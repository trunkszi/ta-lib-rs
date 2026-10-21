//! Momentum oscillators and directional movement indicators.
// Generated signatures mirror the TA-Lib parameter lists.
#![allow(clippy::too_many_arguments)]

crate::ta_indicators! {
    /// Compute AC (Accelerator/Decelerator Oscillator).
    ac => AC, AcStream {
        inputs: [high, low],
        opts: [fast_period: i32, slow_period: i32, signal_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ADX (Average Directional Movement Index).
    adx => ADX, AdxStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ADXR (Average Directional Movement Index Rating).
    adxr => ADXR, AdxrStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute AO (Awesome Oscillator).
    ao => AO, AoStream {
        inputs: [high, low],
        opts: [fast_period: i32, slow_period: i32],
        outs: [outreal: f64],
    };

    /// Compute APO (Absolute Price Oscillator).
    apo => APO, ApoStream {
        inputs: [inreal],
        opts: [fast_period: i32, slow_period: i32, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute AROON (Aroon).
    aroon => AROON, AroonStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outaroondown: f64, outaroonup: f64],
    };

    /// Compute AROONOSC (Aroon Oscillator).
    aroonosc => AROONOSC, AroonoscStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute BOP (Balance Of Power).
    bop => BOP, BopStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute CCI (Commodity Channel Index).
    cci => CCI, CciStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute CMO (Chande Momentum Oscillator).
    cmo => CMO, CmoStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute CMOU (Chande Momentum Oscillator (Unsmoothed)).
    cmou => CMOU, CmouStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute COPPOCK (Coppock Curve).
    coppock => COPPOCK, CoppockStream {
        inputs: [inreal],
        opts: [wma_period: i32, roc_1_period: i32, roc_2_period: i32],
        outs: [outreal: f64],
    };

    /// Compute DPO (Detrended Price Oscillator).
    dpo => DPO, DpoStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute DX (Directional Movement Index).
    dx => DX, DxStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ER (Kaufman Efficiency Ratio).
    er => ER, ErStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ERI (Elder Ray Index (Bull Power / Bear Power)).
    eri => ERI, EriStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outbullpower: f64, outbearpower: f64],
    };

    /// Compute FOSC (Forecast Oscillator).
    fosc => FOSC, FoscStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute FRACTAL (Williams Fractal).
    fractal => FRACTAL, FractalStream {
        inputs: [high, low],
        opts: [left_bars: i32, right_bars: i32],
        outs: [outswinghigh: i32, outswinglow: i32],
    };

    /// Compute IMI (Intraday Momentum Index).
    imi => IMI, ImiStream {
        inputs: [open, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute KDJ (KDJ Stochastic).
    kdj => KDJ, KdjStream {
        inputs: [high, low, close],
        opts: [fast_k_period: i32, slow_k_period: i32, slow_k_ma: MAType, slow_d_period: i32, slow_d_ma: MAType],
        outs: [outk: f64, outd: f64, outj: f64],
    };

    /// Compute MACD (Moving Average Convergence/Divergence).
    macd => MACD, MacdStream {
        inputs: [inreal],
        opts: [fast_period: i32, slow_period: i32, signal_period: i32],
        outs: [outmacd: f64, outmacdsignal: f64, outmacdhist: f64],
    };

    /// Compute MACDEXT (MACD with controllable MA type).
    macdext => MACDEXT, MacdextStream {
        inputs: [inreal],
        opts: [fast_period: i32, fast_ma: MAType, slow_period: i32, slow_ma: MAType, signal_period: i32, signal_ma: MAType],
        outs: [outmacd: f64, outmacdsignal: f64, outmacdhist: f64],
    };

    /// Compute MACDFIX (Moving Average Convergence/Divergence Fix 12/26).
    macdfix => MACDFIX, MacdfixStream {
        inputs: [inreal],
        opts: [signal_period: i32],
        outs: [outmacd: f64, outmacdsignal: f64, outmacdhist: f64],
    };

    /// Compute MFI (Money Flow Index).
    mfi => MFI, MfiStream {
        inputs: [high, low, close, volume],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MINUS_DI (Minus Directional Indicator).
    minus_di => MINUS_DI, MinusDiStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MINUS_DM (Minus Directional Movement).
    minus_dm => MINUS_DM, MinusDmStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute MOM (Momentum).
    mom => MOM, MomStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute PLUS_DI (Plus Directional Indicator).
    plus_di => PLUS_DI, PlusDiStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute PLUS_DM (Plus Directional Movement).
    plus_dm => PLUS_DM, PlusDmStream {
        inputs: [high, low],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute PPO (Percentage Price Oscillator).
    ppo => PPO, PpoStream {
        inputs: [inreal],
        opts: [fast_period: i32, slow_period: i32, ma_type: MAType],
        outs: [outreal: f64],
    };

    /// Compute QSTICK (Qstick).
    qstick => QSTICK, QstickStream {
        inputs: [open, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ROC (Rate of change : ((price/prevPrice)-1)*100).
    roc => ROC, RocStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ROCP (Rate of change Percentage: (price-prevPrice)/prevPrice).
    rocp => ROCP, RocpStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ROCR (Rate of change ratio: (price/prevPrice)).
    rocr => ROCR, RocrStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ROCR100 (Rate of change ratio 100 scale: (price/prevPrice)*100).
    rocr100 => ROCR100, Rocr100Stream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute RSI (Relative Strength Index).
    rsi => RSI, RsiStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute SMI (Stochastic Momentum Index).
    smi => SMI, SmiStream {
        inputs: [high, low, close],
        opts: [time_period: i32, fast_period: i32, slow_period: i32, signal_period: i32],
        outs: [outsmi: f64, outsmisignal: f64],
    };

    /// Compute STOCH (Stochastic).
    stoch => STOCH, StochStream {
        inputs: [high, low, close],
        opts: [fast_k_period: i32, slow_k_period: i32, slow_k_ma: MAType, slow_d_period: i32, slow_d_ma: MAType],
        outs: [outslowk: f64, outslowd: f64],
    };

    /// Compute STOCHF (Stochastic Fast).
    stochf => STOCHF, StochfStream {
        inputs: [high, low, close],
        opts: [fast_k_period: i32, fast_d_period: i32, fast_d_ma: MAType],
        outs: [outfastk: f64, outfastd: f64],
    };

    /// Compute STOCHRSI (Stochastic Relative Strength Index).
    stochrsi => STOCHRSI, StochrsiStream {
        inputs: [inreal],
        opts: [time_period: i32, fast_k_period: i32, fast_d_period: i32, fast_d_ma: MAType],
        outs: [outfastk: f64, outfastd: f64],
    };

    /// Compute TRIX (1-day Rate-Of-Change (ROC) of a Triple Smooth EMA).
    trix => TRIX, TrixStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute TSI (True Strength Index).
    tsi => TSI, TsiStream {
        inputs: [inreal],
        opts: [first_smoothing_period: i32, second_smoothing_period: i32],
        outs: [outreal: f64],
    };

    /// Compute ULTOSC (Ultimate Oscillator).
    ultosc => ULTOSC, UltoscStream {
        inputs: [high, low, close],
        opts: [first_period: i32, second_period: i32, third_period: i32],
        outs: [outreal: f64],
    };

    /// Compute VHF (Vertical Horizontal Filter).
    vhf => VHF, VhfStream {
        inputs: [inreal],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

    /// Compute VORTEX (Vortex Indicator).
    vortex => VORTEX, VortexStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outplusvi: f64, outminusvi: f64],
    };

    /// Compute WAD (Williams' Accumulation/Distribution).
    wad => WAD, WadStream {
        inputs: [high, low, close],
        opts: [],
        outs: [outreal: f64],
    };

    /// Compute WILLR (Williams' %R).
    willr => WILLR, WillrStream {
        inputs: [high, low, close],
        opts: [time_period: i32],
        outs: [outreal: f64],
    };

}

/// Compute the average directional movement index.
///
/// # Errors
///
/// Returns an error for empty or mismatched inputs or an invalid period.
pub fn average_directional_movement_index(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: Option<usize>,
) -> Result<(Vec<f64>, usize), crate::Error> {
    adx(high, low, close, time_period)
}

/// Compute the negative directional indicator.
///
/// # Errors
///
/// Returns an error for empty or mismatched inputs or an invalid period.
pub fn negative_directional_indicator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: Option<usize>,
) -> Result<(Vec<f64>, usize), crate::Error> {
    minus_di(high, low, close, time_period)
}

/// Compute the positive directional indicator.
///
/// # Errors
///
/// Returns an error for empty or mismatched inputs or an invalid period.
pub fn positive_directional_indicator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    time_period: Option<usize>,
) -> Result<(Vec<f64>, usize), crate::Error> {
    plus_di(high, low, close, time_period)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAType;

    fn bars() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        crate::indicators::test_support::initialize_test_runtime();
        let close: Vec<_> = (0..100).map(|i| 100.0 + f64::from(i)).collect();
        let high = close.iter().map(|v| v + 2.0).collect();
        let low = close.iter().map(|v| v - 2.0).collect();
        (high, low, close)
    }

    fn near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn unit_adx_directional_trend_and_aliases() {
        let (high, low, close) = bars();
        let (values, begin) = adx(&high, &low, &close, Some(14)).unwrap();
        assert_eq!(begin, 27);
        assert_eq!(values.len(), close.len() - begin);
        for value in &values {
            near(*value, 100.0);
        }
        assert_eq!(
            average_directional_movement_index(&high, &low, &close, Some(14)).unwrap(),
            (values, begin)
        );
        assert_eq!(
            negative_directional_indicator(&high, &low, &close, Some(14)).unwrap(),
            minus_di(&high, &low, &close, Some(14)).unwrap()
        );
        assert_eq!(
            positive_directional_indicator(&high, &low, &close, Some(14)).unwrap(),
            plus_di(&high, &low, &close, Some(14)).unwrap()
        );
        let (negative, _) = minus_di(&high, &low, &close, Some(14)).unwrap();
        assert!(negative.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn unit_macd_linear_trend_and_caller_buffers() {
        let (_, _, close) = bars();
        let (line, signal, histogram, begin) = macd(&close, Some(12), Some(26), Some(9)).unwrap();
        assert_eq!(begin, 33);
        assert_eq!(line.len(), close.len() - begin);
        for ((line, signal), histogram) in line.iter().zip(&signal).zip(&histogram) {
            near(*line, 7.0);
            near(*signal, 7.0);
            near(*histogram, 0.0);
        }
        let mut a = vec![f64::NAN; close.len()];
        let mut b = a.clone();
        let mut c = a.clone();
        let range = macd_into(&close, Some(12), Some(26), Some(9), &mut a, &mut b, &mut c).unwrap();
        assert_eq!(range.begin, begin);
        assert_eq!(range.count, line.len());
        assert_eq!(&a[..range.count], line);
        assert_eq!(&b[..range.count], signal);
        assert_eq!(&c[..range.count], histogram);
        assert!(a[range.count..].iter().all(|v| v.is_nan()));
    }

    #[test]
    fn unit_rsi_rising_and_falling_prices() {
        let (_, _, close) = bars();
        let (rising, begin) = rsi(&close, Some(14)).unwrap();
        assert_eq!(begin, 14);
        assert_eq!(rising.len(), close.len() - begin);
        for value in rising {
            near(value, 100.0);
        }
        let falling: Vec<_> = close.into_iter().rev().collect();
        let (values, _) = rsi(&falling, Some(14)).unwrap();
        for value in values {
            near(value, 0.0);
        }
    }

    #[test]
    fn unit_stoch_known_range_and_f32_inputs() {
        let (high, low, close) = bars();
        let (k, d, begin) = stoch(
            &high,
            &low,
            &close,
            Some(5),
            Some(3),
            Some(MAType::Sma),
            Some(3),
            Some(MAType::Sma),
        )
        .unwrap();
        assert_eq!(begin, 8);
        assert_eq!(k.len(), close.len() - begin);
        for (k, d) in k.iter().zip(&d) {
            near(*k, 75.0);
            near(*d, 75.0);
        }
        let high: Vec<_> = high.iter().map(|v| *v as f32).collect();
        let low: Vec<_> = low.iter().map(|v| *v as f32).collect();
        let close: Vec<_> = close.iter().map(|v| *v as f32).collect();
        assert_eq!(
            stoch_f32(
                &high,
                &low,
                &close,
                Some(5),
                Some(3),
                Some(MAType::Sma),
                Some(3),
                Some(MAType::Sma)
            )
            .unwrap(),
            (k, d, begin)
        );
    }

    #[test]
    fn unit_streaming_matches_batch_and_peek_preserves_state() {
        let (high, low, mut close) = bars();
        for (i, value) in close.iter_mut().enumerate() {
            *value += (i as f64 * 0.7).sin();
        }
        let split = 60;
        let (adx_values, adx_begin) = adx(&high, &low, &close, Some(14)).unwrap();
        let (rsi_values, rsi_begin) = rsi(&close, Some(14)).unwrap();
        let (m, s, h, macd_begin) = macd(&close, Some(12), Some(26), Some(9)).unwrap();
        let (k, d, stoch_begin) = stoch(
            &high,
            &low,
            &close,
            Some(5),
            Some(3),
            Some(MAType::Sma),
            Some(3),
            Some(MAType::Sma),
        )
        .unwrap();
        let (mut adx_stream, initial) =
            AdxStream::open(&high[..split], &low[..split], &close[..split], Some(14)).unwrap();
        near(initial, adx_values[split - 1 - adx_begin]);
        let mut filled = vec![f64::NAN; split];
        let (mut rsi_stream, range) =
            RsiStream::open_and_fill(&close[..split], Some(14), &mut filled).unwrap();
        assert_eq!(range.begin, rsi_begin);
        assert_eq!(&filled[..range.count], &rsi_values[..range.count]);
        let (mut macd_stream, _) =
            MacdStream::open(&close[..split], Some(12), Some(26), Some(9)).unwrap();
        let (mut stoch_stream, _) = StochStream::open(
            &high[..split],
            &low[..split],
            &close[..split],
            Some(5),
            Some(3),
            Some(MAType::Sma),
            Some(3),
            Some(MAType::Sma),
        )
        .unwrap();
        let mut cloned = rsi_stream.try_clone().unwrap();
        let before = rsi_stream.value().unwrap();
        let peek = rsi_stream.peek(close[split]).unwrap();
        near(rsi_stream.value().unwrap(), before);
        near(cloned.update(close[split]).unwrap(), peek);
        near(rsi_stream.value().unwrap(), before);
        for i in split..close.len() {
            near(
                adx_stream.update(high[i], low[i], close[i]).unwrap(),
                adx_values[i - adx_begin],
            );
            near(
                rsi_stream.update(close[i]).unwrap(),
                rsi_values[i - rsi_begin],
            );
            let actual = macd_stream.update(close[i]).unwrap();
            near(actual.0, m[i - macd_begin]);
            near(actual.1, s[i - macd_begin]);
            near(actual.2, h[i - macd_begin]);
            let actual = stoch_stream.update(high[i], low[i], close[i]).unwrap();
            near(actual.0, k[i - stoch_begin]);
            near(actual.1, d[i - stoch_begin]);
        }
        assert_eq!(rsi_stream.out_range().unwrap().count, rsi_values.len());
    }

    #[test]
    fn unit_invalid_inputs_fail_without_stream_state() {
        let (high, low, close) = bars();
        assert!(matches!(rsi(&[], None), Err(crate::Error::EmptyInput)));
        assert!(adx(&high[..5], &low, &close, Some(14)).is_err());
        assert!(RsiStream::open(&close[..2], Some(14)).is_err());
        assert!(rsi(&close, Some(0)).is_err());
        assert!(average_directional_movement_index(&high, &low, &close, Some(0)).is_err());
        assert!(negative_directional_indicator(&high, &low, &close, Some(0)).is_err());
        assert!(positive_directional_indicator(&high, &low, &close, Some(0)).is_err());
    }
}
