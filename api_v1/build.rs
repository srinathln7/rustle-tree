

// `dyn` is used to create trait objects which enable dynamic dispatch (involves additional runtime costs). Instead of knowing the exact type at compile time, 
// dynamic dispatch allows method calls to be determined at runtime. This is useful for handling multiple types that implement the same trait, 
// like std::error::Error, in a flexible way. In this case, `dyn std::error::Error` allows the function to return different error types, 
// as long as they implement the std::error::Error trait. The Box is used to allocate the error on the heap, which is necessary because trait 
// objects must be dynamically sized and cannot be stored directly on the stack.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `tonic_build` is a helper library crate that integrates with Tonic, a gRPC client and server implementation in Rust.
    tonic_build::compile_protos("src/proto/rustle_tree.proto")?;
    Ok(())
}

