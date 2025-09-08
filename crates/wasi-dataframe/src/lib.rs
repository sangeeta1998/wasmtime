//! # Wasmtime's [wasi-dataframe] Implementation
//!
//! This crate provides a Wasmtime host implementation of a proposed
//! `dataframe-analysis` interface under the `wasi:accelerator` package.
//! It exposes a subset of Polars operations to guest components.
//!
//! Currently supported operations:
//! - load-csv(path)
//! - filter(df, predicate_string) [very small subset parser]
//! - group-by(df, by_columns)
//! - aggregate(df, aggs) [supports mean(col) and count()]
//! - to-json(df) [serializes a small preview for now]
//

#![deny(missing_docs)]

mod generated {
	wasmtime::component::bindgen!({
		path: "wit",
		world: "wasi:accelerator/imports",
	});
}

// Re-export a public path to the generated bindings for external tests/examples.
pub use self::generated as bindings;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use wasmtime::component::HasData;

use polars::prelude::*;

// Bring generated interface into scope.
use self::generated::wasi::accelerator::dataframe_analysis as wit_df;

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
	fn load_csv(&mut self, path: String) -> Result<wit_df::Dataframe> {
		let lf = LazyCsvReader::new(path)
			.has_header(true)
			.finish()
			.map_err(|e| anyhow!("load-csv failed: {e}"))?;
		let h = self.ctx.new_handle();
		self.ctx.lazyframes.insert(h, lf);
		Ok(h)
	}

	fn filter(&mut self, df: wit_df::Dataframe, f: String) -> Result<wit_df::Dataframe> {
		let lf = self
			.ctx
			.lazyframes
			.get(&df)
			.cloned()
			.ok_or_else(|| anyhow!("invalid dataframe handle"))?;

		let expr = parse_simple_predicate(&f)?;
		let out = lf.filter(expr);
		let h = self.ctx.new_handle();
		self.ctx.lazyframes.insert(h, out);
		Ok(h)
	}

	fn group_by(&mut self, df: wit_df::Dataframe, by_columns: Vec<String>) -> Result<wit_df::Dataframe> {
		let lf = self
			.ctx
			.lazyframes
			.get(&df)
			.cloned()
			.ok_or_else(|| anyhow!("invalid dataframe handle"))?;
		if by_columns.is_empty() {
			return Err(anyhow!("group-by requires at least one column"));
		}
		let out = lf.group_by(by_columns);
		let h = self.ctx.new_handle();
		self.ctx.lazyframes.insert(h, out);
		Ok(h)
	}

	fn aggregate(&mut self, df: wit_df::Dataframe, aggs: Vec<String>) -> Result<wit_df::Dataframe> {
		let lf = self
			.ctx
			.lazyframes
			.get(&df)
			.cloned()
			.ok_or_else(|| anyhow!("invalid dataframe handle"))?;

		let mut agg_exprs: Vec<Expr> = Vec::new();
		if aggs.is_empty() {
			// Default to count if no aggregates provided.
			agg_exprs.push(Expr::count().alias("count"));
		} else {
			for s in aggs.iter() {
				if let Some(col) = s.strip_prefix("mean(").and_then(|t| t.strip_suffix(')')) {
					agg_exprs.push(col(col).mean().alias(&format!("mean_{}", col)));
				} else if s == "count()" {
					agg_exprs.push(Expr::count().alias("count"));
				} else {
					return Err(anyhow!("unsupported aggregation: {s}"));
				}
			}
		}
		let out = lf.agg(agg_exprs);
		let h = self.ctx.new_handle();
		self.ctx.lazyframes.insert(h, out);
		Ok(h)
	}

	fn to_json(&mut self, df: wit_df::Dataframe) -> Result<String> {
		let lf = self
			.ctx
			.lazyframes
			.get(&df)
			.cloned()
			.ok_or_else(|| anyhow!("invalid dataframe handle"))?;

		let df = lf
			.limit(100) // keep output small
			.collect()
			.map_err(|e| anyhow!("collect failed: {e}"))?;

		// Serialize rows to a JSON array of objects (simple/naive implementation)
		let columns = df.get_columns();
		let col_names: Vec<&str> = columns.iter().map(|c| c.name().as_str()).collect();
		let height = df.height();

		let mut rows_json = String::from("[");
		for row_idx in 0..height {
			if row_idx > 0 { rows_json.push(','); }
			rows_json.push('{');
			for (ci, series) in columns.iter().enumerate() {
				if ci > 0 { rows_json.push(','); }
				rows_json.push('"');
				rows_json.push_str(col_names[ci]);
				rows_json.push_str("":");
				let v = series.get(row_idx).map_err(|e| anyhow!("row access failed: {e}"))?;
				rows_json.push_str(&json_value_from_anyvalue(v));
			}
			rows_json.push('}');
		}
		rows_json.push(']');
		Ok(rows_json)
	}
}

fn json_value_from_anyvalue(v: AnyValue) -> String {
	match v {
		AnyValue::Null => "null".to_string(),
		AnyValue::Boolean(b) => if b { "true" } else { "false" }.to_string(),
		AnyValue::Utf8(s) => format!("\"{}\"", escape_json_str(s)),
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

fn parse_simple_predicate(s: &str) -> Result<Expr> {
	// Very small subset parser: "col > number" or "col == number"
	let parts: Vec<&str> = s.split_whitespace().collect();
	if parts.len() != 3 { return Err(anyhow!("unsupported filter predicate: {s}")); }
	let col_name = parts[0];
	let op = parts[1];
	let rhs_str = parts[2];
	let rhs_num = rhs_str.parse::<f64>().map_err(|_| anyhow!("rhs must be number: {rhs_str}"))?;
	let rhs = lit(rhs_num);
	let c = col(col_name);
	let expr = match op {
		">" => c.gt(rhs),
		">=" => c.gt_eq(rhs),
		"<" => c.lt(rhs),
		"<=" => c.lt_eq(rhs),
		"==" => c.eq(rhs),
		"!=" => c.neq(rhs),
		_ => return Err(anyhow!("unsupported operator in predicate: {op}")),
	};
	Ok(expr)
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
	self::generated::wasi::accelerator::dataframe_analysis::add_to_linker::<_, HasWasiDataframe>(l, f)
}

