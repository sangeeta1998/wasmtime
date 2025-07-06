use test_programs::wasi::acc::host_allocator;

fn main() {

    let handle = host_allocator::allocate_buffer(64);

    match handle {
        Ok(h) => println!("Allocated buffer with handle {}", h),
        Err(e) => println!("Failed to allocate buffer: {}", e),
    }


}
