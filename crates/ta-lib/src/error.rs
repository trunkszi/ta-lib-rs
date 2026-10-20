//! Error types and return-code diagnostics for `ta-lib`.

use core::ffi::CStr;

use ta_lib_sys as sys;

/// Detailed information associated with a [`sys::TA_RetCode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetCodeInfo {
    /// Symbolic enum constant name (e.g. `"TA_BAD_PARAM"`).
    pub enum_str: String,
    /// Human-readable description of the return code.
    pub info_str: String,
}

/// Query [`RetCodeInfo`] from the C library for a given [`sys::TA_RetCode`].
#[must_use]
pub fn ret_code_info(code: sys::TA_RetCode) -> RetCodeInfo {
    let mut info = sys::TA_RetCodeInfo {
        enumStr: core::ptr::null(),
        infoStr: core::ptr::null(),
    };
    // SAFETY: `TA_SetRetCodeInfo` always populates `info` with static string pointers, even on
    // unknown codes.
    unsafe {
        sys::TA_SetRetCodeInfo(code, &raw mut info);
    }
    let enum_str = if info.enumStr.is_null() {
        String::new()
    } else {
        // SAFETY: `info.enumStr` points to a NUL-terminated static C string in TA-Lib.
        unsafe { CStr::from_ptr(info.enumStr).to_string_lossy().into_owned() }
    };
    let info_str = if info.infoStr.is_null() {
        String::new()
    } else {
        // SAFETY: `info.infoStr` points to a NUL-terminated static C string in TA-Lib.
        unsafe { CStr::from_ptr(info.infoStr).to_string_lossy().into_owned() }
    };
    RetCodeInfo { enum_str, info_str }
}

/// Errors returned by `ta-lib` operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The runtime lock was poisoned by a panic.
    #[error("TA-Lib runtime lock is poisoned")]
    RuntimePoisoned,
    /// Explicit shutdown was requested while runtime guards remain alive.
    #[error("TA-Lib runtime is retained by active guards")]
    RuntimeInUse,
    /// Input slice is empty.
    #[error("input slice cannot be empty")]
    EmptyInput,
    /// Multi-series input slices do not have equal lengths.
    #[error("input slice length mismatch: expected {expected}, got {actual}")]
    MismatchedInputLength {
        /// Expected slice length.
        expected: usize,
        /// Actual slice length provided.
        actual: usize,
    },
    /// Caller-supplied output slice is smaller than required.
    #[error("output buffer too small: required at least {required}, got {actual}")]
    OutputBufferTooSmall {
        /// Minimum required elements.
        required: usize,
        /// Actual buffer capacity.
        actual: usize,
    },
    /// Parameter is out of valid range (`TA_BAD_PARAM`).
    #[error("bad parameter (TA_BAD_PARAM)")]
    BadParam,
    /// Memory allocation failed inside TA-Lib (`TA_ALLOC_ERR`).
    #[error("memory allocation failed in TA-Lib (TA_ALLOC_ERR)")]
    AllocErr,
    /// Insufficient historical bars for streaming warmup (`TA_INSUFFICIENT_HISTORY`).
    #[error("insufficient history for stream warmup (TA_INSUFFICIENT_HISTORY)")]
    InsufficientHistory,
    /// String parameter contained an interior NUL byte.
    #[error("string parameter contains an interior NUL byte")]
    InteriorNul,
    /// Native TA-Lib error code with diagnostic details.
    #[error("TA-Lib error {code:?} ({enum_str}): {info_str}")]
    TaLib {
        /// Raw C return code.
        code: sys::TA_RetCode,
        /// Symbolic C constant name.
        enum_str: String,
        /// Human-readable error message.
        info_str: String,
    },
}

/// Convert a [`sys::TA_RetCode`] into `Result<(), Error>`.
///
/// # Errors
///
/// Returns an [`Error`] if `code` is not [`sys::TA_RetCode::TA_SUCCESS`].
pub fn check_ret_code(code: sys::TA_RetCode) -> Result<(), Error> {
    match code {
        sys::TA_RetCode::TA_SUCCESS => Ok(()),
        sys::TA_RetCode::TA_BAD_PARAM => Err(Error::BadParam),
        sys::TA_RetCode::TA_ALLOC_ERR => Err(Error::AllocErr),
        sys::TA_RetCode::TA_INSUFFICIENT_HISTORY => Err(Error::InsufficientHistory),
        other => {
            let info = ret_code_info(other);
            Err(Error::TaLib {
                code: other,
                enum_str: info.enum_str,
                info_str: info.info_str,
            })
        }
    }
}
