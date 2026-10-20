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

    fn close(actual: f64, expected: f64) {
        assert!(
            actual == expected
                || (actual.is_nan() && expected.is_nan())
                || (actual - expected).abs() <= 1e-8 * expected.abs().max(1.0),
            "actual={actual}, expected={expected}"
        );
    }

    macro_rules! check_indicator {
        ($test:ident, $batch:ident, $single:ident, $into:ident, $single_into:ident,
         $lookback:ident, $shift:ident, $stream:ident,
         [$($input:ident),+], [$($opt:expr),*], [$($out:ident: $ty:ty: $idx:tt),+], $extract:expr) => {
            #[test]
            fn $test() {
                crate::initialize().expect("initialize TA-Lib");
                // Exactly representable inputs isolate precision conversion from algorithm behavior.
                let data: Vec<f64> = (0..192).map(|i| 0.25 + (i % 19) as f64 / 64.0).collect();
                let data32: Vec<f32> = data.iter().map(|&v| v as f32).collect();
                $(let $input = data.as_slice();)+
                let expected = $batch($($input,)+ $($opt,)*).unwrap();
                let single = $single($({let _ = $input; data32.as_slice()},)+ $($opt,)*).unwrap();
                let warmup = $lookback($($opt,)*).unwrap();
                let begin = expected.begin_index();
                assert_eq!(begin, warmup);
                assert!(begin < data.len() - 1);
                let display_shift = |idx| $shift($($opt,)* idx).unwrap();
                $(assert_eq!(display_shift($idx), 0);)+
                $(let mut $out = vec![<$ty>::default(); data.len()];)+
                let range = $into($($input,)+ $($opt,)* $(&mut $out,)+).unwrap();
                assert_eq!(range.begin, begin);
                assert_eq!(range.count, data.len() - begin);
                $({
                    assert_eq!(expected.$idx.len(), range.count);
                    assert_eq!(single.$idx.len(), range.count);
                    for j in 0..range.count {
                        close($out[j] as f64, expected.$idx[j] as f64);
                        close(single.$idx[j] as f64, expected.$idx[j] as f64);
                    }
                })+
                let range32 = $single_into($({let _ = $input; data32.as_slice()},)+ $($opt,)* $(&mut $out,)+).unwrap();
                assert_eq!(range, range32);
                $(for j in 0..range.count { close($out[j] as f64, expected.$idx[j] as f64); })+
                let (filled, filled_range) = $stream::open_and_fill($($input,)+ $($opt,)* $(&mut $out,)+).unwrap();
                assert_eq!(filled_range, range);
                $(for j in 0..range.count { close($out[j] as f64, expected.$idx[j] as f64); })+
                let filled_value = ($extract)(filled.value().unwrap());
                $(close(filled_value[$idx], expected.$idx[range.count - 1] as f64);)+
                let split = data.len() - 1;
                let (mut state, _) = $stream::open($(&$input[..split],)+ $($opt,)*).unwrap();
                let snapshot = ($extract)(state.value().unwrap());
                let preview = ($extract)(state.peek($({let _ = $input; data[split]},)+).unwrap());
                assert_eq!(snapshot, ($extract)(state.value().unwrap()));
                let mut cloned = state.try_clone().unwrap();
                let latest = ($extract)(state.update($({let _ = $input; data[split]},)+).unwrap());
                let cloned_latest = ($extract)(cloned.update($({let _ = $input; data[split]},)+).unwrap());
                $({
                    close(preview[$idx], expected.$idx[range.count - 1] as f64);
                    close(latest[$idx], expected.$idx[range.count - 1] as f64);
                    close(cloned_latest[$idx], latest[$idx]);
                })+
            }
        };
    }

    trait BeginIndex {
        fn begin_index(&self) -> usize;
    }
    impl BeginIndex for (Vec<f64>, usize) {
        fn begin_index(&self) -> usize {
            self.1
        }
    }

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
        crate::initialize().unwrap();
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
