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

#[cfg(not(feature = "release-tag"))]
crate::ta_indicators! {
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
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };
    check_indicator!(ht_dcperiod_batch_and_stream, ht_dcperiod, ht_dcperiod_f32, ht_dcperiod_into, ht_dcperiod_f32_into, ht_dcperiod_lookback, ht_dcperiod_display_shift, HtDcperiodStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_dcphase_batch_and_stream, ht_dcphase, ht_dcphase_f32, ht_dcphase_into, ht_dcphase_f32_into, ht_dcphase_lookback, ht_dcphase_display_shift, HtDcphaseStream, [inreal], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ht_phasor_batch_and_stream, ht_phasor, ht_phasor_f32, ht_phasor_into, ht_phasor_f32_into, ht_phasor_lookback, ht_phasor_display_shift, HtPhasorStream, [inreal], [], [outinphase: f64: 0, outquadrature: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_sine_batch_and_stream, ht_sine, ht_sine_f32, ht_sine_into, ht_sine_f32_into, ht_sine_lookback, ht_sine_display_shift, HtSineStream, [inreal], [], [outsine: f64: 0, outleadsine: f64: 1], |v: (f64,f64)| vec![v.0,v.1]);

    check_indicator!(ht_trendmode_batch_and_stream, ht_trendmode, ht_trendmode_f32, ht_trendmode_into, ht_trendmode_f32_into, ht_trendmode_lookback, ht_trendmode_display_shift, HtTrendmodeStream, [inreal], [], [outinteger: i32: 0], |v: i32| vec![v as f64]);

    #[cfg(not(feature = "release-tag"))]
    check_indicator!(swak_2php_batch_and_stream, swak_2php, swak_2php_f32, swak_2php_into, swak_2php_f32_into, swak_2php_lookback, swak_2php_display_shift, Swak2phpStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);

    #[cfg(not(feature = "release-tag"))]
    check_indicator!(swak_bp_batch_and_stream, swak_bp, swak_bp_f32, swak_bp_into, swak_bp_f32_into, swak_bp_lookback, swak_bp_display_shift, SwakBpStream, [inreal], [Some(8), Some(0.2)], [outreal: f64: 0], |v: f64| vec![v]);

    #[cfg(not(feature = "release-tag"))]
    check_indicator!(swak_hp_batch_and_stream, swak_hp, swak_hp_f32, swak_hp_into, swak_hp_f32_into, swak_hp_lookback, swak_hp_display_shift, SwakHpStream, [inreal], [Some(8)], [outreal: f64: 0], |v: f64| vec![v]);
}
