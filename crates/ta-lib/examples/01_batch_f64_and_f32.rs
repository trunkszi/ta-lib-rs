//! Example 01: Batch calculations across `f64` and `f32`, OHLCV multi-input indicators,
//! and multi-output indicator destructuring.

use anyhow::{Context, Result};
use ta_lib::{MAType, adx, atr, bbands, ema, initialize, macd, obv, rsi, rsi_f32, sma, stoch};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let len = 40;
    let open: Vec<f64> = (0..len).map(|i| 100.0 + (i as f64) * 0.5).collect();
    let high: Vec<f64> = open.iter().map(|v| v + 1.2).collect();
    let low: Vec<f64> = open.iter().map(|v| v - 0.9).collect();
    let close: Vec<f64> = (0..len)
        .map(|i| 100.2 + (i as f64) * 0.5 + ((i % 5) as f64 - 2.0) * 0.35)
        .collect();
    let volume: Vec<f64> = (0..len).map(|i| 10_000.0 + (i as f64) * 250.0).collect();

    // 1. Single-series indicators (SMA, EMA, RSI)
    let (sma_vals, sma_begin) = sma(&close, Some(5)).context("compute SMA(5)")?;
    let (ema_vals, ema_begin) = ema(&close, Some(5)).context("compute EMA(5)")?;
    let (rsi_vals, rsi_begin) = rsi(&close, Some(6)).context("compute RSI(6)")?;
    println!(
        "[Single-series] SMA(5) begin={sma_begin} latest={:.4} | EMA(5) begin={ema_begin} \
         latest={:.4} | RSI(6) begin={rsi_begin} latest={:.4}",
        sma_vals.last().copied().unwrap_or_default(),
        ema_vals.last().copied().unwrap_or_default(),
        rsi_vals.last().copied().unwrap_or_default(),
    );

    // 2. Single-precision (`_f32`) batch parity check
    let close_f32: Vec<f32> = close.iter().map(|&v| v as f32).collect();
    let (rsi_f32_vals, rsi_f32_begin) =
        rsi_f32(&close_f32, Some(6)).context("compute RSI(6) f32")?;
    assert_eq!(rsi_f32_begin, rsi_begin);
    assert_eq!(rsi_f32_vals.len(), rsi_vals.len());

    // 3. Multi-input OHLCV indicators (ATR, ADX, OBV)
    let (atr_vals, atr_begin) = atr(&high, &low, &close, Some(7)).context("compute ATR(7)")?;
    let (adx_vals, adx_begin) = adx(&high, &low, &close, Some(7)).context("compute ADX(7)")?;
    let (obv_vals, obv_begin) = obv(&close, &volume).context("compute OBV")?;
    println!(
        "[OHLCV]         ATR(7) begin={atr_begin} latest={:.4} | ADX(7) begin={adx_begin} \
         latest={:.4} | OBV begin={obv_begin} latest={:.1}",
        atr_vals.last().copied().unwrap_or_default(),
        adx_vals.last().copied().unwrap_or_default(),
        obv_vals.last().copied().unwrap_or_default(),
    );

    // 4. Multi-output indicators (MACD, Bollinger Bands, Stochastic)
    let (macd_line, signal_line, hist_line, macd_begin) =
        macd(&close, Some(12), Some(26), Some(9)).context("compute MACD(12,26,9)")?;
    println!(
        "[Multi-output]  MACD(12,26,9) begin={macd_begin} latest=({:.4}, {:.4}, {:.4})",
        macd_line.last().copied().unwrap_or_default(),
        signal_line.last().copied().unwrap_or_default(),
        hist_line.last().copied().unwrap_or_default(),
    );

    let (upper, middle, lower, bb_begin) =
        bbands(&close, Some(5), Some(2.0), Some(2.0), Some(MAType::Sma))
            .context("compute BBANDS(5)")?;
    println!(
        "[Multi-output]  BBANDS(5)     begin={bb_begin} latest=({:.4}, {:.4}, {:.4})",
        upper.last().copied().unwrap_or_default(),
        middle.last().copied().unwrap_or_default(),
        lower.last().copied().unwrap_or_default(),
    );

    let (slowk, slowd, stoch_begin) = stoch(
        &high,
        &low,
        &close,
        Some(5),
        Some(3),
        Some(MAType::Sma),
        Some(3),
        Some(MAType::Sma),
    )
    .context("compute STOCH(5,3,3)")?;
    println!(
        "[Multi-output]  STOCH(5,3,3)  begin={stoch_begin} latest=(%K={:.4}, %D={:.4})",
        slowk.last().copied().unwrap_or_default(),
        slowd.last().copied().unwrap_or_default(),
    );

    Ok(())
}
