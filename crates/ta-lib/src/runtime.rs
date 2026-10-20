//! Global TA-Lib lifecycle, version metadata, and runtime configuration APIs.

use core::ffi::CStr;
use std::ffi::CString;

use ta_lib_sys as sys;

use crate::{
    error::{Error, check_ret_code},
    types::{CandleSettingType, Compatibility, FuncUnstId, RangeType},
};

/// Initialize the TA-Lib C library (`TA_Initialize`).
///
/// # Errors
///
/// Returns an [`Error`] if `TA_Initialize` fails.
pub fn initialize() -> Result<(), Error> {
    // SAFETY: `TA_Initialize` is safe to call to initialize TA-Lib global state.
    let rc = unsafe { sys::TA_Initialize() };
    check_ret_code(rc)
}

/// Shut down the TA-Lib C library (`TA_Shutdown`).
///
/// # Errors
///
/// Returns an [`Error`] if `TA_Shutdown` fails.
pub fn shutdown() -> Result<(), Error> {
    // SAFETY: `TA_Shutdown` releases resources initialized by `TA_Initialize`.
    let rc = unsafe { sys::TA_Shutdown() };
    check_ret_code(rc)
}

fn static_c_str(ptr: *const core::ffi::c_char) -> &'static str {
    if ptr.is_null() {
        return "";
    }
    // SAFETY: Version strings returned by TA-Lib are static NUL-terminated ASCII strings.
    unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("") }
}

/// Return the full TA-Lib version string (`TA_GetVersionString`).
#[must_use]
pub fn version_string() -> &'static str {
    // SAFETY: `TA_GetVersionString` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionString() })
}

/// Return the major version string (`TA_GetVersionMajor`).
#[must_use]
pub fn version_major() -> &'static str {
    // SAFETY: `TA_GetVersionMajor` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionMajor() })
}

/// Return the minor version string (`TA_GetVersionMinor`).
#[must_use]
pub fn version_minor() -> &'static str {
    // SAFETY: `TA_GetVersionMinor` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionMinor() })
}

/// Return the patch version string (`TA_GetVersionPatch`).
#[must_use]
pub fn version_patch() -> &'static str {
    // SAFETY: `TA_GetVersionPatch` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionPatch() })
}

/// Return the build date string (`TA_GetVersionDate`).
#[must_use]
pub fn version_date() -> &'static str {
    // SAFETY: `TA_GetVersionDate` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionDate() })
}

/// Return the build time string (`TA_GetVersionTime`).
#[must_use]
pub fn version_time() -> &'static str {
    // SAFETY: `TA_GetVersionTime` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionTime() })
}

/// Return the deprecated build string (`TA_GetVersionBuild`).
#[must_use]
pub fn version_build() -> &'static str {
    // SAFETY: `TA_GetVersionBuild` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionBuild() })
}

/// Return the extra version string (`TA_GetVersionExtra`).
#[must_use]
pub fn version_extra() -> &'static str {
    // SAFETY: `TA_GetVersionExtra` returns a static NUL-terminated string pointer.
    static_c_str(unsafe { sys::TA_GetVersionExtra() })
}

/// Query runtime capability information by key (`TA_GetRuntimeInfo`).
///
/// # Errors
///
/// Returns an [`Error`] if `key` contains an interior NUL byte or is unknown to TA-Lib.
pub fn runtime_info(key: &str) -> Result<i32, Error> {
    let c_key = CString::new(key).map_err(|_| Error::InteriorNul)?;
    let mut val: i32 = 0;
    // SAFETY: `c_key` is a valid NUL-terminated C string and `val` is a valid stack i32.
    let rc = unsafe { sys::TA_GetRuntimeInfo(c_key.as_ptr(), &raw mut val) };
    check_ret_code(rc)?;
    Ok(val)
}

/// Set the unstable warmup period stripped from recursive indicators (`TA_SetUnstablePeriod`).
///
/// # Errors
///
/// Returns an [`Error`] if `id` or `unstable_period` is rejected by TA-Lib.
pub fn set_unstable_period(id: FuncUnstId, unstable_period: u32) -> Result<(), Error> {
    // SAFETY: `TA_SetUnstablePeriod` validates `id` and `unstable_period`.
    let rc = unsafe { sys::TA_SetUnstablePeriod(id, unstable_period) };
    check_ret_code(rc)
}

/// Get the unstable warmup period configured for `id` (`TA_GetUnstablePeriod`).
#[must_use]
pub fn get_unstable_period(id: FuncUnstId) -> u32 {
    // SAFETY: `TA_GetUnstablePeriod` is a pure getter over `id`.
    unsafe { sys::TA_GetUnstablePeriod(id) }
}

/// Set global formula compatibility mode (`TA_SetCompatibility`).
///
/// # Errors
///
/// Returns an [`Error`] if TA-Lib rejects the compatibility value.
pub fn set_compatibility(mode: Compatibility) -> Result<(), Error> {
    // SAFETY: `TA_SetCompatibility` sets the compatibility enum value.
    let rc = unsafe { sys::TA_SetCompatibility(mode.into()) };
    check_ret_code(rc)
}

/// Get the current global formula compatibility mode (`TA_GetCompatibility`).
#[must_use]
pub fn get_compatibility() -> Compatibility {
    // SAFETY: `TA_GetCompatibility` reads the current compatibility enum.
    unsafe { sys::TA_GetCompatibility().into() }
}

/// Customize candlestick pattern recognition thresholds (`TA_SetCandleSettings`).
///
/// # Errors
///
/// Returns an [`Error`] if any setting parameter is invalid.
pub fn set_candle_settings(
    setting_type: CandleSettingType,
    range_type: RangeType,
    avg_period: i32,
    factor: f64,
) -> Result<(), Error> {
    // SAFETY: `TA_SetCandleSettings` validates all scalar arguments.
    let rc = unsafe {
        sys::TA_SetCandleSettings(setting_type.into(), range_type.into(), avg_period, factor)
    };
    check_ret_code(rc)
}

/// Restore default candlestick thresholds for `setting_type` (`TA_RestoreCandleDefaultSettings`).
///
/// # Errors
///
/// Returns an [`Error`] if `setting_type` is invalid.
pub fn restore_candle_default_settings(setting_type: CandleSettingType) -> Result<(), Error> {
    // SAFETY: `TA_RestoreCandleDefaultSettings` resets the specified candle setting.
    let rc = unsafe { sys::TA_RestoreCandleDefaultSettings(setting_type.into()) };
    check_ret_code(rc)
}
