use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use wasmtime_wasi_dataframe::{WasiDataframe, WasiDataframeCtx, WasiDataframeCtxBuilder};

// Bring the generated trait into scope so we can call the host methods.
use wasmtime_wasi_dataframe::bindings::wasi::accelerator::dataframe_analysis::Host as _;

#[test]
fn dataframe_smoke() -> Result<()> {
	// Create a small CSV in a temp dir
	let mut dir = std::env::temp_dir();
	dir.push("wasi_df_test");
	std::fs::create_dir_all(&dir)?;
	let mut csv_path = PathBuf::from(&dir);
	csv_path.push("data.csv");
	let mut f = File::create(&csv_path)?;
	writeln!(f, "city,group,val")?;
	writeln!(f, "A,x,10")?;
	writeln!(f, "A,y,5")?;
	writeln!(f, "B,x,7")?;
	writeln!(f, "B,y,3")?;
	f.flush()?;

	// Build context and call host API directly
	let mut ctx: WasiDataframeCtx = WasiDataframeCtxBuilder::new().build();
	let mut host = WasiDataframe::new(&mut ctx);

	let df = host.load_csv(csv_path.to_string_lossy().to_string())?;
	let df_filtered = host.filter(df, "val > 5".to_string())?;
	let df_grouped = host.group_by(df_filtered, vec!["group".to_string()])?;
	let df_agg = host.aggregate(df_grouped, vec!["mean(val)".to_string(), "count()".to_string()])?;
	let json = host.to_json(df_agg)?;

	// Basic assertions
	assert!(json.starts_with("["));
	assert!(json.contains("mean_val"));
	assert!(json.contains("count"));

	Ok(())
}
