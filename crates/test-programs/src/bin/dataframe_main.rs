use std::path::PathBuf;
use test_programs::wasi::dataframe::dataframe_analysis::{
    Dataframe, aggregate, filter, from_rows, group_by, 
    load_csv, to_json, 
    ColumnFilter, Comparator, Scalar
};

// Creates a mutable `PathBuf` named `p` initialized with the path to the current crate's root directory,
// as determined by the `CARGO_MANIFEST_DIR` environment variable at compile time.
fn fixture_csv() -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("wasi-dataframe");
    p.push("tests");
    p.push("data");
    p.push("sample.csv");
    p.to_string_lossy().to_string()
}

// A test function that loads a CSV file into a dataframe and converts it to JSON format.
fn load_and_json_preview() -> Result<(), String> {
    let df = load_csv(fixture_csv().as_ref()).unwrap();
    let json = to_json(df).unwrap();
    if !json.starts_with("[") || !json.contains("city") {
        Err(format!("Unexpected JSON output: {}", json))
    } else {
        Ok(())
    }
}

// not done yet
// fn filter_with_enums() -> Result<(), String> {
//     let df = load_csv(fixture_csv().as_ref()).unwrap();
//     let filters = vec![ ColumnFilter {
//         column: "val".to_string(),
//         op: Comparator    ::Gt(()),
//         value: Scalar::value(5.0),
//     }];
//     let df2 = host.filter(df, filters)?;
//     let json = host.to_json(df2)?;
//     assert!(json.contains("10") || json.contains("7"));
//     Ok(())
// }

// fn group_and_aggregate() -> Result<()> {
//     let mut ctx = build_ctx();
//     let mut host = WasiDataframe::new(&mut ctx);
//     let df = host.load_csv(fixture_csv())?;
//     let df_g = host.group_by(df, vec!["group".to_string()])?;
//     let df_a = host.aggregate(
//         df_g,
//         vec![
//             wit_df::Aggregation::Count(()),
//             wit_df::Aggregation::Mean("val".to_string()),
//         ],
//     )?;
//     let json = host.to_json(df_a)?;
//     assert!(json.contains("count"));
//     assert!(json.contains("mean_val"));
//     Ok(())
// }

// fn from_rows_constructor() -> Result<()> {
//     let mut ctx = build_ctx();
//     let mut host = WasiDataframe::new(&mut ctx);
//     let cols = vec!["city".to_string(), "group".to_string(), "val".to_string()];
//     let rows = vec![
//         vec!["A".to_string(), "x".to_string(), "10".to_string()],
//         vec!["B".to_string(), "y".to_string(), "5".to_string()],
//     ];
//     let df = host.from_rows(cols, rows)?;
//     let json = host.to_json(df)?;
//     assert!(json.contains("A"));
//     assert!(json.contains("B"));
//     Ok(())
// }

fn main() {
    assert_eq!(load_and_json_preview(), Ok(()));
    // filter_with_enums().unwrap();
    // group_and_aggregate().unwrap();
    // from_rows_constructor().unwrap();
}