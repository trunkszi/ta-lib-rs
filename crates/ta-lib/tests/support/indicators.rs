//! Shared fixtures for indicator batch, `_into`, `f32`, and streaming parity tests.

use std::sync::Once;

/// Initialize the TA-Lib runtime once per test process.
pub(crate) fn initialize_test_runtime() {
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(|| crate::initialize().expect("initialize TA-Lib"));
}

/// Compare floating-point results, treating two NaNs as equal.
pub(crate) fn close(actual: f64, expected: f64) {
    assert!(
        actual == expected
            || (actual.is_nan() && expected.is_nan())
            || (actual - expected).abs() <= 1e-8 * expected.abs().max(1.0),
        "actual={actual}, expected={expected}"
    );
}

/// Extract the trailing begin index from a generated batch return tuple.
pub(crate) trait BeginIndex {
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
impl BeginIndex for (Vec<f64>, Vec<f64>, Vec<f64>, usize) {
    fn begin_index(&self) -> usize {
        self.3
    }
}
impl BeginIndex for (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, usize) {
    fn begin_index(&self) -> usize {
        self.4
    }
}

macro_rules! check_indicator {
    ($test:ident, $batch:ident, $single:ident, $into:ident, $single_into:ident,
     $lookback:ident, $_shift:ident, $stream:ident,
     [$($input:ident),+], [$($opt:expr),*], [$($out:ident: $ty:ty: $idx:tt),+], $extract:expr) => {
        #[test]
        fn $test() {
            initialize_test_runtime();
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
            #[cfg(not(feature = "stable"))]
            {
                let display_shift = |idx| $_shift($($opt,)* idx).unwrap();
                $(assert_eq!(display_shift($idx), 0);)+
            }
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

pub(crate) use check_indicator;
