//! Code generation for `ta_indicators!`.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Ident, Result};

use crate::model::{IndicatorList, IndicatorSpec, OptKind, OutKind};

pub fn expand_all(list: &IndicatorList) -> Result<TokenStream> {
    let mut out = TokenStream::new();
    for spec in &list.items {
        let metadata = emit_metadata(spec);
        let f64_batch = emit_batch_f64(spec);
        let f32_batch = emit_batch_f32(spec);
        let stream = emit_stream(spec);
        out.extend(quote!(#metadata #f64_batch #f32_batch #stream));
    }
    Ok(out)
}

fn c_name(spec: &IndicatorSpec, suffix: &str) -> Ident {
    Ident::new(&format!("TA_{}{suffix}", spec.c_ident), Span::call_site())
}

fn opt_rust_type(kind: OptKind) -> TokenStream {
    match kind {
        OptKind::Int => quote!(Option<usize>),
        OptKind::Real => quote!(Option<f64>),
        OptKind::MaType => quote!(Option<::ta_lib::types::MAType>),
    }
}

fn opt_c_expr(name: &Ident, kind: OptKind) -> TokenStream {
    match kind {
        OptKind::Int => quote! { match #name {
            Some(v) => i32::try_from(v).map_err(|_| ::ta_lib::error::Error::BadParam)?,
            None => ::ta_lib::sys::TA_INTEGER_DEFAULT,
        } },
        OptKind::Real => quote!(#name.unwrap_or(::ta_lib::sys::TA_REAL_DEFAULT)),
        OptKind::MaType => quote!(#name.unwrap_or(::ta_lib::types::MAType::Default).into()),
    }
}

fn out_elem_type(kind: OutKind) -> TokenStream {
    match kind {
        OutKind::F64 => quote!(f64),
        OutKind::I32 => quote!(i32),
    }
}

struct Parameters<'a> {
    in_names: &'a [Ident],
    opt_params_decl: Vec<TokenStream>,
    opt_names: Vec<&'a Ident>,
    opt_c_lets: Vec<TokenStream>,
    opt_c_vars: Vec<Ident>,
    out_names: Vec<&'a Ident>,
    out_slice_params: Vec<TokenStream>,
    out_types: Vec<TokenStream>,
}

impl<'a> Parameters<'a> {
    fn new(spec: &'a IndicatorSpec) -> Self {
        Self {
            in_names: &spec.inputs,
            opt_params_decl: spec
                .opts
                .iter()
                .map(|o| {
                    let name = &o.name;
                    let ty = opt_rust_type(o.kind);
                    quote!(#name: #ty)
                })
                .collect(),
            opt_names: spec.opts.iter().map(|o| &o.name).collect(),
            opt_c_lets: spec
                .opts
                .iter()
                .map(|o| {
                    let name = format_ident!("c_opt_{}", o.name);
                    let value = opt_c_expr(&o.name, o.kind);
                    quote!(let #name = #value;)
                })
                .collect(),
            opt_c_vars: spec
                .opts
                .iter()
                .map(|o| format_ident!("c_opt_{}", o.name))
                .collect(),
            out_names: spec.outs.iter().map(|o| &o.name).collect(),
            out_slice_params: spec
                .outs
                .iter()
                .map(|o| {
                    let name = &o.name;
                    let ty = out_elem_type(o.kind);
                    quote!(#name: &mut [#ty])
                })
                .collect(),
            out_types: spec.outs.iter().map(|o| out_elem_type(o.kind)).collect(),
        }
    }
}

fn emit_metadata(spec: &IndicatorSpec) -> TokenStream {
    let Parameters {
        opt_params_decl,
        opt_c_lets,
        opt_c_vars,
        ..
    } = Parameters::new(spec);
    let fn_lookback = format_ident!("{}_lookback", spec.fn_name);
    let fn_display_shift = format_ident!("{}_display_shift", spec.fn_name);
    let c_lookback = c_name(spec, "_Lookback");
    let c_display_shift = c_name(spec, "_DisplayShift");
    quote! {
        /// Return the warmup bar count for the configured indicator.
        /// # Errors
        /// Returns `BadParam` for invalid parameters or a negative native lookback.
        pub fn #fn_lookback(#(#opt_params_decl),*) -> Result<usize, ::ta_lib::error::Error> {
            #(#opt_c_lets)*
            // SAFETY: The native lookback accepts these scalar parameters.
            let lb = unsafe { ::ta_lib::sys::#c_lookback(#(#opt_c_vars),*) };
            usize::try_from(lb).map_err(|_| ::ta_lib::error::Error::BadParam)
        }
        /// Return the chart display shift for a zero-based output index.
        /// # Errors
        /// Returns `BadParam` for invalid parameters or output indices.
        #[cfg(not(feature = "stable"))]
        pub fn #fn_display_shift(#(#opt_params_decl,)* output_idx: usize) -> Result<i32, ::ta_lib::error::Error> {
            #(#opt_c_lets)*
            let index = i32::try_from(output_idx).map_err(|_| ::ta_lib::error::Error::BadParam)?;
            // SAFETY: The native metadata query accepts scalar parameters.
            let shift = unsafe { ::ta_lib::sys::#c_display_shift(#(#opt_c_vars,)* index) };
            if shift == ::ta_lib::sys::TA_INTEGER_DEFAULT { return Err(::ta_lib::error::Error::BadParam); }
            Ok(shift)
        }
    }
}

fn emit_batch_f64(spec: &IndicatorSpec) -> TokenStream {
    emit_batch(spec, false)
}
fn emit_batch_f32(spec: &IndicatorSpec) -> TokenStream {
    emit_batch(spec, true)
}

fn emit_batch(spec: &IndicatorSpec, single: bool) -> TokenStream {
    let Parameters {
        in_names,
        opt_params_decl,
        opt_names,
        opt_c_lets,
        opt_c_vars,
        out_names,
        out_slice_params,
        out_types,
    } = Parameters::new(spec);
    let first_in = &in_names[0];
    let rest_ins = &in_names[1..];
    let fn_name = if single {
        format_ident!("{}_f32", spec.fn_name)
    } else {
        spec.fn_name.clone()
    };
    let fn_into = format_ident!("{}_into", fn_name);
    let fn_lookback = format_ident!("{}_lookback", spec.fn_name);
    let c_batch = if single {
        format_ident!("TA_S_{}", spec.c_ident)
    } else {
        c_name(spec, "")
    };
    let input_type = if single { quote!(f32) } else { quote!(f64) };
    let attrs = &spec.attrs;
    quote! {
        /// Calculate into caller-owned buffers without Rust output allocation.
        /// Each output needs `input.len().saturating_sub(lookback)` elements.
        /// # Errors
        /// Returns an error for empty/mismatched inputs, invalid options, short outputs or native failures.
        #[allow(clippy::too_many_arguments, clippy::type_complexity)]
        pub fn #fn_into(#(#in_names: &[#input_type],)* #(#opt_params_decl,)* #(#out_slice_params,)*)
            -> Result<::ta_lib::types::OutRange, ::ta_lib::error::Error> {
            let len = #first_in.len();
            if len == 0 { return Err(::ta_lib::error::Error::EmptyInput); }
            #(if #rest_ins.len() != len {
                return Err(::ta_lib::error::Error::MismatchedInputLength { expected: len, actual: #rest_ins.len() });
            })*
            let lb = #fn_lookback(#(#opt_names),*)?;
            let required = len.saturating_sub(lb);
            #(if #out_names.len() < required {
                return Err(::ta_lib::error::Error::OutputBufferTooSmall { required, actual: #out_names.len() });
            })*
            #(#opt_c_lets)*
            let end = i32::try_from(len - 1).map_err(|_| ::ta_lib::error::Error::BadParam)?;
            let (mut begin, mut count) = (0_i32, 0_i32);
            // SAFETY: Inputs cover the range, outputs cover its maximum lookback-adjusted size.
            // Empty output slices supply non-null dangling pointers when no output is possible.
            let rc = unsafe { ::ta_lib::sys::#c_batch(0, end, #(#in_names.as_ptr(),)* #(#opt_c_vars,)*
                &raw mut begin, &raw mut count, #(#out_names.as_mut_ptr(),)*) };
            ::ta_lib::error::check_ret_code(rc)?;
            Ok(::ta_lib::types::OutRange {
                begin: usize::try_from(begin).map_err(|_| ::ta_lib::error::Error::BadParam)?,
                count: usize::try_from(count).map_err(|_| ::ta_lib::error::Error::BadParam)?,
            })
        }
        #(#attrs)*
        /// # Errors
        /// Returns an error for empty/mismatched inputs, invalid options or native failures.
        #[allow(clippy::too_many_arguments, clippy::type_complexity)]
        pub fn #fn_name(#(#in_names: &[#input_type],)* #(#opt_params_decl,)*)
            -> Result<(#(Vec<#out_types>,)* usize), ::ta_lib::error::Error> {
            #(let mut #out_names: Vec<#out_types> = vec![Default::default(); #first_in.len()];)*
            let range = #fn_into(#(#in_names,)* #(#opt_names,)* #(&mut #out_names,)*)?;
            #(#out_names.truncate(range.count);)*
            Ok((#(#out_names,)* range.begin))
        }
    }
}

fn emit_stream(spec: &IndicatorSpec) -> TokenStream {
    let Parameters {
        in_names,
        opt_params_decl,
        opt_names,
        opt_c_lets,
        opt_c_vars,
        out_names,
        out_slice_params,
        out_types,
    } = Parameters::new(spec);
    let first_in = &in_names[0];
    let rest_ins = &in_names[1..];
    let stream_ident = &spec.stream_ident;
    let fn_lookback = format_ident!("{}_lookback", spec.fn_name);
    let c_stream_ty = c_name(spec, "_Stream");
    let c_open = c_name(spec, "_Open");
    let c_open_and_fill = c_name(spec, "_OpenAndFill");
    let c_update = c_name(spec, "_Update");
    let c_peek = c_name(spec, "_Peek");
    let c_value = c_name(spec, "_Value");
    let c_out_range = c_name(spec, "_OutRange");
    let c_advance = c_name(spec, "_Advance");
    let c_clone = c_name(spec, "_Clone");
    let c_close = c_name(spec, "_Close");
    let stream_val_ty = if out_types.len() == 1 {
        let t = &out_types[0];
        quote!(#t)
    } else {
        quote!((#(#out_types),*))
    };
    let stream_val_tuple = if out_names.len() == 1 {
        let n = out_names[0];
        quote!(#n)
    } else {
        quote!((#(#out_names),*))
    };
    let stream_val_init: Vec<_> = out_names
        .iter()
        .zip(&out_types)
        .map(|(n, t)| quote!(let mut #n: #t = Default::default();))
        .collect();
    quote! {
        /// RAII streaming state machine for incremental bar-by-bar calculation.
        #[derive(Debug)]
        pub struct #stream_ident {
            ptr: core::ptr::NonNull<::ta_lib::sys::#c_stream_ty>,
        }

        // SAFETY: Each TA-Lib stream instance owns an independent heap state block and is safe to move across threads.
        unsafe impl Send for #stream_ident {}

        impl #stream_ident {
            /// Warm up a new stream on historical bars and return the stream along with the latest output value.
            ///
            /// # Errors
            ///
            /// Returns an error if inputs are empty, mismatched, insufficient history is provided, or parameters are invalid.
            #[allow(clippy::too_many_arguments, clippy::type_complexity)]
            pub fn open(
                #(#in_names: &[f64],)*
                #(#opt_params_decl,)*
            ) -> Result<(Self, #stream_val_ty), ::ta_lib::error::Error> {
                let len = #first_in.len();
                if len == 0 {
                    return Err(::ta_lib::error::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != len {
                        return Err(::ta_lib::error::Error::MismatchedInputLength {
                            expected: len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                #(#opt_c_lets)*
                let hist_len = i32::try_from(len).map_err(|_| ::ta_lib::error::Error::BadParam)?;
                let mut raw: *mut ::ta_lib::sys::#c_stream_ty = core::ptr::null_mut();
                #(#stream_val_init)*
                // SAFETY: Input slices have `len` elements and output pointers reference valid stack variables.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_open(
                        &raw mut raw,
                        #(#in_names.as_ptr(),)*
                        hist_len,
                        #(#opt_c_vars,)*
                        #(&raw mut #out_names,)*
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(raw).ok_or(::ta_lib::error::Error::AllocErr)?;
                Ok((Self { ptr }, #stream_val_tuple))
            }

            /// Warm up a new stream on historical bars while filling caller-provided output buffers for the history.
            ///
            /// # Errors
            ///
            /// Returns an error if inputs are empty, mismatched, output buffers are too small, history is insufficient, or parameters are invalid.
            #[allow(clippy::too_many_arguments, clippy::type_complexity)]
            pub fn open_and_fill(
                #(#in_names: &[f64],)*
                #(#opt_params_decl,)*
                #(#out_slice_params,)*
            ) -> Result<(Self, ::ta_lib::types::OutRange), ::ta_lib::error::Error> {
                let len = #first_in.len();
                if len == 0 {
                    return Err(::ta_lib::error::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != len {
                        return Err(::ta_lib::error::Error::MismatchedInputLength {
                            expected: len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                let lb = #fn_lookback(#(#opt_names),*)?;
                if len <= lb { return Err(::ta_lib::error::Error::InsufficientHistory); }
                let required = len - lb;
                #(
                    if #out_names.len() < required {
                        return Err(::ta_lib::error::Error::OutputBufferTooSmall {
                            required,
                            actual: #out_names.len(),
                        });
                    }
                )*
                #(#opt_c_lets)*
                let hist_len = i32::try_from(len).map_err(|_| ::ta_lib::error::Error::BadParam)?;
                let mut raw: *mut ::ta_lib::sys::#c_stream_ty = core::ptr::null_mut();
                let mut out_begin: i32 = 0;
                let mut out_count: i32 = 0;
                // SAFETY: Inputs cover the history; outputs hold its lookback-adjusted count.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_open_and_fill(
                        &raw mut raw,
                        #(#in_names.as_ptr(),)*
                        hist_len,
                        #(#opt_c_vars,)*
                        &raw mut out_begin,
                        &raw mut out_count,
                        #(#out_names.as_mut_ptr(),)*
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(raw).ok_or(::ta_lib::error::Error::AllocErr)?;
                let stream = Self { ptr };
                Ok((
                    stream,
                    ::ta_lib::types::OutRange {
                        begin: usize::try_from(out_begin).map_err(|_| ::ta_lib::error::Error::BadParam)?,
                        count: usize::try_from(out_count).map_err(|_| ::ta_lib::error::Error::BadParam)?,
                    },
                ))
            }

            /// Commit one closed bar to the stream and return the newly computed output value.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream update fails.
            pub fn update(&mut self, #(#in_names: f64),*) -> Result<#stream_val_ty, ::ta_lib::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is a valid non-null stream pointer owned exclusively by `self`.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_update(
                        self.ptr.as_ptr(),
                        #(#in_names,)*
                        #(&raw mut #out_names,)*
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Evaluate a forming bar without mutating internal stream state.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream peek fails.
            pub fn peek(&self, #(#in_names: f64),*) -> Result<#stream_val_ty, ::ta_lib::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is valid and `TA_*_Peek` takes a const stream pointer without mutating state.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_peek(
                        self.ptr.as_ptr(),
                        #(#in_names,)*
                        #(&raw mut #out_names,)*
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Read the most recently committed output value without advancing the stream.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream value query fails.
            pub fn value(&self) -> Result<#stream_val_ty, ::ta_lib::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is valid and `TA_*_Value` reads the latest value into stack variables.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_value(
                        self.ptr.as_ptr(),
                        #(&raw mut #out_names,)*
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Query the cumulative output range (`begin` and `count`) of the stream so far.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream out-range query fails.
            pub fn out_range(&self) -> Result<::ta_lib::types::OutRange, ::ta_lib::error::Error> {
                let mut out_begin: i32 = 0;
                let mut out_count: i32 = 0;
                // SAFETY: `self.ptr` is valid and `TA_*_OutRange` writes to stack integers.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_out_range(
                        self.ptr.as_ptr(),
                        &raw mut out_begin,
                        &raw mut out_count,
                    )
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                Ok(::ta_lib::types::OutRange {
                    begin: usize::try_from(out_begin).map_err(|_| ::ta_lib::error::Error::BadParam)?,
                    count: usize::try_from(out_count).map_err(|_| ::ta_lib::error::Error::BadParam)?,
                })
            }

            /// Skip one unprocessed bar via `TA_*_Advance`, incrementing `outRangeCount`.
            ///
            /// Recurrence state and `value()` stay unchanged. Use `update` to commit a bar.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream advance fails.
            pub fn advance(&mut self) -> Result<(), ::ta_lib::error::Error> {
                // SAFETY: `self.ptr` is valid and uniquely borrowed via `&mut self`.
                let ret_code = unsafe { ::ta_lib::sys::#c_advance(self.ptr.as_ptr()) };
                ::ta_lib::error::check_ret_code(ret_code)
            }

            /// Deep-copy the stream state into a new independent stream handle.
            ///
            /// # Errors
            ///
            /// Returns an error if memory allocation fails in the C library.
            pub fn try_clone(&self) -> Result<Self, ::ta_lib::error::Error> {
                let mut cloned: *mut ::ta_lib::sys::#c_stream_ty = core::ptr::null_mut();
                // SAFETY: `self.ptr` is valid and `TA_*_Clone` allocates a fresh copy into `cloned`.
                let ret_code = unsafe {
                    ::ta_lib::sys::#c_clone(self.ptr.as_ptr(), &raw mut cloned)
                };
                ::ta_lib::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(cloned).ok_or(::ta_lib::error::Error::AllocErr)?;
                Ok(Self { ptr })
            }
        }

        impl Drop for #stream_ident {
            fn drop(&mut self) {
                // SAFETY: `self.ptr` was allocated by `TA_*_Open` / `OpenAndFill` / `Clone` and is freed once here.
                unsafe {
                    drop(::ta_lib::sys::#c_close(self.ptr.as_ptr()));
                }
            }
        }
    }
}
