//! Procedural macros for generating safe, ergonomic TA-Lib batch and streaming wrappers.

mod expand;
mod model;

use proc_macro::TokenStream;

/// Generate safe batch (`f64` and `f32`), zero-allocation (`_into`), `lookback`,
/// and RAII `Stream` bindings for one or more TA-Lib indicators.
///
/// # Errors
///
/// Emits a compile error if the indicator declaration syntax is invalid.
#[proc_macro]
pub fn ta_indicators(input: TokenStream) -> TokenStream {
    let list = syn::parse_macro_input!(input as model::IndicatorList);
    match expand::expand_all(&list) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
