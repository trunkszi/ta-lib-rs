//! Batch and streaming math transform indicators.

crate::ta_indicators! {
    /// Compute ACOS (Vector Trigonometric ACos).
    acos => ACOS, AcosStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute ASIN (Vector Trigonometric ASin).
    asin => ASIN, AsinStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute ATAN (Vector Trigonometric ATan).
    atan => ATAN, AtanStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute CEIL (Vector Ceil).
    ceil => CEIL, CeilStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute COS (Vector Trigonometric Cos).
    cos => COS, CosStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute COSH (Vector Trigonometric Cosh).
    cosh => COSH, CoshStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute EXP (Vector Arithmetic Exp).
    exp => EXP, ExpStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute FLOOR (Vector Floor).
    floor => FLOOR, FloorStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute LN (Vector Log Natural).
    ln => LN, LnStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute LOG10 (Vector Log10).
    log10 => LOG10, Log10Stream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute SIN (Vector Trigonometric Sin).
    sin => SIN, SinStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute SINH (Vector Trigonometric Sinh).
    sinh => SINH, SinhStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute SQRT (Vector Square Root).
    sqrt => SQRT, SqrtStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute TAN (Vector Trigonometric Tan).
    tan => TAN, TanStream {
        inputs: [inreal],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute TANH (Vector Trigonometric Tanh).
    tanh => TANH, TanhStream {
        inputs: [inreal],
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

    check_indicator!(acos_batch_and_stream, acos, acos_f32, acos_into, acos_f32_into, acos_lookback, acos_display_shift, AcosStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(asin_batch_and_stream, asin, asin_f32, asin_into, asin_f32_into, asin_lookback, asin_display_shift, AsinStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(atan_batch_and_stream, atan, atan_f32, atan_into, atan_f32_into, atan_lookback, atan_display_shift, AtanStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ceil_batch_and_stream, ceil, ceil_f32, ceil_into, ceil_f32_into, ceil_lookback, ceil_display_shift, CeilStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(cos_batch_and_stream, cos, cos_f32, cos_into, cos_f32_into, cos_lookback, cos_display_shift, CosStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(cosh_batch_and_stream, cosh, cosh_f32, cosh_into, cosh_f32_into, cosh_lookback, cosh_display_shift, CoshStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(exp_batch_and_stream, exp, exp_f32, exp_into, exp_f32_into, exp_lookback, exp_display_shift, ExpStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(floor_batch_and_stream, floor, floor_f32, floor_into, floor_f32_into, floor_lookback, floor_display_shift, FloorStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ln_batch_and_stream, ln, ln_f32, ln_into, ln_f32_into, ln_lookback, ln_display_shift, LnStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(log10_batch_and_stream, log10, log10_f32, log10_into, log10_f32_into, log10_lookback, log10_display_shift, Log10Stream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(sin_batch_and_stream, sin, sin_f32, sin_into, sin_f32_into, sin_lookback, sin_display_shift, SinStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(sinh_batch_and_stream, sinh, sinh_f32, sinh_into, sinh_f32_into, sinh_lookback, sinh_display_shift, SinhStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(sqrt_batch_and_stream, sqrt, sqrt_f32, sqrt_into, sqrt_f32_into, sqrt_lookback, sqrt_display_shift, SqrtStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(tan_batch_and_stream, tan, tan_f32, tan_into, tan_f32_into, tan_lookback, tan_display_shift, TanStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(tanh_batch_and_stream, tanh, tanh_f32, tanh_into, tanh_f32_into, tanh_lookback, tanh_display_shift, TanhStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    #[test]
    fn analytic_transforms() {
        crate::initialize().unwrap();
        let input = [0.0, 0.25, 0.5, 1.0];
        let actual = sin(&input).unwrap();
        assert_eq!(actual.1, 0);
        for (actual, input) in actual.0.iter().zip(input) {
            close(*actual, input.sin());
        }
        assert_eq!(sqrt(&[1.0, 4.0, 9.0]).unwrap().0, vec![1.0, 2.0, 3.0]);
        assert_eq!(ceil(&[-1.5, 0.5, 2.0]).unwrap().0, vec![-1.0, 1.0, 2.0]);
        assert_eq!(floor(&[-1.5, 0.5, 2.0]).unwrap().0, vec![-2.0, 0.0, 2.0]);
        for (actual, input) in ln(&[1.0, 2.0, 4.0])
            .unwrap()
            .0
            .iter()
            .zip([1.0_f64, 2.0, 4.0])
        {
            close(*actual, input.ln());
        }
    }
}
