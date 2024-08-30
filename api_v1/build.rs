fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/proto/rustle_tree.proto")?;
    Ok(())
}
