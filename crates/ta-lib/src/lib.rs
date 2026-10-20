//! High-level, safe Rust wrapper for the [TA-Lib](https://ta-lib.org/) technical analysis library.
#![allow(unused_imports)]

pub mod abstract_api;
pub mod error;
pub mod indicators;
pub mod runtime;
pub mod types;

pub use error::{Error, RetCodeInfo, check_ret_code, ret_code_info};
pub use indicators::*;
pub use runtime::*;
pub use ta_lib_macros::ta_indicators;
pub use ta_lib_sys as sys;
pub use types::{
    CandleSettingType, Compatibility, FuncUnstId, MAType, MovingAverageType, OutRange, RangeType,
};
