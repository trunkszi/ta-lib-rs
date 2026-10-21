//! Example 02: Zero-allocation caller-owned output buffers (`_into` / `_f32_into`),
//! `<indicator>_lookback` sizing, and `<indicator>_display_shift` metadata.

use anyhow::{Context, Result};
use ta_lib::{
    MAType, bbands_into, bbands_lookback, initialize, rsi, rsi_f32_into, rsi_into, rsi_lookback,
};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let closes = [
        100.0, 101.4, 102.8, 101.9, 103.5, 104.8, 104.0, 105.6, 106.9, 106.1, 107.7, 109.0, 108.2,
        109.8, 111.1, 110.3,
    ];
    let closes_f32: Vec<f32> = closes.iter().map(|&v| v as f32).collect();

    // 1. Exact buffer sizing via `<indicator>_lookback`
    let period = Some(5);
    let lb = rsi_lookback(period).context("query RSI(5) lookback")?;
    let exact_cap = closes.len().saturating_sub(lb);
    let mut rsi_buf = vec![0.0_f64; exact_cap];

    let range = rsi_into(&closes, period, &mut rsi_buf).context("compute RSI(5) into slice")?;
    let (expected_rsi, expected_begin) = rsi(&closes, period).context("compute allocating RSI")?;
    assert_eq!(range.begin, expected_begin);
    assert_eq!(&rsi_buf[..range.count], expected_rsi.as_slice());
    println!(
        "RSI(5) zero-alloc _into: lookback={lb}, begin={}, count={}, latest={:.4}",
        range.begin,
        range.count,
        rsi_buf[range.count - 1]
    );

    // 2. Reusing the same output buffer for `_f32_into`
    let range_f32 =
        rsi_f32_into(&closes_f32, period, &mut rsi_buf).context("compute RSI(5) f32 into slice")?;
    assert_eq!(range_f32, range);

    // 3. Reusing multi-output caller buffers across multiple parameter sweeps
    let mut upper = vec![0.0_f64; closes.len()];
    let mut middle = vec![0.0_f64; closes.len()];
    let mut lower = vec![0.0_f64; closes.len()];
    for win in [3usize, 5, 8] {
        let bb_lb =
            bbands_lookback(Some(win), Some(2.0), Some(2.0), Some(MAType::Sma)).context("bb lb")?;
        let bb_range = bbands_into(
            &closes,
            Some(win),
            Some(2.0),
            Some(2.0),
            Some(MAType::Sma),
            &mut upper,
            &mut middle,
            &mut lower,
        )
        .with_context(|| format!("BBANDS({win}) into reusable buffers"))?;
        println!(
            "BBANDS({win}) sweep: lookback={bb_lb}, count={}, latest_mid={:.4}",
            bb_range.count,
            middle[bb_range.count - 1]
        );
    }

    // 4. Chart display shift metadata (`pre-release` feature)
    #[cfg(not(feature = "stable"))]
    {
        use ta_lib::{ichimoku_display_shift, sma_display_shift};
        let sma_shift = sma_display_shift(Some(5), 0).context("SMA display shift")?;
        let senkou_a_shift =
            ichimoku_display_shift(Some(9), Some(26), Some(52), 2).context("Ichimoku span A")?;
        let senkou_b_shift =
            ichimoku_display_shift(Some(9), Some(26), Some(52), 3).context("Ichimoku span B")?;
        println!(
            "Display shifts: SMA[0]={sma_shift}, Ichimoku SenkouA[2]={senkou_a_shift}, \
             SenkouB[3]={senkou_b_shift}"
        );
    }

    Ok(())
}
