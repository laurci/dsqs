use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path =
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("dsqs_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["../proto/dsqs.proto"], &["../proto/"])?;
    Ok(())
}
