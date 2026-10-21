//! Example 06: Reflective Abstract API — runtime catalog discovery, parameter constraint
//! introspection, XML function catalog, and dynamic `ParamHolder` execution.

use anyhow::{Context, Result};
use ta_lib::{
    FuncHandle, InputParameterType, OptInputDataSet, OutputParameterType, ParamHolder,
    abstract_api::{
        all_functions, for_each_function, function_description_xml, functions_in_group, groups,
    },
    initialize,
};

fn main() -> Result<()> {
    initialize().context("initialize TA-Lib runtime")?;

    // 1. Enumerate all function groups and count indicators per group
    let group_list = groups().context("list function groups")?;
    let all_funcs = all_functions().context("list all functions")?;
    println!(
        "Catalog summary: {} groups, {} total functions (XML catalog={} bytes)",
        group_list.len(),
        all_funcs.len(),
        function_description_xml().len(),
    );
    for group in &group_list {
        let members = functions_in_group(group).with_context(|| format!("group {group}"))?;
        println!("  - {group:24}: {:3} functions", members.len());
    }

    // 2. Traverse function metadata snapshots with `for_each_function`
    let mut multi_output_count = 0usize;
    for_each_function(&mut |info| {
        if info.output_count > 1 {
            multi_output_count += 1;
        }
    })
    .context("traverse catalog via for_each_function")?;
    println!("Indicators with multiple outputs: {multi_output_count}");

    // 3. Dynamically inspect and execute a real-input indicator ("BBANDS")
    let bb_handle = FuncHandle::get("BBANDS").context("resolve BBANDS handle")?;
    let bb_info = bb_handle.info().context("BBANDS info")?;
    println!(
        "\nInspecting {} ({}) [inputs={}, opts={}, outs={}]",
        bb_info.name,
        bb_info.hint,
        bb_info.input_count,
        bb_info.opt_input_count,
        bb_info.output_count,
    );
    for idx in 0..bb_info.opt_input_count {
        let opt = bb_handle
            .opt_input_parameter_info(idx as u32)
            .context("opt_input info")?;
        match &opt.data_set {
            OptInputDataSet::IntegerRange(r) => println!(
                "  opt[{idx}] {} ({}): int {}..={} default={}",
                opt.name, opt.display_name, r.min, r.max, opt.default_value
            ),
            OptInputDataSet::RealRange(r) => println!(
                "  opt[{idx}] {} ({}): real {:.2}..={:.2} default={}",
                opt.name, opt.display_name, r.min, r.max, opt.default_value
            ),
            OptInputDataSet::IntegerList(list) => println!(
                "  opt[{idx}] {} ({}): {} choices",
                opt.name,
                opt.display_name,
                list.len()
            ),
            OptInputDataSet::RealList(list) => println!(
                "  opt[{idx}] {} ({}): {} choices",
                opt.name,
                opt.display_name,
                list.len()
            ),
        }
    }

    // 4. Dynamically execute a price-input indicator ("ATR") via `set_input_price`
    let atr_handle = FuncHandle::get("ATR").context("resolve ATR handle")?;
    assert_eq!(
        atr_handle.input_parameter_info(0)?.parameter_type,
        InputParameterType::Price
    );
    assert_eq!(
        atr_handle.output_parameter_info(0)?.parameter_type,
        OutputParameterType::Real
    );

    let high = [102.0, 103.5, 104.0, 103.2, 105.0, 106.2, 105.8, 107.1];
    let low = [99.5, 100.8, 101.2, 100.4, 102.1, 103.4, 103.0, 104.2];
    let close = [101.0, 102.8, 102.1, 102.9, 104.5, 105.1, 105.4, 106.6];
    let mut atr_out = [0.0_f64; 8];

    let mut holder = ParamHolder::new(&atr_handle).context("allocate ATR ParamHolder")?;
    holder
        .set_input_price(0, None, Some(&high), Some(&low), Some(&close), None, None)
        .context("bind OHLC price components")?;
    holder
        .set_opt_input_integer(0, 3)
        .context("set ATR period=3")?;
    holder
        .set_output_real(0, &mut atr_out)
        .context("bind ATR output")?;

    let range = holder
        .call(0, close.len() - 1)
        .context("execute dynamic ATR(3)")?;
    drop(holder);
    println!(
        "Dynamic ATR(3): begin={}, count={}, values={:?}",
        range.begin,
        range.count,
        &atr_out[..range.count]
    );

    Ok(())
}
