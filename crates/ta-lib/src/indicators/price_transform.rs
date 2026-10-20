//! Batch and streaming price transform indicators.

// OHLC streams with four outputs retain the C ABI parameter structure.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

crate::ta_indicators! {
    /// Compute AVGDEV (Average Deviation).
    avgdev => AVGDEV, AvgdevStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute AVGPRICE (Average Price).
    avgprice => AVGPRICE, AvgpriceStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute HA (Heikin-Ashi Candles).
    ha => HA, HaStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outhaopen: f64, outhahigh: f64, outhalow: f64, outhaclose: f64],
    };

    /// Compute MEDPRICE (Median Price).
    medprice => MEDPRICE, MedpriceStream {
        inputs: [high, low],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute TYPPRICE (Typical Price).
    typprice => TYPPRICE, TyppriceStream {
        inputs: [high, low, close],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute WCLPRICE (Weighted Close Price).
    wclprice => WCLPRICE, WclpriceStream {
        inputs: [high, low, close],
        opts: [],
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
    impl BeginIndex for (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, usize) {
        fn begin_index(&self) -> usize {
            self.4
        }
    }

    check_indicator!(avgdev_batch_and_stream, avgdev, avgdev_f32, avgdev_into, avgdev_f32_into, avgdev_lookback, avgdev_display_shift, AvgdevStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(avgprice_batch_and_stream, avgprice, avgprice_f32, avgprice_into, avgprice_f32_into, avgprice_lookback, avgprice_display_shift, AvgpriceStream, [open, high, low, close], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ha_batch_and_stream, ha, ha_f32, ha_into, ha_f32_into, ha_lookback, ha_display_shift, HaStream, [open, high, low, close], [], [outhaopen: f64: 0, outhahigh: f64: 1, outhalow: f64: 2, outhaclose: f64: 3], |v: (f64,f64,f64,f64)| vec![v.0,v.1,v.2,v.3]);

    check_indicator!(medprice_batch_and_stream, medprice, medprice_f32, medprice_into, medprice_f32_into, medprice_lookback, medprice_display_shift, MedpriceStream, [high, low], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(typprice_batch_and_stream, typprice, typprice_f32, typprice_into, typprice_f32_into, typprice_lookback, typprice_display_shift, TyppriceStream, [high, low, close], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(wclprice_batch_and_stream, wclprice, wclprice_f32, wclprice_into, wclprice_f32_into, wclprice_lookback, wclprice_display_shift, WclpriceStream, [high, low, close], [], [outreal: f64: 0], |v: f64| vec![v]);

    #[test]
    fn ohlc_prices_preserve_distinct_inputs() {
        crate::indicators::math_operators::initialize_test_runtime();
        let open = [10.0, 12.0, 14.0];
        let high = [14.0, 16.0, 18.0];
        let low = [8.0, 10.0, 12.0];
        let close = [12.0, 14.0, 16.0];
        assert_eq!(
            avgprice(&open, &high, &low, &close).unwrap().0,
            vec![11.0, 13.0, 15.0]
        );
        assert_eq!(medprice(&high, &low).unwrap().0, vec![11.0, 13.0, 15.0]);
        self::close(typprice(&high, &low, &close).unwrap().0[0], 34.0 / 3.0);
        self::close(wclprice(&high, &low, &close).unwrap().0[0], 11.5);
        let (ha_open, ha_high, ha_low, ha_close, begin) = ha(&open, &high, &low, &close).unwrap();
        assert_eq!(begin, 0);
        assert_eq!(ha_close, vec![11.0, 13.0, 15.0]);
        assert_eq!(ha_open, vec![11.0, 11.0, 12.0]);
        assert_eq!(ha_high, high);
        assert_eq!(ha_low, low);
    }
}
