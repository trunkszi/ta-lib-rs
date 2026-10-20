//! Indicator group: volume.
// TA-Lib C signatures mirror their full parameter lists; some exceed clippy's default limit.
#![allow(clippy::too_many_arguments)]

crate::ta_indicators! {
    /// Compute AD (Chaikin A/D Line).
    ad => AD, AdStream {
        inputs: [high, low, close, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute ADOSC (Chaikin A/D Oscillator).
    adosc => ADOSC, AdoscStream {
        inputs: [high, low, close, volume],
        opts: [fast_period: i32, slow_period: i32],
        outs: [outreal: f64],
    };

    /// Compute CMF (Chaikin Money Flow).
    cmf => CMF, CmfStream {
        inputs: [high, low, close, volume],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute EFI (Elder's Force Index).
    efi => EFI, EfiStream {
        inputs: [close, volume],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute EMV (Arms Ease of Movement).
    emv => EMV, EmvStream {
        inputs: [high, low, volume],
        opts: [time_period: i32, volume_divisor: f64],
        outs: [outreal: f64],
    };

    /// Compute KVO (Klinger Volume Oscillator).
    kvo => KVO, KvoStream {
        inputs: [high, low, close, volume],
        opts: [fast_period: i32, slow_period: i32, signal_period: i32],
        outs: [outkvo: f64, outkvosignal: f64],
    };

    /// Compute MARKETFI (Market Facilitation Index).
    marketfi => MARKETFI, MarketfiStream {
        inputs: [high, low, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute NVI (Negative Volume Index).
    nvi => NVI, NviStream {
        inputs: [close, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute OBV (On Balance Volume).
    obv => OBV, ObvStream {
        inputs: [inreal, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute PVI (Positive Volume Index).
    pvi => PVI, PviStream {
        inputs: [close, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute PVO (Percentage Volume Oscillator).
    pvo => PVO, PvoStream {
        inputs: [volume],
        opts: [fast_period: i32, slow_period: i32, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute PVT (Price Volume Trend).
    pvt => PVT, PvtStream {
        inputs: [close, volume],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute RVOL (Relative Volume).
    rvol => RVOL, RvolStream {
        inputs: [volume],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute VWAP (Volume Weighted Average Price).
    vwap => VWAP, VwapStream {
        inputs: [high, low, close, volume],
        opts: [],
        outs: [outreal: f64],
    };

}

/// Backward-compatible alias for [`obv`].
///
/// # Errors
///
/// Returns an error if the input slices are empty or have mismatched lengths.
pub fn on_balance_volume(
    inreal: &[f64],
    volume: &[f64],
) -> Result<(Vec<f64>, usize), crate::error::Error> {
    obv(inreal, volume)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_support::{
        BeginIndex, check_indicator, close, initialize_test_runtime,
    };

    check_indicator!(obv_batch_and_stream, obv, obv_f32, obv_into, obv_f32_into, obv_lookback, obv_display_shift, ObvStream, [inreal, volume], [], [outreal: f64: 0], |v: f64| vec![v]);

    check_indicator!(ad_batch_and_stream, ad, ad_f32, ad_into, ad_f32_into, ad_lookback, ad_display_shift, AdStream, [high, low, close, volume], [], [outreal: f64: 0], |v: f64| vec![v]);

    #[test]
    fn on_balance_volume_alias_matches_generated_function() {
        let close = [10.0, 11.0, 10.5, 12.0, 11.5, 13.0];
        let volume = [100.0, 200.0, 300.0, 400.0, 500.0, 600.0];
        assert_eq!(
            on_balance_volume(&close, &volume).unwrap(),
            obv(&close, &volume).unwrap()
        );
    }

    #[test]
    fn on_balance_volume_accumulates_signed_volume() {
        let close = [10.0, 11.0, 10.5, 12.0];
        let volume = [100.0, 200.0, 300.0, 400.0];
        let (values, begin) = on_balance_volume(&close, &volume).unwrap();
        assert_eq!(begin, 0);
        assert_eq!(values, vec![100.0, 300.0, 0.0, 400.0]);
    }
}
