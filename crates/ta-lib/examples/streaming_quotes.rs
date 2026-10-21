//! Stateful RAII streaming example (`open`, `peek`, `update`, `try_clone`, and `open_and_fill`).

use anyhow::{Context, Result};
use ta_lib::{BbandsStream, MAType, RsiStream, initialize, rsi};

fn main() -> Result<()> {
    initialize().context("failed to initialize TA-Lib runtime")?;

    let history = [100.0, 101.2, 102.5, 101.8, 103.4, 104.9, 104.1, 105.7];
    let live_bars = [106.8, 105.9, 107.4, 108.6];

    // 1. Open an RSI(5) stream warmed up from historical bars
    let (mut stream, initial_rsi) =
        RsiStream::open(&history, Some(5)).context("open RSI(5) stream")?;
    println!("Initial warmed-up RSI(5): {initial_rsi:.4}");
    assert_eq!(stream.value()?, initial_rsi);

    // 2. Preview intrabar ticks with `peek` (does not mutate committed state)
    let preview_high = stream.peek(109.0).context("peek high tick")?;
    let preview_low = stream.peek(99.0).context("peek low tick")?;
    assert_eq!(stream.value()?, initial_rsi);
    println!("Intrabar RSI(5) preview: if 109.0 -> {preview_high:.4}, if 99.0 -> {preview_low:.4}");

    // 3. Branch state with `try_clone` for counterfactual scenario simulation
    let mut shock_branch = stream.try_clone().context("clone RSI stream")?;
    let shock_rsi = shock_branch.update(95.0).context("update shock branch")?;
    println!("Counterfactual shock branch RSI(5) at 95.0: {shock_rsi:.4}");
    assert_eq!(stream.value()?, initial_rsi);

    // 4. Commit live closed bars with `update` and verify exact parity with batch `rsi`
    for &bar in &live_bars {
        let current = stream.update(bar).context("commit closed bar")?;
        println!("Committed bar {bar:.2} -> RSI(5) = {current:.4}");
    }

    let mut full_series = history.to_vec();
    full_series.extend_from_slice(&live_bars);
    let (batch_rsi, _) = rsi(&full_series, Some(5)).context("compute batch RSI(5)")?;
    let latest_batch = batch_rsi.last().copied().context("empty batch output")?;
    assert!((stream.value()? - latest_batch).abs() < 1e-10);

    // 5. Multi-output streaming with `open_and_fill`
    let mut upper = vec![0.0; history.len()];
    let mut middle = vec![0.0; history.len()];
    let mut lower = vec![0.0; history.len()];
    let (mut bb_stream, bb_range) = BbandsStream::open_and_fill(
        &history,
        Some(4),
        Some(2.0),
        Some(2.0),
        Some(MAType::Sma),
        &mut upper,
        &mut middle,
        &mut lower,
    )
    .context("open and fill BBANDS(4) stream")?;
    let (u, m, l) = bb_stream.update(106.8).context("update BBANDS stream")?;
    println!(
        "BBANDS(4) filled {} historical outputs (begin={}), next=({u:.4}, {m:.4}, {l:.4})",
        bb_range.count, bb_range.begin
    );

    Ok(())
}
