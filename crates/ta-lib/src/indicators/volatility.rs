//! Indicator group: volatility.

crate::ta_indicators! {
    /// Compute ADR (Average Day Range).
    adr => ADR, AdrStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ATR (Average True Range).
    atr => ATR, AtrStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute CVI (Chaikin's Volatility).
    cvi => CVI, CviStream {
        inputs: [high, low],
        opts: [time_period: i32, roc_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MASSI (Mass Index).
    massi => MASSI, MassiStream {
        inputs: [high, low],
        opts: [fast_period: i32, slow_period: i32],
        outs: [outreal: f64],
    };

    /// Compute NATR (Normalized Average True Range).
    natr => NATR, NatrStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute RVI (Relative Volatility Index).
    rvi => RVI, RviStream {
        inputs: [inreal],
        opts: [time_period: i32, stddev_period: i32],
        outs: [outreal: f64],
    };

    /// Compute TRANGE (True Range).
    trange => TRANGE, TrangeStream {
        inputs: [high, low, close],
        opts: [],
        outs: [outreal: f64],
    };

}

/// Backward-compatible alias for [`atr`]; accepts a bare period or `Some(period)`.
///
/// # Errors
///
/// Returns an error if the input slices are empty, have mismatched lengths, or the period is out of
/// range.
pub fn average_true_range(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: impl Into<Option<usize>>,
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    atr(high, low, close, time_period.into())
}

/// Backward-compatible alias for [`natr`]; accepts a bare period or `Some(period)`.
///
/// # Errors
///
/// Returns an error if the input slices are empty, have mismatched lengths, or the period is out of
/// range.
pub fn normalized_average_true_range(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: impl Into<Option<usize>>,
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    natr(high, low, close, time_period.into())
}

/// Backward-compatible alias for [`trange`].
///
/// # Errors
///
/// Returns an error if the input slices are empty or have mismatched lengths.
pub fn true_range(
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    trange(high, low, close)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };

    check_indicator!(atr_batch_and_stream, atr, atr_f32, atr_into, atr_f32_into, atr_lookback, atr_display_shift, AtrStream, [high, low, close], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(natr_batch_and_stream, natr, natr_f32, natr_into, natr_f32_into, natr_lookback, natr_display_shift, NatrStream, [high, low, close], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(trange_batch_and_stream, trange, trange_f32, trange_into, trange_f32_into, trange_lookback, trange_display_shift, TrangeStream, [high, low, close], [], [outreal: f64: 0], |v: f64| vec![v]);

    fn bars(len: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let close: Vec<f64> = (1..=len).map(|value| 100.0 + value as f64).collect();
        let high: Vec<f64> = close.iter().map(|value| value + 2.0).collect();
        let low: Vec<f64> = close.iter().map(|value| value - 2.0).collect();
        (high, low, close)
    }

    #[test]
    fn volatility_aliases_match_generated_functions() {
        let (high, low, close) = bars(20);
        assert_eq!(
            average_true_range(&high, &low, &close, 5).unwrap(),
            atr(&high, &low, &close, Some(5)).unwrap()
        );
        assert_eq!(
            normalized_average_true_range(&high, &low, &close, Some(5)).unwrap(),
            natr(&high, &low, &close, Some(5)).unwrap()
        );
        assert_eq!(
            true_range(&high, &low, &close).unwrap(),
            trange(&high, &low, &close).unwrap()
        );
    }

    #[test]
    fn true_range_reports_warmup_offset() {
        let (high, low, close) = bars(10);
        let (values, begin) = true_range(&high, &low, &close).unwrap();
        assert_eq!(begin + values.len(), high.len());
        assert!(values.iter().all(|value| value.is_finite() && *value > 0.0));
    }
}
