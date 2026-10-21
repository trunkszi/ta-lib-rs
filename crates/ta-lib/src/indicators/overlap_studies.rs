//! Indicator group: overlap_studies.
// TA-Lib C signatures mirror their full parameter lists; some exceed clippy's default limit.
#![allow(clippy::too_many_arguments)]

crate::ta_indicators! {
    /// Compute ACCBANDS (Acceleration Bands).
    accbands => ACCBANDS, AccbandsStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outrealupperband: f64, outrealmiddleband: f64, outreallowerband: f64],
    };

    /// Compute BBANDS (Bollinger Bands).
    bbands => BBANDS, BbandsStream {
        inputs: [inreal],
        opts: [time_period: i32, deviations_up: f64, deviations_down: f64, ma_type: MAType],
        outs: [outrealupperband: f64, outrealmiddleband: f64, outreallowerband: f64],
    };

    /// Compute DEMA (Double Exponential Moving Average).
    dema => DEMA, DemaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute DONCHIAN (Donchian Channels).
    donchian => DONCHIAN, DonchianStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outrealupperband: f64, outrealmiddleband: f64, outreallowerband: f64],
    };

    /// Compute EMA (Exponential Moving Average).
    ema => EMA, EmaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute HMA (Hull Moving Average).
    hma => HMA, HmaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute HT_TRENDLINE (Hilbert Transform - Instantaneous Trendline).
    ht_trendline => HT_TRENDLINE, HtTrendlineStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute KAMA (Kaufman Adaptive Moving Average).
    kama => KAMA, KamaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute KC (Keltner Channels).
    kc => KC, KcStream {
        inputs: [high, low, close],
        opts: [time_period: i32, atr_period: i32, deviations: f64],
        outs: [outrealupperband: f64, outrealmiddleband: f64, outreallowerband: f64],
    };

    /// Compute MA (Moving average).
    ma => MA, MaStream {
        inputs: [inreal],
        opts: [time_period: i32, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute MAMA (MESA Adaptive Moving Average).
    mama => MAMA, MamaStream {
        inputs: [inreal],
        opts: [fast_limit: f64, slow_limit: f64],
        outs: [outmama: f64, outfama: f64],
    };

    /// Compute MAVP (Moving average with variable period).
    mavp => MAVP, MavpStream {
        inputs: [inreal, inperiods],
        opts: [minimum_period: i32, maximum_period: i32, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute MIDPOINT (MidPoint over period).
    midpoint => MIDPOINT, MidpointStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MIDPRICE (Midpoint Price over period).
    midprice => MIDPRICE, MidpriceStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute RMA (Wilder's Smoothed Moving Average).
    rma => RMA, RmaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute SAR (Parabolic SAR).
    sar => SAR, SarStream {
        inputs: [high, low],
        opts: [acceleration_factor: f64, af_maximum: f64],
        outs: [outreal: f64],
    };

    /// Compute SAREXT (Parabolic SAR - Extended).
    sarext => SAREXT, SarextStream {
        inputs: [high, low],
        opts: [start_value: f64, offset_on_reverse: f64, af_init_long: f64, af_long: f64, af_max_long: f64, af_init_short: f64, af_short: f64, af_max_short: f64],
        outs: [outreal: f64],
    };

    /// Compute SMA (Simple Moving Average).
    sma => SMA, SmaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute SUPERTREND (SuperTrend).
    supertrend => SUPERTREND, SupertrendStream {
        inputs: [high, low, close],
        opts: [time_period: i32, multiplier: f64],
        outs: [outsupertrend: f64, outtrend: i32],
    };

    /// Compute T3 (Triple Exponential Moving Average (T3)).
    t3 => T3, T3Stream {
        inputs: [inreal],
        opts: [time_period: i32, volume_factor: f64],
        outs: [outreal: f64],
    };

    /// Compute TEMA (Triple Exponential Moving Average).
    tema => TEMA, TemaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute TRIMA (Triangular Moving Average).
    trima => TRIMA, TrimaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute VWMA (Volume Weighted Moving Average).
    vwma => VWMA, VwmaStream {
        inputs: [inreal, volume],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute WMA (Weighted Moving Average).
    wma => WMA, WmaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ZLEMA (Zero-Lag Exponential Moving Average).
    zlema => ZLEMA, ZlemaStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

}

/// Backward-compatible alias for [`sma`]; accepts a bare period or `Some(period)`.
///
/// # Errors
///
/// Returns an error if `inreal` is empty or the period is out of range.
pub fn simple_moving_average(
    inreal: &[f64],
    time_period: impl Into<Option<usize>>,
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    sma(inreal, time_period.into())
}

/// Backward-compatible alias for [`ema`]; accepts a bare period or `Some(period)`.
///
/// # Errors
///
/// Returns an error if `inreal` is empty or the period is out of range.
pub fn exponential_moving_average(
    inreal: &[f64],
    time_period: impl Into<Option<usize>>,
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    ema(inreal, time_period.into())
}

/// Backward-compatible alias for [`bbands`]; parameters accept bare values or `Option`s.
///
/// # Errors
///
/// Returns an error if `inreal` is empty or any parameter is out of range.
#[allow(clippy::type_complexity)]
pub fn bollinger_bands(
    inreal: &[f64],
    time_period: impl Into<Option<usize>>,
    deviations_up: impl Into<Option<f64>>,
    deviations_down: impl Into<Option<f64>>,
    ma_type: impl Into<Option<crate::types::MAType>>,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, usize), crate::error::Error> {
    bbands(
        inreal,
        time_period.into(),
        deviations_up.into(),
        deviations_down.into(),
        ma_type.into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };

    check_indicator!(bbands_batch_and_stream, bbands, bbands_f32, bbands_into, bbands_f32_into, bbands_lookback, bbands_display_shift, BbandsStream, [inreal], [Some(5), Some(2.0), Some(2.0), Some(crate::types::MAType::Sma)], [outrealupperband: f64: 0, outrealmiddleband: f64: 1, outreallowerband: f64: 2], |v: (f64,f64,f64)| vec![v.0,v.1,v.2]);

    fn ramp(len: usize) -> Vec<f64> {
        (1..=len).map(|value| value as f64).collect()
    }

    #[test]
    fn moving_average_aliases_match_generated_functions() {
        let data = ramp(12);
        assert_eq!(
            simple_moving_average(&data, 3).unwrap(),
            sma(&data, Some(3)).unwrap()
        );
        assert_eq!(
            simple_moving_average(&data, Some(3)).unwrap(),
            sma(&data, Some(3)).unwrap()
        );
        let (values, begin) = simple_moving_average(&data, 3).unwrap();
        assert_eq!(begin, 2);
        assert_eq!(
            values,
            vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0]
        );
        assert_eq!(
            exponential_moving_average(&data, 3).unwrap(),
            ema(&data, Some(3)).unwrap()
        );
    }

    #[test]
    fn bollinger_alias_matches_generated_function() {
        let data = ramp(20);
        assert_eq!(
            bollinger_bands(&data, 5, 2.0, 2.0, crate::types::MAType::Sma).unwrap(),
            bbands(
                &data,
                Some(5),
                Some(2.0),
                Some(2.0),
                Some(crate::types::MAType::Sma)
            )
            .unwrap()
        );
    }
}
