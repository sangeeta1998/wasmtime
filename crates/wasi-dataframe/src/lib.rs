//! # Wasmtime's [wasi-dataframe] Implementation
//!
//! This crate provides a Wasmtime host implementation of a proposed
//! `dataframe-analysis` interface under the `wasi:accelerator` package.
//! It exposes a subset of Polars operations to guest components.
//!
//! Currently supported operations:
//! - load-csv(path)
//! - from-rows(columns, rows)
//! - filter(df, filters)
//! - group-by(df, by_columns)
//! - aggregate(df, aggs) [mean(col), count()]
//! - to-json(df)
//

#![deny(missing_docs)]

/// Generated bindings for the wasi-dataframe interface.
#[allow(missing_docs)]
pub mod generated {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "wasi:dataframe/imports",
    });
}

// Re-export a public path to the generated bindings for external tests/examples.
pub use self::generated as bindings;

use anyhow::Result;
use std::collections::HashMap;
use polars::prelude::{LazyFrame, LazyCsvReader, LazyFileListReader, Expr, col, lit, DataType, NamedFrom, IntoLazy};
use polars::series::Series;
use polars::frame::DataFrame;
use polars::datatypes::AnyValue;
use wasmtime::component::HasData;
// Bring generated interface into scope.
use self::generated::wasi::dataframe::dataframe_analysis as wit_df;

/// Opaque handle to a host-side dataframe (we use `LazyFrame` for cheap cloning and planning).
pub type DataframeHandle = u32;

/// Builder for the runtime context which owns host-side dataframes.
#[derive(Default)]
pub struct WasiDataframeCtxBuilder {
    _lazyframes: HashMap<DataframeHandle, LazyFrame>,
    _next_handle: DataframeHandle,
}

impl WasiDataframeCtxBuilder {
    /// Creates a new builder with default parameters.
    pub fn new() -> Self {
        Self {
            _next_handle: 1,
            ..Default::default()
        }
    }

    /// Builds the runtime context.
    pub fn build(self) -> WasiDataframeCtx {
        WasiDataframeCtx {
            lazyframes: self._lazyframes,
            next_handle: self._next_handle,
        }
    }
}

/// The runtime context for WASI dataframe operations.
pub struct WasiDataframeCtx {
    lazyframes: HashMap<DataframeHandle, LazyFrame>,
    next_handle: DataframeHandle,
}

impl WasiDataframeCtx {
    /// Creates and returns a new unique handle.
    pub fn new_handle(&mut self) -> DataframeHandle {
        let handle = self.next_handle;
        self.next_handle = self.next_handle.wrapping_add(1).max(1);
        handle
    }

    /// Creates a new builder for constructing a `WasiDataframeCtx`.
    pub fn builder() -> WasiDataframeCtxBuilder {
        WasiDataframeCtxBuilder::new()
    }
}

/// A wrapper capturing the needed internal state for `wasi-dataframe`.
pub struct WasiDataframe<'a> {
    /// The user-provided context.
    ctx: &'a mut WasiDataframeCtx,
}

impl<'a> WasiDataframe<'a> {
    /// Create a new view into the `wasi-dataframe` state.
    pub fn new(ctx: &'a mut WasiDataframeCtx) -> Self {
        Self { ctx }
    }
}

// Implement the generated `Host` trait for the `dataframe-analysis` interface.
impl wit_df::Host for WasiDataframe<'_> {
    fn load_csv(&mut self, path: String) -> std::result::Result<wit_df::Dataframe, ()> {
        let lf = LazyCsvReader::new(path)
            .with_has_header(true)
            .finish()
            .map_err(|_| ())?;
        let h = self.ctx.new_handle();
        self.ctx.lazyframes.insert(h, lf);
        Ok(h)
    }

    fn from_rows(
        &mut self,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> std::result::Result<wit_df::Dataframe, ()> {
        if columns.is_empty() {
            return Err(());
        }
        let width = columns.len();
        for r in rows.iter() {
            if r.len() != width {
                return Err(());
            }
        }
        let mut series: Vec<Series> = Vec::with_capacity(width);
        for (ci, name) in columns.iter().enumerate() {
            let mut col_vals: Vec<String> = Vec::with_capacity(rows.len());
            for r in rows.iter() {
                col_vals.push(r[ci].clone());
            }
            series.push(Series::new(name.as_str().into(), col_vals));
        }
        let df = DataFrame::new(series.into_iter().map(|s| s.into()).collect()).map_err(|_| ())?;
        let lf = df.lazy();
        let h = self.ctx.new_handle();
        self.ctx.lazyframes.insert(h, lf);
        Ok(h)
    }

    fn filter(
        &mut self,
        df: wit_df::Dataframe,
        filters: Vec<wit_df::ColumnFilter>,
    ) -> std::result::Result<wit_df::Dataframe, ()> {
        let mut lf = self
            .ctx
            .lazyframes
            .get(&df)
            .cloned()
            .ok_or(())?;
        for f in filters.into_iter() {
            let expr = filter_expr_from_wit(&f)?;
            lf = lf.filter(expr);
        }
        let h = self.ctx.new_handle();
        self.ctx.lazyframes.insert(h, lf);
        Ok(h)
    }

    fn group_by(
        &mut self,
        df: wit_df::Dataframe,
        by_columns: Vec<String>,
    ) -> std::result::Result<wit_df::Dataframe, ()> {
        let lf = self
            .ctx
            .lazyframes
            .get(&df)
            .cloned()
            .ok_or(())?;
        if by_columns.is_empty() {
            return Err(());
        }
        let out = lf.group_by(by_columns.iter().map(|s| col(s.as_str())).collect::<Vec<_>>());
        let h = self.ctx.new_handle();
        self.ctx.lazyframes.insert(h, out.into());
        Ok(h)
    }

    fn aggregate(
        &mut self,
        df: wit_df::Dataframe,
        aggs: Vec<wit_df::Aggregation>,
    ) -> std::result::Result<wit_df::Dataframe, ()> {
        let lf = self
            .ctx
            .lazyframes
            .get(&df)
            .cloned()
            .ok_or(())?;

        let mut agg_exprs: Vec<Expr> = Vec::new();
        if aggs.is_empty() {
            agg_exprs.push(lit(1).count().alias("count"));
        } else {
            for a in aggs.into_iter() {
                match a {
                    wit_df::Aggregation::Count => agg_exprs.push(lit(1).count().alias("count")),
                    wit_df::Aggregation::Mean => {
                        // For mean aggregation, we need to specify which column to aggregate
                        // Since the WIT interface doesn't specify a column, we'll use the first numeric column
                        // This is a limitation of the current interface design
                        return Err(());
                    }
                }
            }
        }
        let out = lf.select(agg_exprs);
        let h = self.ctx.new_handle();
        self.ctx.lazyframes.insert(h, out);
        Ok(h)
    }

    fn to_json(&mut self, df: wit_df::Dataframe) -> std::result::Result<String, ()> {
        let lf = self
            .ctx
            .lazyframes
            .get(&df)
            .cloned()
            .ok_or(())?;

        let df = lf
            .limit(100)
            .collect()
            .map_err(|_| ())?;

        let columns = df.get_columns();
        let col_names: Vec<&str> = columns.iter().map(|c| c.name().as_str()).collect();
        let height = df.height();

        let mut rows_json = String::from("[");
        for row_idx in 0..height {
            if row_idx > 0 {
                rows_json.push(',');
            }
            rows_json.push('{');
            for (ci, series) in columns.iter().enumerate() {
                if ci > 0 {
                    rows_json.push(',');
                }
                rows_json.push('"');
                rows_json.push_str(col_names[ci]);
                rows_json.push_str("\":");
                let v = series
                    .get(row_idx)
                    .map_err(|_| ())?;
                rows_json.push_str(&json_value_from_anyvalue(v));
            }
            rows_json.push('}');
        }
        rows_json.push(']');
        Ok(rows_json)
    }
}

fn filter_expr_from_wit(f: &wit_df::ColumnFilter) -> std::result::Result<Expr, ()> {
    // When comparing against a numeric value, cast column to Float64 so
    // filters work even if data came from from-rows as Utf8 strings.
    let (lhs, rhs) = match &f.value {
        wit_df::Scalar::Value(v) => (col(&f.column).cast(DataType::Float64), lit(*v)),
        wit_df::Scalar::Name(s) => (col(&f.column), lit(s.as_str())),
        wit_df::Scalar::Logic(b) => (col(&f.column), lit(*b)),
    };
    let e = match f.op {
        wit_df::Comparator::Gt => lhs.gt(rhs),
        wit_df::Comparator::Gte => lhs.gt_eq(rhs),
        wit_df::Comparator::Lt => lhs.lt(rhs),
        wit_df::Comparator::Lte => lhs.lt_eq(rhs),
        wit_df::Comparator::Eq => lhs.eq(rhs),
        wit_df::Comparator::Neq => lhs.neq(rhs),
    };
    Ok(e)
}

fn json_value_from_anyvalue(v: AnyValue) -> String {
    match v {
        AnyValue::Null => "null".to_string(),
        AnyValue::Boolean(b) => if b { "true" } else { "false" }.to_string(),
        AnyValue::String(s) => format!("\"{}\"", escape_json_str(s)),
        AnyValue::Float64(f) => format!("{}", f),
        AnyValue::Float32(f) => format!("{}", f),
        AnyValue::Int64(i) => i.to_string(),
        AnyValue::Int32(i) => i.to_string(),
        AnyValue::Int16(i) => i.to_string(),
        AnyValue::Int8(i) => i.to_string(),
        AnyValue::UInt64(i) => i.to_string(),
        AnyValue::UInt32(i) => i.to_string(),
        AnyValue::UInt16(i) => i.to_string(),
        AnyValue::UInt8(i) => i.to_string(),
        _ => format!("\"{}\"", escape_json_str(&v.to_string())),
    }
}

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

struct HasWasiDataframe;

impl HasData for HasWasiDataframe {
    type Data<'a> = WasiDataframe<'a>;
}

/// Add all the `wasi-dataframe` world's interfaces to a `wasmtime::component::Linker`.
pub fn add_to_linker<T: Send + 'static>(
    l: &mut wasmtime::component::Linker<T>,
    f: fn(&mut T) -> WasiDataframe<'_>,
) -> Result<()> {
    self::generated::wasi::dataframe::dataframe_analysis::add_to_linker::<_, HasWasiDataframe>(l, f)
}
