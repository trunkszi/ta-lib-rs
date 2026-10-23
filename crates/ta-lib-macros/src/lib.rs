//! Procedural macros for generating safe, ergonomic TA-Lib batch and streaming wrappers.

mod app_macros;
mod derive_stream;
mod emit_feature_set;
mod expand;
mod model;
mod registry;

use proc_macro::TokenStream;

/// Generate safe batch (`f64` and `f32`), zero-allocation (`_into`), `lookback`,
/// `display_shift`, and RAII `Stream` bindings for one or more TA-Lib indicators.
///
/// If `=> C_IDENT, StreamIdent` is omitted after the indicator function name, the C symbol
/// (`UPPER_SNAKE_CASE`) and stream struct (`<PascalCase>Stream`) are inferred automatically.
///
/// # Errors
///
/// Emits a compile error if the indicator declaration syntax is invalid.
#[proc_macro]
pub fn ta_indicators(input: TokenStream) -> TokenStream {
    let list = syn::parse_macro_input!(input as model::IndicatorList);
    expand::expand_all(&list)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Invoke a TA-Lib indicator or stream constructor with semicolon-separated inputs and options,
/// automatically wrapping option expressions in `Some(...)` and mapping `_` to `None`.
///
/// Prefixing with `padded:` (e.g., `ta_call!(padded: ta_lib::rsi(&close; 14))`) allocates a
/// full-length vector equal to the input slice length, fills the warmup prefix `0..lookback`
/// with `f64::NAN`, and computes the indicator in-place into `&mut out[lookback..]`.
#[proc_macro]
pub fn ta_call(input: TokenStream) -> TokenStream {
    let call = syn::parse_macro_input!(input as app_macros::TaCallInput);
    app_macros::expand_ta_call(&call)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Execute multiple batch indicators and align every output vector to the shared post-warmup
/// timeline (`max_begin..`), preventing lookahead bias and index misalignment.
///
/// Returns `Result<TaAlignedBatch, ta_lib::Error>` containing `.begin`, `.count`, and all
/// named output vectors truncated/sliced to the exact same bar index range `0..count`.
#[proc_macro]
pub fn ta_batch_aligned(input: TokenStream) -> TokenStream {
    let batch = syn::parse_macro_input!(input as app_macros::BatchAlignedInput);
    app_macros::expand_batch_aligned(&batch)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Invoke any TA-Lib indicator dynamically by string name through the reflective Abstract API
/// (`FuncHandle` + `ParamHolder`) in a single declarative expression.
#[proc_macro]
pub fn ta_dyn_call(input: TokenStream) -> TokenStream {
    let dyn_call = syn::parse_macro_input!(input as app_macros::DynCallInput);
    app_macros::expand_dyn_call(&dyn_call)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive a unified multi-indicator columnar batch (`<Name>Batch`) and native C-stream
/// incremental engine (`<Name>Stream`) from a factor/signal snapshot struct.
///
/// Automatically infers canonical OHLCV inputs (`open`, `high`, `low`, `close`, `volume`),
/// supports shorthand attributes (`#[ta(ema(9))]`, `#[ta(atr(14))]`, `#[ta(macd(12, 26, 9), out =
/// 2)]`), and honors `#[serde(rename / rename_all)]` alongside `#[ta(rename / rename_all)]`.
#[proc_macro_derive(FeatureSet, attributes(ta, serde))]
pub fn derive_feature_set(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    derive_stream::StreamModel::parse(&input)
        .map(|model| emit_feature_set::expand(&model))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Alias for [`FeatureSet`]: derive a unified multi-indicator `<Name>Batch` and `<Name>Stream`.
#[proc_macro_derive(TaStream, attributes(ta, serde))]
pub fn derive_ta_stream(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    derive_stream::StreamModel::parse(&input)
        .map(|model| emit_feature_set::expand(&model))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
