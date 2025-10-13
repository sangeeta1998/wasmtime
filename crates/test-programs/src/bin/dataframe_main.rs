use test_programs::wasi::dataframe::dataframe_analysis::{
    from_rows, to_json
};

fn main() {
    println!("=== WebAssembly DataFrame Test ===");
    
    // Test 1: Basic dataframe creation and JSON conversion
    println!("Test 1: Creating dataframe from rows...");
    
    let columns = vec!["city".to_string(), "group".to_string(), "val".to_string()];
    let rows = vec![
        vec!["A".to_string(), "x".to_string(), "10".to_string()],
        vec!["A".to_string(), "y".to_string(), "5".to_string()],
        vec!["B".to_string(), "x".to_string(), "7".to_string()],
        vec!["B".to_string(), "y".to_string(), "3".to_string()],
    ];
    
    println!("Columns: {:?}", columns);
    println!("Rows: {:?}", rows);
    
    // Create dataframe
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
    
    // Convert to JSON
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
    
    // Show result
    println!("JSON result: {}", json);
    
    // Basic validation
    if json.starts_with("[") && json.contains("A") {
        println!("✅ Test 1 passed: Basic dataframe operations work");
    } else {
        println!("❌ Test 1 failed: Invalid JSON output");
        std::process::exit(1);
    }
    
    // Test 2: Aggregation::Count over the whole dataframe
    println!("\nTest 2: Aggregation::Count over all rows...");
    {
        use test_programs::wasi::dataframe::dataframe_analysis::{aggregate, Aggregation};
        // Recreate the dataframe for a fresh handle
        let df2 = match from_rows(&columns, &rows) {
            Ok(df) => df,
            Err(e) => {
                println!("❌ Failed to create dataframe for aggregation: {:?}", e);
                std::process::exit(1);
            }
        };
        let aggs = vec![Aggregation::Count];
        let agg_df = match aggregate(df2, &aggs) {
            Ok(df) => df,
            Err(()) => {
                println!("❌ aggregate(count) returned error");
                std::process::exit(1);
            }
        };
        let agg_json = match to_json(agg_df) {
            Ok(s) => s,
            Err(()) => {
                println!("❌ Failed to to_json aggregated dataframe");
                std::process::exit(1);
            }
        };
        println!("Aggregation result JSON: {}", agg_json);
        if !agg_json.contains("count") {
            println!("❌ Aggregation output missing 'count' field");
            std::process::exit(1);
        }
        println!("✅ Aggregation::Count test passed");
    }
    
    println!("🎉 Basic test completed successfully!");
    println!("\n📋 Advanced Operations Status:");
    println!("✅ from_rows() - Working");
    println!("✅ to_json() - Working");
    println!("🧪 filter() - Implemented in host, needs integration testing");
    println!("🧪 group_by() - Implemented in host, needs integration testing");
    println!("🧪 aggregate() - Implemented in host, needs integration testing");
    println!("🧪 load_csv() - Implemented in host, needs integration testing");
    println!("\n💡 Note: Advanced operations are implemented in the host code");
    println!("   but may need additional integration work for the test environment.");
}