//! Core domain types and enumerations for `ta-lib`.

use ta_lib_sys as sys;

/// Valid output range written by a batch or streaming TA-Lib calculation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OutRange {
    /// Index of the first input bar that produced an output value (`outBegIdx`).
    pub begin: usize,
    /// Number of valid output elements written starting at index 0 (`outNBElement`).
    pub count: usize,
}

/// Moving average algorithm type (`TA_MAType`).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum MAType {
    /// Simple Moving Average (`TA_MAType_SMA`).
    Sma = sys::TA_MAType::TA_MAType_SMA as u32,
    /// Exponential Moving Average (`TA_MAType_EMA`).
    Ema = sys::TA_MAType::TA_MAType_EMA as u32,
    /// Weighted Moving Average (`TA_MAType_WMA`).
    Wma = sys::TA_MAType::TA_MAType_WMA as u32,
    /// Double Exponential Moving Average (`TA_MAType_DEMA`).
    Dema = sys::TA_MAType::TA_MAType_DEMA as u32,
    /// Triple Exponential Moving Average (`TA_MAType_TEMA`).
    Tema = sys::TA_MAType::TA_MAType_TEMA as u32,
    /// Triangular Moving Average (`TA_MAType_TRIMA`).
    Trima = sys::TA_MAType::TA_MAType_TRIMA as u32,
    /// Kaufman Adaptive Moving Average (`TA_MAType_KAMA`).
    Kama = sys::TA_MAType::TA_MAType_KAMA as u32,
    /// MESA Adaptive Moving Average (`TA_MAType_MAMA`).
    Mama = sys::TA_MAType::TA_MAType_MAMA as u32,
    /// Tillson T3 Moving Average (`TA_MAType_T3`).
    T3 = sys::TA_MAType::TA_MAType_T3 as u32,
    /// Hull Moving Average (`TA_MAType_HMA`).
    Hma = sys::TA_MAType::TA_MAType_HMA as u32,
    /// Disabled smoothing (`TA_MAType_DISABLED`).
    Disabled = sys::TA_MAType::TA_MAType_DISABLED as u32,
    /// Indicator-specific default moving average (`TA_MAType_DEFAULT`).
    #[default]
    Default = sys::TA_MAType::TA_MAType_DEFAULT as u32,
    /// Zero-Lag Exponential Moving Average (`TA_MAType_ZLEMA`).
    Zlema = sys::TA_MAType::TA_MAType_ZLEMA as u32,
    /// Wilder's Running Moving Average (`TA_MAType_RMA`).
    Rma = sys::TA_MAType::TA_MAType_RMA as u32,
    /// Variable Index Dynamic Average (`TA_MAType_VIDYA`).
    Vidya = sys::TA_MAType::TA_MAType_VIDYA as u32,
    /// Arnaud Legoux Moving Average (`TA_MAType_ALMA`).
    Alma = sys::TA_MAType::TA_MAType_ALMA as u32,
}

/// Backward-compatible alias for [`MAType`].
pub type MovingAverageType = MAType;

#[allow(non_upper_case_globals)]
impl MAType {
    /// Legacy variant alias for [`MAType::Sma`].
    pub const SimpleMovingAverage: Self = Self::Sma;
    /// Legacy variant alias for [`MAType::Ema`].
    pub const ExponentialMovingAverage: Self = Self::Ema;
    /// Legacy variant alias for [`MAType::Wma`].
    pub const WeightedMovingAverage: Self = Self::Wma;
    /// Legacy variant alias for [`MAType::Dema`].
    pub const DoubleExponentialMovingAverage: Self = Self::Dema;
    /// Legacy variant alias for [`MAType::Tema`].
    pub const TripleExponentialMovingAverage: Self = Self::Tema;
    /// Legacy variant alias for [`MAType::Trima`].
    pub const TriangularMovingAverage: Self = Self::Trima;
    /// Legacy variant alias for [`MAType::Kama`].
    pub const KaufmanAdaptiveMovingAverage: Self = Self::Kama;
    /// Legacy variant alias for [`MAType::Mama`].
    pub const MesaAdaptiveMovingAverage: Self = Self::Mama;
    /// Legacy variant alias for [`MAType::T3`].
    pub const TripleGeneralizedDoubleExponentialMovingAverage: Self = Self::T3;
}

impl From<MAType> for sys::TA_MAType {
    fn from(value: MAType) -> Self {
        match value {
            MAType::Sma => Self::TA_MAType_SMA,
            MAType::Ema => Self::TA_MAType_EMA,
            MAType::Wma => Self::TA_MAType_WMA,
            MAType::Dema => Self::TA_MAType_DEMA,
            MAType::Tema => Self::TA_MAType_TEMA,
            MAType::Trima => Self::TA_MAType_TRIMA,
            MAType::Kama => Self::TA_MAType_KAMA,
            MAType::Mama => Self::TA_MAType_MAMA,
            MAType::T3 => Self::TA_MAType_T3,
            MAType::Hma => Self::TA_MAType_HMA,
            MAType::Disabled => Self::TA_MAType_DISABLED,
            MAType::Default => Self::TA_MAType_DEFAULT,
            MAType::Zlema => Self::TA_MAType_ZLEMA,
            MAType::Rma => Self::TA_MAType_RMA,
            MAType::Vidya => Self::TA_MAType_VIDYA,
            MAType::Alma => Self::TA_MAType_ALMA,
        }
    }
}

impl From<sys::TA_MAType> for MAType {
    fn from(value: sys::TA_MAType) -> Self {
        match value {
            sys::TA_MAType::TA_MAType_SMA => Self::Sma,
            sys::TA_MAType::TA_MAType_EMA => Self::Ema,
            sys::TA_MAType::TA_MAType_WMA => Self::Wma,
            sys::TA_MAType::TA_MAType_DEMA => Self::Dema,
            sys::TA_MAType::TA_MAType_TEMA => Self::Tema,
            sys::TA_MAType::TA_MAType_TRIMA => Self::Trima,
            sys::TA_MAType::TA_MAType_KAMA => Self::Kama,
            sys::TA_MAType::TA_MAType_MAMA => Self::Mama,
            sys::TA_MAType::TA_MAType_T3 => Self::T3,
            sys::TA_MAType::TA_MAType_HMA => Self::Hma,
            sys::TA_MAType::TA_MAType_DISABLED => Self::Disabled,
            sys::TA_MAType::TA_MAType_DEFAULT => Self::Default,
            sys::TA_MAType::TA_MAType_ZLEMA => Self::Zlema,
            sys::TA_MAType::TA_MAType_RMA => Self::Rma,
            sys::TA_MAType::TA_MAType_VIDYA => Self::Vidya,
            sys::TA_MAType::TA_MAType_ALMA => Self::Alma,
        }
    }
}

/// Calculation compatibility mode (`TA_Compatibility`).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Compatibility {
    /// Standard TA-Lib calculation mode (`TA_COMPATIBILITY_DEFAULT`).
    #[default]
    Default = sys::TA_Compatibility::TA_COMPATIBILITY_DEFAULT as u32,
    /// MetaStock compatibility mode (`TA_COMPATIBILITY_METASTOCK`).
    Metastock = sys::TA_Compatibility::TA_COMPATIBILITY_METASTOCK as u32,
}

impl From<Compatibility> for sys::TA_Compatibility {
    fn from(value: Compatibility) -> Self {
        match value {
            Compatibility::Default => Self::TA_COMPATIBILITY_DEFAULT,
            Compatibility::Metastock => Self::TA_COMPATIBILITY_METASTOCK,
        }
    }
}

impl From<sys::TA_Compatibility> for Compatibility {
    fn from(value: sys::TA_Compatibility) -> Self {
        match value {
            sys::TA_Compatibility::TA_COMPATIBILITY_DEFAULT => Self::Default,
            sys::TA_Compatibility::TA_COMPATIBILITY_METASTOCK => Self::Metastock,
        }
    }
}

/// Range measurement type for candlestick settings (`TA_RangeType`).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RangeType {
    /// Open-to-close real body (`TA_RangeType_RealBody`).
    RealBody = sys::TA_RangeType::TA_RangeType_RealBody as u32,
    /// High-to-low total range (`TA_RangeType_HighLow`).
    HighLow = sys::TA_RangeType::TA_RangeType_HighLow as u32,
    /// Combined upper and lower shadows (`TA_RangeType_Shadows`).
    Shadows = sys::TA_RangeType::TA_RangeType_Shadows as u32,
}

impl From<RangeType> for sys::TA_RangeType {
    fn from(value: RangeType) -> Self {
        match value {
            RangeType::RealBody => Self::TA_RangeType_RealBody,
            RangeType::HighLow => Self::TA_RangeType_HighLow,
            RangeType::Shadows => Self::TA_RangeType_Shadows,
        }
    }
}

/// Candlestick setting classification (`TA_CandleSettingType`).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CandleSettingType {
    /// Long real body.
    BodyLong = sys::TA_CandleSettingType::TA_BodyLong as u32,
    /// Very long real body.
    BodyVeryLong = sys::TA_CandleSettingType::TA_BodyVeryLong as u32,
    /// Short real body.
    BodyShort = sys::TA_CandleSettingType::TA_BodyShort as u32,
    /// Doji body threshold.
    BodyDoji = sys::TA_CandleSettingType::TA_BodyDoji as u32,
    /// Long shadow.
    ShadowLong = sys::TA_CandleSettingType::TA_ShadowLong as u32,
    /// Very long shadow.
    ShadowVeryLong = sys::TA_CandleSettingType::TA_ShadowVeryLong as u32,
    /// Short shadow.
    ShadowShort = sys::TA_CandleSettingType::TA_ShadowShort as u32,
    /// Very short shadow.
    ShadowVeryShort = sys::TA_CandleSettingType::TA_ShadowVeryShort as u32,
    /// Near price distance threshold.
    Near = sys::TA_CandleSettingType::TA_Near as u32,
    /// Far price distance threshold.
    Far = sys::TA_CandleSettingType::TA_Far as u32,
    /// Equal price distance threshold.
    Equal = sys::TA_CandleSettingType::TA_Equal as u32,
    /// Apply to all candle settings (`TA_AllCandleSettings`).
    AllCandleSettings = sys::TA_CandleSettingType::TA_AllCandleSettings as u32,
}

impl From<CandleSettingType> for sys::TA_CandleSettingType {
    fn from(value: CandleSettingType) -> Self {
        match value {
            CandleSettingType::BodyLong => Self::TA_BodyLong,
            CandleSettingType::BodyVeryLong => Self::TA_BodyVeryLong,
            CandleSettingType::BodyShort => Self::TA_BodyShort,
            CandleSettingType::BodyDoji => Self::TA_BodyDoji,
            CandleSettingType::ShadowLong => Self::TA_ShadowLong,
            CandleSettingType::ShadowVeryLong => Self::TA_ShadowVeryLong,
            CandleSettingType::ShadowShort => Self::TA_ShadowShort,
            CandleSettingType::ShadowVeryShort => Self::TA_ShadowVeryShort,
            CandleSettingType::Near => Self::TA_Near,
            CandleSettingType::Far => Self::TA_Far,
            CandleSettingType::Equal => Self::TA_Equal,
            CandleSettingType::AllCandleSettings => Self::TA_AllCandleSettings,
        }
    }
}

/// Unstable period function identifier (`TA_FuncUnstId`).
pub type FuncUnstId = sys::TA_FuncUnstId;

impl TryFrom<sys::TA_RangeType> for RangeType {
    type Error = crate::Error;
    fn try_from(value: sys::TA_RangeType) -> Result<Self, Self::Error> {
        Ok(match value {
            sys::TA_RangeType::TA_RangeType_RealBody => Self::RealBody,
            sys::TA_RangeType::TA_RangeType_HighLow => Self::HighLow,
            sys::TA_RangeType::TA_RangeType_Shadows => Self::Shadows,
        })
    }
}

impl TryFrom<sys::TA_CandleSettingType> for CandleSettingType {
    type Error = crate::Error;
    fn try_from(value: sys::TA_CandleSettingType) -> Result<Self, Self::Error> {
        Ok(match value {
            sys::TA_CandleSettingType::TA_BodyLong => Self::BodyLong,
            sys::TA_CandleSettingType::TA_BodyVeryLong => Self::BodyVeryLong,
            sys::TA_CandleSettingType::TA_BodyShort => Self::BodyShort,
            sys::TA_CandleSettingType::TA_BodyDoji => Self::BodyDoji,
            sys::TA_CandleSettingType::TA_ShadowLong => Self::ShadowLong,
            sys::TA_CandleSettingType::TA_ShadowVeryLong => Self::ShadowVeryLong,
            sys::TA_CandleSettingType::TA_ShadowShort => Self::ShadowShort,
            sys::TA_CandleSettingType::TA_ShadowVeryShort => Self::ShadowVeryShort,
            sys::TA_CandleSettingType::TA_Near => Self::Near,
            sys::TA_CandleSettingType::TA_Far => Self::Far,
            sys::TA_CandleSettingType::TA_Equal => Self::Equal,
            sys::TA_CandleSettingType::TA_AllCandleSettings => Self::AllCandleSettings,
        })
    }
}
