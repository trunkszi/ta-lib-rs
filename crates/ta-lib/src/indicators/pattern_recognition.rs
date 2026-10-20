//! Candlestick pattern recognition with integer signals and incremental streams.

crate::ta_indicators! {
    /// Compute CDL2CROWS (Two Crows).
    cdl2crows => CDL2CROWS, Cdl2crowsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3BLACKCROWS (Three Black Crows).
    cdl3blackcrows => CDL3BLACKCROWS, Cdl3blackcrowsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3INSIDE (Three Inside Up/Down).
    cdl3inside => CDL3INSIDE, Cdl3insideStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3LINESTRIKE (Three-Line Strike).
    cdl3linestrike => CDL3LINESTRIKE, Cdl3linestrikeStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3OUTSIDE (Three Outside Up/Down).
    cdl3outside => CDL3OUTSIDE, Cdl3outsideStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3STARSINSOUTH (Three Stars In The South).
    cdl3starsinsouth => CDL3STARSINSOUTH, Cdl3starsinsouthStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDL3WHITESOLDIERS (Three Advancing White Soldiers).
    cdl3whitesoldiers => CDL3WHITESOLDIERS, Cdl3whitesoldiersStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLABANDONEDBABY (Abandoned Baby).
    cdlabandonedbaby => CDLABANDONEDBABY, CdlabandonedbabyStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLADVANCEBLOCK (Advance Block).
    cdladvanceblock => CDLADVANCEBLOCK, CdladvanceblockStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLBELTHOLD (Belt-hold).
    cdlbelthold => CDLBELTHOLD, CdlbeltholdStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLBREAKAWAY (Breakaway).
    cdlbreakaway => CDLBREAKAWAY, CdlbreakawayStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLCLOSINGMARUBOZU (Closing Marubozu).
    cdlclosingmarubozu => CDLCLOSINGMARUBOZU, CdlclosingmarubozuStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLCONCEALBABYSWALL (Concealing Baby Swallow).
    cdlconcealbabyswall => CDLCONCEALBABYSWALL, CdlconcealbabyswallStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLCOUNTERATTACK (Counterattack).
    cdlcounterattack => CDLCOUNTERATTACK, CdlcounterattackStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLDARKCLOUDCOVER (Dark Cloud Cover).
    cdldarkcloudcover => CDLDARKCLOUDCOVER, CdldarkcloudcoverStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLDOJI (Doji).
    cdldoji => CDLDOJI, CdldojiStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLDOJISTAR (Doji Star).
    cdldojistar => CDLDOJISTAR, CdldojistarStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLDRAGONFLYDOJI (Dragonfly Doji).
    cdldragonflydoji => CDLDRAGONFLYDOJI, CdldragonflydojiStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLENGULFING (Engulfing Pattern).
    cdlengulfing => CDLENGULFING, CdlengulfingStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLEVENINGDOJISTAR (Evening Doji Star).
    cdleveningdojistar => CDLEVENINGDOJISTAR, CdleveningdojistarStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLEVENINGSTAR (Evening Star).
    cdleveningstar => CDLEVENINGSTAR, CdleveningstarStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLGAPSIDESIDEWHITE (Up/Down-gap side-by-side white lines).
    cdlgapsidesidewhite => CDLGAPSIDESIDEWHITE, CdlgapsidesidewhiteStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLGRAVESTONEDOJI (Gravestone Doji).
    cdlgravestonedoji => CDLGRAVESTONEDOJI, CdlgravestonedojiStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHAMMER (Hammer).
    cdlhammer => CDLHAMMER, CdlhammerStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHANGINGMAN (Hanging Man).
    cdlhangingman => CDLHANGINGMAN, CdlhangingmanStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHARAMI (Harami Pattern).
    cdlharami => CDLHARAMI, CdlharamiStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHARAMICROSS (Harami Cross Pattern).
    cdlharamicross => CDLHARAMICROSS, CdlharamicrossStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHIGHWAVE (High-Wave Candle).
    cdlhighwave => CDLHIGHWAVE, CdlhighwaveStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHIKKAKE (Hikkake Pattern).
    cdlhikkake => CDLHIKKAKE, CdlhikkakeStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHIKKAKEMOD (Modified Hikkake Pattern).
    cdlhikkakemod => CDLHIKKAKEMOD, CdlhikkakemodStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLHOMINGPIGEON (Homing Pigeon).
    cdlhomingpigeon => CDLHOMINGPIGEON, CdlhomingpigeonStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLIDENTICAL3CROWS (Identical Three Crows).
    cdlidentical3crows => CDLIDENTICAL3CROWS, Cdlidentical3crowsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLINNECK (In-Neck Pattern).
    cdlinneck => CDLINNECK, CdlinneckStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLINVERTEDHAMMER (Inverted Hammer).
    cdlinvertedhammer => CDLINVERTEDHAMMER, CdlinvertedhammerStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLKICKING (Kicking).
    cdlkicking => CDLKICKING, CdlkickingStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLKICKINGBYLENGTH (Kicking - bull/bear determined by the longer marubozu).
    cdlkickingbylength => CDLKICKINGBYLENGTH, CdlkickingbylengthStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLLADDERBOTTOM (Ladder Bottom).
    cdlladderbottom => CDLLADDERBOTTOM, CdlladderbottomStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLLONGLEGGEDDOJI (Long Legged Doji).
    cdllongleggeddoji => CDLLONGLEGGEDDOJI, CdllongleggeddojiStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLLONGLINE (Long Line Candle).
    cdllongline => CDLLONGLINE, CdllonglineStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLMARUBOZU (Marubozu).
    cdlmarubozu => CDLMARUBOZU, CdlmarubozuStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLMATCHINGLOW (Matching Low).
    cdlmatchinglow => CDLMATCHINGLOW, CdlmatchinglowStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLMATHOLD (Mat Hold).
    cdlmathold => CDLMATHOLD, CdlmatholdStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLMORNINGDOJISTAR (Morning Doji Star).
    cdlmorningdojistar => CDLMORNINGDOJISTAR, CdlmorningdojistarStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLMORNINGSTAR (Morning Star).
    cdlmorningstar => CDLMORNINGSTAR, CdlmorningstarStream {
        inputs: [open, high, low, close],
        opts: [penetration: f64],
        outs: [outinteger: i32],
    };

    /// Compute CDLONNECK (On-Neck Pattern).
    cdlonneck => CDLONNECK, CdlonneckStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLPIERCING (Piercing Pattern).
    cdlpiercing => CDLPIERCING, CdlpiercingStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLRICKSHAWMAN (Rickshaw Man).
    cdlrickshawman => CDLRICKSHAWMAN, CdlrickshawmanStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLRISEFALL3METHODS (Rising/Falling Three Methods).
    cdlrisefall3methods => CDLRISEFALL3METHODS, Cdlrisefall3methodsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSEPARATINGLINES (Separating Lines).
    cdlseparatinglines => CDLSEPARATINGLINES, CdlseparatinglinesStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSHOOTINGSTAR (Shooting Star).
    cdlshootingstar => CDLSHOOTINGSTAR, CdlshootingstarStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSHORTLINE (Short Line Candle).
    cdlshortline => CDLSHORTLINE, CdlshortlineStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSPINNINGTOP (Spinning Top).
    cdlspinningtop => CDLSPINNINGTOP, CdlspinningtopStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSTALLEDPATTERN (Stalled Pattern).
    cdlstalledpattern => CDLSTALLEDPATTERN, CdlstalledpatternStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLSTICKSANDWICH (Stick Sandwich).
    cdlsticksandwich => CDLSTICKSANDWICH, CdlsticksandwichStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLTAKURI (Takuri (Dragonfly Doji with very long lower shadow)).
    cdltakuri => CDLTAKURI, CdltakuriStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLTASUKIGAP (Tasuki Gap).
    cdltasukigap => CDLTASUKIGAP, CdltasukigapStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLTHRUSTING (Thrusting Pattern).
    cdlthrusting => CDLTHRUSTING, CdlthrustingStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLTRISTAR (Tristar Pattern).
    cdltristar => CDLTRISTAR, CdltristarStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLUNIQUE3RIVER (Unique 3 River).
    cdlunique3river => CDLUNIQUE3RIVER, Cdlunique3riverStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLUPSIDEGAP2CROWS (Upside Gap Two Crows).
    cdlupsidegap2crows => CDLUPSIDEGAP2CROWS, Cdlupsidegap2crowsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

    /// Compute CDLXSIDEGAP3METHODS (Upside/Downside Gap Three Methods).
    cdlxsidegap3methods => CDLXSIDEGAP3METHODS, Cdlxsidegap3methodsStream {
        inputs: [open, high, low, close],
        opts: [],
        outs: [outinteger: i32],
    };

}

#[cfg(test)]
mod unit_scenario {
    use std::sync::Once;

    use super::*;

    struct Candles {
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    }

    fn candles(tail: &[[f64; 4]]) -> Candles {
        static INIT: Once = Once::new();
        INIT.call_once(|| crate::initialize().expect("initialize TA-Lib"));
        let rows = std::iter::repeat_n([100.0, 102.0, 99.0, 101.0], 16)
            .chain(tail.iter().copied())
            .collect::<Vec<_>>();
        Candles {
            open: rows.iter().map(|r| r[0]).collect(),
            high: rows.iter().map(|r| r[1]).collect(),
            low: rows.iter().map(|r| r[2]).collect(),
            close: rows.iter().map(|r| r[3]).collect(),
        }
    }

    fn float_inputs(c: &Candles) -> [Vec<f32>; 4] {
        [&c.open, &c.high, &c.low, &c.close].map(|input| input.iter().map(|&v| v as f32).collect())
    }

    /// A zero-body candle is a doji; an ordinary body is not, in batch and stream form.
    #[test]
    fn doji_batch_float_and_stream() {
        let c = candles(&[
            [100.0, 102.0, 98.0, 100.0],
            [100.0, 102.0, 99.0, 101.0],
            [100.0, 102.0, 98.0, 100.0],
        ]);
        let lookback = cdldoji_lookback().unwrap();
        assert_eq!(lookback, 10);
        let (values, begin) = cdldoji(&c.open, &c.high, &c.low, &c.close).unwrap();
        assert_eq!(begin, lookback);
        assert_eq!(values.len(), c.open.len() - begin);
        assert!(values[..16 - begin].iter().all(|&v| v == 0));
        assert_eq!(&values[16 - begin..], &[100, 0, 100]);
        let [open, high, low, close] = float_inputs(&c);
        assert_eq!(
            cdldoji_f32(&open, &high, &low, &close).unwrap(),
            (values.clone(), begin)
        );
        let warm = lookback + 1;
        let (mut stream, current) = CdldojiStream::open(
            &c.open[..warm],
            &c.high[..warm],
            &c.low[..warm],
            &c.close[..warm],
        )
        .unwrap();
        assert_eq!(current, values[0]);
        for i in warm..c.open.len() {
            assert_eq!(
                stream
                    .peek(c.open[i], c.high[i], c.low[i], c.close[i])
                    .unwrap(),
                values[i - begin]
            );
            let previous = stream.value().unwrap();
            assert_eq!(previous, values[i - begin - 1]);
            assert_eq!(
                stream
                    .update(c.open[i], c.high[i], c.low[i], c.close[i])
                    .unwrap(),
                values[i - begin]
            );
        }
    }

    /// Strict engulfing preserves bullish +100 and bearish -100 integer directions.
    #[test]
    fn engulfing_batch_float_and_stream() {
        let c = candles(&[
            [102.0, 103.0, 100.0, 101.0],
            [100.0, 104.0, 99.0, 103.0],
            [104.0, 105.0, 98.0, 99.0],
        ]);
        let lookback = cdlengulfing_lookback().unwrap();
        assert_eq!(lookback, 2);
        let (values, begin) = cdlengulfing(&c.open, &c.high, &c.low, &c.close).unwrap();
        assert_eq!(begin, lookback);
        assert_eq!(values.len(), c.open.len() - begin);
        assert_eq!(&values[17 - begin..], &[100, -100]);
        let [open, high, low, close] = float_inputs(&c);
        assert_eq!(
            cdlengulfing_f32(&open, &high, &low, &close).unwrap(),
            (values.clone(), begin)
        );
        let warm = lookback + 1;
        let mut buffer = vec![i32::MIN; warm];
        let (mut stream, range) = CdlengulfingStream::open_and_fill(
            &c.open[..warm],
            &c.high[..warm],
            &c.low[..warm],
            &c.close[..warm],
            &mut buffer,
        )
        .unwrap();
        assert_eq!(
            range,
            crate::OutRange {
                begin,
                count: warm - begin
            }
        );
        assert_eq!(&buffer[..range.count], &values[..range.count]);
        assert!(buffer[range.count..].iter().all(|&v| v == i32::MIN));
        for i in warm..c.open.len() {
            assert_eq!(
                stream
                    .update(c.open[i], c.high[i], c.low[i], c.close[i])
                    .unwrap(),
                values[i - begin]
            );
        }
    }

    /// A gapped short star followed by a deep recovery passes only the matching penetration
    /// threshold.
    #[test]
    fn morning_star_batch_float_and_stream() {
        let c = candles(&[
            [110.0, 111.0, 99.0, 100.0],
            [98.0, 99.0, 97.0, 98.5],
            [99.0, 108.0, 98.0, 107.0],
        ]);
        let lookback = cdlmorningstar_lookback(Some(0.3)).unwrap();
        assert_eq!(lookback, 12);
        assert_eq!(cdlmorningstar_lookback(None).unwrap(), lookback);
        let (values, begin) =
            cdlmorningstar(&c.open, &c.high, &c.low, &c.close, Some(0.3)).unwrap();
        assert_eq!(begin, lookback);
        assert_eq!(values.len(), c.open.len() - begin);
        assert_eq!(values.last(), Some(&100));
        assert!(values[..values.len() - 1].iter().all(|&v| v == 0));
        let (stricter, _) = cdlmorningstar(&c.open, &c.high, &c.low, &c.close, Some(0.8)).unwrap();
        assert_eq!(stricter.last(), Some(&0));
        let [open, high, low, close] = float_inputs(&c);
        assert_eq!(
            cdlmorningstar_f32(&open, &high, &low, &close, Some(0.3)).unwrap(),
            (values.clone(), begin)
        );
        let warm = lookback + 1;
        let (mut stream, current) = CdlmorningstarStream::open(
            &c.open[..warm],
            &c.high[..warm],
            &c.low[..warm],
            &c.close[..warm],
            Some(0.3),
        )
        .unwrap();
        assert_eq!(current, values[0]);
        for i in warm..c.open.len() {
            assert_eq!(
                stream
                    .update(c.open[i], c.high[i], c.low[i], c.close[i])
                    .unwrap(),
                values[i - begin]
            );
        }
    }

    /// Invalid series dimensions, insufficient warmup and negative penetration fail explicitly.
    #[test]
    fn invalid_pattern_inputs() {
        let c = candles(&[]);
        assert_eq!(cdldoji(&[], &[], &[], &[]), Err(crate::Error::EmptyInput));
        assert!(matches!(
            cdlengulfing(&c.open, &c.high[..2], &c.low, &c.close),
            Err(crate::Error::MismatchedInputLength { .. })
        ));
        assert_eq!(
            cdlmorningstar_lookback(Some(-0.1)),
            Err(crate::Error::BadParam)
        );
        assert!(matches!(
            CdlmorningstarStream::open(
                &c.open[..1],
                &c.high[..1],
                &c.low[..1],
                &c.close[..1],
                Some(0.3)
            ),
            Err(crate::Error::InsufficientHistory)
        ));
    }
}
