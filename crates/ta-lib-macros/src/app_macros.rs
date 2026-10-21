//! Function-like application macros: `ta_call!`, `ta_batch_aligned!`, and `ta_dyn_call!`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, LitStr, Result, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::{
    derive_stream::{resolve_runtime_path, wrap_opt_expr},
    model::OutKind,
    registry::output_kinds,
};

/// Parsed representation of `ta_call!([padded:] func(in1, in2; opt1, _, opt3; out1, out2))`.
pub struct TaCallInput {
    pub padded: bool,
    pub func: syn::ExprPath,
    pub inputs: Vec<Expr>,
    pub opts: Vec<Expr>,
    pub outputs: Vec<Expr>,
}

impl Parse for TaCallInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let fork = input.fork();
        let padded = if let Ok(kw) = fork.parse::<Ident>()
            && kw == "padded"
            && fork.peek(Token![:])
            && !fork.peek2(Token![:])
        {
            input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            true
        } else {
            false
        };

        let func: syn::ExprPath = input.parse()?;
        let content;
        parenthesized!(content in input);

        let mut inputs = Vec::new();
        while !content.is_empty() && !content.peek(Token![;]) {
            inputs.push(content.parse::<Expr>()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        let mut opts = Vec::new();
        if content.peek(Token![;]) {
            content.parse::<Token![;]>()?;
            while !content.is_empty() && !content.peek(Token![;]) {
                opts.push(content.parse::<Expr>()?);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        let mut outputs = Vec::new();
        if content.peek(Token![;]) {
            content.parse::<Token![;]>()?;
            while !content.is_empty() {
                outputs.push(content.parse::<Expr>()?);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        }

        if !content.is_empty() {
            return Err(content.error("unexpected token in `ta_call!` arguments"));
        }

        Ok(Self {
            padded,
            func,
            inputs,
            opts,
            outputs,
        })
    }
}

pub fn expand_ta_call(call: &TaCallInput) -> Result<TokenStream> {
    let func = &call.func;
    let inputs = &call.inputs;
    let opts: Vec<TokenStream> = call.opts.iter().map(wrap_opt_expr).collect();
    let outputs = &call.outputs;
    if !call.padded {
        return Ok(quote! {
            #func(#(#inputs,)* #(#opts,)* #(#outputs),*)
        });
    }

    let runtime = resolve_runtime_path(None)?;
    if inputs.is_empty() {
        return Err(syn::Error::new(
            call.func.path.segments.last().unwrap().ident.span(),
            "`ta_call!(padded: ...)` requires at least one input slice",
        ));
    }
    let last_seg = call.func.path.segments.last().expect("non-empty path");
    let fn_name = last_seg.ident.to_string();
    let base_name = fn_name.strip_suffix("_f32").unwrap_or(&fn_name);

    let mut fn_lb = call.func.clone();
    fn_lb.path.segments.last_mut().unwrap().ident =
        Ident::new(&format!("{base_name}_lookback"), last_seg.ident.span());

    let mut fn_into = call.func.clone();
    fn_into.path.segments.last_mut().unwrap().ident =
        Ident::new(&format!("{fn_name}_into"), last_seg.ident.span());

    let kinds = output_kinds(base_name);
    let out_ids: Vec<Ident> = (0..kinds.len())
        .map(|i| format_ident!("__out_{i}"))
        .collect();
    let out_decls: Vec<TokenStream> = kinds
        .iter()
        .zip(&out_ids)
        .map(|(&kind, id)| match kind {
            OutKind::F64 => quote!(let mut #id: ::std::vec::Vec<f64> = vec![f64::NAN; __len];),
            OutKind::I32 => quote!(let mut #id: ::std::vec::Vec<i32> = vec![0_i32; __len];),
        })
        .collect();
    let ret_expr = if out_ids.len() == 1 {
        let id = &out_ids[0];
        quote!(#id)
    } else {
        quote!((#(#out_ids),*))
    };
    let first_in = &inputs[0];
    let rest_ins = &inputs[1..];

    Ok(quote! {
        (|| -> ::core::result::Result<_, #runtime::Error> {
            let __in0 = #first_in;
            let __len = __in0.len();
            if __len == 0 {
                return Err(#runtime::Error::EmptyInput);
            }
            let __lb = #fn_lb(#(#opts),*)?;
            #(#out_decls)*
            if __len > __lb {
                let _ = #fn_into(__in0, #(#rest_ins,)* #(#opts,)* #(&mut #out_ids[__lb..]),*)?;
            }
            Ok(#ret_expr)
        })()
    })
}

pub enum BatchFieldBinding {
    Single(Ident),
    Tuple(Vec<Ident>),
}

pub enum BatchCallExpr {
    Shorthand(TaCallInput),
    Standard(Expr),
}

pub struct BatchEntry {
    pub binding: BatchFieldBinding,
    pub call: BatchCallExpr,
}

pub struct BatchAlignedInput {
    pub entries: Vec<BatchEntry>,
}

impl Parse for BatchAlignedInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut entries = Vec::new();
        while !input.is_empty() {
            let binding = if input.peek(syn::token::Paren) {
                let inner;
                parenthesized!(inner in input);
                let idents: Punctuated<Ident, Token![,]> =
                    inner.parse_terminated(Ident::parse, Token![,])?;
                if idents.is_empty() {
                    return Err(inner.error("tuple binding in `ta_batch_aligned!` cannot be empty"));
                }
                BatchFieldBinding::Tuple(idents.into_iter().collect())
            } else {
                let id: Ident = input.parse()?;
                BatchFieldBinding::Single(id)
            };

            input.parse::<Token![:]>()?;

            let fork = input.fork();
            let call = if fork.parse::<TaCallInput>().is_ok()
                && (fork.is_empty() || fork.peek(Token![,]))
            {
                let parsed: TaCallInput = input.parse()?;
                BatchCallExpr::Shorthand(parsed)
            } else {
                BatchCallExpr::Standard(input.parse::<Expr>()?)
            };

            entries.push(BatchEntry { binding, call });
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        if entries.is_empty() {
            return Err(input.error("`ta_batch_aligned!` requires at least one indicator entry"));
        }
        Ok(Self { entries })
    }
}

pub fn expand_batch_aligned(batch: &BatchAlignedInput) -> Result<TokenStream> {
    let runtime = resolve_runtime_path(None)?;

    let mut all_fields = Vec::new();
    let mut type_params = Vec::new();
    let mut eval_stmts = Vec::new();
    let mut begin_vars = Vec::new();
    let mut trim_stmts = Vec::new();

    for (idx, entry) in batch.entries.iter().enumerate() {
        let begin_id = format_ident!("__begin_{idx}");
        begin_vars.push(begin_id.clone());

        let call_ts = match &entry.call {
            BatchCallExpr::Shorthand(c) => expand_ta_call(c)?,
            BatchCallExpr::Standard(e) => quote!(#e),
        };

        match &entry.binding {
            BatchFieldBinding::Single(ident) => {
                let ty_param = format_ident!("__T{}", all_fields.len());
                type_params.push(ty_param);
                all_fields.push(ident.clone());
                eval_stmts.push(quote! {
                    let (mut #ident, #begin_id) = (#call_ts)?;
                });
                trim_stmts.push(quote! {
                    let __skip = __max_begin.saturating_sub(#begin_id);
                    if __skip >= #ident.len() {
                        #ident.clear();
                    } else if __skip > 0 {
                        drop(#ident.drain(0..__skip));
                    }
                    if #ident.len() < __aligned_count {
                        __aligned_count = #ident.len();
                    }
                });
            }
            BatchFieldBinding::Tuple(idents) => {
                for ident in idents {
                    let ty_param = format_ident!("__T{}", all_fields.len());
                    type_params.push(ty_param);
                    all_fields.push(ident.clone());
                    trim_stmts.push(quote! {
                        let __skip = __max_begin.saturating_sub(#begin_id);
                        if __skip >= #ident.len() {
                            #ident.clear();
                        } else if __skip > 0 {
                            drop(#ident.drain(0..__skip));
                        }
                        if #ident.len() < __aligned_count {
                            __aligned_count = #ident.len();
                        }
                    });
                }
                eval_stmts.push(quote! {
                    let (#(mut #idents,)* #begin_id) = (#call_ts)?;
                });
            }
        }
    }

    let first_field = &all_fields[0];
    let serde_crate_path = format!("{}::serde", quote!(#runtime).to_string().replace(' ', ""));
    Ok(quote! {
        (|| -> ::core::result::Result<_, #runtime::Error> {
            #[derive(
                Debug,
                Clone,
                PartialEq,
                #runtime::serde::Serialize,
                #runtime::serde::Deserialize,
            )]
            #[serde(crate = #serde_crate_path)]
            pub struct TaAlignedBatch<#(#type_params),*> {
                /// Shared warmup index (`max(begin_i)`) in the original input series.
                pub begin: usize,
                /// Number of aligned output bars shared across all indicator vectors.
                pub count: usize,
                #(pub #all_fields: ::std::vec::Vec<#type_params>,)*
            }

            #(#eval_stmts)*

            let mut __max_begin: usize = 0;
            #(
                if #begin_vars > __max_begin {
                    __max_begin = #begin_vars;
                }
            )*

            let mut __aligned_count: usize = usize::MAX;
            #(#trim_stmts)*
            if __aligned_count == usize::MAX {
                __aligned_count = #first_field.len();
            }
            #(
                if #all_fields.len() > __aligned_count {
                    #all_fields.truncate(__aligned_count);
                }
            )*

            Ok(TaAlignedBatch {
                begin: __max_begin,
                count: __aligned_count,
                #(#all_fields,)*
            })
        })()
    })
}

pub struct DynCallInput {
    pub indicator: LitStr,
    pub range: Option<Expr>,
    pub real_inputs: Vec<Expr>,
    pub int_inputs: Vec<Expr>,
    pub price_inputs: Vec<Expr>,
    pub int_opts: Vec<Expr>,
    pub real_opts: Vec<Expr>,
    pub real_outputs: Vec<Expr>,
    pub int_outputs: Vec<Expr>,
}

impl Parse for DynCallInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let indicator: LitStr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let mut range = None;
        let mut real_inputs = Vec::new();
        let mut int_inputs = Vec::new();
        let mut price_inputs = Vec::new();
        let mut int_opts = Vec::new();
        let mut real_opts = Vec::new();
        let mut real_outputs = Vec::new();
        let mut int_outputs = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "range" => {
                    range = Some(input.parse::<Expr>()?);
                }
                "real_inputs" => real_inputs = parse_expr_brackets(input)?,
                "int_inputs" => int_inputs = parse_expr_brackets(input)?,
                "price_inputs" => price_inputs = parse_expr_brackets(input)?,
                "int_opts" => int_opts = parse_expr_brackets(input)?,
                "real_opts" => real_opts = parse_expr_brackets(input)?,
                "real_outputs" => real_outputs = parse_expr_brackets(input)?,
                "int_outputs" => int_outputs = parse_expr_brackets(input)?,
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown `ta_dyn_call!` section `{other}`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if real_inputs.is_empty() && int_inputs.is_empty() && price_inputs.is_empty() {
            return Err(syn::Error::new(
                indicator.span(),
                "`ta_dyn_call!` requires at least one of `real_inputs`, `int_inputs`, or \
                 `price_inputs`",
            ));
        }
        if real_outputs.is_empty() && int_outputs.is_empty() {
            return Err(syn::Error::new(
                indicator.span(),
                "`ta_dyn_call!` requires at least one of `real_outputs` or `int_outputs`",
            ));
        }

        Ok(Self {
            indicator,
            range,
            real_inputs,
            int_inputs,
            price_inputs,
            int_opts,
            real_opts,
            real_outputs,
            int_outputs,
        })
    }
}

fn parse_expr_brackets(input: ParseStream<'_>) -> Result<Vec<Expr>> {
    let content;
    bracketed!(content in input);
    let list: Punctuated<Expr, Token![,]> = content.parse_terminated(Expr::parse, Token![,])?;
    Ok(list.into_iter().collect())
}

pub fn expand_dyn_call(dyn_call: &DynCallInput) -> Result<TokenStream> {
    let runtime = resolve_runtime_path(None)?;
    let name = &dyn_call.indicator;

    let mut set_stmts = Vec::new();
    for (idx, expr) in dyn_call.real_inputs.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        set_stmts.push(quote! {
            __holder.set_input_real(#idx_u32, #expr)?;
        });
    }
    for (idx, expr) in dyn_call.int_inputs.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        set_stmts.push(quote! {
            __holder.set_input_integer(#idx_u32, #expr)?;
        });
    }
    for (idx, expr) in dyn_call.price_inputs.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        set_stmts.push(quote! {
            let (__o, __h, __l, __c, __v, __oi) = #expr;
            __holder.set_input_price(#idx_u32, __o, __h, __l, __c, __v, __oi)?;
        });
    }
    for (idx, expr) in dyn_call.int_opts.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        let opt = wrap_opt_expr(expr);
        set_stmts.push(quote! {
            let __opt_val: i32 = match #opt {
                ::core::option::Option::Some(v) => v,
                ::core::option::Option::None => #runtime::sys::TA_INTEGER_DEFAULT,
            };
            __holder.set_opt_input_integer(#idx_u32, __opt_val)?;
        });
    }
    for (idx, expr) in dyn_call.real_opts.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        let opt = wrap_opt_expr(expr);
        set_stmts.push(quote! {
            let __opt_val: f64 = match #opt {
                ::core::option::Option::Some(v) => v,
                ::core::option::Option::None => #runtime::sys::TA_REAL_DEFAULT,
            };
            __holder.set_opt_input_real(#idx_u32, __opt_val)?;
        });
    }
    for (idx, expr) in dyn_call.real_outputs.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        set_stmts.push(quote! {
            __holder.set_output_real(#idx_u32, #expr)?;
        });
    }
    for (idx, expr) in dyn_call.int_outputs.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        set_stmts.push(quote! {
            __holder.set_output_integer(#idx_u32, #expr)?;
        });
    }

    let range_stmt = if let Some(r_expr) = &dyn_call.range {
        quote! {
            let __range = #r_expr;
            let (__start, __end) = (
                *::core::ops::RangeBounds::start_bound(&__range),
                *::core::ops::RangeBounds::end_bound(&__range),
            );
            let __start_idx = match __start {
                ::core::ops::Bound::Included(v) => v,
                ::core::ops::Bound::Excluded(v) => v + 1,
                ::core::ops::Bound::Unbounded => 0,
            };
            let __end_idx = match __end {
                ::core::ops::Bound::Included(v) => v,
                ::core::ops::Bound::Excluded(v) => v.saturating_sub(1),
                ::core::ops::Bound::Unbounded => return Err(#runtime::Error::BadParam),
            };
        }
    } else if let Some(first_in) = dyn_call
        .real_inputs
        .first()
        .or_else(|| dyn_call.int_inputs.first())
    {
        quote! {
            let __len = (#first_in).len();
            if __len == 0 {
                return Err(#runtime::Error::EmptyInput);
            }
            let __start_idx = 0usize;
            let __end_idx = __len - 1;
        }
    } else {
        return Err(syn::Error::new(
            dyn_call.indicator.span(),
            "`range: start..=end` is required when only `price_inputs` are supplied",
        ));
    };

    Ok(quote! {
        (|| -> ::core::result::Result<#runtime::OutRange, #runtime::Error> {
            #range_stmt
            let __handle = #runtime::FuncHandle::get(#name)?;
            let mut __holder = #runtime::ParamHolder::new(&__handle)?;
            #(#set_stmts)*
            __holder.call(__start_idx, __end_idx)
        })()
    })
}
