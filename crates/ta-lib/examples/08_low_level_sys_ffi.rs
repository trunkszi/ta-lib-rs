//! Example 08: Direct low-level `ta-lib-sys` C FFI invocation (`TA_SMA`, `TA_RSI_Open` /
//! `TA_RSI_Update` / `TA_RSI_Close`) and native `TA_RetCode` diagnostics.

use core::ptr;

use anyhow::{Context, Result, bail};
use ta_lib::{check_ret_code, initialize, ret_code_info, sys};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    let prices = [10.0_f64, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
    let period = 3;

    // 1. Direct C batch call (`sys::TA_SMA_Lookback` and `sys::TA_SMA`)
    // SAFETY: `TA_SMA_Lookback` is a pure scalar function.
    let lookback = unsafe { sys::TA_SMA_Lookback(period) };
    let mut out_begin: i32 = 0;
    let mut out_count: i32 = 0;
    let mut out_buf = [0.0_f64; 7];

    // SAFETY: `prices` has 7 elements covering `0..=6`, and `out_buf` has capacity >= 7 - lookback.
    let rc = unsafe {
        sys::TA_SMA(
            0,
            (prices.len() - 1) as i32,
            prices.as_ptr(),
            period,
            &raw mut out_begin,
            &raw mut out_count,
            out_buf.as_mut_ptr(),
        )
    };
    check_ret_code(rc).context("raw TA_SMA call failed")?;
    println!(
        "[Raw FFI batch] TA_SMA(3): lookback={lookback}, begin={out_begin}, values={:?}",
        &out_buf[..(out_count as usize)]
    );

    // 2. Direct C streaming lifecycle (`TA_RSI_Open`, `TA_RSI_Update`, `TA_RSI_Close`)
    let mut raw_stream: *mut sys::TA_RSI_Stream = ptr::null_mut();
    let mut initial_rsi = 0.0_f64;
    // SAFETY: `prices` is valid for `prices.len()` elements and out-pointers are valid stack vars.
    let open_rc = unsafe {
        sys::TA_RSI_Open(
            &raw mut raw_stream,
            prices.as_ptr(),
            prices.len() as i32,
            period,
            &raw mut initial_rsi,
        )
    };
    check_ret_code(open_rc).context("raw TA_RSI_Open failed")?;
    if raw_stream.is_null() {
        bail!("TA_RSI_Open returned a null stream pointer");
    }

    let mut next_rsi = 0.0_f64;
    // SAFETY: `raw_stream` is a live stream handle allocated by `TA_RSI_Open`.
    let update_rc = unsafe { sys::TA_RSI_Update(raw_stream, 17.5, &raw mut next_rsi) };
    // Always close the raw C stream handle before checking the result.
    // SAFETY: `raw_stream` is freed exactly once here.
    let close_rc = unsafe { sys::TA_RSI_Close(raw_stream) };
    check_ret_code(update_rc).context("raw TA_RSI_Update failed")?;
    check_ret_code(close_rc).context("raw TA_RSI_Close failed")?;
    println!("[Raw FFI stream] TA_RSI(3): open={initial_rsi:.4} -> update(17.5)={next_rsi:.4}");

    // 3. Inspect native `TA_RetCode` diagnostics via `ret_code_info`
    for code in [
        sys::TA_RetCode::TA_SUCCESS,
        sys::TA_RetCode::TA_BAD_PARAM,
        sys::TA_RetCode::TA_INSUFFICIENT_HISTORY,
    ] {
        let info = ret_code_info(code);
        println!(
            "[RetCodeInfo] {code:?} (raw={}): {} — {}",
            code.raw(),
            info.enum_str,
            info.info_str
        );
    }

    Ok(())
}
