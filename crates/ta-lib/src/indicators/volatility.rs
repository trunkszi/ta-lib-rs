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

    /// Compute BBW (Bollinger BandWidth).
    bbw => BBW, BbwStream {
        inputs: [inreal],
        opts: [time_period: i32, deviations_up: f64, deviations_down: f64, ma_type: MAType],
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

    /// Compute PERCENTB (Bollinger Bands %B).
    percentb => PERCENTB, PercentbStream {
        inputs: [inreal],
        opts: [time_period: i32, deviations_up: f64, deviations_down: f64, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute ROGERSSATCHELL (Rogers-Satchell Volatility).
    rogerssatchell => ROGERSSATCHELL, RogerssatchellStream {
        inputs: [open, high, low, close],
        opts: [time_period: i32, annualization: f64],
        outs: [outreal: f64],
    };

    /// Compute RVI (Relative Volatility Index).
    rvi => RVI, RviStream {
        inputs: [inreal],
        opts: [time_period: i32, stddev_period: i32],
        outs: [outreal: f64],
    };

    /// Compute RVIR (Relative Volatility Index, refined high/low form).
    rvir => RVIR, RvirStream {
        inputs: [high, low],
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
