//! Global TA-Lib lifecycle, version metadata, and runtime configuration APIs.

use core::ffi::CStr;
use std::sync::{Mutex, MutexGuard};

use ta_lib_sys as sys;

use crate::{
    error::{Error, check_ret_code},
    types::{CandleSettingType, Compatibility, FuncUnstId, RangeType},
};

#[derive(Default)]
struct RuntimeState {
    initialized: bool,
    manual_owner: bool,
    guards: usize,
    shutdown_error: Option<Error>,
}

static RUNTIME: Mutex<RuntimeState> = Mutex::new(RuntimeState {
    initialized: false,
    manual_owner: false,
    guards: 0,
    shutdown_error: None,
});

fn runtime_lock() -> Result<MutexGuard<'static, RuntimeState>, Error> {
    let state = RUNTIME.lock().map_err(|_| Error::RuntimePoisoned)?;
    if let Some(error) = &state.shutdown_error {
        return Err(error.clone());
    }
    Ok(state)
}

impl RuntimeState {
    fn initialize(&mut self) -> Result<(), Error> {
        if !self.initialized {
            // SAFETY: All high-level lifecycle and configuration calls hold RUNTIME.
            check_ret_code(unsafe { sys::TA_Initialize() })?;
            self.initialized = true;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        if self.initialized {
            // SAFETY: The caller holds RUNTIME and has verified there are no guards.
            // Applications must also quiesce calculations and drop streams/holders.
            check_ret_code(unsafe { sys::TA_Shutdown() })?;
            self.initialized = false;
        }
        self.manual_owner = false;
        self.shutdown_error = None;
        Ok(())
    }
}

fn configured_runtime() -> Result<MutexGuard<'static, RuntimeState>, Error> {
    let mut state = runtime_lock()?;
    if !state.initialized {
        state.initialize()?;
        state.manual_owner = true;
    }
    Ok(state)
}

/// Initialize TA-Lib once without resetting existing process-wide settings.
///
/// # Errors
/// Returns an error on native initialization failure or a poisoned runtime lock.
pub fn initialize() -> Result<(), Error> {
    ensure_initialized()
}

/// Ensure TA-Lib is initialized, retaining explicit ownership until `shutdown`.
/// Repeated calls preserve unstable periods, compatibility and candle settings.
///
/// # Errors
/// Returns an error on native initialization failure, a prior guard shutdown failure,
/// or a poisoned runtime lock.
pub fn ensure_initialized() -> Result<(), Error> {
    let mut state = runtime_lock()?;
    state.initialize()?;
    state.manual_owner = true;
    Ok(())
}

/// Shut down TA-Lib after all calculations, streams and holders have finished.
/// Active runtime guards reject shutdown. Repeated shutdown is harmless.
/// This lock serializes runtime controls, not indicator calculations; callers
/// must quiesce calculations before changing configuration or shutting down.
///
/// # Errors
/// Returns an error for active guards, a poisoned lock or a native shutdown failure.
pub fn shutdown() -> Result<(), Error> {
    let mut state = RUNTIME.lock().map_err(|_| Error::RuntimePoisoned)?;
    if state.guards != 0 {
        return Err(Error::RuntimeInUse);
    }
    state.shutdown()
}

/// Retain an initialized TA-Lib runtime until this guard is dropped.
/// The last guard shuts down a guard-owned runtime. An explicit `initialize` or
/// `ensure_initialized` call retains it until explicit `shutdown` instead.
/// Guards do not synchronize calculations or authorize concurrent setting changes.
#[derive(Debug)]
#[must_use]
pub struct RuntimeGuard {
    _private: (),
}

impl RuntimeGuard {
    /// Acquire a runtime reference without resetting an existing runtime.
    /// # Errors
    /// Returns an error if initialization fails, the lock is poisoned or the guard count overflows.
    pub fn new() -> Result<Self, Error> {
        let mut state = runtime_lock()?;
        let guards = state.guards.checked_add(1).ok_or(Error::BadParam)?;
        state.initialize()?;
        state.guards = guards;
        Ok(Self { _private: () })
    }
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = RUNTIME.lock() {
            state.guards -= 1;
            if state.guards == 0
                && !state.manual_owner
                && let Err(error) = state.shutdown()
            {
                // Preserve failure for the next fallible runtime operation;
                // explicit shutdown can retry the native cleanup.
                state.shutdown_error = Some(error);
            }
        }
    }
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
#[cfg(not(feature = "release-tag"))]
pub fn runtime_info(key: &str) -> Result<i32, Error> {
    let _state = configured_runtime()?;
    let c_key = std::ffi::CString::new(key).map_err(|_| Error::InteriorNul)?;
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
    let _state = configured_runtime()?;
    // SAFETY: `TA_SetUnstablePeriod` validates `id` and `unstable_period`.
    let rc = unsafe { sys::TA_SetUnstablePeriod(id, unstable_period) };
    check_ret_code(rc)
}

/// Get the unstable warmup period configured for `id` (`TA_GetUnstablePeriod`).
/// # Panics
/// Panics if runtime initialization or locking fails; use `try_get_unstable_period`
/// to handle that failure explicitly.
#[must_use]
pub fn get_unstable_period(id: FuncUnstId) -> u32 {
    try_get_unstable_period(id).expect("TA-Lib runtime unavailable")
}

/// Read an unstable period, reporting runtime initialization or lock failures.
/// # Errors
/// Returns an error if the runtime cannot be initialized or locked.
pub fn try_get_unstable_period(id: FuncUnstId) -> Result<u32, Error> {
    let _state = configured_runtime()?;
    // SAFETY: `TA_GetUnstablePeriod` is a pure getter over `id`.
    Ok(unsafe { sys::TA_GetUnstablePeriod(id) })
}

/// Set global formula compatibility mode (`TA_SetCompatibility`).
///
/// # Errors
///
/// Returns an [`Error`] if TA-Lib rejects the compatibility value.
pub fn set_compatibility(mode: Compatibility) -> Result<(), Error> {
    let _state = configured_runtime()?;
    // SAFETY: `TA_SetCompatibility` sets the compatibility enum value.
    let rc = unsafe { sys::TA_SetCompatibility(mode.into()) };
    check_ret_code(rc)
}

/// Get the current global formula compatibility mode (`TA_GetCompatibility`).
/// # Panics
/// Panics if runtime initialization or locking fails; use `try_get_compatibility`
/// to handle that failure explicitly.
#[must_use]
pub fn get_compatibility() -> Compatibility {
    try_get_compatibility().expect("TA-Lib runtime unavailable")
}

/// Read compatibility, reporting runtime initialization or lock failures.
/// # Errors
/// Returns an error if the runtime cannot be initialized or locked.
pub fn try_get_compatibility() -> Result<Compatibility, Error> {
    let _state = configured_runtime()?;
    // SAFETY: `TA_GetCompatibility` reads the current compatibility enum.
    Ok(unsafe { sys::TA_GetCompatibility().into() })
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
    let _state = configured_runtime()?;
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
    let _state = configured_runtime()?;
    // SAFETY: `TA_RestoreCandleDefaultSettings` resets the specified candle setting.
    let rc = unsafe { sys::TA_RestoreCandleDefaultSettings(setting_type.into()) };
    check_ret_code(rc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutRange;

    #[test]
    fn integration_runtime_lifecycle_and_concurrent_initialization() {
        // A separate process keeps deliberate configuration/shutdown transitions
        // from racing unrelated indicator tests in this test executable.
        const CHILD: &str = "TA_LIB_RUNTIME_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "runtime::tests::integration_runtime_lifecycle_and_concurrent_initialization",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        shutdown().unwrap();
        let first = RuntimeGuard::new().unwrap();
        let second = RuntimeGuard::new().unwrap();
        assert_eq!(shutdown(), Err(Error::RuntimeInUse));
        drop(first);
        assert_eq!(shutdown(), Err(Error::RuntimeInUse));
        drop(second);
        assert!(!runtime_lock().unwrap().initialized);

        let id = FuncUnstId::TA_FUNC_UNST_EMA;
        set_unstable_period(id, 17).unwrap();
        set_compatibility(Compatibility::Metastock).unwrap();
        set_candle_settings(
            CandleSettingType::try_from(sys::TA_CandleSettingType::TA_BodyDoji).unwrap(),
            RangeType::try_from(sys::TA_RangeType::TA_RangeType_HighLow).unwrap(),
            0,
            0.0,
        )
        .unwrap();
        let mut threads = Vec::new();
        for _ in 0..8 {
            threads.push(std::thread::spawn(|| {
                initialize().unwrap();
                ensure_initialized().unwrap();
                let guard = RuntimeGuard::new().unwrap();
                assert_eq!(get_unstable_period(FuncUnstId::TA_FUNC_UNST_EMA), 17);
                assert_eq!(get_compatibility(), Compatibility::Default);
                drop(guard);
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(get_unstable_period(id), 17);
        assert_eq!(get_compatibility(), Compatibility::Default);
        let open = [100.0; 20];
        let high = [101.0; 20];
        let low = [99.0; 20];
        let close = [100.1; 20];
        let (custom, _) = crate::cdldoji(&open, &high, &low, &close).unwrap();
        assert!(!custom.is_empty());
        assert!(custom.iter().all(|v| *v == 0));
        restore_candle_default_settings(CandleSettingType::AllCandleSettings).unwrap();
        let (default, _) = crate::cdldoji(&open, &high, &low, &close).unwrap();
        assert!(!default.is_empty());
        assert!(default.iter().all(|v| *v == 100));
        shutdown().unwrap();
        shutdown().unwrap();
        assert_eq!(try_get_unstable_period(id).unwrap(), 0);
        assert_eq!(try_get_compatibility().unwrap(), Compatibility::Default);
        shutdown().unwrap();
        let guard = RuntimeGuard::new().unwrap();
        ensure_initialized().unwrap();
        drop(guard);
        assert!(runtime_lock().unwrap().initialized);
        shutdown().unwrap();
    }

    #[test]
    fn unit_exact_batch_buffers_and_zero_output_warmup() {
        ensure_initialized().unwrap();
        let input = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output = [f64::NAN; 3];
        let range = crate::sma_into(&input, Some(3), &mut output).unwrap();
        assert_eq!(range, OutRange { begin: 2, count: 3 });
        assert_eq!(output, [2.0, 3.0, 4.0]);
        assert_eq!(
            crate::sma_into(&input, Some(3), &mut [0.0; 2]),
            Err(Error::OutputBufferTooSmall {
                required: 3,
                actual: 2
            })
        );
        let single = [1.0_f32, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            crate::sma_f32_into(&single, Some(3), &mut output).unwrap(),
            range
        );
        assert_eq!(output, [2.0, 3.0, 4.0]);
        assert_eq!(
            crate::sma_f32_into(&single, Some(3), &mut [0.0; 2]),
            Err(Error::OutputBufferTooSmall {
                required: 3,
                actual: 2
            })
        );
        assert_eq!(
            crate::sma_into(&input[..2], Some(3), &mut [])
                .unwrap()
                .count,
            0
        );
        assert_eq!(
            crate::sma_f32_into(&single[..2], Some(3), &mut [])
                .unwrap()
                .count,
            0
        );
        let mut indices = [0; 3];
        let range = crate::maxindex_into(&input, Some(3), &mut indices).unwrap();
        assert_eq!(range.count, 3);
        assert_eq!(indices, [2, 3, 4]);
    }

    #[test]
    fn unit_stream_exact_history_buffers_and_skip_semantics() {
        ensure_initialized().unwrap();
        let input = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output = [0.0; 3];
        let (mut stream, range) =
            crate::SmaStream::open_and_fill(&input, Some(3), &mut output).unwrap();
        assert_eq!(range, OutRange { begin: 2, count: 3 });
        assert_eq!(output, [2.0, 3.0, 4.0]);
        assert!(matches!(
            crate::SmaStream::open_and_fill(&input, Some(3), &mut [0.0; 2]),
            Err(Error::OutputBufferTooSmall {
                required: 3,
                actual: 2
            })
        ));
        assert!(matches!(
            crate::SmaStream::open_and_fill(&input[..2], Some(3), &mut []),
            Err(Error::InsufficientHistory)
        ));
        let mut branch = stream.try_clone().unwrap();
        let value = stream.value().unwrap();
        let before = stream.out_range().unwrap();
        assert!(stream.update(f64::NAN).is_err());
        assert_eq!(stream.out_range().unwrap(), before);
        stream.advance().unwrap();
        assert_eq!(stream.value().unwrap(), value);
        assert_eq!(stream.out_range().unwrap().count, before.count + 1);
        assert_eq!(stream.update(6.0).unwrap(), branch.update(6.0).unwrap());
    }

    #[test]
    fn unit_multi_output_exact_macd_buffers() {
        ensure_initialized().unwrap();
        let input: Vec<_> = (0..50).map(f64::from).collect();
        let lb = crate::macd_lookback(Some(12), Some(26), Some(9)).unwrap();
        let required = input.len() - lb;
        let mut line = vec![0.0; required];
        let mut signal = line.clone();
        let mut hist = line.clone();
        let range = crate::macd_into(
            &input,
            Some(12),
            Some(26),
            Some(9),
            &mut line,
            &mut signal,
            &mut hist,
        )
        .unwrap();
        assert_eq!(range.count, required);
        assert_eq!(
            crate::macd(&input, Some(12), Some(26), Some(9)).unwrap(),
            (line, signal, hist, range.begin)
        );
    }
}
