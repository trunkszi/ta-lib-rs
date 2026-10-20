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
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };
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
        initialize_test_runtime();
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
