//! Model and attribute parser for `#[derive(FeatureSet)]` and `#[derive(TaStream)]`.

use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, ast::NestedMeta};
use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, ExprPath, Field, Fields, Ident, Lit, Meta, Path,
    Result, Token, Type, Visibility, ext::IdentExt, punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    model::OutKind,
    registry::{default_inputs, output_kinds, sort_canonical_inputs},
};

#[derive(Default, FromDeriveInput)]
#[darling(attributes(ta), allow_unknown_fields)]
struct ContainerOpts {
    stream: Option<String>,
    batch: Option<String>,
    rename_all: Option<String>,
    crate_path: Option<String>,
}

#[derive(Default, FromField)]
#[darling(attributes(ta), allow_unknown_fields)]
struct FieldOpts {
    indicator: Option<String>,
    stream: Option<String>,
    rename: Option<String>,
    out: Option<usize>,
}

pub struct StreamMember {
    pub ident: Ident,
    pub ty: Type,
    pub column: String,
    pub fn_into_ident: Ident,
    pub fn_lookback_ident: Ident,
    pub stream_ty: Ident,
    pub inputs: Vec<Ident>,
    pub opts: Vec<TokenStream>,
    pub out_kinds: &'static [OutKind],
    pub out_index: Option<usize>,
}

pub struct StreamModel {
    pub ident: Ident,
    pub batch_ident: Ident,
    pub iter_ident: Ident,
    pub stream_ident: Ident,
    pub vis: Visibility,
    pub runtime: Path,
    pub inputs: Vec<Ident>,
    pub members: Vec<StreamMember>,
}

impl StreamModel {
    pub fn parse(input: &DeriveInput) -> Result<Self> {
        let opts = ContainerOpts::from_derive_input(input)
            .map_err(|e| syn::Error::new(input.span(), e.to_string()))?;
        let container_inputs = parse_container_ta_attrs(&input.attrs)?;
        if input.generics.lifetimes().next().is_some()
            || input.generics.type_params().next().is_some()
            || input.generics.const_params().next().is_some()
        {
            return Err(syn::Error::new(
                input.generics.span(),
                "FeatureSet / TaStream does not support generic parameters on the struct",
            ));
        }

        let runtime = resolve_runtime_path(opts.crate_path.as_deref())?;
        let serde_rename_all = parse_serde_container_rename_all(&input.attrs)?;
        let rename_all = opts.rename_all.or(serde_rename_all);
        if let Some(style) = rename_all.as_deref() {
            apply_rename("field_name", Some(style))?;
        }

        let Data::Struct(data) = &input.data else {
            return Err(syn::Error::new(
                input.span(),
                "FeatureSet / TaStream requires a struct with named fields",
            ));
        };
        let Fields::Named(fields) = &data.fields else {
            return Err(syn::Error::new(
                data.fields.span(),
                "FeatureSet / TaStream requires named fields",
            ));
        };
        if fields.named.is_empty() {
            return Err(syn::Error::new(
                fields.span(),
                "FeatureSet / TaStream requires at least one indicator field",
            ));
        }

        let structure = synstructure::Structure::new(input);
        let mut members = Vec::with_capacity(structure.variants()[0].bindings().len());
        let mut columns = BTreeSet::new();
        let mut inferred_inputs = Vec::new();
        let mut seen_inputs = BTreeSet::new();

        for field in &fields.named {
            let member = parse_member(field, rename_all.as_deref(), &mut columns)?;
            for inp in &member.inputs {
                let key = inp.to_string();
                if seen_inputs.insert(key) {
                    inferred_inputs.push(inp.clone());
                }
            }
            members.push(member);
        }

        let inputs = if container_inputs.is_empty() {
            sort_canonical_inputs(inferred_inputs)
        } else {
            for member in &members {
                for inp in &member.inputs {
                    if !container_inputs.iter().any(|c| c == inp) {
                        return Err(syn::Error::new(
                            inp.span(),
                            format!(
                                "field input `{inp}` is not declared in container \
                                 `#[ta(inputs(...))]`"
                            ),
                        ));
                    }
                }
            }
            container_inputs
        };

        let base_pascal = input.ident.unraw().to_string().to_upper_camel_case();
        let stream_name = match opts.stream {
            Some(name) if !name.trim().is_empty() => name,
            Some(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "stream name must not be empty",
                ));
            }
            None => format!("{base_pascal}Stream"),
        };
        let batch_name = match opts.batch {
            Some(name) if !name.trim().is_empty() => name,
            Some(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "batch name must not be empty",
                ));
            }
            None => format!("{base_pascal}Batch"),
        };
        let iter_name = format!("{batch_name}Iter");

        Ok(Self {
            ident: input.ident.clone(),
            batch_ident: Ident::new(&batch_name, input.ident.span()),
            iter_ident: Ident::new(&iter_name, input.ident.span()),
            stream_ident: Ident::new(&stream_name, input.ident.span()),
            vis: input.vis.clone(),
            runtime,
            inputs,
            members,
        })
    }
}

pub fn resolve_runtime_path(override_path: Option<&str>) -> Result<Path> {
    if let Some(custom) = override_path {
        return syn::parse_str(custom);
    }
    let default_path = match proc_macro_crate::crate_name("ta-lib") {
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            format!("::{}", name.replace('-', "_"))
        }
        _ => "::ta_lib".to_owned(),
    };
    syn::parse_str(&default_path)
}

fn parse_container_ta_attrs(attrs: &[Attribute]) -> Result<Vec<Ident>> {
    let mut inputs = Vec::new();
    let mut seen = BTreeSet::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("ta")) {
        let Meta::List(list) = &attr.meta else {
            return Err(syn::Error::new(
                attr.span(),
                "expected `#[ta(...)]` parenthesized attribute",
            ));
        };
        let nested = NestedMeta::parse_meta_list(list.tokens.clone())?;
        for item in nested {
            let NestedMeta::Meta(meta) = item else {
                return Err(syn::Error::new(
                    attr.span(),
                    "expected named `ta` attribute",
                ));
            };
            let name = meta
                .path()
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default();
            match name.as_str() {
                "stream" | "batch" | "rename_all" | "crate_path" => {}
                "inputs" => {
                    let Meta::List(in_list) = &meta else {
                        return Err(syn::Error::new(
                            meta.span(),
                            "expected `inputs(col1, col2, ...)`",
                        ));
                    };
                    let parsed: Punctuated<Ident, Token![,]> =
                        in_list.parse_args_with(Punctuated::parse_terminated)?;
                    if parsed.is_empty() {
                        return Err(syn::Error::new(
                            in_list.span(),
                            "`inputs(...)` cannot be empty",
                        ));
                    }
                    for ident in parsed {
                        if !seen.insert(ident.to_string()) {
                            return Err(syn::Error::new(
                                ident.span(),
                                format!("duplicate container input `{ident}`"),
                            ));
                        }
                        inputs.push(ident);
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        meta.span(),
                        format!("unknown container `ta` attribute `{other}`"),
                    ));
                }
            }
        }
    }
    Ok(inputs)
}

fn parse_member(
    field: &Field,
    rename_all: Option<&str>,
    columns: &mut BTreeSet<String>,
) -> Result<StreamMember> {
    let f_opts =
        FieldOpts::from_field(field).map_err(|e| syn::Error::new(field.span(), e.to_string()))?;
    if f_opts.indicator.is_some() && f_opts.stream.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "use either `indicator = \"...\"` or `stream = \"...\"`, not both",
        ));
    }
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| syn::Error::new(field.span(), "expected named field"))?;
    let serde_rename = parse_serde_field_rename(&field.attrs)?;
    let column = match f_opts.rename.or(serde_rename) {
        Some(r) => r,
        None => apply_rename(&ident.unraw().to_string(), rename_all)?,
    };
    if column.is_empty() || column.contains('\0') || !columns.insert(column.clone()) {
        return Err(syn::Error::new(
            field.span(),
            "field names must be nonempty, unique, and contain no NUL",
        ));
    }

    let (shorthand_ind, explicit_inputs, opts) = parse_field_lists(&field.attrs)?;
    if shorthand_ind.is_some() && (f_opts.indicator.is_some() || f_opts.stream.is_some()) {
        return Err(syn::Error::new(
            field.span(),
            "cannot combine shorthand indicator call with `indicator = ...` or `stream = ...`",
        ));
    }

    let (fn_name_str, stream_ty) = resolve_indicator_names(
        shorthand_ind.as_deref().or(f_opts.indicator.as_deref()),
        f_opts.stream.as_deref(),
        &ident,
        field.span(),
    )?;

    let inputs = if explicit_inputs.is_empty() {
        default_inputs(&fn_name_str)
            .iter()
            .map(|s| Ident::new(s, field.span()))
            .collect()
    } else {
        explicit_inputs
    };

    let out_kinds = output_kinds(&fn_name_str);
    if let Some(idx) = f_opts.out
        && idx >= out_kinds.len()
    {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "`out = {idx}` is out of bounds for indicator `{fn_name_str}` (which has {} \
                 outputs)",
                out_kinds.len()
            ),
        ));
    }

    let fn_into_ident = Ident::new(&format!("{fn_name_str}_into"), field.span());
    let fn_lookback_ident = Ident::new(&format!("{fn_name_str}_lookback"), field.span());

    Ok(StreamMember {
        ident,
        ty: field.ty.clone(),
        column,
        fn_into_ident,
        fn_lookback_ident,
        stream_ty,
        inputs,
        opts,
        out_kinds,
        out_index: f_opts.out,
    })
}

fn resolve_indicator_names(
    indicator: Option<&str>,
    stream: Option<&str>,
    field_ident: &Ident,
    span: Span,
) -> Result<(String, Ident)> {
    if let Some(st) = stream {
        let trimmed = st.trim();
        if trimmed.is_empty() {
            return Err(syn::Error::new(span, "`stream` name cannot be empty"));
        }
        let base = trimmed.strip_suffix("Stream").unwrap_or(trimmed);
        let fn_name = base.to_snake_case();
        return Ok((fn_name, Ident::new(trimmed, span)));
    }
    let raw = match indicator {
        Some(ind) if !ind.trim().is_empty() => ind.trim().to_owned(),
        Some(_) => return Err(syn::Error::new(span, "`indicator` name cannot be empty")),
        None => field_ident.unraw().to_string(),
    };
    let (fn_name, stream_name) = if let Some(base) = raw.strip_suffix("Stream") {
        (base.to_snake_case(), raw)
    } else {
        let snake = raw.to_lowercase();
        let stream = format!("{}Stream", snake.to_upper_camel_case());
        (snake, stream)
    };
    Ok((fn_name, Ident::new(&stream_name, span)))
}

fn parse_field_lists(
    attrs: &[Attribute],
) -> Result<(Option<String>, Vec<Ident>, Vec<TokenStream>)> {
    let mut shorthand_ind = None;
    let mut inputs = Vec::new();
    let mut opts = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("ta")) {
        let Meta::List(list) = &attr.meta else {
            return Err(syn::Error::new(
                attr.span(),
                "expected `#[ta(...)]` parenthesized attribute",
            ));
        };
        let nested = NestedMeta::parse_meta_list(list.tokens.clone())?;
        for item in nested {
            let NestedMeta::Meta(meta) = item else {
                return Err(syn::Error::new(
                    attr.span(),
                    "expected named `ta` attribute",
                ));
            };
            let name = meta
                .path()
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default();
            match name.as_str() {
                "indicator" | "stream" | "rename" | "out" => {}
                "inputs" => {
                    let Meta::List(in_list) = &meta else {
                        return Err(syn::Error::new(meta.span(), "expected `inputs(col1, ...)`"));
                    };
                    let parsed: Punctuated<Ident, Token![,]> =
                        in_list.parse_args_with(Punctuated::parse_terminated)?;
                    if parsed.is_empty() {
                        return Err(syn::Error::new(
                            in_list.span(),
                            "`inputs(...)` cannot be empty",
                        ));
                    }
                    inputs.extend(parsed);
                }
                "opts" => {
                    let Meta::List(opt_list) = &meta else {
                        return Err(syn::Error::new(meta.span(), "expected `opts(...)`"));
                    };
                    let parsed: Punctuated<Expr, Token![,]> =
                        opt_list.parse_args_with(Punctuated::parse_terminated)?;
                    for expr in parsed {
                        opts.push(wrap_opt_expr(&expr));
                    }
                }
                other if !other.is_empty() => {
                    if shorthand_ind.is_some() {
                        return Err(syn::Error::new(
                            meta.span(),
                            "multiple indicator shorthands on the same field",
                        ));
                    }
                    shorthand_ind = Some(other.to_owned());
                    match &meta {
                        Meta::Path(_) => {}
                        Meta::List(opt_list) => {
                            let parsed: Punctuated<Expr, Token![,]> =
                                opt_list.parse_args_with(Punctuated::parse_terminated)?;
                            for expr in parsed {
                                opts.push(wrap_opt_expr(&expr));
                            }
                        }
                        Meta::NameValue(nv) => {
                            let Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) = &nv.value
                            else {
                                return Err(syn::Error::new(
                                    meta.span(),
                                    format!("expected argument list or string for `{other}`"),
                                ));
                            };
                            let parsed: Punctuated<Expr, Token![,]> =
                                syn::parse::Parser::parse_str(
                                    Punctuated::parse_terminated,
                                    &s.value(),
                                )?;
                            for expr in parsed {
                                opts.push(wrap_opt_expr(&expr));
                            }
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(meta.span(), "invalid `ta` field attribute"));
                }
            }
        }
    }
    Ok((shorthand_ind, inputs, opts))
}

pub fn wrap_opt_expr(expr: &Expr) -> TokenStream {
    match expr {
        Expr::Infer(_) => quote!(::core::option::Option::None),
        Expr::Path(ExprPath { path, .. }) if path.is_ident("None") => {
            quote!(::core::option::Option::None)
        }
        Expr::Call(call)
            if matches!(
                call.func.as_ref(),
                Expr::Path(ExprPath { path, .. }) if path.is_ident("Some")
            ) =>
        {
            quote!(#expr)
        }
        _ => quote!(::core::option::Option::Some(#expr)),
    }
}

pub fn apply_rename(value: &str, style: Option<&str>) -> Result<String> {
    Ok(match style {
        None => value.to_owned(),
        Some("snake_case") => value.to_snake_case(),
        Some("camelCase") => value.to_lower_camel_case(),
        Some("PascalCase") => value.to_upper_camel_case(),
        Some("SCREAMING_SNAKE_CASE") => value.to_shouty_snake_case(),
        Some("kebab-case") => value.to_case(Case::Kebab),
        Some("SCREAMING-KEBAB-CASE") => value.to_case(Case::Cobol),
        Some("lowercase") => value.to_lowercase(),
        Some("UPPERCASE") => value.to_uppercase(),
        Some(_) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "unsupported `rename_all` case style",
            ));
        }
    })
}

fn parse_serde_container_rename_all(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut rename_all = None;
    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let metas: Punctuated<Meta, Token![,]> =
            list.parse_args_with(Punctuated::parse_terminated)?;
        for meta in metas {
            if meta.path().is_ident("rename_all") && rename_all.is_none() {
                rename_all = extract_lit_str(&meta);
            }
        }
    }
    Ok(rename_all)
}

fn parse_serde_field_rename(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut rename = None;
    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let metas: Punctuated<Meta, Token![,]> =
            list.parse_args_with(Punctuated::parse_terminated)?;
        for meta in metas {
            if meta.path().is_ident("rename") && rename.is_none() {
                rename = extract_lit_str(&meta);
            }
        }
    }
    Ok(rename)
}

fn extract_lit_str(meta: &Meta) -> Option<String> {
    let Meta::NameValue(nv) = meta else {
        return None;
    };
    let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = &nv.value
    else {
        return None;
    };
    Some(s.value())
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::StreamModel;

    #[test]
    fn parses_shorthand_and_infers_canonical_ohlcv_inputs() {
        let input: syn::DeriveInput = parse_quote! {
            #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
            pub struct AlphaSnapshot {
                #[ta(ema(9))]
                pub fast_ema: f64,
                #[ta(atr(14))]
                pub atr_14: f64,
                #[ta(macd(12, 26, 9), out = 2)]
                pub macd_hist: f64,
            }
        };
        let model = StreamModel::parse(&input).unwrap();
        assert_eq!(model.stream_ident, "AlphaSnapshotStream");
        assert_eq!(model.batch_ident, "AlphaSnapshotBatch");
        let input_names: Vec<String> = model.inputs.iter().map(ToString::to_string).collect();
        assert_eq!(input_names, vec!["high", "low", "close"]);
        assert_eq!(model.members[0].column, "FAST-EMA");
        assert_eq!(model.members[0].stream_ty, "EmaStream");
        assert_eq!(model.members[1].stream_ty, "AtrStream");
        assert_eq!(model.members[2].out_index, Some(2));
    }

    #[test]
    fn rejects_invalid_derive_inputs() {
        let invalid: [syn::DeriveInput; 4] = [
            parse_quote!(
                enum Bad {
                    A,
                }
            ),
            parse_quote!(
                struct Tuple(f64);
            ),
            parse_quote!(
                struct Empty {}
            ),
            parse_quote!(
                struct OutOfBounds {
                    #[ta(ema(9), out = 1)]
                    fast_ema: f64,
                }
            ),
        ];
        for item in invalid {
            assert!(StreamModel::parse(&item).is_err());
        }
    }
}
