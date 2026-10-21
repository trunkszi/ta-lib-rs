//! Integration and edge-case tests for `ta_call!`, `ta_batch_aligned!`, `ta_dyn_call!`,
//! `#[derive(FeatureSet)]`, and `#[derive(TaStream)]`.

#[cfg(feature = "pre-release")]
use serde::{Deserialize, Serialize};
use ta_lib::{Error, MAType, initialize, ta_batch_aligned, ta_call, ta_dyn_call};
#[cfg(feature = "pre-release")]
use ta_lib::{FeatureSet, TaStream};

#[cfg(feature = "pre-release")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FeatureSet)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub struct StrategyFactors {
    #[ta(ema(5))]
    pub fast_ema: f64,
    #[ta(sma(8))]
    pub slow_sma: f64,
    #[ta(atr(5))]
    pub atr_5: f64,
    #[ta(bbands(5, 2.0, 2.0, MAType::Sma))]
    pub bands: (f64, f64, f64),
    #[ta(macd(4, 8, 3), out = 2)]
    pub macd_hist: f64,
    #[ta(cdldoji)]
    pub doji: i32,
}

#[cfg(feature = "pre-release")]
#[derive(Debug, Clone, PartialEq, TaStream)]
#[ta(stream = "CustomRsiEngine", batch = "CustomRsiColumns")]
pub struct CustomRsiView {
    #[ta(rsi(6), rename = "rsi_fast")]
    pub rsi_6: f64,
}

fn sample_ohlc(len: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut open = Vec::with_capacity(len);
    let mut high = Vec::with_capacity(len);
    let mut low = Vec::with_capacity(len);
    let mut close = Vec::with_capacity(len);
    for i in 0..len {
        let base = (i as f64).mul_add(0.45, 100.0) + ((i % 5) as f64 - 2.0) * 0.35;
        let o = if i % 7 == 0 { base } else { base - 0.15 };
        open.push(o);
        high.push(base + 0.90);
        low.push(base - 0.85);
        close.push(base);
    }
    (open, high, low, close)
}

#[test]
fn ta_call_standard_and_padded_modes_preserve_invariants() {
    initialize().unwrap();
    let (_, high, low, close) = sample_ohlc(32);

    let (ema_vals, ema_begin) = ta_call!(ta_lib::ema(&close; 5)).unwrap();
    let padded_ema = ta_call!(padded: ta_lib::ema(&close; 5)).unwrap();
    assert_eq!(padded_ema.len(), close.len());
    assert!(padded_ema[..ema_begin].iter().all(|v| v.is_nan()));
    assert_eq!(
        &padded_ema[ema_begin..ema_begin + ema_vals.len()],
        ema_vals.as_slice()
    );

    // Padded call on history shorter than lookback fills entire output with NaN without failing.
    let short = &close[..3];
    let short_padded = ta_call!(padded: ta_lib::ema(short; 10)).unwrap();
    assert_eq!(short_padded.len(), short.len());
    assert!(short_padded.iter().all(|v| v.is_nan()));

    // Multi-output tuple padded call.
    let (upper_p, mid_p, lower_p) =
        ta_call!(padded: ta_lib::bbands(&close; 5, 2.0, 2.0, _)).unwrap();
    assert_eq!(upper_p.len(), close.len());
    assert_eq!(mid_p.len(), close.len());
    assert_eq!(lower_p.len(), close.len());
    assert!(mid_p[..4].iter().all(|v| v.is_nan()));

    // In-place `; &mut out` syntax on `atr_into`.
    let mut out_atr = vec![0.0; close.len()];
    let atr_range = ta_call!(ta_lib::atr_into(&high, &low, &close; 5; &mut out_atr)).unwrap();
    assert_eq!(atr_range.begin, 5);
    assert_eq!(atr_range.count, close.len() - 5);
}

#[test]
fn ta_batch_aligned_trims_to_max_begin_and_serializes() {
    initialize().unwrap();
    let (_, high, low, close) = sample_ohlc(40);

    let batch = ta_batch_aligned! {
        ema_5: ta_lib::ema(&close; 5),
        sma_10: ta_lib::sma(&close; 10),
        atr_6: ta_lib::atr(&high, &low, &close; 6),
        (bb_up, bb_mid, bb_low): ta_lib::bbands(&close; 5, _, _, MAType::Sma),
    }
    .unwrap();

    assert_eq!(batch.begin, 9);
    assert_eq!(batch.count, close.len() - 9);
    assert_eq!(batch.ema_5.len(), batch.count);
    assert_eq!(batch.sma_10.len(), batch.count);
    assert_eq!(batch.atr_6.len(), batch.count);
    assert_eq!(batch.bb_up.len(), batch.count);
    assert_eq!(batch.bb_mid.len(), batch.count);
    assert_eq!(batch.bb_low.len(), batch.count);

    let json = serde_json::to_string(&batch).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["begin"], batch.begin);
    assert_eq!(parsed["count"], batch.count);
}

#[cfg(feature = "pre-release")]
#[test]
fn feature_set_batch_and_stream_stay_consistent_across_modes() {
    initialize().unwrap();
    let (open, high, low, close) = sample_ohlc(48);

    assert_eq!(
        StrategyFactors::FIELD_NAMES,
        &[
            "FAST-EMA",
            "SLOW-SMA",
            "ATR-5",
            "BANDS",
            "MACD-HIST",
            "DOJI"
        ]
    );
    assert_eq!(CustomRsiView::FIELD_NAMES, &["rsi_fast"]);

    let aligned = StrategyFactors::compute_aligned(&open, &high, &low, &close).unwrap();
    let padded = StrategyFactors::compute_padded(&open, &high, &low, &close).unwrap();
    assert_eq!(padded.len(), close.len());
    assert_eq!(aligned.begin + aligned.count, close.len());

    // Every aligned row must match the corresponding tail row of `padded`.
    for i in 0..aligned.len() {
        let row_a = aligned.row(i).unwrap();
        let row_p = padded.row(aligned.begin + i).unwrap();
        assert!((row_a.fast_ema - row_p.fast_ema).abs() < 1e-12);
        assert!((row_a.slow_sma - row_p.slow_sma).abs() < 1e-12);
        assert!((row_a.atr_5 - row_p.atr_5).abs() < 1e-12);
        assert!((row_a.macd_hist - row_p.macd_hist).abs() < 1e-12);
        assert_eq!(row_a.doji, row_p.doji);
    }

    // `compute_padded` on insufficient history still guarantees `row(index)` works without panic.
    let short_padded =
        StrategyFactors::compute_padded(&open[..4], &high[..4], &low[..4], &close[..4]).unwrap();
    assert_eq!(short_padded.len(), 4);
    for idx in 0..short_padded.len() {
        let r = short_padded.row(idx).unwrap();
        assert!(r.fast_ema.is_nan());
        assert!(r.bands.0.is_nan());
    }

    let split = 36;
    let (mut stream, filled) = StrategyFactors::open_and_fill(
        &open[..split],
        &high[..split],
        &low[..split],
        &close[..split],
    )
    .unwrap();
    let batch_head = StrategyFactors::compute_aligned(
        &open[..split],
        &high[..split],
        &low[..split],
        &close[..split],
    )
    .unwrap();
    assert_eq!(filled.begin, batch_head.begin);
    assert_eq!(filled.count, batch_head.count);
    assert_eq!(filled.iter().next_back(), Some(stream.value().unwrap()));

    // Verify `peek` does not mutate state and `try_clone` produces identical updates.
    let mut cloned = stream.try_clone().unwrap();
    let peeked = stream
        .peek(open[split], high[split], low[split], close[split])
        .unwrap();
    let committed = stream
        .next(open[split], high[split], low[split], close[split])
        .unwrap();
    let clone_committed = cloned
        .update(open[split], high[split], low[split], close[split])
        .unwrap();
    assert!((peeked.fast_ema - committed.fast_ema).abs() < 1e-12);
    assert_eq!(committed, clone_committed);
}

#[test]
fn ta_dyn_call_handles_valid_and_empty_ranges() {
    initialize().unwrap();
    let (open, high, low, close) = sample_ohlc(30);
    let mut out = vec![0.0; close.len()];

    let range = ta_dyn_call! {
        "ATR",
        price_inputs: [(Some(&open[..]), Some(&high[..]), Some(&low[..]), Some(&close[..]), None, None)],
        int_opts: [5],
        real_outputs: [&mut out],
    }
    .unwrap();
    assert_eq!(range.begin, 5);
    assert_eq!(range.count, close.len() - 5);

    // Empty exclusive range `0..0` must fail with `Error::BadParam` instead of underflowing to
    // 0..=0.
    let err = ta_dyn_call! {
        "SMA",
        range: 0..0,
        real_inputs: [&close],
        int_opts: [5],
        real_outputs: [&mut out],
    }
    .unwrap_err();
    assert_eq!(err, Error::BadParam);
}
