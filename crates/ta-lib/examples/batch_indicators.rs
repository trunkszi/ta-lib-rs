//! Batch calculation example (`f64`, `f32`, caller-owned `_into` buffers, and lookback metadata).

use anyhow::{Context, Result};
use ta_lib::{
    MAType, bbands, bbands_into, ema, initialize, macd, rsi, rsi_f32, rsi_into, rsi_lookback, sma,
};

fn main() -> Result<()> {
    initialize().context("failed to initialize TA-Lib runtime")?;

    let closes = [
        100.0, 101.5, 102.0, 100.8, 103.2, 104.5, 103.9, 105.1, 106.4, 105.8, 107.2, 108.0, 107.4,
        109.1, 110.3, 109.6, 111.0, 112.4, 111.8, 113.5, 114.2, 113.7, 115.1, 116.0, 115.4, 117.2,
        118.5, 117.9, 119.3, 120.1, 119.5, 121.0, 122.3, 121.7, 123.4, 124.0,
    ];

    // 1. Standard allocating batch indicators (f64)
    let (sma_vals, sma_begin) = sma(&closes, Some(5)).context("compute SMA(5)")?;
    let (ema_vals, ema_begin) = ema(&closes, Some(5)).context("compute EMA(5)")?;
    let (rsi_vals, rsi_begin) = rsi(&closes, Some(6)).context("compute RSI(6)")?;
    println!(
        "SMA(5): begin={sma_begin}, count={}, latest={:.4}",
        sma_vals.len(),
        sma_vals.last().copied().unwrap_or_default()
    );
    println!(
        "EMA(5): begin={ema_begin}, count={}, latest={:.4}",
        ema_vals.len(),
        ema_vals.last().copied().unwrap_or_default()
    );
    println!(
        "RSI(6): begin={rsi_begin}, count={}, latest={:.4}",
        rsi_vals.len(),
        rsi_vals.last().copied().unwrap_or_default()
    );

    // 2. Single-precision (f32) input batch calculation
    let closes_f32: Vec<f32> = closes.iter().map(|&v| v as f32).collect();
    let (rsi_f32_vals, rsi_f32_begin) = rsi_f32(&closes_f32, Some(6)).context("compute RSI f32")?;
    assert_eq!(rsi_f32_begin, rsi_begin);
    assert_eq!(rsi_f32_vals.len(), rsi_vals.len());

    // 3. Multi-output batch indicators: MACD and Bollinger Bands
    let (macd_line, signal_line, hist_line, macd_begin) =
        macd(&closes, Some(12), Some(26), Some(9)).context("compute MACD(12, 26, 9)")?;
    println!(
        "MACD(12,26,9): begin={macd_begin}, count={}, latest=({:.4}, {:.4}, {:.4})",
        macd_line.len(),
        macd_line.last().copied().unwrap_or_default(),
        signal_line.last().copied().unwrap_or_default(),
        hist_line.last().copied().unwrap_or_default()
    );

    let (upper, middle, lower, bbands_begin) =
        bbands(&closes, Some(5), Some(2.0), Some(2.0), Some(MAType::Sma))
            .context("compute BBANDS(5, 2.0, 2.0)")?;
    println!(
        "BBANDS(5): begin={bbands_begin}, latest=({:.4}, {:.4}, {:.4})",
        upper.last().copied().unwrap_or_default(),
        middle.last().copied().unwrap_or_default(),
        lower.last().copied().unwrap_or_default()
    );

    // 4. Caller-owned output buffers (`_into`) for zero output-vector allocation
    let lookback = rsi_lookback(Some(6)).context("query RSI(6) lookback")?;
    let mut rsi_buf = vec![0.0_f64; closes.len() - lookback];
    let range = rsi_into(&closes, Some(6), &mut rsi_buf).context("compute RSI(6) into buffer")?;
    assert_eq!(range.begin, lookback);
    assert_eq!(&rsi_buf[..range.count], rsi_vals.as_slice());

    let mut upper_buf = vec![0.0_f64; closes.len()];
    let mut middle_buf = vec![0.0_f64; closes.len()];
    let mut lower_buf = vec![0.0_f64; closes.len()];
    let bb_range = bbands_into(
        &closes,
        Some(5),
        Some(2.0),
        Some(2.0),
        Some(MAType::Sma),
        &mut upper_buf,
        &mut middle_buf,
        &mut lower_buf,
    )
    .context("compute BBANDS into caller buffers")?;
    assert_eq!(&upper_buf[..bb_range.count], upper.as_slice());

    Ok(())
}
