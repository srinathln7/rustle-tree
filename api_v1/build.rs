fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/proto/rustle_tree.proto")?;
    Ok(())
}

// use prost_build::Config;

// fn main() {
//     let mut config = Config::new();

//     // Use field attributes to include `serde::Serialize`
//     config.type_attribute(
//         ".rustle_tree.TreeNode",
//         "#[derive(serde::Serialize)]"
//     );

//     // Compile the proto files
//     config.compile_protos(&["src/proto/rustle_tree.proto"]).unwrap();
// }
