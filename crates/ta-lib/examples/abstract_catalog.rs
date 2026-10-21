//! Reflective Abstract API example (catalog discovery, metadata inspection, and dynamic execution).

use anyhow::{Context, Result};
use ta_lib::{
    FuncHandle, InputParameterType, OptInputDataSet, ParamHolder,
    abstract_api::{all_functions, for_each_function, functions_in_group, groups},
    initialize,
};

fn main() -> Result<()> {
    initialize().context("failed to initialize TA-Lib runtime")?;

    // 1. Inspect function groups and total catalog size
    let group_names = groups().context("list TA-Lib function groups")?;
    let function_names = all_functions().context("list all TA-Lib functions")?;
    println!(
        "TA-Lib catalog: {} groups, {} total indicators",
        group_names.len(),
        function_names.len()
    );
    for group in &group_names {
        let members = functions_in_group(group).with_context(|| format!("list group {group}"))?;
        println!("  - {group}: {} indicators", members.len());
    }

    // 2. Count candlestick patterns via `for_each_function`
    let mut pattern_count = 0usize;
    for_each_function(&mut |info| {
        if info.group == "Pattern Recognition" {
            pattern_count += 1;
        }
    })
    .context("traverse function catalog")?;
    println!("Visited {pattern_count} Pattern Recognition indicators via for_each_function");

    // 3. Dynamically inspect and execute an indicator by name ("EMA")
    let handle = FuncHandle::get("EMA").context("resolve EMA handle")?;
    let info = handle.info().context("query EMA metadata")?;
    println!(
        "\nResolved {} ({}) in group '{}' [inputs={}, opt_inputs={}, outputs={}]",
        info.name, info.hint, info.group, info.input_count, info.opt_input_count, info.output_count
    );

    let in_param = handle
        .input_parameter_info(0)
        .context("query EMA input 0")?;
    assert_eq!(in_param.parameter_type, InputParameterType::Real);

    let opt_param = handle
        .opt_input_parameter_info(0)
        .context("query EMA opt_input 0")?;
    if let OptInputDataSet::IntegerRange(range) = opt_param.data_set {
        println!(
            "  opt_input[0] '{}' ({}) range: {}..={} (default {})",
            opt_param.name, opt_param.display_name, range.min, range.max, opt_param.default_value
        );
    }

    let prices = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0];
    let mut output = [0.0_f64; 8];
    let mut holder = ParamHolder::new(&handle).context("allocate EMA ParamHolder")?;
    holder
        .set_input_real(0, &prices)
        .context("bind EMA input")?;
    holder
        .set_opt_input_integer(0, 3)
        .context("set EMA period=3")?;
    holder
        .set_output_real(0, &mut output)
        .context("bind EMA output")?;

    let lookback = holder.lookback().context("query EMA(3) lookback")?;
    let range = holder
        .call(0, prices.len() - 1)
        .context("execute dynamic EMA(3)")?;
    drop(holder);

    println!(
        "Dynamic EMA(3): lookback={lookback}, begin={}, values={:?}",
        range.begin,
        &output[..range.count]
    );

    Ok(())
}
