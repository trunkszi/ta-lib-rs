//! High-level, safe Rust wrapper for the [TA-Lib](https://ta-lib.org/) technical analysis library.

extern crate self as ta_lib;

pub mod abstract_api;
pub mod error;
pub mod indicators;
pub mod runtime;
pub mod types;

pub use abstract_api::{
    FuncHandle, FuncInfo, InputParameterInfo, InputParameterType, IntegerRange, NamedInteger,
    NamedReal, OptInputDataSet, OptInputParameterInfo, OptInputParameterType, OutputParameterInfo,
    OutputParameterType, ParamHolder, RealRange,
};
pub use anyhow;
pub use error::{Error, Result, RetCodeInfo, check_ret_code, ret_code_info};
pub use indicators::*;
pub use runtime::*;
pub use ta_lib_macros::ta_indicators;
pub use ta_lib_sys as sys;
pub use types::{
    CandleSettingType, Compatibility, FuncUnstId, MAType, MovingAverageType, OutRange, RangeType,
};

#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
mod readme_doctests {}
