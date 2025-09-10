# Implemented WIT Functions (Subset of Polars)

Implements a minimal set of WIT-exposed functions to interact with the Polars data processing library. These functions are designed to be simple, composable, and extensible for future acceleration.

### `load_csv(path: string) -> dataframe`
Uses Polars `LazyCsvReader` to load a CSV lazily; returns a new handle.  
**Why**: Gives a simple path to evaluate when/what to accelerate, without committing to full expression parsing yet.

### `filter(df, filter: string) -> dataframe`
Very minimal parser for expressions like `"col > 10"`, `"col == 7"`, etc.  
**Why**: Enables basic filtering logic while keeping the implementation lightweight.

### `group_by(df, by_columns: list<string>) -> dataframe`
Delegates to Polars `group_by`.  
**Why**: Group-by is common and can benefit from parallel/accelerated backends later.

### `aggregate(df, aggs: list<string>) -> dataframe`
Supports `"mean(col)"` and `"count()"`. Defaults to `count()` if empty.  
**Why**: Simple, composable, and representative of useful analytics; easy to extend with more functions later.

### `to_json(df) -> string`
Collects with a `.limit(100)` preview and serializes rows to a JSON array.  
**Why**: Quick way to surface results in tests/demos; avoids adding external JSON dependencies for now.
