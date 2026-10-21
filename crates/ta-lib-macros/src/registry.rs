//! Canonical input and output signature registry for all TA-Lib indicators.

use crate::model::OutKind;

/// Canonical OHLCV priority order when inferring container input parameters.
const OHLCV_ORDER: &[&str] = &["open", "high", "low", "close", "volume"];

/// Return the default input slice names for a TA-Lib indicator function name (`snake_case`).
pub fn default_inputs(indicator: &str) -> &'static [&'static str] {
    if indicator.starts_with("cdl") {
        return &["open", "high", "low", "close"];
    }
    match indicator {
        "bop" | "avgprice" | "asi" | "si" | "ha" | "rogerssatchell" => {
            &["open", "high", "low", "close"]
        }
        "imi" | "qstick" => &["open", "close"],
        "ac" | "ao" | "aroon" | "aroonosc" | "fractal" | "minus_dm" | "plus_dm" | "fisher"
        | "donchian" | "midprice" | "sar" | "sarext" | "frama" | "ichimoku" | "zigzag"
        | "medprice" | "adr" | "cvi" | "massi" | "rvir" => &["high", "low"],
        "adx" | "adxr" | "cci" | "dx" | "eri" | "kdj" | "minus_di" | "plus_di" | "smi"
        | "stoch" | "stochf" | "ultosc" | "vortex" | "wad" | "willr" | "chop" | "choptr"
        | "ibs" | "pso" | "sqzmom" | "wavetrend" | "accbands" | "kc" | "supertrend" | "cksp"
        | "typprice" | "wclprice" | "atr" | "natr" | "trange" => &["high", "low", "close"],
        "mfi" | "ad" | "adosc" | "cmf" | "vwap" | "kvo" => &["high", "low", "close", "volume"],
        "marketfi" | "emv" => &["high", "low", "volume"],
        "efi" | "nvi" | "obv" | "pvi" | "pvt" | "vwma" => &["close", "volume"],
        "pvo" | "rvol" => &["volume"],
        "add" | "div" | "mult" | "sub" | "beta" | "correl" => &["inreal0", "inreal1"],
        "mavp" => &["close", "inperiods"],
        _ => &["close"],
    }
}

/// Return the output element kinds (`f64` or `i32`) for a TA-Lib indicator function name.
pub fn output_kinds(indicator: &str) -> &'static [OutKind] {
    if indicator.starts_with("cdl") {
        return &[OutKind::I32];
    }
    match indicator {
        "ht_trendmode" | "maxindex" | "minindex" => &[OutKind::I32],
        "minmaxindex" | "fractal" => &[OutKind::I32, OutKind::I32],
        "sqzmom" | "supertrend" => &[OutKind::F64, OutKind::I32],
        "zigzag" => &[OutKind::F64, OutKind::I32, OutKind::I32],
        "ha" | "ichimoku" => &[OutKind::F64, OutKind::F64, OutKind::F64, OutKind::F64],
        "kdj" | "macd" | "macdext" | "macdfix" | "accbands" | "bbands" | "donchian" | "kc" => {
            &[OutKind::F64, OutKind::F64, OutKind::F64]
        }
        "ht_phasor" | "ht_sine" | "minmax" | "aroon" | "eri" | "smi" | "stoch" | "stochf"
        | "stochrsi" | "vortex" | "fisher" | "kst" | "kstext" | "wavetrend" | "mama" | "cksp"
        | "kvo" => &[OutKind::F64, OutKind::F64],
        _ => &[OutKind::F64],
    }
}

/// Sort inferred input identifiers so that standard OHLCV series appear in canonical
/// `(open, high, low, close, volume)` order followed by any custom series in declaration order.
pub fn sort_canonical_inputs(raw_inputs: Vec<syn::Ident>) -> Vec<syn::Ident> {
    let mut ordered = Vec::with_capacity(raw_inputs.len());
    for &canonical in OHLCV_ORDER {
        if let Some(found) = raw_inputs.iter().find(|id| *id == canonical) {
            ordered.push(found.clone());
        }
    }
    for id in raw_inputs {
        if !OHLCV_ORDER.iter().any(|&c| id == c) {
            ordered.push(id);
        }
    }
    ordered
}
