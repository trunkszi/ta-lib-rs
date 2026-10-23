//! Example 04: Candlestick pattern recognition (61 CDL indicators) and global
//! `CandleSettingType` / `RangeType` customization.

use anyhow::{Context, Result};
use ta_lib::{
    CandleSettingType, RangeType, cdldoji, cdldoji_lookback, cdlengulfing, cdlhammer,
    cdlmorningstar, initialize, restore_candle_default_settings, set_candle_settings,
};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    // Construct a 15-bar OHLC series ending with a classic Doji and Bullish Engulfing bar
    let open = [
        100.0, 101.0, 102.0, 101.5, 100.8, 101.2, 102.4, 103.0, 102.2, 101.0, 100.5, 100.0, 100.02,
        101.0, 99.2,
    ];
    let high = [
        101.5, 102.5, 103.2, 102.8, 102.0, 102.6, 103.8, 104.2, 103.5, 102.4, 101.8, 101.2, 101.8,
        101.4, 102.5,
    ];
    let low = [
        99.0, 100.0, 101.0, 100.2, 99.5, 100.0, 101.2, 101.8, 101.0, 99.8, 99.2, 98.8, 98.2, 99.6,
        99.0,
    ];
    let close = [
        101.0, 102.0, 101.5, 100.8, 101.2, 102.4, 103.0, 102.2, 101.0, 100.5, 100.0, 100.1, 100.03,
        99.8, 102.1,
    ];

    // 1. Evaluate candlestick patterns with default settings
    let (doji_signals, doji_begin) =
        cdldoji(&open, &high, &low, &close).context("compute CDLDOJI")?;
    let (engulfing_signals, eng_begin) =
        cdlengulfing(&open, &high, &low, &close).context("compute CDLENGULFING")?;
    let (hammer_signals, _) = cdlhammer(&open, &high, &low, &close).context("compute CDLHAMMER")?;
    let (mstar_signals, _) =
        cdlmorningstar(&open, &high, &low, &close, Some(0.3)).context("compute CDLMORNINGSTAR")?;

    println!(
        "Default CDLDOJI       (begin={doji_begin}): {:?}",
        doji_signals
    );
    println!(
        "Default CDLENGULFING  (begin={eng_begin}): {:?}",
        engulfing_signals
    );
    println!("Default CDLHAMMER     signals: {:?}", hammer_signals);
    println!("Default CDLMORNINGSTAR signals: {:?}", mstar_signals);

    // 2. Customize global Doji body threshold via `set_candle_settings`
    let default_lb = cdldoji_lookback().context("default CDLDOJI lookback")?;
    set_candle_settings(CandleSettingType::BodyDoji, RangeType::HighLow, 5, 0.05)
        .context("tighten BodyDoji setting")?;
    let custom_lb = cdldoji_lookback().context("custom CDLDOJI lookback")?;
    let (custom_doji, _) = cdldoji(&open, &high, &low, &close).context("custom CDLDOJI")?;
    println!(
        "Custom BodyDoji(period=5, factor=0.05): lookback {default_lb} -> {custom_lb}, \
         signals={:?}",
        custom_doji
    );

    // 3. Restore default candle settings
    restore_candle_default_settings(CandleSettingType::AllCandleSettings)
        .context("restore default candle settings")?;
    assert_eq!(cdldoji_lookback()?, default_lb);

    Ok(())
}
