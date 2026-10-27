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

fn scalar_target(m: &StreamMember) -> Option<usize> {
    (m.out_kinds.len() == 1).then_some(0).or(m.out_index)
}

fn emit_align_columns(
    target: TokenStream,
    field_idents: &[&Ident],
    begin_vars: &[Ident],
) -> TokenStream {
    quote! {
        let mut __aligned_count = __len.saturating_sub(__max_begin);
        #(
            let __skip = __max_begin.saturating_sub(#begin_vars);
            if __skip > 0 {
                let __n = __skip.min(#target.#field_idents.len());
                drop(#target.#field_idents.drain(0..__n));
            }
            if #target.#field_idents.len() < __aligned_count {
                __aligned_count = #target.#field_idents.len();
            }
        )*
        #(#target.#field_idents.truncate(__aligned_count);)*
        #target.begin = __max_begin;
        #target.count = __aligned_count;
    }
}

fn emit_input_len_check(inputs: &[Ident], runtime: &syn::Path) -> TokenStream {
    let first_in = &inputs[0];
    let rest_ins = &inputs[1..];
    quote! {
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
    let input_len_check = emit_input_len_check(inputs, runtime);

    let columns: Vec<&str> = model.members.iter().map(|m| m.column.as_str()).collect();
    let field_idents: Vec<&Ident> = model.members.iter().map(|m| &m.ident).collect();
    let field_tys: Vec<&syn::Type> = model.members.iter().map(|m| &m.ty).collect();

    let lookback_vars: Vec<Ident> = model
        .members
        .iter()
        .map(|m| format_ident!("__lb_{}", m.ident))
        .collect();
    let begin_vars: Vec<Ident> = model
        .members
        .iter()
        .map(|m| format_ident!("__beg_{}", m.ident))
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
        .zip(&begin_vars)
        .map(|((m, lb_id), beg_id)| emit_member_aligned_into(m, lb_id, beg_id, runtime))
        .collect();

    let padded_member_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .zip(&lookback_vars)
        .map(|(m, lb_id)| emit_member_padded_into(m, lb_id, runtime))
        .collect();

    let serde_crate_path = format!("{}::serde", quote!(#runtime).to_string().replace(' ', ""));
    let align_self = emit_align_columns(quote!(self), &field_idents, &begin_vars);

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
                #input_len_check
                let mut __max_begin: usize = 0;
                #(#lookback_stmts)*
                #(#aligned_member_stmts)*
                #align_self
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
                #input_len_check
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

fn emit_member_aligned_into(
    m: &StreamMember,
    lb_id: &Ident,
    beg_id: &Ident,
    runtime: &syn::Path,
) -> TokenStream {
    let f = &m.ident;
    let fn_into = &m.fn_into_ident;
    let ins = &m.inputs;
    let opts = &m.opts;
    if let Some(target_idx) = scalar_target(m) {
        let target_def = default_expr(m.out_kinds[target_idx]);
        let (scratch_decls, args) =
            scratch_buffers_for_target(m, target_idx, quote!(&mut self.#f), quote!(__req));
        quote! {
            self.#f.clear();
            let __req = __len.saturating_sub(#lb_id);
            let #beg_id = if __req > 0 {
                self.#f.resize(__req, #target_def);
                #(#scratch_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(#args),*)?;
                self.#f.truncate(__r.count);
                if __r.begin > __max_begin { __max_begin = __r.begin; }
                __r.begin
            } else {
                #lb_id
            };
        }
    } else {
        let (buf_ids, buf_decls) = tuple_buffers(m, quote!(__req));
        quote! {
            self.#f.clear();
            let __req = __len.saturating_sub(#lb_id);
            let #beg_id = if __req > 0 {
                #(#buf_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(&mut #buf_ids),*)?;
                self.#f.reserve(__r.count);
                for __idx in 0..__r.count {
                    self.#f.push((#(#buf_ids[__idx]),*));
                }
                if __r.begin > __max_begin { __max_begin = __r.begin; }
                __r.begin
            } else {
                #lb_id
            };
        }
    }
}

fn emit_member_padded_into(m: &StreamMember, lb_id: &Ident, runtime: &syn::Path) -> TokenStream {
    let f = &m.ident;
    let fn_into = &m.fn_into_ident;
    let ins = &m.inputs;
    let opts = &m.opts;
    if let Some(target_idx) = scalar_target(m) {
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
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(#args),*)?;
                if __r.count == 0 {
                    self.#f.fill(#pad);
                } else {
                    if __r.begin != #lb_id {
                        self.#f.copy_within(#lb_id..(#lb_id + __r.count), __r.begin);
                        self.#f[..__r.begin].fill(#pad);
                    }
                    let __end = __r.begin + __r.count;
                    if __end < __len {
                        self.#f[__end..].fill(#pad);
                    }
                }
                if __r.begin > __max_begin { __max_begin = __r.begin; }
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
                #(#buf_decls)*
                let __r = #runtime::#fn_into(#(#ins,)* #(#opts,)* #(&mut #buf_ids),*)?;
                if __r.count == 0 {
                    self.#f.resize(__len, #pad_tuple);
                } else {
                    self.#f.resize(__r.begin, #pad_tuple);
                    self.#f.reserve(__len.saturating_sub(__r.begin));
                    for __idx in 0..__r.count {
                        self.#f.push((#(#buf_ids[__idx]),*));
                    }
                    if self.#f.len() < __len {
                        self.#f.resize(__len, #pad_tuple);
                    }
                }
                if __r.begin > __max_begin { __max_begin = __r.begin; }
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
    let input_len_check = emit_input_len_check(inputs, runtime);

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

    let begin_vars: Vec<Ident> = model
        .members
        .iter()
        .map(|m| format_ident!("__beg_{}", m.ident))
        .collect();
    let open_and_fill_stmts: Vec<TokenStream> = model
        .members
        .iter()
        .zip(&begin_vars)
        .map(|(m, beg_id)| emit_member_open_and_fill(m, beg_id, runtime))
        .collect();

    let first_field = &model.members[0].ident;
    let rest_fields: Vec<&Ident> = model.members[1..].iter().map(|m| &m.ident).collect();
    let align_batch = emit_align_columns(quote!(__batch), &field_idents, &begin_vars);

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
                #input_len_check
                let mut __batch = #batch_ident::with_capacity(__len);
                let mut __max_begin: usize = 0;
                #(#open_and_fill_stmts)*
                #align_batch
                Ok((Self { #(#field_idents),* }, __batch))
            }

            /// Commit one closed bar across all member streams and return the new snapshot.
            /// # Errors
            /// Returns an error if any underlying C stream update fails.
            #[allow(clippy::too_many_arguments)]
            #[inline]
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
            #[inline]
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
            #[inline]
            pub fn peek(
                &self,
                #(#inputs: f64),*
            ) -> ::core::result::Result<#ident, #runtime::Error> {
                Ok(#ident { #(#peek_calls),* })
            }

            /// Read the most recently committed snapshot without advancing any member stream.
            /// # Errors
            /// Returns an error if any underlying C stream value query fails.
            #[inline]
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

fn emit_member_open_and_fill(m: &StreamMember, beg_id: &Ident, runtime: &syn::Path) -> TokenStream {
    let f = &m.ident;
    let st = &m.stream_ty;
    let fn_lb = &m.fn_lookback_ident;
    let ins = &m.inputs;
    let opts = &m.opts;
    if let Some(target_idx) = scalar_target(m) {
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
            let #beg_id = __r.begin;
            if #beg_id > __max_begin { __max_begin = #beg_id; }
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
            let #beg_id = __r.begin;
            if #beg_id > __max_begin { __max_begin = #beg_id; }
        }
    }
}
