//! Batch and streaming math operators indicators.

crate::ta_indicators! {
    /// Compute ADD (Vector Arithmetic Add).
    add => ADD, AddStream {
        inputs: [inreal0, inreal1],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute CUMSUM (Cumulative Sum).
    cumsum => CUMSUM, CumsumStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute DIV (Vector Arithmetic Div).
    div => DIV, DivStream {
        inputs: [inreal0, inreal1],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute MAX (Highest value over a specified period).
    max => MAX, MaxStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MAXINDEX (Index of highest value over a specified period).
    maxindex => MAXINDEX, MaxindexStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outinteger: i32],
    };

    /// Compute MIN (Lowest value over a specified period).
    min => MIN, MinStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MININDEX (Index of lowest value over a specified period).
    minindex => MININDEX, MinindexStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outinteger: i32],
    };

    /// Compute MINMAX (Lowest and highest values over a specified period).
    minmax => MINMAX, MinmaxStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outmin: f64, outmax: f64],
    };

    /// Compute MINMAXINDEX (Indexes of lowest and highest values over a specified period).
    minmaxindex => MINMAXINDEX, MinmaxindexStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outminidx: i32, outmaxidx: i32],
    };

    /// Compute MULT (Vector Arithmetic Mult).
    mult => MULT, MultStream {
        inputs: [inreal0, inreal1],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute SUB (Vector Arithmetic Subtraction).
    sub => SUB, SubStream {
        inputs: [inreal0, inreal1],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute SUM (Summation).
    sum => SUM, SumStream {
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
    impl BeginIndex for (Vec<i32>, Vec<i32>, usize) {
        fn begin_index(&self) -> usize {
            self.2
        }
    }

    check_indicator!(add_batch_and_stream, add, add_f32, add_into, add_f32_into, add_lookback, add_display_shift, AddStream, [inreal0, inreal1], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(cumsum_batch_and_stream, cumsum, cumsum_f32, cumsum_into, cumsum_f32_into, cumsum_lookback, cumsum_display_shift, CumsumStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(div_batch_and_stream, div, div_f32, div_into, div_f32_into, div_lookback, div_display_shift, DivStream, [inreal0, inreal1], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(max_batch_and_stream, max, max_f32, max_into, max_f32_into, max_lookback, max_display_shift, MaxStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(maxindex_batch_and_stream, maxindex, maxindex_f32, maxindex_into, maxindex_f32_into, maxindex_lookback, maxindex_display_shift, MaxindexStream, [inreal], [Some(8)], [outinteger: i32: 0], |v: i32| vec![v as f64]);

    check_indicator!(min_batch_and_stream, min, min_f32, min_into, min_f32_into, min_lookback, min_display_shift, MinStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(minindex_batch_and_stream, minindex, minindex_f32, minindex_into, minindex_f32_into, minindex_lookback, minindex_display_shift, MinindexStream, [inreal], [Some(8)], [outinteger: i32: 0], |v: i32| vec![v as f64]);

    check_indicator!(minmax_batch_and_stream, minmax, minmax_f32, minmax_into, minmax_f32_into, minmax_lookback, minmax_display_shift, MinmaxStream, [inreal], [Some(8)], [outmin: f64: 0, outmax: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(minmaxindex_batch_and_stream, minmaxindex, minmaxindex_f32, minmaxindex_into, minmaxindex_f32_into, minmaxindex_lookback, minmaxindex_display_shift, MinmaxindexStream, [inreal], [Some(8)], [outminidx: i32: 0, outmaxidx: i32: 1], |v: (i32,i32)| vec![v.0 as f64,v.1 as f64]);

    check_indicator!(mult_batch_and_stream, mult, mult_f32, mult_into, mult_f32_into, mult_lookback, mult_display_shift, MultStream, [inreal0, inreal1], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(sub_batch_and_stream, sub, sub_f32, sub_into, sub_f32_into, sub_lookback, sub_display_shift, SubStream, [inreal0, inreal1], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(sum_batch_and_stream, sum, sum_f32, sum_into, sum_f32_into, sum_lookback, sum_display_shift, SumStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    #[test]
    fn arithmetic_and_window_positions() {
        crate::initialize().unwrap();
        let a = [2.0, 5.0, 1.0, 7.0];
        let b = [1.0, 2.0, 4.0, 2.0];
        assert_eq!(add(&a, &b).unwrap(), (vec![3.0, 7.0, 5.0, 9.0], 0));
        assert_eq!(sub(&a, &b).unwrap().0, vec![1.0, 3.0, -3.0, 5.0]);
        assert_eq!(mult(&a, &b).unwrap().0, vec![2.0, 10.0, 4.0, 14.0]);
        assert_eq!(div(&a, &b).unwrap().0, vec![2.0, 2.5, 0.25, 3.5]);
        assert_eq!(sum(&a, Some(2)).unwrap(), (vec![7.0, 6.0, 8.0], 1));
        assert_eq!(
            minmaxindex(&a, Some(2)).unwrap(),
            (vec![0, 2, 2], vec![1, 1, 3], 1)
        );
        assert!(add(&a, &b[..2]).is_err());
        assert!(sum_into(&a, Some(2), &mut [0.0; 2]).is_err());
        assert!(sum(&[], Some(2)).is_err());
        assert!(sum_display_shift(Some(2), 1).is_err());
    }
}
