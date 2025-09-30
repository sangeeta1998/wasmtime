use test_programs::wasi::dataframe::dataframe_analysis::{
    from_rows, to_json
};

fn main() {
    // Simple test: Create dataframe and convert to JSON
    // This follows the wasi-accelerator pattern of using programmatic data
    
    println!("Starting WebAssembly dataframe test...");
    
    // Create dataframe programmatically (no file system needed)
    let columns = vec!["city".to_string(), "group".to_string(), "val".to_string()];
    let rows = vec![
        vec!["A".to_string(), "x".to_string(), "10".to_string()],
        vec!["A".to_string(), "y".to_string(), "5".to_string()],
        vec!["B".to_string(), "x".to_string(), "7".to_string()],
        vec!["B".to_string(), "y".to_string(), "3".to_string()],
    ];
    
    println!("Creating dataframe from rows...");
    let df = match from_rows(&columns, &rows) {
        Ok(df) => {
            println!("✅ DataFrame created successfully");
            df
        },
        Err(e) => {
            println!("❌ Failed to create dataframe: {:?}", e);
            std::process::exit(1);
        }
    };
    
    println!("Converting dataframe to JSON...");
    let json = match to_json(df) {
        Ok(json) => {
            println!("✅ JSON conversion successful");
            json
        },
        Err(e) => {
            println!("❌ Failed to convert to JSON: {:?}", e);
            std::process::exit(1);
        }
    };
    
    // Validate the result
    println!("JSON result: {}", json);
    if json.starts_with("[") && json.contains("A") && json.contains("B") {
        println!("✅ DataFrame creation and JSON conversion test passed");
    } else {
        println!("❌ DataFrame creation test failed");
        std::process::exit(1);
    }
    
    println!("🎉 WebAssembly dataframe test completed successfully!");
}