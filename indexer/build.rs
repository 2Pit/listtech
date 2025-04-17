fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();

    config.compile_well_known_types();
    // .type_attribute(".", "#[derive(Debug)]");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/api/proto")
        .compile_protos_with_config(
            config,
            &["proto/api.proto"],
            &["proto", "../corelib/proto/third_party"],
        )?;

    println!("cargo:rerun-if-changed=proto/api.proto");
    println!("cargo:rerun-if-changed=../corelib/proto/third_party");

    Ok(())
}
