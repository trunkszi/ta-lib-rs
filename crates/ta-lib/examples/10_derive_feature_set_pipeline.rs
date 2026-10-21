//! Example 10: Unified batch & streaming factor panel via `#[derive(FeatureSet)]`.
//!
//! Demonstrates:
//! 1. Shorthand indicator attributes (`#[ta(ema(5))]`, `#[ta(atr(5))]`, `#[ta(macd(6, 13, 4), out =
//!    2)]`) with automatic canonical OHLCV input inference (`high, low, close`) and Serde
//!    compatibility.
//! 2. `AlphaFactor::compute_aligned` + iterating over rows (`for factor in &batch`).
//! 3. `AlphaFactorBatch::with_capacity` + `compute_padded_into` for zero-allocation buffer reuse
//!    and zero-copy in-place `f64::NAN` warmup padding.
//! 4. `AlphaFactorStream::open_and_fill` single-pass historical batch + live C state machine
//!    warmup.
//! 5. Real-time `peek` (intrabar zero-side-effect preview), `next` (`O(1)` bar commit), and
//!    `try_clone`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ta_lib::{FeatureSet, MAType, initialize};

/// Unified multi-indicator factor snapshot.
///
/// `#[derive(FeatureSet)]` automatically infers the canonical `(high, low, close)` input signature
/// across all fields and generates:
/// - `AlphaFactorBatch` (Structure-of-Arrays columnar batch with `compute_aligned_into` &
///   `compute_padded_into`)
/// - `AlphaFactorStream` (RAII native C-stream pipeline with `open`, `open_and_fill`, `peek`,
///   `next`, `try_clone`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FeatureSet)]
#[serde(rename_all = "snake_case")]
pub struct AlphaFactor {
    #[ta(ema(5))]
    pub fast_ema: f64,

    #[ta(ema(10))]
    pub slow_ema: f64,

    #[ta(rsi(6))]
    pub rsi_6: f64,

    #[ta(atr(5))]
    pub atr_5: f64,

    #[ta(macd(6, 13, 4), out = 2)]
    pub macd_hist: f64,

    #[ta(bbands(5, 2.0, 2.0, MAType::Sma))]
    pub bbands: (f64, f64, f64),
}

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let high: Vec<f64> = (0..40).map(|i| 101.5 + (i as f64) * 0.5).collect();
    let low: Vec<f64> = (0..40).map(|i| 99.0 + (i as f64) * 0.45).collect();
    let close: Vec<f64> = (0..40)
        .map(|i| 100.2 + (i as f64) * 0.48 + ((i % 4) as f64 - 1.5) * 0.3)
        .collect();

    println!("Field names: {:?}", AlphaFactor::FIELD_NAMES);

    // 1. Aligned SoA Batch (trimmed to `max_begin`, zero NaNs, direct row iteration)
    let aligned_batch =
        AlphaFactor::compute_aligned(&high, &low, &close).context("compute_aligned")?;
    let last_from_iter = (&aligned_batch).into_iter().next_back().unwrap();
    println!(
        "[1. Aligned Batch] begin={}, count={}, latest fast_ema={:.4}, macd_hist={:.4}",
        aligned_batch.begin,
        aligned_batch.len(),
        last_from_iter.fast_ema,
        last_from_iter.macd_hist
    );

    // 2. Zero-allocation reusable SoA Batch + in-place `NaN` warmup padding (`len == close.len()`)
    let mut reusable_batch = AlphaFactorBatch::with_capacity(close.len());
    let pad_range = reusable_batch
        .compute_padded_into(&high, &low, &close)
        .context("compute_padded_into")?;
    assert_eq!(reusable_batch.len(), close.len());
    assert!(reusable_batch.fast_ema[0].is_nan());
    assert_eq!(reusable_batch.row(close.len() - 1).unwrap(), last_from_iter);
    println!(
        "[2. Padded Batch]  total_rows={}, warmup_begin={}, row[0].fast_ema={}",
        reusable_batch.len(),
        pad_range.begin,
        reusable_batch.fast_ema[0]
    );

    // 3. Single-pass `open_and_fill`: warms up live C streams AND fills aligned historical batch
    let (mut stream, filled_batch) =
        AlphaFactorStream::open_and_fill(&high, &low, &close).context("open_and_fill")?;
    assert_eq!(filled_batch, aligned_batch);
    assert_eq!(stream.value()?, last_from_iter);
    println!(
        "[3. open_and_fill] single-pass batch rows={} matches compute_aligned exactly",
        filled_batch.len()
    );

    // 4. Live intrabar `peek`, counterfactual `try_clone`, and bar-close `next` (`update`)
    let preview = stream.peek(122.5, 119.8, 121.9).context("stream.peek")?;
    let mut shadow_stream = stream.try_clone().context("stream.try_clone")?;
    let committed = stream.next(122.5, 119.8, 121.9).context("stream.next")?;
    let shadow_committed = shadow_stream
        .update(122.5, 119.8, 121.9)
        .context("shadow_stream.update")?;
    assert_eq!(preview, committed);
    assert_eq!(committed, shadow_committed);

    let json = serde_json::to_string(&committed).context("serialize AlphaFactor to JSON")?;
    println!("[4. Live Stream]   committed JSON = {json}");

    Ok(())
}
