# ta-lib-rs

Rust 2024 bindings for TA-Lib **0.8.3**, covering all **237 indicators** in its
10 function groups. The workspace provides typed batch and streaming APIs, plus
a reflective Abstract API for selecting indicators at runtime.

## Workspace architecture

| Crate | Responsibility |
| --- | --- |
| [`ta-lib-sys`](crates/ta-lib-sys) | Builds the C library statically with CMake and generates raw FFI declarations from upstream `ABI.manifest`. |
| [`ta-lib-macros`](crates/ta-lib-macros) | Implements `ta_indicators!`, generating typed batch, caller-buffer, metadata and RAII streaming bindings. |
| [`ta-lib`](crates/ta-lib) | Exposes safe indicator functions, domain types, errors, runtime configuration and the Abstract API. |

All 237 indicators have typed batch and streaming bindings:

| Function group | Indicators |
| --- | ---: |
| Cycle Indicators | 8 |
| Math Operators | 12 |
| Math Transform | 15 |
| Momentum Indicators | 63 |
| Overlap Studies | 34 |
| Pattern Recognition | 61 |
| Price Transform | 6 |
| Statistic Functions | 13 |
| Volatility Indicators | 11 |
| Volume Indicators | 14 |
| **Total** | **237** |

Indicator functions and stream types are re-exported at the `ta_lib` crate root;
reflective functions live in `ta_lib::abstract_api`.

## Build and dependency setup

Use the toolchain in [`rust-toolchain.toml`](rust-toolchain.toml), which includes
Rustfmt and Clippy. Building requires a C compiler, CMake and a native build tool
supported by CMake. The Rust edition is 2024; the checked-in toolchain selects
nightly.

For a local checkout, add the high-level crate to your application's manifest:

```toml
[dependencies]
ta-lib = { path = "/path/to/ta-lib-rs/crates/ta-lib" }
```

`ta-lib-sys` disables shared-library builds and development tools and enables a
position-independent static C library. It does not require a separately installed
TA-Lib shared library or libclang/bindgen. On Unix, it also links the system math
library.

Source selection follows this order:

1. `TA_LIB_DIR`, if set, must name a TA-Lib **source tree**, containing
   `CMakeLists.txt`, `ABI.manifest` and `include/ta_libc.h`.
2. `vendor/ta-lib`, if present in the workspace.
3. The Cargo target directory's `vendor-cache/ta-lib` cache.
4. A download from the upstream Git repository, using the build script's pinned
   revision.

Use the matching 0.8.3 source tree when overriding the source. `TA_LIB_DIR` is not
an installed-library directory, and an override is not version-checked. For an
offline build, prepare the source tree and Cargo dependencies beforehand:

```sh
export TA_LIB_DIR=/path/to/ta-lib-source
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

## Runtime lifecycle

Call `ta_lib::initialize()` before calculations. Runtime settings such as
compatibility, unstable periods and candle settings are global: configure them
before concurrent calculations rather than changing them while work is running.
Call `shutdown()` only after all calculations and native handles have finished;
it is not a per-indicator cleanup operation. Streaming states and parameter
holders free their own native allocations when dropped.

## Batch calculations: f64 and f32

Each indicator has an allocating `f64` function and an `_f32` variant accepting
single-precision inputs. Real outputs use `f64` in both variants. Integer-valued
indicator outputs use `i32`.

Optional parameters use `Option<usize>`, `Option<f64>` or `Option<MAType>`;
`None` requests the native default. For example, `Some(MAType::Sma)` selects simple
moving-average smoothing where supported.

```rust
use ta_lib::{Error, initialize, rsi, rsi_f32};

fn main() -> Result<(), Error> {
    initialize()?;
    let prices = [10.0, 11.0, 12.0, 11.0, 13.0, 14.0];
    let (values, begin) = rsi(&prices, Some(3))?;
    assert_eq!(begin, 3);
    assert_eq!(values.len(), prices.len() - begin);

    let prices_f32: Vec<f32> = prices.iter().map(|p| *p as f32).collect();
    let (values_f32, begin_f32) = rsi_f32(&prices_f32, Some(3))?;
    assert_eq!(begin_f32, begin);
    assert_eq!(values_f32, values);
    Ok(())
}
```

Outputs are packed: `values[0]` corresponds to input bar `begin`, not bar zero.
For multiple outputs, the vectors precede the final begin index. For example,
`macd` returns `(macd, signal, histogram, begin)`. Input series must be nonempty
and have matching lengths. A valid calculation with insufficient warmup bars can
return no output; check the returned length.

## Caller-owned output buffers

`_into` and `_f32_into` avoid Rust output-vector allocation by writing into
caller-provided slices. Allocate these buffers once and reuse them. This is a
wrapper allocation guarantee, not a blanket guarantee about every native
indicator's internal implementation.

```rust
use ta_lib::{Error, initialize, rsi_into};

fn main() -> Result<(), Error> {
    initialize()?;
    let prices = [10.0, 11.0, 12.0, 11.0, 13.0, 14.0];
    let mut output = [0.0; 6];
    let range = rsi_into(&prices, Some(3), &mut output)?;
    assert_eq!(range.begin, 3);
    let valid = &output[..range.count];
    assert_eq!(valid.len(), 3);
    Ok(())
}
```

Every output slice must have at least `input.len().saturating_sub(lookback)`
elements (or `input.len()` for a reusable upper bound). Only
`output[..range.count]` contains calculated values. `OutRange.begin` is the
corresponding input offset and `OutRange.count` is the number of written
elements.

## Lookback and chart display shift

`<indicator>_lookback` returns the warmup bar count for the supplied options.
`<indicator>_display_shift` returns the chart placement shift for a zero-based
output index. Display shift describes placement; it does not change the packed
output layout.

```rust
use ta_lib::{Error, initialize, rsi_display_shift, rsi_lookback};

fn main() -> Result<(), Error> {
    initialize()?;
    assert_eq!(rsi_lookback(Some(3))?, 3);
    assert_eq!(rsi_display_shift(Some(3), 0)?, 0);
    Ok(())
}
```

## RAII streaming

Each indicator has a dedicated stream type, such as `RsiStream`, `AdxStream` or
`MacdStream`. `open` warms the state from historical bars and returns the latest
output. `open_and_fill` also writes historical outputs into caller-owned buffers.
Opening allocates native state, which is released automatically on drop.

```rust
use ta_lib::{Error, RsiStream, initialize, rsi};

fn main() -> Result<(), Error> {
    initialize()?;
    let history = [10.0, 11.0, 12.0, 11.0, 13.0, 14.0];
    let (mut stream, initial) = RsiStream::open(&history, Some(3))?;
    assert_eq!(stream.value()?, initial);

    let forming = stream.peek(15.0)?;
    assert_eq!(stream.value()?, initial); // A preview does not commit the bar.
    let closed = stream.update(15.0)?;
    assert!((closed - forming).abs() < 1e-10);

    let mut independent = stream.try_clone()?;
    independent.update(9.0)?;
    assert_eq!(stream.value()?, closed);

    let mut complete = history.to_vec();
    complete.push(15.0);
    let (batch, _) = rsi(&complete, Some(3))?;
    assert!((closed - batch.last().copied().unwrap()).abs() < 1e-10);
    Ok(())
}
```

`update` commits one closed bar; `peek` evaluates a forming bar without mutating
committed state; `value` reads the latest committed output. `out_range` exposes
the cumulative output range. `try_clone` deep-copies native state for independent
continuation. Insufficient history at stream opening returns an error rather
than a ready stream. Streams can be moved between threads but do not provide
shared concurrent mutation.

## Reflective Abstract API

Use the Abstract API when the indicator name or parameters are chosen at runtime.
`groups`, `functions_in_group` and `all_functions` return owned names.
`function_description_xml` exposes the library's XML catalog, while
`for_each_function` visits owned `FuncInfo` snapshots.

`FuncHandle` resolves names and exposes function metadata, required inputs,
optional inputs and outputs. Optional-input datasets describe `RealRange`,
`IntegerRange`, `RealList` and `IntegerList` constraints. Input flags identify
which price components an indicator consumes.

```rust
use ta_lib::{Error, abstract_api, initialize};
use ta_lib::abstract_api::{FuncHandle, InputParameterType, ParamHolder};

fn main() -> Result<(), Error> {
    initialize()?;
    assert_eq!(abstract_api::groups()?.len(), 10);
    assert_eq!(abstract_api::all_functions()?.len(), 237);
    let handle = FuncHandle::get("SMA")?;
    let info = handle.info()?;
    assert_eq!(info.name, "SMA");
    let input_info = handle.input_parameter_info(0)?;
    assert_eq!(input_info.parameter_type, InputParameterType::Real);

    let close = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut output = [0.0; 5];
    let mut params = ParamHolder::new(&handle)?;
    // SMA consumes a real input series; price-based indicators use set_input_price.
    params.set_input_real(0, &close)?;
    params.set_opt_input_integer(0, 3)?;
    params.set_output_real(0, &mut output)?;
    assert_eq!(params.lookback()?, 2);
    assert_eq!(params.display_shift(0)?, 0);
    let range = params.call(0, close.len() - 1)?;
    drop(params); // Release the output borrow before reading the buffer.
    assert_eq!(range.begin, 2);
    assert_eq!(&output[..range.count], &[2.0, 3.0, 4.0]);
    Ok(())
}
```

`ParamHolder` owns its native allocation and borrows registered slices for its
lifetime. Bind real or integer inputs with `set_input_real` or
`set_input_integer`; bind price components with `set_input_price`. Configure
optional values with `set_opt_input_integer` or `set_opt_input_real`, and bind
outputs with `set_output_real` or `set_output_integer`. Match these methods to
the function's parameter metadata.

`call(start, end)` uses **inclusive** input indices. Inputs must cover `end`, and
output buffers must hold at least `(end + 1).saturating_sub(start.max(lookback))`
elements. Read only the returned output count after dropping the holder. A holder
may be called again while its buffers remain bound.

All fallible wrappers return `ta_lib::Error`, including native return-code
information for errors without a dedicated Rust variant. Raw FFI remains
available through `ta_lib::sys` for callers that need to manage C contracts
directly.

## License

The Rust workspace uses the BSD-3-Clause license. TA-Lib's upstream license is
included with its source distribution.
