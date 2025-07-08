use test_programs::wasi::accelerator::host_allocator::{
    MatrixDimensions, 
    allocate_buffer,
    free_buffer, 
    write_to_host, 
    register_matrix_dimensions, 
    get_matrix_dimensions,
    read_from_host,
    matrix_multiply_f32,
};

fn main() {
    let handle = allocate_buffer(64).unwrap();

    assert_eq!(handle, 1);

    assert!(free_buffer(1).is_ok());

    let handle = allocate_buffer(64).unwrap();

    assert_eq!(handle, 2);

    let a_data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let b_data: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
    let dims_a = MatrixDimensions { rows: 2, cols: 2 };
    let dims_b = MatrixDimensions { rows: 2, cols: 2 };

    let a_bytes = f32_vec_to_bytes(&a_data);
    let b_bytes = f32_vec_to_bytes(&b_data);

    let handle_a = allocate_buffer(a_bytes.len() as u64).unwrap();
    let handle_b = allocate_buffer(b_bytes.len() as u64).unwrap();

    assert_eq!(handle_a, 3);
    assert_eq!(handle_b, 4);

    assert!(write_to_host(&a_bytes, handle_a, 0).is_ok()); 
    assert!(register_matrix_dimensions(handle_a, dims_a).is_ok());

    assert!(write_to_host(&b_bytes, handle_b, 0).is_ok());
    println!("[Client Wasm] Wrote B data to host");
    assert!(register_matrix_dimensions(handle_b, dims_b).is_ok());
    println!("[Client Wasm] Registered B dimensions");

    // 3. Perform matrix multiplication
    let handle_c = matrix_multiply_f32(handle_a, handle_b).unwrap();
    println!(
        "[Client Wasm] Matrix multiplication done. Result C handle: {}",
        handle_c
    );

    // 4. Get dimensions of C and read C back
    let dims_c = get_matrix_dimensions(handle_c).unwrap();
    println!(
        "[Client Wasm] Got C dimensions: {}x{}",
        dims_c.rows, dims_c.cols
    );

    let c_byte_len = (dims_c.rows * dims_c.cols * std::mem::size_of::<f32>() as u32) as u64;
    let c_bytes = read_from_host(handle_c, 0, c_byte_len).unwrap();
    assert_eq!(c_bytes.len(), 16 );

    let c_data = bytes_to_f32_vec(&c_bytes).unwrap();
    println!("[Client Wasm] Result C: {:?}", c_data);

    // Expected: [19.0, 22.0, 43.0, 50.0]
    let expected_c: Vec<f32> = vec![19.0, 22.0, 43.0, 50.0];
    assert!( c_data
        .iter()
        .zip(expected_c.iter())
        .all(|(a, b)| (a - b).abs() < f32::EPSILON));
 
    // 5. Free host buffers
    assert!(free_buffer(handle_a).is_ok());
    assert!(free_buffer(handle_b).is_ok());
    assert!(free_buffer(handle_c).is_ok());
    println!("[Client Wasm] Freed all handles.");

}



fn f32_vec_to_bytes(data: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(data.len() * std::mem::size_of::<f32>());
    for val in data {
        bytes.extend_from_slice(&val.to_ne_bytes());
    }
    bytes
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return None;
    }
    let mut result = Vec::with_capacity(bytes.len() / std::mem::size_of::<f32>());
    for chunk in bytes.chunks_exact(std::mem::size_of::<f32>()) {
        result.push(f32::from_ne_bytes(chunk.try_into().unwrap()));
    }
    Some(result)
}
