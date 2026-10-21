//! Integration tests for the reflective Abstract API and `ParamHolder`.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Once,
};

use ta_lib::{
    Error, OutRange,
    abstract_api::{
        FuncHandle, InputParameterType, OptInputDataSet, OptInputParameterType,
        OutputParameterType, ParamHolder, all_functions, for_each_function,
        function_description_xml, functions_in_group, groups,
    },
    sys,
};

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| ta_lib::initialize().expect("initialize TA-Lib"));
}

/// The reflective tables expose all ten groups, all 201 functions, and the XML payload.
#[test]
fn catalog_exposes_all_groups_and_functions() {
    init();
    let groups = groups().expect("group table");
    assert_eq!(groups.len(), 10);
    assert!(groups.iter().any(|g| g == "Overlap Studies"));
    assert!(groups.iter().any(|g| g == "Momentum Indicators"));
    let mut functions = all_functions().expect("function catalog");
    assert_eq!(functions.len(), 201);
    assert!(functions.iter().any(|name| name == "SMA"));
    let overlap = functions_in_group("Overlap Studies").expect("overlap");
    assert!(overlap.iter().any(|name| name == "SMA"));
    let mut visited = Vec::new();
    for_each_function(&mut |info| visited.push(info.name.clone())).expect("traversal");
    visited.sort();
    functions.sort();
    assert_eq!(visited, functions);
    let xml = function_description_xml();
    assert!(xml.starts_with("<?xml") && xml.contains("<FinancialFunctions>"));
}

fn sma_holder<'a>(input: &'a [f64], output: &'a mut [f64]) -> ParamHolder<'a> {
    let handle = FuncHandle::get("SMA").expect("SMA handle");
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder.set_input_real(0, input).expect("bind real");
    holder.set_opt_input_integer(0, 3).expect("set period");
    holder.set_output_real(0, output).expect("bind output");
    holder
}

/// Function metadata and a dynamic SMA invocation agree with the static indicator.
#[test]
fn dynamic_sma_matches_native_values() {
    init();
    let handle = FuncHandle::get("SMA").expect("SMA handle");
    let info = handle.info().expect("SMA info");
    assert_eq!(
        (info.name.as_str(), info.group.as_str()),
        ("SMA", "Overlap Studies")
    );
    let input_info = handle.input_parameter_info(0).expect("input info");
    assert_eq!(input_info.parameter_type, InputParameterType::Real);
    let opt = handle.opt_input_parameter_info(0).expect("opt input info");
    assert_eq!(opt.parameter_type, OptInputParameterType::IntegerRange);
    let OptInputDataSet::IntegerRange(range) = opt.data_set else {
        panic!("SMA period should use an integer range");
    };
    assert!(range.min <= 2 && range.max >= 1_000);
    let output_info = handle.output_parameter_info(0).expect("output info");
    assert_eq!(output_info.parameter_type, OutputParameterType::Real);

    let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let mut output = [0.0_f64; 5];
    let holder = sma_holder(&input, &mut output);
    assert_eq!(holder.lookback().expect("lookback"), 2);
    let range = holder.call(0, 4).expect("call SMA");
    assert_eq!((range.begin, range.count), (2, 3));
    drop(holder);
    assert_eq!(output[..3], [2.0, 3.0, 4.0]);
}

/// Output capacity is `end + 1 - max(start, lookback)`, not the raw requested span.
#[test]
fn call_capacity_accounts_for_lookback() {
    init();
    let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let mut exact = [0.0; 3];
    let holder = sma_holder(&input, &mut exact);
    assert_eq!(holder.call(0, 4), Ok(OutRange { begin: 2, count: 3 }));
    drop(holder);
    assert_eq!(exact, [2.0, 3.0, 4.0]);

    let mut short = [0.0; 2];
    let holder = sma_holder(&input, &mut short);
    assert_eq!(
        holder.call(0, 4),
        Err(Error::OutputBufferTooSmall {
            required: 3,
            actual: 2
        })
    );

    let mut empty: [f64; 0] = [];
    let holder = sma_holder(&input, &mut empty);
    assert_eq!(holder.call(0, 0), Ok(OutRange { begin: 0, count: 0 }));
}

/// A callback panic is re-raised after the synchronous traversal returns.
#[test]
#[should_panic(expected = "boom")]
fn for_each_function_propagates_callback_panic() {
    init();
    assert!(for_each_function(&mut |_| panic!("boom")).is_err());
}

/// Once a callback panics, later functions are not visited.
#[test]
fn for_each_function_stops_after_callback_panic() {
    init();
    let mut calls = 0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        for_each_function(&mut |_| {
            calls += 1;
            if calls == 5 {
                panic!("boom");
            }
        })
        .unwrap();
    }));
    assert!(result.is_err());
    assert_eq!(calls, 5);
}

/// Unflagged price components do not constrain the recorded input length.
#[test]
fn set_input_price_ignores_unflagged_components() {
    init();
    let high = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
    let low = [9.0, 10.0, 11.0, 12.0, 13.0, 14.0];
    let close = [9.5, 10.5, 11.5, 12.5, 13.5, 14.5];
    let one_bar = [1.0];

    let handle = FuncHandle::get("TRANGE").expect("TRANGE handle");
    let input_info = handle.input_parameter_info(0).expect("input info");
    assert_eq!(input_info.parameter_type, InputParameterType::Price);
    assert_ne!(input_info.flags & sys::TA_IN_PRICE_HIGH, 0);
    assert_eq!(input_info.flags & sys::TA_IN_PRICE_VOLUME, 0);
    let mut output = [0.0; 6];
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder
        .set_input_price(
            0,
            None,
            Some(&high),
            Some(&low),
            Some(&close),
            Some(&one_bar),
            None,
        )
        .expect("bind price");
    holder.set_output_real(0, &mut output).expect("bind output");
    let range = holder.call(0, 5).expect("call TRANGE");
    assert_eq!((range.begin, range.count), (1, 5));
    drop(holder);
    assert!(
        output[..range.count]
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
    );

    let handle = FuncHandle::get("ATR").expect("ATR handle");
    let mut output = [0.0; 6];
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder
        .set_input_price(
            0,
            None,
            Some(&high),
            Some(&low),
            Some(&close),
            Some(&one_bar),
            None,
        )
        .expect("bind price");
    holder.set_opt_input_integer(0, 5).expect("set period");
    holder.set_output_real(0, &mut output).expect("bind output");
    let range = holder.call(0, 5).expect("call ATR");
    assert_eq!((range.begin, range.count), (5, 1));
    drop(holder);
    let (batch, batch_begin) = ta_lib::atr(&high, &low, &close, Some(5)).expect("batch ATR");
    assert_eq!(batch_begin, range.begin);
    assert!((output[0] - batch[0]).abs() < 1e-12);
}

/// Real optional inputs cover both a continuous range and a bounded penetration list.
#[test]
fn set_opt_input_real_covers_bbands_and_candle_penetration() {
    init();
    let data: Vec<f64> = (0..20).map(|index| 100.0 + index as f64).collect();
    let handle = FuncHandle::get("BBANDS").expect("BBANDS handle");
    let (mut upper, mut middle, mut lower) = ([0.0; 20], [0.0; 20], [0.0; 20]);
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder.set_input_real(0, &data).expect("bind real");
    holder.set_opt_input_integer(0, 5).expect("set period");
    holder.set_opt_input_real(1, 2.0).expect("upper deviations");
    holder.set_opt_input_real(2, 2.0).expect("lower deviations");
    holder
        .set_opt_input_integer(3, ta_lib::MAType::Sma as i32)
        .expect("set MA type");
    holder.set_output_real(0, &mut upper).expect("upper output");
    holder
        .set_output_real(1, &mut middle)
        .expect("middle output");
    holder.set_output_real(2, &mut lower).expect("lower output");
    let range = holder.call(0, 19).expect("call BBANDS");
    assert_eq!((range.begin, range.count), (4, 16));
    drop(holder);
    for index in 0..range.count {
        assert!(upper[index] >= middle[index] && middle[index] >= lower[index]);
        assert!(middle[index].is_finite());
    }

    let bars = 40;
    let open: Vec<f64> = (0..bars).map(|index| 100.0 + (index % 3) as f64).collect();
    let high: Vec<f64> = open.iter().map(|value| value + 2.0).collect();
    let low: Vec<f64> = open.iter().map(|value| value - 2.0).collect();
    let close = open.clone();
    let handle = FuncHandle::get("CDLMORNINGSTAR").expect("CDLMORNINGSTAR handle");
    let ball = handle
        .opt_input_parameter_info(0)
        .expect("penetration metadata");
    assert_eq!(ball.parameter_type, OptInputParameterType::RealRange);
    assert!(matches!(ball.data_set, OptInputDataSet::RealRange(_)));
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder
        .set_input_price(
            0,
            Some(&open),
            Some(&high),
            Some(&low),
            Some(&close),
            None,
            None,
        )
        .expect("bind price");
    holder.set_opt_input_real(0, 0.3).expect("set penetration");
    let mut pattern = [0_i32; 40];
    holder
        .set_output_integer(0, &mut pattern)
        .expect("bind integer output");
    let range = holder.call(0, 39).expect("call CDLMORNINGSTAR");
    assert_eq!((range.begin, range.count), (12, 28));
    drop(holder);
    assert!(
        pattern[..range.count]
            .iter()
            .all(|value| matches!(*value, -100 | 0 | 100))
    );
}

/// Integer-list datasets expose the named moving-average choices.
#[test]
fn opt_input_integer_list_describes_ma_types() {
    init();
    let handle = FuncHandle::get("MA").expect("MA handle");
    let info = handle
        .opt_input_parameter_info(1)
        .expect("MA type metadata");
    assert_eq!(info.parameter_type, OptInputParameterType::IntegerList);
    let OptInputDataSet::IntegerList(list) = info.data_set else {
        panic!("MA type should use an integer list");
    };
    assert!(!list.is_empty());
    assert!(
        list.iter()
            .any(|entry| entry.value == ta_lib::MAType::Sma as i32 && !entry.name.is_empty())
    );
}

/// Integer outputs are bound through `set_output_integer` and match the static function.
#[test]
fn set_output_integer_matches_static_maxindex() {
    init();
    let data = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
    let handle = FuncHandle::get("MAXINDEX").expect("MAXINDEX handle");
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder.set_input_real(0, &data).expect("bind real");
    holder.set_opt_input_integer(0, 5).expect("set period");
    let mut output = [0_i32; 8];
    holder
        .set_output_integer(0, &mut output)
        .expect("bind integer output");
    assert_eq!(
        handle
            .output_parameter_info(0)
            .expect("output info")
            .parameter_type,
        OutputParameterType::Integer
    );
    let range = holder.call(0, 7).expect("call MAXINDEX");
    let (expected, expected_begin) = ta_lib::maxindex(&data, Some(5)).expect("batch MAXINDEX");
    assert_eq!(range.begin, expected_begin);
    assert_eq!(range.count, expected.len());
    drop(holder);
    assert_eq!(&output[..range.count], expected.as_slice());
}

/// Invalid names, indices, and input kinds fail explicitly.
#[test]
fn parameter_errors_are_explicit() {
    init();
    assert!(FuncHandle::get("NOPE").is_err());
    assert!(functions_in_group("NOPE").is_err());
    assert_eq!(FuncHandle::get("SMA\0").unwrap_err(), Error::InteriorNul);
    assert_eq!(
        functions_in_group("bad\0group").unwrap_err(),
        Error::InteriorNul
    );

    let handle = FuncHandle::get("SMA").expect("SMA handle");
    assert!(handle.input_parameter_info(9).is_err());
    assert!(handle.opt_input_parameter_info(9).is_err());
    assert!(handle.output_parameter_info(9).is_err());

    let input = [1.0_f64; 5];
    let mut output = [0.0_f64; 5];
    let mut holder = ParamHolder::new(&handle).expect("param holder");
    holder.set_input_real(0, &input).expect("bind real");
    assert!(holder.set_input_integer(0, &[1, 2, 3]).is_err());
    holder.set_opt_input_integer(0, 3).expect("set period");
    holder.set_output_real(0, &mut output).expect("bind output");
}
