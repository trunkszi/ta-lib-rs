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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };
    check_indicator!(ht_dcperiod_batch_and_stream, ht_dcperiod, ht_dcperiod_f32, ht_dcperiod_into, ht_dcperiod_f32_into, ht_dcperiod_lookback, ht_dcperiod_display_shift, HtDcperiodStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_dcphase_batch_and_stream, ht_dcphase, ht_dcphase_f32, ht_dcphase_into, ht_dcphase_f32_into, ht_dcphase_lookback, ht_dcphase_display_shift, HtDcphaseStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_phasor_batch_and_stream, ht_phasor, ht_phasor_f32, ht_phasor_into, ht_phasor_f32_into, ht_phasor_lookback, ht_phasor_display_shift, HtPhasorStream, [inreal], [], [outinphase: f64: 0, outquadrature: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_sine_batch_and_stream, ht_sine, ht_sine_f32, ht_sine_into, ht_sine_f32_into, ht_sine_lookback, ht_sine_display_shift, HtSineStream, [inreal], [], [outsine: f64: 0, outleadsine: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_trendmode_batch_and_stream, ht_trendmode, ht_trendmode_f32, ht_trendmode_into, ht_trendmode_f32_into, ht_trendmode_lookback, ht_trendmode_display_shift, HtTrendmodeStream, [inreal], [], [outinteger: i32: 0], |v: i32| vec![v as f64]);
}
