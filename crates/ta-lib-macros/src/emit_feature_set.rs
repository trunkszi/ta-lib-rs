//! Code generation for `#[derive(FeatureSet)]` and `#[derive(TaStream)]`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Index};

use crate::{
    derive_stream::{StreamMember, StreamModel},
    model::OutKind,
};

pub fn expand(model: &StreamModel) -> TokenStream {
    let struct_impl = emit_struct_impl(model);
    let batch_items = emit_batch(model);
    let stream_items = emit_stream(model);
    quote! {
        #struct_impl
        #batch_items
        #stream_items
    }
}

fn pad_expr(kind: OutKind) -> TokenStream {
    match kind {
        OutKind::F64 => quote!(f64::NAN),
        OutKind::I32 => quote!(0_i32),
    }
}

fn default_expr(kind: OutKind) -> TokenStream {
    match kind {
        OutKind::F64 => quote!(0.0_f64),
        OutKind::I32 => quote!(0_i32),
    }
}

fn scalar_type(kind: OutKind) -> TokenStream {
    match kind {
        OutKind::F64 => quote!(f64),
        OutKind::I32 => quote!(i32),
    }
}

fn extract_stream_val(m: &StreamMember, raw_expr: TokenStream) -> TokenStream {
    if let Some(idx) = m.out_index {
        let index = Index::from(idx);
        quote!((#raw_expr).#index)
    } else {
        raw_expr
    }
}

fn emit_struct_impl(model: &StreamModel) -> TokenStream {
    let ident = &model.ident;
    let batch_ident = &model.batch_ident;
    let stream_ident = &model.stream_ident;
    let runtime = &model.runtime;
    let inputs = &model.inputs;
    let first_in = &inputs[0];
    let columns: Vec<&str> = model.members.iter().map(|m| m.column.as_str()).collect();

    quote! {
        impl #ident {
            /// Normalized column/field names for the feature set.
            pub const FIELD_NAMES: &'static [&'static str] = &[#(#columns),*];

            /// Compute all indicators in batch mode aligned to `max(begin)` without `NaN` padding.
            /// # Errors
            /// Returns an error if inputs are empty, mismatched, or parameters are invalid.
            #[allow(clippy::too_many_arguments)]
            pub fn compute_aligned(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<#batch_ident, #runtime::Error> {
                let mut batch = #batch_ident::with_capacity(#first_in.len());
                let _ = batch.compute_aligned_into(#(#inputs),*)?;
                Ok(batch)
            }

            /// Compute all indicators in batch mode with warmup prefixes padded with `f64::NAN`.
            /// # Errors
            /// Returns an error if inputs are empty, mismatched, or parameters are invalid.
            #[allow(clippy::too_many_arguments)]
            pub fn compute_padded(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<#batch_ident, #runtime::Error> {
                let mut batch = #batch_ident::with_capacity(#first_in.len());
                let _ = batch.compute_padded_into(#(#inputs),*)?;
                Ok(batch)
            }

            /// Warm up a new multi-indicator stream and return the latest snapshot.
            /// # Errors
            /// Returns an error if any member indicator fails warmup validation or allocation.
            #[allow(clippy::too_many_arguments)]
            pub fn open_stream(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<(#stream_ident, Self), #runtime::Error> {
                #stream_ident::open(#(#inputs),*)
            }

            /// Warm up a new multi-indicator stream while filling an aligned batch in one pass.
            /// # Errors
            /// Returns an error if history is insufficient or any member indicator fails.
            #[allow(clippy::too_many_arguments)]
            pub fn open_and_fill(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<(#stream_ident, #batch_ident), #runtime::Error> {
                #stream_ident::open_and_fill(#(#inputs),*)
            }
        }
    }
}

fn emit_batch(model: &StreamModel) -> TokenStream {
    let ident = &model.ident;
    let batch_ident = &model.batch_ident;
    let iter_ident = &model.iter_ident;
    let vis = &model.vis;
    let runtime = &model.runtime;
    let inputs = &model.inputs;
    let first_in = &inputs[0];
    let rest_ins = &inputs[1..];

    let columns: Vec<&str> = model.members.iter().map(|m| m.column.as_str()).collect();
    let field_idents: Vec<&Ident> = model.members.iter().map(|m| &m.ident).collect();
    let field_tys: Vec<&syn::Type> = model.members.iter().map(|m| &m.ty).collect();

    let lookback_vars: Vec<Ident> = model
        .members
        .iter()
        .map(|m| format_ident!("__lb_{}", m.ident))
        .collect();
    let lookback_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .zip(&lookback_vars)
        .map(|(m, lb_id)| {
            let fn_lb = &m.fn_lookback_ident;
            let opts = &m.opts;
            quote! {
                let #lb_id = #runtime::#fn_lb(#(#opts),*)?;
                if #lb_id > __max_begin {
                    __max_begin = #lb_id;
                }
            }
        })
        .collect();

    let aligned_member_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .zip(&lookback_vars)
        .map(|(m, lb_id)| emit_member_aligned_into(m, lb_id, runtime))
        .collect();

    let padded_member_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .zip(&lookback_vars)
        .map(|(m, lb_id)| emit_member_padded_into(m, lb_id, runtime))
        .collect();

    let serde_crate_path = format!("{}::serde", quote!(#runtime).to_string().replace(' ', ""));

    quote! {
        /// Structure-of-Arrays (SoA) columnar batch generated by `#[derive(FeatureSet)]`.
        #[derive(
            Debug,
            Clone,
            PartialEq,
            #runtime::serde::Serialize,
            #runtime::serde::Deserialize,
        )]
        #[serde(crate = #serde_crate_path)]
        #vis struct #batch_ident {
            /// Warmup offset (`max(begin_i)`) in the original input series.
            pub begin: usize,
            /// Number of rows shared across all column vectors.
            pub count: usize,
            #(
                /// Column vector for the corresponding indicator field.
                pub #field_idents: ::std::vec::Vec<#field_tys>,
            )*
        }

        impl #batch_ident {
            /// Normalized column/field names for the batch.
            pub const FIELD_NAMES: &'static [&'static str] = &[#(#columns),*];

            /// Pre-allocate column vectors with the given capacity for zero-allocation reuse.
            #[must_use]
            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    begin: 0,
                    count: 0,
                    #(#field_idents: ::std::vec::Vec::with_capacity(capacity),)*
                }
            }

            /// Return the number of rows currently stored in the batch.
            #[must_use]
            pub const fn len(&self) -> usize {
                self.count
            }

            /// Return `true` if the batch contains no rows.
            #[must_use]
            pub const fn is_empty(&self) -> bool {
                self.count == 0
            }

            /// Clear all column vectors while retaining their allocated capacities.
            pub fn clear(&mut self) {
                self.begin = 0;
                self.count = 0;
                #(self.#field_idents.clear();)*
            }

            /// Project row `index` (`0..self.count`) as an owned snapshot struct.
            #[must_use]
            pub fn row(&self, index: usize) -> ::core::option::Option<#ident> {
                if index >= self.count {
                    return ::core::option::Option::None;
                }
                ::core::option::Option::Some(#ident {
                    #(#field_idents: self.#field_idents[index],)*
                })
            }

            /// Iterate over all rows in the batch as snapshot structs.
            #[must_use]
            pub const fn iter(&self) -> #iter_ident<'_> {
                #iter_ident {
                    batch: self,
                    front: 0,
                    back: self.count,
                }
            }

            /// Compute all indicators into this batch's reusable buffers, aligning every column
            /// to start at `max(begin_i)` without `NaN` padding.
            /// # Errors
            /// Returns an error if inputs are empty, mismatched, or parameters are invalid.
            #[allow(clippy::too_many_arguments)]
            pub fn compute_aligned_into(
                &mut self,
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<#runtime::OutRange, #runtime::Error> {
                let __len = #first_in.len();
                if __len == 0 {
                    return Err(#runtime::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != __len {
                        return Err(#runtime::Error::MismatchedInputLength {
                            expected: __len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                let mut __max_begin: usize = 0;
                #(#lookback_stmts)*
                let __aligned_count = __len.saturating_sub(__max_begin);
                #(#aligned_member_stmts)*
                self.begin = __max_begin;
                self.count = __aligned_count;
                Ok(#runtime::OutRange {
                    begin: self.begin,
                    count: self.count,
                })
            }

            /// Compute all indicators into this batch's reusable buffers with `f64::NAN` warmup padding.
            /// # Errors
            /// Returns an error if inputs are empty, mismatched, or parameters are invalid.
            #[allow(clippy::too_many_arguments)]
            pub fn compute_padded_into(
                &mut self,
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<#runtime::OutRange, #runtime::Error> {
                let __len = #first_in.len();
                if __len == 0 {
                    return Err(#runtime::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != __len {
                        return Err(#runtime::Error::MismatchedInputLength {
                            expected: __len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                let mut __max_begin: usize = 0;
                #(#lookback_stmts)*
                #(#padded_member_stmts)*
                self.begin = __max_begin;
                self.count = __len;
                Ok(#runtime::OutRange {
                    begin: self.begin,
                    count: self.count,
                })
            }
        }

        /// Row iterator over a [`#batch_ident`].
        #[derive(Debug, Clone)]
        #vis struct #iter_ident<'a> {
            batch: &'a #batch_ident,
            front: usize,
            back: usize,
        }

        impl Iterator for #iter_ident<'_> {
            type Item = #ident;

            fn next(&mut self) -> ::core::option::Option<Self::Item> {
                if self.front >= self.back {
                    return ::core::option::Option::None;
                }
                let idx = self.front;
                self.front += 1;
                self.batch.row(idx)
            }

            fn size_hint(&self) -> (usize, ::core::option::Option<usize>) {
                let rem = self.back.saturating_sub(self.front);
                (rem, ::core::option::Option::Some(rem))
            }
        }

        impl DoubleEndedIterator for #iter_ident<'_> {
            fn next_back(&mut self) -> ::core::option::Option<Self::Item> {
                if self.front >= self.back {
                    return ::core::option::Option::None;
                }
                self.back -= 1;
                self.batch.row(self.back)
            }
        }

        impl ExactSizeIterator for #iter_ident<'_> {}

        impl<'a> IntoIterator for &'a #batch_ident {
            type Item = #ident;
            type IntoIter = #iter_ident<'a>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    }
}

fn scratch_buffers_for_target(
    m: &StreamMember,
    target_idx: usize,
    target_slice: TokenStream,
    cap_expr: TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let f = &m.ident;
    let mut scratch_decls = Vec::new();
    let mut args = Vec::new();
    for (i, &kind) in m.out_kinds.iter().enumerate() {
        if i == target_idx {
            args.push(target_slice.clone());
        } else {
            let s_id = format_ident!("__scratch_{f}_{i}");
            let s_def = default_expr(kind);
            let s_ty = scalar_type(kind);
            scratch_decls
                .push(quote!(let mut #s_id: ::std::vec::Vec<#s_ty> = vec![#s_def; #cap_expr];));
            args.push(quote!(&mut #s_id));
        }
    }
    (scratch_decls, args)
}

fn tuple_buffers(m: &StreamMember, cap_expr: TokenStream) -> (Vec<Ident>, Vec<TokenStream>) {
    let f = &m.ident;
    let buf_ids: Vec<Ident> = (0..m.out_kinds.len())
        .map(|i| format_ident!("__buf_{f}_{i}"))
        .collect();
    let buf_decls = m
        .out_kinds
        .iter()
        .zip(&buf_ids)
        .map(|(&kind, b_id)| {
            let def = default_expr(kind);
            let ty = scalar_type(kind);
            quote!(let mut #b_id: ::std::vec::Vec<#ty> = vec![#def; #cap_expr];)
        })
        .collect();
    (buf_ids, buf_decls)
}

fn emit_member_aligned_into(m: &StreamMember, lb_id: &Ident, runtime: &syn::Path) -> TokenStream {
    let f = &m.ident;
    let fn_into = &m.fn_into_ident;
    let ins = &m.inputs;
    let opts = &m.opts;

    if m.out_kinds.len() == 1 {
        let def = default_expr(m.out_kinds[0]);
        quote! {
            self.#f.clear();
            let __req = __len.saturating_sub(#lb_id);
            if __aligned_count > 0 && __req > 0 {
                self.#f.resize(__req, #def);
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* &mut self.#f)?;
                let __skip = __max_begin.saturating_sub(__r.begin);
                if __skip > 0 { drop(self.#f.drain(0..__skip.min(self.#f.len()))); }
                self.#f.truncate(__aligned_count);
            }
        }
    } else if let Some(target_idx) = m.out_index {
        let target_def = default_expr(m.out_kinds[target_idx]);
        let (scratch_decls, args) =
            scratch_buffers_for_target(m, target_idx, quote!(&mut self.#f), quote!(__req));
        quote! {
            self.#f.clear();
            let __req = __len.saturating_sub(#lb_id);
            if __aligned_count > 0 && __req > 0 {
                self.#f.resize(__req, #target_def);
                #(#scratch_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(#args),*)?;
                let __skip = __max_begin.saturating_sub(__r.begin);
                if __skip > 0 { drop(self.#f.drain(0..__skip.min(self.#f.len()))); }
                self.#f.truncate(__aligned_count);
            }
        }
    } else {
        let (buf_ids, buf_decls) = tuple_buffers(m, quote!(__req));
        quote! {
            self.#f.clear();
            let __req = __len.saturating_sub(#lb_id);
            if __aligned_count > 0 && __req > 0 {
                #(#buf_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(&mut #buf_ids),*)?;
                let __skip = __max_begin.saturating_sub(__r.begin);
                self.#f.reserve(__aligned_count);
                for __idx in 0..__aligned_count {
                    let __src = __skip + __idx;
                    self.#f.push((#(#buf_ids[__src]),*));
                }
            }
        }
    }
}

fn emit_member_padded_into(m: &StreamMember, lb_id: &Ident, runtime: &syn::Path) -> TokenStream {
    let f = &m.ident;
    let fn_into = &m.fn_into_ident;
    let ins = &m.inputs;
    let opts = &m.opts;

    if m.out_kinds.len() == 1 {
        let pad = pad_expr(m.out_kinds[0]);
        let def = default_expr(m.out_kinds[0]);
        quote! {
            self.#f.clear();
            if __len <= #lb_id {
                self.#f.resize(__len, #pad);
            } else {
                self.#f.resize(__len, #def);
                self.#f[..#lb_id].fill(#pad);
                let _ = #runtime::#fn_into(#(#ins,)* #(#opts,)* &mut self.#f[#lb_id..])?;
            }
        }
    } else if let Some(target_idx) = m.out_index {
        let pad = pad_expr(m.out_kinds[target_idx]);
        let def = default_expr(m.out_kinds[target_idx]);
        let (scratch_decls, args) = scratch_buffers_for_target(
            m,
            target_idx,
            quote!(&mut self.#f[#lb_id..]),
            quote!(__len - #lb_id),
        );
        quote! {
            self.#f.clear();
            if __len <= #lb_id {
                self.#f.resize(__len, #pad);
            } else {
                self.#f.resize(__len, #def);
                self.#f[..#lb_id].fill(#pad);
                #(#scratch_decls)*
                let _ = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(#args),*)?;
            }
        }
    } else {
        let pad_elems: Vec<TokenStream> = m.out_kinds.iter().copied().map(pad_expr).collect();
        let pad_tuple = quote!((#(#pad_elems),*));
        let (buf_ids, buf_decls) = tuple_buffers(m, quote!(__len - #lb_id));
        quote! {
            self.#f.clear();
            if __len <= #lb_id {
                self.#f.resize(__len, #pad_tuple);
            } else {
                self.#f.resize(#lb_id, #pad_tuple);
                self.#f.reserve(__len - #lb_id);
                #(#buf_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(&mut #buf_ids),*)?;
                for __idx in 0..__r.count {
                    self.#f.push((#(#buf_ids[__idx]),*));
                }
            }
        }
    }
}

fn emit_stream(model: &StreamModel) -> TokenStream {
    let ident = &model.ident;
    let batch_ident = &model.batch_ident;
    let stream_ident = &model.stream_ident;
    let vis = &model.vis;
    let runtime = &model.runtime;
    let inputs = &model.inputs;
    let first_in = &inputs[0];
    let rest_ins = &inputs[1..];

    let columns: Vec<&str> = model.members.iter().map(|m| m.column.as_str()).collect();
    let field_idents: Vec<&Ident> = model.members.iter().map(|m| &m.ident).collect();
    let stream_tys: Vec<&Ident> = model.members.iter().map(|m| &m.stream_ty).collect();

    let open_calls: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| {
            let f = &m.ident;
            let st = &m.stream_ty;
            let ins = &m.inputs;
            let opts = &m.opts;
            let val_id = format_ident!("__val_{f}");
            let extracted = extract_stream_val(m, quote!(#val_id));
            quote! {
                let (#f, #val_id) = #runtime::#st::open(#(#ins,)* #(#opts),*)?;
                let #val_id = #extracted;
            }
        })
        .collect();
    let val_idents: Vec<Ident> = model
        .members
        .iter()
        .map(|m| format_ident!("__val_{}", m.ident))
        .collect();

    let update_calls: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| {
            let f = &m.ident;
            let ins = &m.inputs;
            let val = extract_stream_val(m, quote!(self.#f.update(#(#ins),*)?));
            quote!(#f: #val)
        })
        .collect();

    let peek_calls: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| {
            let f = &m.ident;
            let ins = &m.inputs;
            let val = extract_stream_val(m, quote!(self.#f.peek(#(#ins),*)?));
            quote!(#f: #val)
        })
        .collect();

    let value_calls: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| {
            let f = &m.ident;
            let val = extract_stream_val(m, quote!(self.#f.value()?));
            quote!(#f: #val)
        })
        .collect();

    let open_and_fill_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| emit_member_open_and_fill(m, runtime))
        .collect();

    let first_field = &model.members[0].ident;
    let rest_fields: Vec<&Ident> = model.members[1..].iter().map(|m| &m.ident).collect();

    let accessors: Vec<TokenStream> = model
        .members
        .iter()
        .map(|m| {
            let f = &m.ident;
            let st = &m.stream_ty;
            let get_ref = format_ident!("{f}_stream");
            let get_mut = format_ident!("{f}_stream_mut");
            quote! {
                /// Borrow the underlying indicator stream for this field.
                #[must_use]
                pub const fn #get_ref(&self) -> &#runtime::#st {
                    &self.#f
                }

                /// Mutably borrow the underlying indicator stream for this field.
                pub const fn #get_mut(&mut self) -> &mut #runtime::#st {
                    &mut self.#f
                }
            }
        })
        .collect();

    quote! {
        /// Multi-indicator RAII streaming pipeline generated by `#[derive(FeatureSet)]`.
        #[derive(Debug)]
        #vis struct #stream_ident {
            #(#field_idents: #runtime::#stream_tys,)*
        }

        impl #stream_ident {
            /// Normalized column/field names for the multi-indicator pipeline.
            pub const FIELD_NAMES: &'static [&'static str] = &[#(#columns),*];

            /// Warm up all member indicator streams on historical bar slices.
            /// # Errors
            /// Returns an error if any member indicator fails warmup validation or allocation.
            #[allow(clippy::too_many_arguments)]
            pub fn open(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<(Self, #ident), #runtime::Error> {
                #(#open_calls)*
                Ok((
                    Self { #(#field_idents),* },
                    #ident { #(#field_idents: #val_idents),* },
                ))
            }

            /// Warm up all member indicator streams while filling an aligned [`#batch_ident`].
            /// # Errors
            /// Returns an error if history is insufficient or any member stream fails.
            #[allow(clippy::too_many_arguments)]
            pub fn open_and_fill(
                #(#inputs: &[f64]),*
            ) -> ::core::result::Result<(Self, #batch_ident), #runtime::Error> {
                let __len = #first_in.len();
                if __len == 0 {
                    return Err(#runtime::Error::EmptyInput);
                }
                #(
                    if #rest_ins.len() != __len {
                        return Err(#runtime::Error::MismatchedInputLength {
                            expected: __len,
                            actual: #rest_ins.len(),
                        });
                    }
                )*
                let mut __batch = #batch_ident::with_capacity(__len);
                let mut __max_begin: usize = 0;
                #(#open_and_fill_stmts)*
                let __aligned_count = __len.saturating_sub(__max_begin);
                #(
                    let __skip = __batch.#field_idents.len().saturating_sub(__aligned_count);
                    if __skip > 0 {
                        drop(__batch.#field_idents.drain(0..__skip));
                    }
                )*
                __batch.begin = __max_begin;
                __batch.count = __aligned_count;
                Ok((Self { #(#field_idents),* }, __batch))
            }

            /// Commit one closed bar across all member streams and return the new snapshot.
            /// # Errors
            /// Returns an error if any underlying C stream update fails.
            #[allow(clippy::too_many_arguments)]
            pub fn update(
                &mut self,
                #(#inputs: f64),*
            ) -> ::core::result::Result<#ident, #runtime::Error> {
                Ok(#ident { #(#update_calls),* })
            }

            /// Alias for [`Self::update`]: commit one closed bar and return the new snapshot.
            /// # Errors
            /// Returns an error if any underlying C stream update fails.
            #[allow(clippy::too_many_arguments)]
            pub fn next(
                &mut self,
                #(#inputs: f64),*
            ) -> ::core::result::Result<#ident, #runtime::Error> {
                self.update(#(#inputs),*)
            }

            /// Evaluate a forming bar across all member streams without mutating state.
            /// # Errors
            /// Returns an error if any underlying C stream peek fails.
            #[allow(clippy::too_many_arguments)]
            pub fn peek(
                &self,
                #(#inputs: f64),*
            ) -> ::core::result::Result<#ident, #runtime::Error> {
                Ok(#ident { #(#peek_calls),* })
            }

            /// Read the most recently committed snapshot without advancing any member stream.
            /// # Errors
            /// Returns an error if any underlying C stream value query fails.
            pub fn value(&self) -> ::core::result::Result<#ident, #runtime::Error> {
                Ok(#ident { #(#value_calls),* })
            }

            /// Return the aligned output range (`max(begin)` and `min(count)`).
            /// # Errors
            /// Returns an error if any underlying C stream out-range query fails.
            pub fn out_range(&self) -> ::core::result::Result<#runtime::OutRange, #runtime::Error> {
                let mut agg = self.#first_field.out_range()?;
                #(
                    let r = self.#rest_fields.out_range()?;
                    if r.begin > agg.begin { agg.begin = r.begin; }
                    if r.count < agg.count { agg.count = r.count; }
                )*
                Ok(agg)
            }

            /// Advance all member streams by one skipped bar (`TA_*_Advance`).
            /// # Errors
            /// Returns an error if any underlying C stream advance fails.
            pub fn advance(&mut self) -> ::core::result::Result<(), #runtime::Error> {
                #(self.#field_idents.advance()?;)*
                Ok(())
            }

            /// Deep-copy all member streams into an independent pipeline.
            /// # Errors
            /// Returns an error if any underlying C stream clone fails.
            pub fn try_clone(&self) -> ::core::result::Result<Self, #runtime::Error> {
                Ok(Self { #(#field_idents: self.#field_idents.try_clone()?),* })
            }

            #(#accessors)*
        }
    }
}

fn emit_member_open_and_fill(m: &StreamMember, runtime: &syn::Path) -> TokenStream {
    let f = &m.ident;
    let st = &m.stream_ty;
    let fn_lb = &m.fn_lookback_ident;
    let ins = &m.inputs;
    let opts = &m.opts;

    if m.out_kinds.len() == 1 {
        let def = default_expr(m.out_kinds[0]);
        quote! {
            let __lb = #runtime::#fn_lb(#(#opts),*)?;
            if __len <= __lb { return Err(#runtime::Error::InsufficientHistory); }
            __batch.#f.resize(__len - __lb, #def);
            let (#f, __r) = #runtime::#st::open_and_fill(#(#ins,)* #(#opts,)* &mut __batch.#f)?;
            __batch.#f.truncate(__r.count);
            if __r.begin > __max_begin { __max_begin = __r.begin; }
        }
    } else if let Some(target_idx) = m.out_index {
        let def = default_expr(m.out_kinds[target_idx]);
        let (scratch_decls, args) = scratch_buffers_for_target(
            m,
            target_idx,
            quote!(&mut __batch.#f),
            quote!(__len - __lb),
        );
        quote! {
            let __lb = #runtime::#fn_lb(#(#opts),*)?;
            if __len <= __lb { return Err(#runtime::Error::InsufficientHistory); }
            __batch.#f.resize(__len - __lb, #def);
            #(#scratch_decls)*
            let (#f, __r) = #runtime::#st::open_and_fill(#(#ins,)* #(#opts,)* #(#args),*)?;
            __batch.#f.truncate(__r.count);
            if __r.begin > __max_begin { __max_begin = __r.begin; }
        }
    } else {
        let (buf_ids, buf_decls) = tuple_buffers(m, quote!(__len - __lb));
        quote! {
            let __lb = #runtime::#fn_lb(#(#opts),*)?;
            if __len <= __lb { return Err(#runtime::Error::InsufficientHistory); }
            #(#buf_decls)*
            let (#f, __r) = #runtime::#st::open_and_fill(#(#ins,)* #(#opts,)* #(&mut #buf_ids),*)?;
            __batch.#f.reserve(__r.count);
            for __idx in 0..__r.count { __batch.#f.push((#(#buf_ids[__idx]),*)); }
            if __r.begin > __max_begin { __max_begin = __r.begin; }
        }
    }
}
