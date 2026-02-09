fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/blog.proto");
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        .compile_protos(&["proto/blog.proto"], &["proto"])?;
    Ok(())
}
