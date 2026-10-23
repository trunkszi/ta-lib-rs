//! Example 03: Stateful RAII streaming state machines (`open`, `open_and_fill`, `peek`,
//! `update`, `value`, `out_range`, `advance`, and `try_clone`).

use anyhow::{Context, Result};
use ta_lib::{BbandsStream, MAType, MacdStream, RsiStream, initialize, rsi};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let history = [
        100.0, 101.2, 102.5, 101.8, 103.4, 104.9, 104.1, 105.7, 106.3, 105.5, 107.0, 108.2, 107.6,
        109.1, 110.4, 109.8, 111.2, 112.5, 111.9, 113.4, 114.6, 114.0, 115.5, 116.8, 116.2, 117.7,
        119.0, 118.4, 119.9, 121.2, 120.6, 122.1, 123.4, 122.8, 124.3, 125.6,
    ];
    let live_bars = [126.4, 125.2, 127.8, 129.1];

    // 1. Open an RSI(6) stream warmed up from historical bars
    let (mut rsi_stream, initial_rsi) =
        RsiStream::open(&history, Some(6)).context("open RSI(6) stream")?;
    println!(
        "RSI(6) opened: initial={initial_rsi:.4}, out_range={:?}",
        rsi_stream.out_range()?
    );

    // 2. Preview intrabar forming ticks with `peek` without mutating stream state
    let bull_tick = rsi_stream.peek(132.0).context("peek bullish tick")?;
    let bear_tick = rsi_stream.peek(118.0).context("peek bearish tick")?;
    assert_eq!(rsi_stream.value()?, initial_rsi);
    println!("RSI(6) intrabar peek: @132.0 -> {bull_tick:.4}, @118.0 -> {bear_tick:.4}");

    // 3. Clone stream state with `try_clone` for independent counterfactual branching
    let mut crash_branch = rsi_stream.try_clone().context("clone RSI stream")?;
    let crash_rsi = crash_branch
        .update(112.0)
        .context("update crash scenario")?;
    println!("Counterfactual crash branch @112.0 -> RSI(6) = {crash_rsi:.4}");
    assert_eq!(rsi_stream.value()?, initial_rsi);

    // 4. Commit closed bars with `update` and verify parity with batch calculation
    for &bar in &live_bars {
        let updated = rsi_stream.update(bar).context("update closed bar")?;
        println!("Committed closed bar {bar:.2} -> RSI(6) = {updated:.4}");
    }

    let mut all_bars = history.to_vec();
    all_bars.extend_from_slice(&live_bars);
    let (batch_rsi, _) = rsi(&all_bars, Some(6)).context("compute batch RSI(6)")?;
    assert!((rsi_stream.value()? - batch_rsi.last().copied().unwrap()).abs() < 1e-10);

    // 5. Multi-output streaming (`MacdStream::open` and `BbandsStream::open_and_fill`)
    let (mut macd_stream, (m0, s0, h0)) =
        MacdStream::open(&history, Some(12), Some(26), Some(9)).context("open MACD stream")?;
    let (m1, s1, h1) = macd_stream.update(126.4).context("update MACD stream")?;
    println!(
        "MACD stream: initial=({m0:.4}, {s0:.4}, {h0:.4}) -> next=({m1:.4}, {s1:.4}, {h1:.4})"
    );

    let mut upper = vec![0.0; history.len()];
    let mut middle = vec![0.0; history.len()];
    let mut lower = vec![0.0; history.len()];
    let (mut bb_stream, bb_range) = BbandsStream::open_and_fill(
        &history,
        Some(5),
        Some(2.0),
        Some(2.0),
        Some(MAType::Sma),
        &mut upper,
        &mut middle,
        &mut lower,
    )
    .context("open_and_fill BBANDS(5) stream")?;
    bb_stream.advance().context("advance skipped bar")?;
    println!(
        "BBANDS(5) open_and_fill: filled={} bars, out_range after advance={:?}",
        bb_range.count,
        bb_stream.out_range()?
    );

    Ok(())
}
