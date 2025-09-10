use anyhow::Result;
use std::path::PathBuf;

use wasmtime_wasi_dataframe::{WasiDataframe, WasiDataframeCtx, WasiDataframeCtxBuilder};
use wasmtime_wasi_dataframe::bindings::wasi::accelerator::dataframe_analysis as wit_df;
use wit_df::Host as _;

fn build_ctx() -> WasiDataframeCtx { WasiDataframeCtxBuilder::new().build() }

fn fixture_csv() -> String {
	let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	p.push("tests");
	p.push("data");
	p.push("sample.csv");
	p.to_string_lossy().to_string()
}

#[test]
fn load_and_json_preview() -> Result<()> {
	let mut ctx = build_ctx();
	let mut host = WasiDataframe::new(&mut ctx);
	let df = host.load_csv(fixture_csv())?;
	let json = host.to_json(df)?;
	assert!(json.starts_with("["));
	assert!(json.contains("city"));
	Ok(())
}

#[test]
fn filter_with_enums() -> Result<()> {
	let mut ctx = build_ctx();
	let mut host = WasiDataframe::new(&mut ctx);
	let df = host.load_csv(fixture_csv())?;
	let filters = vec![wit_df::ColumnFilter{ column: "val".to_string(), op: wit_df::Comparator::Gt(()), value: wit_df::Scalar::F64(5.0)}];
	let df2 = host.filter(df, filters)?;
	let json = host.to_json(df2)?;
	assert!(json.contains("10") || json.contains("7"));
	Ok(())
}

#[test]
fn group_and_aggregate() -> Result<()> {
	let mut ctx = build_ctx();
	let mut host = WasiDataframe::new(&mut ctx);
	let df = host.load_csv(fixture_csv())?;
	let df_g = host.group_by(df, vec!["group".to_string()])?;
	let df_a = host.aggregate(df_g, vec![wit_df::Aggregation::Count(()), wit_df::Aggregation::Mean("val".to_string())])?;
	let json = host.to_json(df_a)?;
	assert!(json.contains("count"));
	assert!(json.contains("mean_val"));
	Ok(())
}

#[test]
fn from_rows_constructor() -> Result<()> {
	let mut ctx = build_ctx();
	let mut host = WasiDataframe::new(&mut ctx);
	let cols = vec!["city".to_string(), "group".to_string(), "val".to_string()];
	let rows = vec![
		vec!["A".to_string(), "x".to_string(), "10".to_string()],
		vec!["B".to_string(), "y".to_string(), "5".to_string()],
	];
	let df = host.from_rows(cols, rows)?;
	let json = host.to_json(df)?;
	assert!(json.contains("A"));
	assert!(json.contains("B"));
	Ok(())
}
