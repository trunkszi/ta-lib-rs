//! Code generation for `ta_indicators!`.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Ident, Result};

use crate::model::{IndicatorList, IndicatorSpec, OptKind, OutKind};

pub fn expand_all(list: &IndicatorList) -> Result<TokenStream> {
    let mut out = TokenStream::new();
    for item in &list.items {
        out.extend(expand_one(item)?);
    }
    Ok(out)
}

fn opt_rust_type(kind: OptKind) -> TokenStream {
    match kind {
        OptKind::Int => quote!(Option<usize>),
        OptKind::Real => quote!(Option<f64>),
        OptKind::MaType => quote!(Option<crate::types::MAType>),
    }
}

fn opt_c_expr(name: &Ident, kind: OptKind) -> TokenStream {
    match kind {
        OptKind::Int => quote! {
            match #name {
                Some(v) => i32::try_from(v).map_err(|_| crate::error::Error::BadParam)?,
                None => ta_lib_sys::TA_INTEGER_DEFAULT,
            }
        },
        OptKind::Real => quote! {
            #name.unwrap_or(ta_lib_sys::TA_REAL_DEFAULT)
        },
        OptKind::MaType => quote! {
            #name.unwrap_or(crate::types::MAType::Default).into()
        },
    }
}

fn out_elem_type(kind: OutKind) -> TokenStream {
    match kind {
        OutKind::F64 => quote!(f64),
        OutKind::I32 => quote!(i32),
    }
}

fn expand_one(spec: &IndicatorSpec) -> Result<TokenStream> {
    let attrs = &spec.attrs;
    let fn_name = &spec.fn_name;
    let fn_into = format_ident!("{}_into", fn_name);
    let fn_f32 = format_ident!("{}_f32", fn_name);
    let fn_f32_into = format_ident!("{}_f32_into", fn_name);
    let fn_lookback = format_ident!("{}_lookback", fn_name);
    let fn_display_shift = format_ident!("{}_display_shift", fn_name);
    let stream_ident = &spec.stream_ident;

    let c_batch = Ident::new(&format!("TA_{}", spec.c_ident), Span::call_site());
    let c_batch_s = Ident::new(&format!("TA_S_{}", spec.c_ident), Span::call_site());
    let c_lookback = Ident::new(&format!("TA_{}_Lookback", spec.c_ident), Span::call_site());
    let c_display_shift = Ident::new(
        &format!("TA_{}_DisplayShift", spec.c_ident),
        Span::call_site(),
    );
    let c_stream_ty = Ident::new(&format!("TA_{}_Stream", spec.c_ident), Span::call_site());
    let c_open = Ident::new(&format!("TA_{}_Open", spec.c_ident), Span::call_site());
    let c_open_and_fill = Ident::new(
        &format!("TA_{}_OpenAndFill", spec.c_ident),
        Span::call_site(),
    );
    let c_update = Ident::new(&format!("TA_{}_Update", spec.c_ident), Span::call_site());
    let c_peek = Ident::new(&format!("TA_{}_Peek", spec.c_ident), Span::call_site());
    let c_value = Ident::new(&format!("TA_{}_Value", spec.c_ident), Span::call_site());
    let c_out_range = Ident::new(&format!("TA_{}_OutRange", spec.c_ident), Span::call_site());
    let c_advance = Ident::new(&format!("TA_{}_Advance", spec.c_ident), Span::call_site());
    let c_clone = Ident::new(&format!("TA_{}_Clone", spec.c_ident), Span::call_site());
    let c_close = Ident::new(&format!("TA_{}_Close", spec.c_ident), Span::call_site());

    let in_names = &spec.inputs;
    let first_in = &in_names[0];
    let rest_ins = &in_names[1..];

    let opt_params_decl: Vec<TokenStream> = spec
        .opts
        .iter()
        .map(|o| {
            let n = &o.name;
            let ty = opt_rust_type(o.kind);
            quote!(#n: #ty)
        })
        .collect();
    let opt_names: Vec<&Ident> = spec.opts.iter().map(|o| &o.name).collect();
    let opt_c_lets: Vec<TokenStream> = spec
        .opts
        .iter()
        .map(|o| {
            let c_var = format_ident!("c_opt_{}", o.name);
            let expr = opt_c_expr(&o.name, o.kind);
            quote!(let #c_var = #expr;)
        })
        .collect();
    let opt_c_vars: Vec<Ident> = spec
        .opts
        .iter()
        .map(|o| format_ident!("c_opt_{}", o.name))
        .collect();

    let out_names: Vec<&Ident> = spec.outs.iter().map(|o| &o.name).collect();
    let out_types: Vec<TokenStream> = spec.outs.iter().map(|o| out_elem_type(o.kind)).collect();
    let out_slice_params: Vec<TokenStream> = spec
        .outs
        .iter()
        .map(|o| {
            let n = &o.name;
            let ty = out_elem_type(o.kind);
            quote!(#n: &mut [#ty])
        })
        .collect();
    let out_vec_types: Vec<TokenStream> = out_types.iter().map(|t| quote!(Vec<#t>)).collect();
    let batch_ret_ty = if out_vec_types.len() == 1 {
        let t0 = &out_vec_types[0];
        quote!((#t0, usize))
    } else {
        quote!((#(#out_vec_types,)* usize))
    };
    let stream_val_ty = if out_types.len() == 1 {
        let t0 = &out_types[0];
        quote!(#t0)
    } else {
        quote!((#(#out_types),*))
    };
    let stream_val_init: Vec<TokenStream> = spec
        .outs
        .iter()
        .map(|o| {
            let n = &o.name;
            match o.kind {
                OutKind::F64 => quote!(let mut #n = 0.0_f64;),
                OutKind::I32 => quote!(let mut #n = 0_i32;),
            }
        })
        .collect();
    let stream_val_tuple = if out_names.len() == 1 {
        let n0 = out_names[0];
        quote!(#n0)
    } else {
        quote!((#(#out_names),*))
    };

    let vec_allocs: Vec<TokenStream> = spec
        .outs
        .iter()
        .map(|o| {
            let n = &o.name;
            let ty = out_elem_type(o.kind);
            let zero = match o.kind {
                OutKind::F64 => quote!(0.0_f64),
                OutKind::I32 => quote!(0_i32),
            };
            quote!(let mut #n: Vec<#ty> = vec![#zero; #first_in.len()];)
        })
        .collect();
    let vec_truncates: Vec<TokenStream> = out_names
        .iter()
        .map(|n| quote!(#n.truncate(range.count);))
        .collect();
    let batch_ok_expr = if out_names.len() == 1 {
        let n0 = out_names[0];
        quote!(Ok((#n0, range.begin)))
    } else {
        quote!(Ok((#(#out_names,)* range.begin)))
    };

    Ok(quote! {
        /// Compute the lookback period (warmup bar count) for this indicator.
        ///
        /// # Errors
        ///
        /// Returns [`crate::error::Error::BadParam`] if any optional parameter is out of range.
        pub fn #fn_lookback(#(#opt_params_decl),*) -> Result<usize, crate::error::Error> {
            #(#opt_c_lets)*
            // SAFETY: Calling pure C lookback function with validated scalar parameters.
            let lb = unsafe { ta_lib_sys::#c_lookback(#(#opt_c_vars),*) };
            usize::try_from(lb).map_err(|_| crate::error::Error::BadParam)
        }

        /// Query the chart display shift for the specified `output_idx` (0-based).
        ///
        /// # Errors
        ///
        /// Returns [`crate::error::Error::BadParam`] if any optional parameter or `output_idx` is invalid.
        pub fn #fn_display_shift(
            #(#opt_params_decl,)*
            output_idx: usize,
        ) -> Result<i32, crate::error::Error> {
            #(#opt_c_lets)*
            let out_idx = i32::try_from(output_idx).map_err(|_| crate::error::Error::BadParam)?;
            // SAFETY: Calling pure C display-shift function with scalar parameters.
            let shift = unsafe { ta_lib_sys::#c_display_shift(#(#opt_c_vars,)* out_idx) };
            if shift == ta_lib_sys::TA_INTEGER_DEFAULT {
                return Err(crate::error::Error::BadParam);
            }
            Ok(shift)
        }

        /// Compute the indicator into caller-provided mutable output slices without heap allocation.
        ///
        /// # Errors
        ///
        /// Returns an error if input slices are empty, input slice lengths do not match,
        /// output buffers are smaller than the input length, or the underlying TA-Lib function fails.
        pub fn #fn_into(
            #(#in_names: &[f64],)*
            #(#opt_params_decl,)*
            #(#out_slice_params,)*
        ) -> Result<crate::types::OutRange, crate::error::Error> {
            let len = #first_in.len();
            if len == 0 {
                return Err(crate::error::Error::EmptyInput);
            }
            #(
                if #rest_ins.len() != len {
                    return Err(crate::error::Error::MismatchedInputLength {
                        expected: len,
                        actual: #rest_ins.len(),
                    });
                }
            )*
            #(
                if #out_names.len() < len {
                    return Err(crate::error::Error::OutputBufferTooSmall {
                        required: len,
                        actual: #out_names.len(),
                    });
                }
            )*
            #(#opt_c_lets)*
            let end_idx = i32::try_from(len - 1).map_err(|_| crate::error::Error::BadParam)?;
            let mut out_begin: i32 = 0;
            let mut out_count: i32 = 0;
            // SAFETY: All input and output slices are verified to have at least `len` elements.
            let ret_code = unsafe {
                ta_lib_sys::#c_batch(
                    0,
                    end_idx,
                    #(#in_names.as_ptr(),)*
                    #(#opt_c_vars,)*
                    &raw mut out_begin,
                    &raw mut out_count,
                    #(#out_names.as_mut_ptr(),)*
                )
            };
            crate::error::check_ret_code(ret_code)?;
            Ok(crate::types::OutRange {
                begin: usize::try_from(out_begin).unwrap_or(0),
                count: usize::try_from(out_count).unwrap_or(0),
            })
        }

        #(#attrs)*
        ///
        /// # Errors
        ///
        /// Returns an error if input slices are empty, mismatched, or parameters are out of range.
        pub fn #fn_name(
            #(#in_names: &[f64],)*
            #(#opt_params_decl,)*
        ) -> Result<#batch_ret_ty, crate::error::Error> {
            if #first_in.is_empty() {
                return Err(crate::error::Error::EmptyInput);
            }
            #(#vec_allocs)*
            let range = #fn_into(
                #(#in_names,)*
                #(#opt_names,)*
                #(&mut #out_names,)*
            )?;
            #(#vec_truncates)*
            #batch_ok_expr
        }

        /// Compute the indicator over single-precision (`f32`) inputs into caller-provided output slices.
        ///
        /// # Errors
        ///
        /// Returns an error if input slices are empty, mismatched, output slices are too small, or parameters are invalid.
        pub fn #fn_f32_into(
            #(#in_names: &[f32],)*
            #(#opt_params_decl,)*
            #(#out_slice_params,)*
        ) -> Result<crate::types::OutRange, crate::error::Error> {
            let len = #first_in.len();
            if len == 0 {
                return Err(crate::error::Error::EmptyInput);
            }
            #(
                if #rest_ins.len() != len {
                    return Err(crate::error::Error::MismatchedInputLength {
                        expected: len,
                        actual: #rest_ins.len(),
                    });
                }
            )*
            #(
                if #out_names.len() < len {
                    return Err(crate::error::Error::OutputBufferTooSmall {
                        required: len,
                        actual: #out_names.len(),
                    });
                }
            )*
            #(#opt_c_lets)*
            let end_idx = i32::try_from(len - 1).map_err(|_| crate::error::Error::BadParam)?;
            let mut out_begin: i32 = 0;
            let mut out_count: i32 = 0;
            // SAFETY: All f32 input and output slices have at least `len` elements.
            let ret_code = unsafe {
                ta_lib_sys::#c_batch_s(
                    0,
                    end_idx,
                    #(#in_names.as_ptr(),)*
                    #(#opt_c_vars,)*
                    &raw mut out_begin,
                    &raw mut out_count,
                    #(#out_names.as_mut_ptr(),)*
                )
            };
            crate::error::check_ret_code(ret_code)?;
            Ok(crate::types::OutRange {
                begin: usize::try_from(out_begin).unwrap_or(0),
                count: usize::try_from(out_count).unwrap_or(0),
            })
        }

        /// Compute the indicator over single-precision (`f32`) inputs, returning allocated output vectors.
        ///
        /// # Errors
        ///
        /// Returns an error if input slices are empty, mismatched, or parameters are invalid.
        pub fn #fn_f32(
            #(#in_names: &[f32],)*
            #(#opt_params_decl,)*
        ) -> Result<#batch_ret_ty, crate::error::Error> {
            if #first_in.is_empty() {
                return Err(crate::error::Error::EmptyInput);
            }
            #(#vec_allocs)*
            let range = #fn_f32_into(
                #(#in_names,)*
                #(#opt_names,)*
                #(&mut #out_names,)*
            )?;
            #(#vec_truncates)*
            #batch_ok_expr
        }

        /// RAII streaming state machine for incremental bar-by-bar calculation.
        #[derive(Debug)]
        pub struct #stream_ident {
            ptr: core::ptr::NonNull<ta_lib_sys::#c_stream_ty>,
        }

        // SAFETY: Each TA-Lib stream instance owns an independent heap state block and is safe to move across threads.
        unsafe impl Send for #stream_ident {}

        impl #stream_ident {
            /// Warm up a new stream on historical bars and return the stream along with the latest output value.
            ///
            /// # Errors
            ///
            /// Returns an error if inputs are empty, mismatched, insufficient history is provided, or parameters are invalid.
            pub fn open(
                #(#in_names: &[f64],)*
                #(#opt_params_decl,)*
            ) -> Result<(Self, #stream_val_ty), crate::error::Error> {
                let len = #first_in.len();
                if len == 0 {
                    return Err(crate::error::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != len {
                        return Err(crate::error::Error::MismatchedInputLength {
                            expected: len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                #(#opt_c_lets)*
                let hist_len = i32::try_from(len).map_err(|_| crate::error::Error::BadParam)?;
                let mut raw: *mut ta_lib_sys::#c_stream_ty = core::ptr::null_mut();
                #(#stream_val_init)*
                // SAFETY: Input slices have `len` elements and output pointers reference valid stack variables.
                let ret_code = unsafe {
                    ta_lib_sys::#c_open(
                        &raw mut raw,
                        #(#in_names.as_ptr(),)*
                        hist_len,
                        #(#opt_c_vars,)*
                        #(&raw mut #out_names,)*
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(raw).ok_or(crate::error::Error::AllocErr)?;
                Ok((Self { ptr }, #stream_val_tuple))
            }

            /// Warm up a new stream on historical bars while filling caller-provided output buffers for the history.
            ///
            /// # Errors
            ///
            /// Returns an error if inputs are empty, mismatched, output buffers are too small, or parameters are invalid.
            pub fn open_and_fill(
                #(#in_names: &[f64],)*
                #(#opt_params_decl,)*
                #(#out_slice_params,)*
            ) -> Result<(Self, crate::types::OutRange), crate::error::Error> {
                let len = #first_in.len();
                if len == 0 {
                    return Err(crate::error::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != len {
                        return Err(crate::error::Error::MismatchedInputLength {
                            expected: len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                #(
                    if #out_names.len() < len {
                        return Err(crate::error::Error::OutputBufferTooSmall {
                            required: len,
                            actual: #out_names.len(),
                        });
                    }
                )*
                #(#opt_c_lets)*
                let hist_len = i32::try_from(len).map_err(|_| crate::error::Error::BadParam)?;
                let mut raw: *mut ta_lib_sys::#c_stream_ty = core::ptr::null_mut();
                let mut out_begin: i32 = 0;
                let mut out_count: i32 = 0;
                // SAFETY: Input and output slices are verified to hold at least `len` elements.
                let ret_code = unsafe {
                    ta_lib_sys::#c_open_and_fill(
                        &raw mut raw,
                        #(#in_names.as_ptr(),)*
                        hist_len,
                        #(#opt_c_vars,)*
                        &raw mut out_begin,
                        &raw mut out_count,
                        #(#out_names.as_mut_ptr(),)*
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(raw).ok_or(crate::error::Error::AllocErr)?;
                Ok((
                    Self { ptr },
                    crate::types::OutRange {
                        begin: usize::try_from(out_begin).unwrap_or(0),
                        count: usize::try_from(out_count).unwrap_or(0),
                    },
                ))
            }

            /// Commit one closed bar to the stream and return the newly computed output value.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream update fails.
            pub fn update(&mut self, #(#in_names: f64),*) -> Result<#stream_val_ty, crate::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is a valid non-null stream pointer owned exclusively by `self`.
                let ret_code = unsafe {
                    ta_lib_sys::#c_update(
                        self.ptr.as_ptr(),
                        #(#in_names,)*
                        #(&raw mut #out_names,)*
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Evaluate a forming bar without mutating internal stream state.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream peek fails.
            pub fn peek(&self, #(#in_names: f64),*) -> Result<#stream_val_ty, crate::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is valid and `TA_*_Peek` takes a const stream pointer without mutating state.
                let ret_code = unsafe {
                    ta_lib_sys::#c_peek(
                        self.ptr.as_ptr(),
                        #(#in_names,)*
                        #(&raw mut #out_names,)*
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Read the most recently committed output value without advancing the stream.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream value query fails.
            pub fn value(&self) -> Result<#stream_val_ty, crate::error::Error> {
                #(#stream_val_init)*
                // SAFETY: `self.ptr` is valid and `TA_*_Value` reads the latest value into stack variables.
                let ret_code = unsafe {
                    ta_lib_sys::#c_value(
                        self.ptr.as_ptr(),
                        #(&raw mut #out_names,)*
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                Ok(#stream_val_tuple)
            }

            /// Query the cumulative output range (`begin` and `count`) of the stream so far.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream out-range query fails.
            pub fn out_range(&self) -> Result<crate::types::OutRange, crate::error::Error> {
                let mut out_begin: i32 = 0;
                let mut out_count: i32 = 0;
                // SAFETY: `self.ptr` is valid and `TA_*_OutRange` writes to stack integers.
                let ret_code = unsafe {
                    ta_lib_sys::#c_out_range(
                        self.ptr.as_ptr(),
                        &raw mut out_begin,
                        &raw mut out_count,
                    )
                };
                crate::error::check_ret_code(ret_code)?;
                Ok(crate::types::OutRange {
                    begin: usize::try_from(out_begin).unwrap_or(0),
                    count: usize::try_from(out_count).unwrap_or(0),
                })
            }

            /// Advance the stream state after a previous `peek` or step.
            ///
            /// # Errors
            ///
            /// Returns an error if the underlying C stream advance fails.
            pub fn advance(&mut self) -> Result<(), crate::error::Error> {
                // SAFETY: `self.ptr` is valid and uniquely borrowed via `&mut self`.
                let ret_code = unsafe { ta_lib_sys::#c_advance(self.ptr.as_ptr()) };
                crate::error::check_ret_code(ret_code)
            }

            /// Deep-copy the stream state into a new independent stream handle.
            ///
            /// # Errors
            ///
            /// Returns an error if memory allocation fails in the C library.
            pub fn try_clone(&self) -> Result<Self, crate::error::Error> {
                let mut cloned: *mut ta_lib_sys::#c_stream_ty = core::ptr::null_mut();
                // SAFETY: `self.ptr` is valid and `TA_*_Clone` allocates a fresh copy into `cloned`.
                let ret_code = unsafe {
                    ta_lib_sys::#c_clone(self.ptr.as_ptr(), &raw mut cloned)
                };
                crate::error::check_ret_code(ret_code)?;
                let ptr = core::ptr::NonNull::new(cloned).ok_or(crate::error::Error::AllocErr)?;
                Ok(Self { ptr })
            }
        }

        impl Drop for #stream_ident {
            fn drop(&mut self) {
                // SAFETY: `self.ptr` was allocated by `TA_*_Open` / `OpenAndFill` / `Clone` and is freed once here.
                unsafe {
                    drop(ta_lib_sys::#c_close(self.ptr.as_ptr()));
                }
            }
        }
    })
}
