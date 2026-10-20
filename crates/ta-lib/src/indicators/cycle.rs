//! Batch and streaming cycle indicators.

crate::ta_indicators! {
    /// Compute HT_DCPERIOD (Hilbert Transform - Dominant Cycle Period).
    ht_dcperiod => HT_DCPERIOD, HtDcperiodStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute HT_DCPHASE (Hilbert Transform - Dominant Cycle Phase).
    ht_dcphase => HT_DCPHASE, HtDcphaseStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute HT_PHASOR (Hilbert Transform - Phasor Components).
    ht_phasor => HT_PHASOR, HtPhasorStream {
        inputs: [inreal],
        opts: [],
        outs: [outinphase: f64, outquadrature: f64],
    };

    /// Compute HT_SINE (Hilbert Transform - SineWave).
    ht_sine => HT_SINE, HtSineStream {
        inputs: [inreal],
        opts: [],
        outs: [outsine: f64, outleadsine: f64],
    };

    /// Compute HT_TRENDMODE (Hilbert Transform - Trend vs Cycle Mode).
    ht_trendmode => HT_TRENDMODE, HtTrendmodeStream {
        inputs: [inreal],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute SWAK_2PHP (Swiss Army Knife - Two-Pole High-Pass Filter).
    swak_2php => SWAK_2PHP, Swak2phpStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute SWAK_BP (Swiss Army Knife - Band-Pass Filter).
    swak_bp => SWAK_BP, SwakBpStream {
        inputs: [inreal],
        opts: [time_period: i32, delta: f64],
        outs: [outreal: f64],
    };

    /// Compute SWAK_HP (Swiss Army Knife - High-Pass Filter).
    swak_hp => SWAK_HP, SwakHpStream {
        inputs: [inreal],
        opts: [time_period: i32],
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
                crate::indicators::math_operators::initialize_test_runtime();
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
    impl BeginIndex for (Vec<i32>, usize) {
        fn begin_index(&self) -> usize {
            self.1
        }
    }
    impl BeginIndex for (Vec<f64>, Vec<f64>, usize) {
        fn begin_index(&self) -> usize {
            self.2
        }
    }

    check_indicator!(ht_dcperiod_batch_and_stream, ht_dcperiod, ht_dcperiod_f32, ht_dcperiod_into, ht_dcperiod_f32_into, ht_dcperiod_lookback, ht_dcperiod_display_shift, HtDcperiodStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_dcphase_batch_and_stream, ht_dcphase, ht_dcphase_f32, ht_dcphase_into, ht_dcphase_f32_into, ht_dcphase_lookback, ht_dcphase_display_shift, HtDcphaseStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_phasor_batch_and_stream, ht_phasor, ht_phasor_f32, ht_phasor_into, ht_phasor_f32_into, ht_phasor_lookback, ht_phasor_display_shift, HtPhasorStream, [inreal], [], [outinphase: f64: 0, outquadrature: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_sine_batch_and_stream, ht_sine, ht_sine_f32, ht_sine_into, ht_sine_f32_into, ht_sine_lookback, ht_sine_display_shift, HtSineStream, [inreal], [], [outsine: f64: 0, outleadsine: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_trendmode_batch_and_stream, ht_trendmode, ht_trendmode_f32, ht_trendmode_into, ht_trendmode_f32_into, ht_trendmode_lookback, ht_trendmode_display_shift, HtTrendmodeStream, [inreal], [], [outinteger: i32: 0], |v: i32| vec![v as f64]);

    check_indicator!(swak_2php_batch_and_stream, swak_2php, swak_2php_f32, swak_2php_into, swak_2php_f32_into, swak_2php_lookback, swak_2php_display_shift, Swak2phpStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(swak_bp_batch_and_stream, swak_bp, swak_bp_f32, swak_bp_into, swak_bp_f32_into, swak_bp_lookback, swak_bp_display_shift, SwakBpStream, [inreal], [Some(8), Some(0.2)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(swak_hp_batch_and_stream, swak_hp, swak_hp_f32, swak_hp_into, swak_hp_f32_into, swak_hp_lookback, swak_hp_display_shift, SwakHpStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);
}
