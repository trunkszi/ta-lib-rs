//! Example 05: Runtime lifecycle (`RuntimeGuard`), version metadata, `UnstablePeriod`
//! stripping for recursive indicators, and `Compatibility` modes.

use anyhow::{Context, Result};
use ta_lib::{
    Compatibility, FuncUnstId, RuntimeGuard, ema, ema_lookback, get_compatibility,
    get_unstable_period, set_compatibility, set_unstable_period, version_build, version_date,
    version_extra, version_major, version_minor, version_patch, version_string, version_time,
};

fn main() -> Result<()> {
    // 1. Reference-counted RAII runtime guard (`RuntimeGuard::new`)
    let _guard = RuntimeGuard::new().context("acquire TA-Lib RuntimeGuard")?;

    println!(
        "TA-Lib version: {} (major={}, minor={}, patch={}, build='{}', date='{}', time='{}', \
         extra='{}')",
        version_string(),
        version_major(),
        version_minor(),
        version_patch(),
        version_build(),
        version_date(),
        version_time(),
        version_extra(),
    );

    #[cfg(not(feature = "stable"))]
    {
        let init_count =
            ta_lib::runtime_info("count.initialize").context("query count.initialize")?;
        let simd = ta_lib::runtime_info("simd.x86_64.avx2").unwrap_or(0);
        println!("Runtime info: count.initialize={init_count}, simd.x86_64.avx2={simd}");
    }

    // 2. Effect of `set_unstable_period` on recursive indicators (e.g. EMA)
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64) * 0.8).collect();
    set_unstable_period(FuncUnstId::TA_FUNC_UNST_EMA, 0).context("reset EMA unstable period")?;
    let base_lb = ema_lookback(Some(5)).context("EMA(5) base lookback")?;
    let (base_ema, base_begin) = ema(&prices, Some(5)).context("compute base EMA(5)")?;

    set_unstable_period(FuncUnstId::TA_FUNC_UNST_EMA, 6).context("set EMA unstable period = 6")?;
    assert_eq!(get_unstable_period(FuncUnstId::TA_FUNC_UNST_EMA), 6);
    let stripped_lb = ema_lookback(Some(5)).context("EMA(5) stripped lookback")?;
    let (stripped_ema, stripped_begin) =
        ema(&prices, Some(5)).context("compute stripped EMA(5)")?;

    println!(
        "EMA(5) unstable period=0: lookback={base_lb}, begin={base_begin}, len={}",
        base_ema.len()
    );
    println!(
        "EMA(5) unstable period=6: lookback={stripped_lb}, begin={stripped_begin}, len={}",
        stripped_ema.len()
    );
    assert_eq!(stripped_lb, base_lb + 6);
    assert_eq!(stripped_ema.as_slice(), &base_ema[6..]);

    // Restore EMA unstable period to 0
    set_unstable_period(FuncUnstId::TA_FUNC_UNST_EMA, 0).context("restore EMA unstable period")?;

    // 3. Compatibility mode API
    set_compatibility(Compatibility::Metastock).context("set Metastock compatibility")?;
    set_compatibility(Compatibility::Default).context("restore Default compatibility")?;
    assert_eq!(get_compatibility(), Compatibility::Default);

    Ok(())
}
