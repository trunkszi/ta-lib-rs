//! Example 07: Using the `ta_indicators!` procedural macro (`ta-lib-macros`) directly
//! to generate custom domain-named batch, `_f32`, `_into`, `lookback`, and RAII `Stream` bindings.

use anyhow::{Context, Result};
use ta_lib::{MAType, initialize};

/// A user-defined module generated via `ta_lib::ta_indicators!`.
///
/// Each entry automatically expands into:
/// - `<name>` (`f64` allocating batch function)
/// - `<name>_f32` (`f32` allocating batch function)
/// - `<name>_into` and `<name>_f32_into` (zero-allocation caller-slice functions)
/// - `<name>_lookback` (and `<name>_display_shift` on `pre-release`)
/// - `<StreamName>` (RAII streaming state machine with `open`, `open_and_fill`, `peek`, `update`,
///   `try_clone`)
#[allow(clippy::too_many_arguments)]
mod custom_strategy {
    ta_lib::ta_indicators! {
        /// Custom single-input trend moving average binding (`TA_SMA`).
        trend_ma => SMA, TrendMaStream {
            inputs: [close],
            opts: [window: i32],
            outs: [ma: f64],
        };

        /// Custom multi-output volatility envelope binding (`TA_BBANDS`).
        envelope_bands => BBANDS, EnvelopeBandsStream {
            inputs: [close],
            opts: [window: i32, upper_sigma: f64, lower_sigma: f64, smoothing: MAType],
            outs: [upper: f64, mid: f64, lower: f64],
        };

        /// Custom multi-input OHLC volatility stop binding (`TA_ATR`).
        true_range_stop => ATR, TrueRangeStopStream {
            inputs: [high, low, close],
            opts: [window: i32],
            outs: [atr: f64],
        };

        /// Custom integer-output candlestick detector (`TA_CDLDOJI`).
        doji_detector => CDLDOJI, DojiDetectorStream {
            inputs: [open, high, low, close],
            opts: [],
            outs: [signal: i32],
        };
    }
}

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let open = [
        100.0, 101.0, 102.0, 101.5, 102.8, 103.5, 103.0, 104.2, 105.0, 104.6, 105.8, 106.0, 106.01,
    ];
    let high = [
        101.2, 102.3, 103.1, 102.6, 104.0, 104.8, 104.2, 105.5, 106.2, 105.8, 107.0, 107.2, 107.5,
    ];
    let low = [
        99.1, 100.2, 101.0, 100.4, 101.9, 102.6, 102.1, 103.3, 104.0, 103.7, 104.9, 105.1, 104.8,
    ];
    let close = [
        100.8, 101.9, 101.4, 102.5, 103.6, 103.1, 104.0, 104.9, 104.5, 105.6, 106.1, 106.0, 106.02,
    ];

    // 1. Call macro-generated `f64` and `_f32` batch functions
    let (ma_vals, ma_begin) =
        custom_strategy::trend_ma(&close, Some(4)).context("custom trend_ma(4)")?;
    let close_f32: Vec<f32> = close.iter().map(|&v| v as f32).collect();
    let (ma_f32, _) =
        custom_strategy::trend_ma_f32(&close_f32, Some(4)).context("custom trend_ma_f32(4)")?;
    assert_eq!(ma_vals.len(), ma_f32.len());
    println!(
        "[Macro batch] trend_ma(4): begin={ma_begin}, count={}, latest={:.4}",
        ma_vals.len(),
        ma_vals.last().copied().unwrap_or_default()
    );

    // 2. Call macro-generated `_lookback` and zero-allocation `_into`
    let lb = custom_strategy::true_range_stop_lookback(Some(4)).context("ATR lookback")?;
    let mut atr_buf = vec![0.0_f64; close.len() - lb];
    let atr_range =
        custom_strategy::true_range_stop_into(&high, &low, &close, Some(4), &mut atr_buf)
            .context("true_range_stop_into")?;
    println!(
        "[Macro _into] true_range_stop(4): lookback={lb}, begin={}, latest={:.4}",
        atr_range.begin,
        atr_buf[atr_range.count - 1]
    );

    // 3. Call macro-generated multi-output stream (`EnvelopeBandsStream`)
    let (mut env_stream, (u0, m0, l0)) = custom_strategy::EnvelopeBandsStream::open(
        &close,
        Some(4),
        Some(2.0),
        Some(2.0),
        Some(MAType::Ema),
    )
    .context("open custom EnvelopeBandsStream")?;
    let (u_peek, m_peek, l_peek) = env_stream.peek(107.5).context("peek custom stream")?;
    let (u1, m1, l1) = env_stream.update(107.5).context("update custom stream")?;
    assert!((m_peek - m1).abs() < 1e-10);
    println!(
        "[Macro stream] EnvelopeBandsStream: open=({u0:.4}, {m0:.4}, {l0:.4}) -> update=({u1:.4}, \
         {m1:.4}, {l1:.4}) [peek=({u_peek:.4}, {m_peek:.4}, {l_peek:.4})]"
    );

    // 4. Call macro-generated integer-output candlestick detector
    let (doji_signals, doji_begin) =
        custom_strategy::doji_detector(&open, &high, &low, &close).context("doji_detector")?;
    println!(
        "[Macro i32]   doji_detector: begin={doji_begin}, signals={:?}",
        doji_signals
    );

    Ok(())
}
