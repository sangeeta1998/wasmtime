# Wasmtime's wasi-dataframe Implementation

This crate provides a Wasmtime host implementation of a proposed `dataframe-analysis` interface under the `wasi:accelerator` package. It exposes a subset of Polars operations to guest components.

## Current Status

 **Compilation**: Compilation errors have been resolved, and the code compiles successfully.

## Implemented WIT Functions (Subset of Polars)

Implements a minimal set of WIT-exposed functions to interact with the Polars data processing library. These functions are designed to be simple, composable, and extensible for future acceleration.

### `load_csv(path: string) -> result<dataframe>`
Uses Polars `LazyCsvReader` to load a CSV lazily; returns a new handle.  
**Why**: Gives a simple path to evaluate when/what to accelerate, without committing to full expression parsing yet.

### `from_rows(columns: list<string>, rows: list<list<string>>) -> result<dataframe>`
Creates a dataframe from column names and row data.  
**Why**: Enables programmatic dataframe creation for testing and data manipulation.

### `filter(df: dataframe, filters: list<column-filter>) -> result<dataframe>`
Applies column filters with support for various comparators (gt, gte, lt, lte, eq, neq) and scalar values (logic, name, value).  
**Why**: Enables basic filtering logic while keeping the implementation lightweight.

### `group_by(df: dataframe, by_columns: list<string>) -> result<dataframe>`
Delegates to Polars `group_by`.  
**Why**: Group-by is common and can benefit from parallel/accelerated backends later.

### `aggregate(df: dataframe, aggs: list<aggregation>) -> result<dataframe>`
Supports `count` and `mean` aggregations. Defaults to `count()` if empty.  
**Note**: `mean` aggregation currently requires column specification which is not supported by the current WIT interface.  
**Why**: Simple, composable, and representative of useful analytics; easy to extend with more functions later.

### `to_json(df: dataframe) -> result<string>`
Collects with a `.limit(100)` preview and serializes rows to a JSON array.  
**Why**: Quick way to surface results in tests/demos; avoids adding external JSON dependencies for now.

## Recent Fixes

- Fixed import and dependency issues
-  Updated return types to match WIT interface (`Result<T, ()>`)
- Fixed Polars API usage for version 0.49.1
- Updated error handling to use `()` instead of `anyhow::Error`
- Fixed CSV reader, DataFrame construction, and aggregation functions
- Fixed documentation errors for generated code
- Updated WIT interface compatibility (Scalar variants, Comparator enums, Aggregation enums)

## Testing

The main test driver is in `crates/dataframe/tests/main.rs`, but the real test case is in `crates/test-programs/src/bin/dataframe_main.rs` where we should add code as if writing a WebAssembly module.

```bash
cargo check --package wasmtime-wasi-dataframe

cargo check --bin dataframe_main --package test-programs

cargo test --package wasmtime-wasi-dataframe  # (no tests run, but compiles)


**Note**: The test currently has a runtime conflict issue that needs to be resolved separately.
