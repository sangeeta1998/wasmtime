use test_programs::wasi::dataframe::dataframe_analysis::{
    load_csv, to_json
};

fn main() {
    // Simple test that loads CSV and converts to JSON
    // This will be executed as a WebAssembly component by the WASI runtime
    let df = load_csv("/tmp/sample.csv").unwrap();
    let json = to_json(df).unwrap();
    
    // Basic validation
    if json.starts_with("[") && json.contains("city") {
        println!("✅ CSV load and JSON conversion test passed");
    } else {
        println!("❌ CSV load and JSON conversion test failed");
        std::process::exit(1);
    }
}