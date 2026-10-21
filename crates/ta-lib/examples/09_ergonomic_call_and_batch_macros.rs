//! Example 09: Application-level ergonomic macros (`ta_call!`, `ta_batch_aligned!`,
//! `ta_dyn_call!`).
//!
//! Demonstrates:
//! 1. `ta_call!`: Eliminating `Some(...)` / `None` boilerplate with `;` and `_` placeholders.
//! 2. `ta_call!(padded: ...)`: Zero-copy in-place `f64::NAN` warmup padding to match input length.
//! 3. `ta_batch_aligned!`: Ad-hoc multi-indicator batch execution aligned to `max(begin)` + Serde
//!    JSON.
//! 4. `ta_dyn_call!`: Single-expression reflective Abstract API calls by indicator string name.

use anyhow::{Context, Result};
use ta_lib::{MAType, initialize, ta_batch_aligned, ta_call, ta_dyn_call};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let close: Vec<f64> = (0..45)
        .map(|i| 100.0 + (i as f64) * 0.45 + ((i % 5) as f64 - 2.0) * 0.6)
        .collect();

    // 1. `ta_call!`: semicolon separates inputs from options; `_` maps to `None` (TA-Lib default)
    let (rsi_vals, rsi_begin) = ta_call!(ta_lib::rsi(&close; 6)).context("ta_call! rsi(6)")?;
    let (up, mid, low, bb_begin) =
        ta_call!(ta_lib::bbands(&close; 5, _, _, MAType::Ema)).context("ta_call! bbands")?;
    println!(
        "[1. ta_call!] rsi(6): begin={rsi_begin}, len={}, latest={:.4}",
        rsi_vals.len(),
        rsi_vals.last().copied().unwrap_or_default()
    );
    println!(
        "[1. ta_call!] bbands(5, _, _, Ema): begin={bb_begin}, latest=({:.4}, {:.4}, {:.4})",
        up.last().copied().unwrap_or_default(),
        mid.last().copied().unwrap_or_default(),
        low.last().copied().unwrap_or_default()
    );

    // 2. `ta_call!(padded: ...)`: single-allocation in-place `NaN` prefix padding (len ==
    //    close.len())
    let rsi_padded = ta_call!(padded: ta_lib::rsi(&close; 6)).context("padded rsi(6)")?;
    assert_eq!(rsi_padded.len(), close.len());
    assert!(rsi_padded[..rsi_begin].iter().all(|v| v.is_nan()));
    assert!(
        (rsi_padded.last().copied().unwrap() - rsi_vals.last().copied().unwrap()).abs() < 1e-12
    );
    println!(
        "[2. ta_call!(padded:)] rsi_padded.len() == close.len() == {}, warmup[0..{rsi_begin}] = \
         NaN",
        rsi_padded.len()
    );

    // 3. `ta_batch_aligned!`: aligns multiple indicators with different lookbacks to `max_begin`
    let aligned = ta_batch_aligned! {
        fast_ema: ta_lib::ema(&close; 5),
        slow_ema: ta_lib::ema(&close; 12),
        rsi_6: ta_lib::rsi(&close; 6),
        (macd, macd_signal, macd_hist): ta_lib::macd(&close; 6, 13, 5),
        (bb_upper, bb_mid, bb_lower): ta_lib::bbands(&close; 10, _, _, MAType::Sma),
    }
    .context("ta_batch_aligned!")?;

    assert_eq!(aligned.fast_ema.len(), aligned.count);
    assert_eq!(aligned.slow_ema.len(), aligned.count);
    assert_eq!(aligned.rsi_6.len(), aligned.count);
    assert_eq!(aligned.macd_hist.len(), aligned.count);
    assert_eq!(aligned.bb_upper.len(), aligned.count);
    assert_eq!(aligned.begin + aligned.count, close.len());

    let json = serde_json::to_string(&aligned).context("serialize TaAlignedBatch to JSON")?;
    println!(
        "[3. ta_batch_aligned!] shared begin={}, aligned count={}, JSON bytes={}",
        aligned.begin,
        aligned.count,
        json.len()
    );

    // 4. `ta_dyn_call!`: declarative reflective Abstract API call by indicator name
    let mut dyn_macd = vec![0.0_f64; close.len()];
    let mut dyn_sig = vec![0.0_f64; close.len()];
    let mut dyn_hist = vec![0.0_f64; close.len()];
    let dyn_range = ta_dyn_call! {
        "MACD",
        real_inputs: [&close],
        int_opts: [6, 13, 5],
        real_outputs: [&mut dyn_macd, &mut dyn_sig, &mut dyn_hist],
    }
    .context("ta_dyn_call! MACD")?;

    assert_eq!(dyn_range.begin, aligned.begin);
    assert!((dyn_hist[dyn_range.count - 1] - aligned.macd_hist[aligned.count - 1]).abs() < 1e-12);
    println!(
        "[4. ta_dyn_call!] reflective MACD(6,13,5): begin={}, count={}, latest_hist={:.4}",
        dyn_range.begin,
        dyn_range.count,
        dyn_hist[dyn_range.count - 1]
    );

    Ok(())
}
