//! Batch and streaming statistic indicators.

crate::ta_indicators! {
    /// Compute BETA (Beta).
    beta => BETA, BetaStream {
        inputs: [inreal0, inreal1],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute CORREL (Pearson's Correlation Coefficient (r)).
    correl => CORREL, CorrelStream {
        inputs: [inreal0, inreal1],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute KURTOSIS (Rolling Excess Kurtosis).
    kurtosis => KURTOSIS, KurtosisStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute LINEARREG (Linear Regression).
    linearreg => LINEARREG, LinearregStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute LINEARREG_ANGLE (Linear Regression Angle).
    linearreg_angle => LINEARREG_ANGLE, LinearregAngleStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute LINEARREG_INTERCEPT (Linear Regression Intercept).
    linearreg_intercept => LINEARREG_INTERCEPT, LinearregInterceptStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute LINEARREG_SLOPE (Linear Regression Slope).
    linearreg_slope => LINEARREG_SLOPE, LinearregSlopeStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MEDIAN (Rolling Median).
    median => MEDIAN, MedianStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute PERCENTILE (Percentile (nearest rank)).
    percentile => PERCENTILE, PercentileStream {
        inputs: [inreal],
        opts: [time_period: i32, percentile: f64],
        outs: [outreal: f64],
    };

    /// Compute PERCENTRANK (Percent Rank).
    percentrank => PERCENTRANK, PercentrankStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute STDDEV (Standard Deviation).
    stddev => STDDEV, StddevStream {
        inputs: [inreal],
        opts: [time_period: i32, deviations: f64],
        outs: [outreal: f64],
    };

    /// Compute TSF (Time Series Forecast).
    tsf => TSF, TsfStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute VAR (Variance).
    var => VAR, VarStream {
        inputs: [inreal],
        opts: [time_period: i32, deviations: f64],
        outs: [outreal: f64],
    };

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };
    check_indicator!(beta_batch_and_stream, beta, beta_f32, beta_into, beta_f32_into, beta_lookback, beta_display_shift, BetaStream, [inreal0, inreal1], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(correl_batch_and_stream, correl, correl_f32, correl_into, correl_f32_into, correl_lookback, correl_display_shift, CorrelStream, [inreal0, inreal1], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(kurtosis_batch_and_stream, kurtosis, kurtosis_f32, kurtosis_into, kurtosis_f32_into, kurtosis_lookback, kurtosis_display_shift, KurtosisStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(linearreg_batch_and_stream, linearreg, linearreg_f32, linearreg_into, linearreg_f32_into, linearreg_lookback, linearreg_display_shift, LinearregStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(linearreg_angle_batch_and_stream, linearreg_angle, linearreg_angle_f32, linearreg_angle_into, linearreg_angle_f32_into, linearreg_angle_lookback, linearreg_angle_display_shift, LinearregAngleStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(linearreg_intercept_batch_and_stream, linearreg_intercept, linearreg_intercept_f32, linearreg_intercept_into, linearreg_intercept_f32_into, linearreg_intercept_lookback, linearreg_intercept_display_shift, LinearregInterceptStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(linearreg_slope_batch_and_stream, linearreg_slope, linearreg_slope_f32, linearreg_slope_into, linearreg_slope_f32_into, linearreg_slope_lookback, linearreg_slope_display_shift, LinearregSlopeStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(median_batch_and_stream, median, median_f32, median_into, median_f32_into, median_lookback, median_display_shift, MedianStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(percentile_batch_and_stream, percentile, percentile_f32, percentile_into, percentile_f32_into, percentile_lookback, percentile_display_shift, PercentileStream, [inreal], [Some(8), Some(50.0)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(percentrank_batch_and_stream, percentrank, percentrank_f32, percentrank_into, percentrank_f32_into, percentrank_lookback, percentrank_display_shift, PercentrankStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(stddev_batch_and_stream, stddev, stddev_f32, stddev_into, stddev_f32_into, stddev_lookback, stddev_display_shift, StddevStream, [inreal], [Some(8), Some(1.0)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(tsf_batch_and_stream, tsf, tsf_f32, tsf_into, tsf_f32_into, tsf_lookback, tsf_display_shift, TsfStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(var_batch_and_stream, var, var_f32, var_into, var_f32_into, var_lookback, var_display_shift, VarStream, [inreal], [Some(8), Some(1.0)], [outreal: f64: 0], |v: f64| vec![v]);

    #[test]
    fn linear_fit_and_population_variance() {
        initialize_test_runtime();
        let input = [1.0, 3.0, 5.0, 7.0, 9.0];
        assert_eq!(linearreg_slope(&input, Some(3)).unwrap(), (vec![2.0; 3], 2));
        assert_eq!(linearreg(&input, Some(3)).unwrap().0, vec![5.0, 7.0, 9.0]);
        assert_eq!(tsf(&input, Some(3)).unwrap().0, vec![7.0, 9.0, 11.0]);
        for variance in var(&input, Some(3), Some(1.0)).unwrap().0 {
            close(variance, 8.0 / 3.0);
        }
        assert_eq!(correl(&input, &input, Some(3)).unwrap().0, vec![1.0; 3]);
    }
}
