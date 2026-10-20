//! Low-level FFI bindings to the statically linked upstream `TA-Lib` C library.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};

pub type Int32 = c_int;
pub type UInt32 = c_uint;
pub type Int64 = i64;
pub type UInt64 = u64;
pub type TA_Real = f64;
pub type TA_Integer = c_int;
pub type TA_FuncHandle = c_uint;
pub type TA_FuncFlags = c_int;
pub type TA_InputFlags = c_int;
pub type TA_OptInputFlags = c_int;
pub type TA_OutputFlags = c_int;

pub const TA_INTEGER_MIN: TA_Integer = -2_147_483_647;
pub const TA_INTEGER_MAX: TA_Integer = 2_147_483_647;
pub const TA_INTEGER_DEFAULT: TA_Integer = i32::MIN;
pub const TA_INDEX_MAX: TA_Integer = 100_000_000;
pub const TA_MAX_INDEX: TA_Integer = 100_000_000;
pub const TA_UNSTABLE_AUTO_PREC_4: c_uint = 100_000_004;
pub const TA_UNSTABLE_AUTO_PREC_8: c_uint = 100_000_008;

pub const TA_REAL_MIN: TA_Real = -3e+37;
pub const TA_REAL_MAX: TA_Real = 3e+37;
pub const TA_REAL_DEFAULT: TA_Real = -4e+37;

pub const TA_FUNC_FLG_PERIOD1_IDENTITY: TA_FuncFlags = 1;
pub const TA_FUNC_FLG_DISPLAY_SHIFT: TA_FuncFlags = 2;
pub const TA_FUNC_FLG_USES_TRANSCENDENTAL: TA_FuncFlags = 4;
pub const TA_FUNC_FLG_POSITIVE_INPUT: TA_FuncFlags = 8;
pub const TA_FUNC_FLG_OVERLAP: TA_FuncFlags = 0x0100_0000;
pub const TA_FUNC_FLG_STREAM: TA_FuncFlags = 0x0200_0000;
pub const TA_FUNC_FLG_VOLUME: TA_FuncFlags = 0x0400_0000;
pub const TA_FUNC_FLG_UNST_PER: TA_FuncFlags = 0x0800_0000;
pub const TA_FUNC_FLG_CANDLESTICK: TA_FuncFlags = 0x1000_0000;
pub const TA_FUNC_FLG_PATH_DEP: TA_FuncFlags = 0x2000_0000;
pub const TA_FUNC_FLG_NAN_INF_OUT: TA_FuncFlags = 0x4000_0000;

pub const TA_IN_PRICE_OPEN: TA_InputFlags = 0x0000_0001;
pub const TA_IN_PRICE_HIGH: TA_InputFlags = 0x0000_0002;
pub const TA_IN_PRICE_LOW: TA_InputFlags = 0x0000_0004;
pub const TA_IN_PRICE_CLOSE: TA_InputFlags = 0x0000_0008;
pub const TA_IN_PRICE_VOLUME: TA_InputFlags = 0x0000_0010;
pub const TA_IN_PRICE_OPENINTEREST: TA_InputFlags = 0x0000_0020;
pub const TA_IN_PRICE_TIMESTAMP: TA_InputFlags = 0x0000_0040;

pub const TA_OPTIN_IS_PERCENT: TA_OptInputFlags = 0x0010_0000;
pub const TA_OPTIN_IS_DEGREE: TA_OptInputFlags = 0x0020_0000;
pub const TA_OPTIN_IS_CURRENCY: TA_OptInputFlags = 0x0040_0000;
pub const TA_OPTIN_ADVANCED: TA_OptInputFlags = 0x0100_0000;

pub const TA_OUT_LINE: TA_OutputFlags = 0x0000_0001;
pub const TA_OUT_DOT_LINE: TA_OutputFlags = 0x0000_0002;
pub const TA_OUT_DASH_LINE: TA_OutputFlags = 0x0000_0004;
pub const TA_OUT_DOT: TA_OutputFlags = 0x0000_0008;
pub const TA_OUT_HISTO: TA_OutputFlags = 0x0000_0010;
pub const TA_OUT_PATTERN_BOOL: TA_OutputFlags = 0x0000_0020;
pub const TA_OUT_PATTERN_BULL_BEAR: TA_OutputFlags = 0x0000_0040;
pub const TA_OUT_PATTERN_STRENGTH: TA_OutputFlags = 0x0000_0080;
pub const TA_OUT_PATTERN_CONFIRM: TA_OutputFlags = 0x0000_0080;
pub const TA_OUT_POSITIVE: TA_OutputFlags = 0x0000_0100;
pub const TA_OUT_NEGATIVE: TA_OutputFlags = 0x0000_0200;
pub const TA_OUT_ZERO: TA_OutputFlags = 0x0000_0400;
pub const TA_OUT_UPPER_LIMIT: TA_OutputFlags = 0x0000_0800;
pub const TA_OUT_LOWER_LIMIT: TA_OutputFlags = 0x0000_1000;
pub const TA_OUT_NULLABLE: TA_OutputFlags = 0x0000_2000;
pub const TA_OUT_DISPLAY_SHIFT: TA_OutputFlags = 0x0000_4000;
pub const TA_OUT_PATTERN_WEAK: TA_OutputFlags = 0x0000_8000;

pub const TA_FUNC_UNST_COUNT: usize = 37;
pub const TA_MATYPE_MIN: c_uint = 0;
pub const TA_MATYPE_MAX: c_uint = 15;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_RetCode {
    TA_SUCCESS = 0,
    TA_LIB_NOT_INITIALIZE = 1,
    TA_BAD_PARAM = 2,
    TA_ALLOC_ERR = 3,
    TA_GROUP_NOT_FOUND = 4,
    TA_FUNC_NOT_FOUND = 5,
    TA_INVALID_HANDLE = 6,
    TA_INVALID_PARAM_HOLDER = 7,
    TA_INVALID_PARAM_HOLDER_TYPE = 8,
    TA_INVALID_PARAM_FUNCTION = 9,
    TA_INPUT_NOT_ALL_INITIALIZE = 10,
    TA_OUTPUT_NOT_ALL_INITIALIZE = 11,
    TA_OUT_OF_RANGE_START_INDEX = 12,
    TA_OUT_OF_RANGE_END_INDEX = 13,
    TA_INVALID_LIST_TYPE = 14,
    TA_BAD_OBJECT = 15,
    TA_NOT_SUPPORTED = 16,
    TA_INSUFFICIENT_HISTORY = 17,
    TA_INTERNAL_ERROR = 5000,
    TA_UNKNOWN_ERR = 65535,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_Compatibility {
    TA_COMPATIBILITY_DEFAULT = 0,
    TA_COMPATIBILITY_METASTOCK = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_MAType {
    TA_MAType_SMA = 0,
    TA_MAType_EMA = 1,
    TA_MAType_WMA = 2,
    TA_MAType_DEMA = 3,
    TA_MAType_TEMA = 4,
    TA_MAType_TRIMA = 5,
    TA_MAType_KAMA = 6,
    TA_MAType_MAMA = 7,
    TA_MAType_T3 = 8,
    TA_MAType_HMA = 9,
    TA_MAType_DISABLED = 10,
    TA_MAType_DEFAULT = 11,
    TA_MAType_ZLEMA = 12,
    TA_MAType_RMA = 13,
    TA_MAType_VIDYA = 14,
    TA_MAType_ALMA = 15,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_FuncUnstId {
    TA_FUNC_UNST_ADX = 0,
    TA_FUNC_UNST_UNUSED_1 = 1,
    TA_FUNC_UNST_ATR = 2,
    TA_FUNC_UNST_CMO = 3,
    TA_FUNC_UNST_DX = 4,
    TA_FUNC_UNST_EMA = 5,
    TA_FUNC_UNST_HT_DCPERIOD = 6,
    TA_FUNC_UNST_HT_DCPHASE = 7,
    TA_FUNC_UNST_HT_PHASOR = 8,
    TA_FUNC_UNST_HT_SINE = 9,
    TA_FUNC_UNST_HT_TRENDLINE = 10,
    TA_FUNC_UNST_HT_TRENDMODE = 11,
    TA_FUNC_UNST_UNUSED_12 = 12,
    TA_FUNC_UNST_KAMA = 13,
    TA_FUNC_UNST_MAMA = 14,
    TA_FUNC_UNST_UNUSED_15 = 15,
    TA_FUNC_UNST_MINUS_DI = 16,
    TA_FUNC_UNST_MINUS_DM = 17,
    TA_FUNC_UNST_NATR = 18,
    TA_FUNC_UNST_PLUS_DI = 19,
    TA_FUNC_UNST_PLUS_DM = 20,
    TA_FUNC_UNST_RSI = 21,
    TA_FUNC_UNST_UNUSED_22 = 22,
    TA_FUNC_UNST_T3 = 23,
    TA_FUNC_UNST_RMA = 24,
    TA_FUNC_UNST_HA = 25,
    TA_FUNC_UNST_RVI = 26,
    TA_FUNC_UNST_FRAMA = 27,
    TA_FUNC_UNST_MCGD = 28,
    TA_FUNC_UNST_VIDYA = 29,
    TA_FUNC_UNST_STC = 30,
    TA_FUNC_UNST_SWAK_GAUSS = 31,
    TA_FUNC_UNST_SWAK_BUTTER = 32,
    TA_FUNC_UNST_SWAK_HP = 33,
    TA_FUNC_UNST_SWAK_2PHP = 34,
    TA_FUNC_UNST_SWAK_BP = 35,
    TA_FUNC_UNST_FISHER = 36,
    TA_FUNC_UNST_ALL = 65535,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_RangeType {
    TA_RangeType_RealBody = 0,
    TA_RangeType_HighLow = 1,
    TA_RangeType_Shadows = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_CandleSettingType {
    TA_BodyLong = 0,
    TA_BodyVeryLong = 1,
    TA_BodyShort = 2,
    TA_BodyDoji = 3,
    TA_ShadowLong = 4,
    TA_ShadowVeryLong = 5,
    TA_ShadowShort = 6,
    TA_ShadowVeryShort = 7,
    TA_Near = 8,
    TA_Far = 9,
    TA_Equal = 10,
    TA_AllCandleSettings = 11,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_InputParameterType {
    TA_Input_Price = 0,
    TA_Input_Real = 1,
    TA_Input_Integer = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_OptInputParameterType {
    TA_OptInput_RealRange = 0,
    TA_OptInput_RealList = 1,
    TA_OptInput_IntegerRange = 2,
    TA_OptInput_IntegerList = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TA_OutputParameterType {
    TA_Output_Real = 0,
    TA_Output_Integer = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_StringTable {
    pub size: c_uint,
    pub string: *mut *const c_char,
    pub hiddenData: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_RetCodeInfo {
    pub enumStr: *const c_char,
    pub infoStr: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_FuncInfo {
    pub name: *const c_char,
    pub group: *const c_char,
    pub hint: *const c_char,
    pub flags: TA_FuncFlags,
    pub nbInput: c_uint,
    pub nbOptInput: c_uint,
    pub nbOutput: c_uint,
    pub handle: *const TA_FuncHandle,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_RealRange {
    pub min: TA_Real,
    pub max: TA_Real,
    pub precision: TA_Integer,
    pub suggested_start: TA_Real,
    pub suggested_end: TA_Real,
    pub suggested_increment: TA_Real,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_IntegerRange {
    pub min: TA_Integer,
    pub max: TA_Integer,
    pub suggested_start: TA_Integer,
    pub suggested_end: TA_Integer,
    pub suggested_increment: TA_Integer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_RealDataPair {
    pub value: TA_Real,
    pub string: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_IntegerDataPair {
    pub value: TA_Integer,
    pub string: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_RealList {
    pub data: *const TA_RealDataPair,
    pub nbElement: c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_IntegerList {
    pub data: *const TA_IntegerDataPair,
    pub nbElement: c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_InputParameterInfo {
    pub type_: TA_InputParameterType,
    pub paramName: *const c_char,
    pub flags: TA_InputFlags,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_OptInputParameterInfo {
    pub type_: TA_OptInputParameterType,
    pub paramName: *const c_char,
    pub flags: TA_OptInputFlags,
    pub displayName: *const c_char,
    pub dataSet: *const c_void,
    pub defaultValue: TA_Real,
    pub hint: *const c_char,
    pub helpFile: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_OutputParameterInfo {
    pub type_: TA_OutputParameterType,
    pub paramName: *const c_char,
    pub flags: TA_OutputFlags,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TA_ParamHolder {
    pub hiddenData: *mut c_void,
}

pub type TA_CallForEachFunc =
    Option<unsafe extern "C" fn(funcInfo: *const TA_FuncInfo, opaqueData: *mut c_void)>;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use core::mem::{offset_of, size_of};

    use super::*;

    #[test]
    fn abi_manifest_struct_sizes_and_offsets_match() {
        assert_eq!(GENERATED_FUNCTION_COUNT, 3122);
        assert_eq!(GENERATED_STREAM_TYPE_COUNT, 237);

        assert_eq!(size_of::<TA_FuncInfo>(), 48);
        assert_eq!(offset_of!(TA_FuncInfo, name), 0);
        assert_eq!(offset_of!(TA_FuncInfo, group), 8);
        assert_eq!(offset_of!(TA_FuncInfo, hint), 16);
        assert_eq!(offset_of!(TA_FuncInfo, flags), 24);
        assert_eq!(offset_of!(TA_FuncInfo, nbInput), 28);
        assert_eq!(offset_of!(TA_FuncInfo, nbOptInput), 32);
        assert_eq!(offset_of!(TA_FuncInfo, nbOutput), 36);
        assert_eq!(offset_of!(TA_FuncInfo, handle), 40);

        assert_eq!(size_of::<TA_InputParameterInfo>(), 24);
        assert_eq!(size_of::<TA_IntegerDataPair>(), 16);
        assert_eq!(size_of::<TA_IntegerList>(), 16);
        assert_eq!(size_of::<TA_IntegerRange>(), 20);
        assert_eq!(size_of::<TA_OptInputParameterInfo>(), 64);
        assert_eq!(size_of::<TA_OutputParameterInfo>(), 24);
        assert_eq!(size_of::<TA_ParamHolder>(), 8);
        assert_eq!(size_of::<TA_RealDataPair>(), 16);
        assert_eq!(size_of::<TA_RealList>(), 16);
        assert_eq!(size_of::<TA_RealRange>(), 48);
        assert_eq!(size_of::<TA_RetCodeInfo>(), 16);
        assert_eq!(size_of::<TA_StringTable>(), 24);
    }

    #[test]
    fn static_c_library_links_and_executes_sma() {
        // SAFETY: TA_Initialize and TA_SMA are called with valid stack buffers.
        unsafe {
            assert_eq!(TA_Initialize(), TA_RetCode::TA_SUCCESS);
            let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
            let mut out = [0.0_f64; 5];
            let mut beg = 0_i32;
            let mut nb = 0_i32;
            let rc = TA_SMA(
                0,
                4,
                input.as_ptr(),
                3,
                &raw mut beg,
                &raw mut nb,
                out.as_mut_ptr(),
            );
            assert_eq!(rc, TA_RetCode::TA_SUCCESS);
            assert_eq!(beg, 2);
            assert_eq!(nb, 3);
            assert!((out[0] - 2.0).abs() < 1e-12);
            assert!((out[1] - 3.0).abs() < 1e-12);
            assert!((out[2] - 4.0).abs() < 1e-12);
        }
    }
}
