# Repository guide

This file defines the operational contract for coding agents and contributors
working in ta-lib-rs. Read it before planning, reviewing, or editing repository
files. Preserve unrelated staged and unstaged work and establish file ownership
before concurrent work. Change staging, history, or remote branches only when
the current user explicitly requests those operations.

## Project overview

The workspace provides Rust 2024 bindings and procedural macros for the upstream
TA-Lib C library. It is a synchronous compute library with typed batch functions,
caller-owned output buffers, incremental indicator state, and reflective calls.
Application scheduling and networking belong to callers.

Default `pre-release` builds track upstream `main`. The `stable` feature resolves
an upstream release tag. Select the intended profile explicitly when validating
feature-dependent interfaces; inspect generated metadata rather than assuming
both upstream revisions expose identical indicator or streaming capabilities.

## Workspace layout

```text
ta-lib-rs/
|-- Cargo.toml                       # Workspace dependencies, metadata, and lints
|-- Cargo.lock                       # Resolved Rust dependency versions
|-- AGENTS.md                        # Agent maintenance contract
|-- README.md                        # Public architecture, APIs, and examples
|-- LICENSE                          # Distribution terms
|-- rust-toolchain.toml              # Selected Rust toolchain and components
|-- rustfmt.toml                     # Formatting configuration
|-- .github/workflows/ci.yml          # Hosted CI and native prerequisites
+-- crates/
    |-- ta-lib-sys/                  # L1: native build, FFI, and metadata
    |-- ta-lib-macros/               # L2: declaration and application macros
    +-- ta-lib/                      # L3: safe public APIs, tests, and examples
```

Keep native source caches and generated bindings beneath Cargo build output.
`TA_LIB_DIR` may select an existing upstream source checkout for a controlled
build. Keep local planning material under ignored `.scratch/` paths.

## Crate layering and responsibilities

| Layer | Crate | Responsibility |
| --- | --- | --- |
| L1 | `ta-lib-sys` | Fetch or locate upstream source, build its static C library with CMake, and generate FFI and indicator metadata from headers and `ABI.manifest`. |
| L2 | `ta-lib-macros` | Expand `ta_indicators!`, `ta_call!`, `ta_batch_aligned!`, `ta_dyn_call!`, and `FeatureSet` / `TaStream` derives. |
| L3 | `ta-lib` | Provide safe batch and streaming APIs, aligned/padded feature sets, runtime settings, the reflective Abstract API, examples, and integration tests. |

Dependencies point from the safe crate to its macro and FFI foundations. Keep
native ABI details in the FFI layer and safety invariants in the safe wrappers.
Generated code must preserve caller imports, Serde attributes, output ordering,
lookback alignment, and error propagation.

## Build, lint, and test commands

Run commands from the workspace root with the toolchain in
`rust-toolchain.toml`. Native builds require Git, a C compiler, CMake, and a
supported native build tool. Bound Cargo and native build concurrency together
when sharing a development machine.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo test -p ta-lib --no-default-features --features stable
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

The GitHub workflow runs formatting, workspace Clippy, workspace tests and
warning-free documentation on ubuntu-24.04 using the nightly toolchain.
Documentation tests and the stable profile remain separate local checks.

Run every progressive example when changing public macros or serialization:

```sh
cargo run -p ta-lib --example 01_batch_f64_and_f32
cargo run -p ta-lib --example 02_zero_alloc_buffers
cargo run -p ta-lib --example 03_streaming_state_machine
cargo run -p ta-lib --example 04_candlestick_patterns
cargo run -p ta-lib --example 05_runtime_and_unstable_periods
cargo run -p ta-lib --example 06_abstract_reflection
cargo run -p ta-lib --example 07_macro_custom_bindings
cargo run -p ta-lib --example 08_low_level_sys_ffi
cargo run -p ta-lib --example 09_ergonomic_call_and_batch_macros
cargo run -p ta-lib --example 10_derive_feature_set_pipeline
```

## Hard invariants and public contracts

### Platform, manifests, and source form

- Target Linux x86_64 with Rust edition 2024 and the declared nightly toolchain.
- Keep every tracked file at most 800 lines and entirely ASCII English, including
  source comments, documentation, diagnostics, and commit messages.
- Declare dependency sources and versions in root `[workspace.dependencies]`.
  Use single-line inline tables; members inherit with `{ workspace = true }`.
- Use sonic-rs 0.5.10 for JSON. Keep `serde` derives for serializable domain types.
  Tests and examples exercise `sonic_rs::to_string`, `sonic_rs::from_str`, and
  `sonic_rs::Value`; import `JsonValueTrait` for typed dynamic-value access.
- Keep manifests and crate files free of legacy JSON/HTTP dependencies and
  asynchronous runtimes, including packages introduced through aliases.
- Keep the compute and FFI APIs synchronous. Preserve declared workspace lints
  and feature profiles; inspect the manifests before changing panic behavior.

### Native ownership and zero-allocation APIs

- Validate slice lengths, output capacity, option ranges, and lookback alignment
  before native calls. Preserve begin/count semantics for empty and short input.
- Keep caller-owned `_into` batch functions and steady-state stream update/peek
  paths free of heap allocation. Initialization and allocating batch APIs have
  distinct allocation contracts.
- Preserve RAII ownership for native stream state. Preview operations leave
  committed state unchanged; cloned streams progress independently.
- Keep upstream global configuration explicit. Tests that mutate runtime or
  unstable-period settings must restore configuration and respect serial test
  execution where required by shared native state.

### Errors, safety, and documentation

- Return typed `ta_lib::Result<T>` failures at safe API boundaries; examples use
  `anyhow::Result` to attach operation context.
- Give every unsafe block and unsafe implementation a `// SAFETY:` comment
  documenting pointer validity, lengths, aliasing, lifetime, and native ownership.
- Give unsafe functions `# Safety`, fallible public APIs `# Errors`, and APIs
  with panic paths `# Panics` rustdoc sections.
- Keep comments focused on durable domain and ownership rules. Keep temporary
  task assignments and validation notes outside maintained source.

## Testing boundaries

| Dimension | Observable contract |
| --- | --- |
| Integration | Safe wrappers and reflective calls match native metadata, signatures, errors, and output ranges. |
| E2E | Public batch/feature-set computations serialize and deserialize through sonic-rs, preserving metadata, field names, values, and typed snapshots. |
| Lifecycle | Stream initialization, preview, commit, cloning, and drop preserve state ownership and committed output. |
| Scenario | Short history, padded warmup, multi-output alignment, invalid parameters, and feature-profile differences retain documented behavior. |

Test public seams rather than generated source spelling. Use controlled input
and independently checkable output. Verify finite numeric rows for JSON round
trips; warmup NaNs have a separate computation contract.

## Change hygiene

1. Read the current task, this guide, affected source, and current manifests.
   Inspect staged and unstaged diffs and preserve unrelated work.
2. Keep changes within the authorized repository and owned paths. Read target
   files again before writing when another worker may be active.
3. Add focused regressions for changed public contracts, then run the affected
   package and the workspace Cargo checks above. Run examples for serialization/API changes.
4. For dependency changes, audit direct and resolved package sources, including
   aliases and development/build dependencies.
5. For documentation changes, verify paths, commands, ASCII text, line counts,
   and public signatures against the current repository.
6. Finish with `git diff --check`. Report exact commands and results, keeping
   local checks distinct from hosted CI and remote publication evidence.
7. Preserve staging and history unless the current user explicitly authorizes
   committing, rewriting, or pushing. Complete validation before publishing.
