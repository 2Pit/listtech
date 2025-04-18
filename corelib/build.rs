fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../_proto");

    let out_dir = "src/proto";
    let proto_files = ["../_proto/indexer.proto", "../_proto/searcher.proto"];

    let includes = ["../_proto"];

    let mut config = prost_build::Config::new();
    config.compile_well_known_types();
    config.out_dir(out_dir);
    // config.type_attribute(".", "#[derive(Debug)]");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos_with_config(config, &proto_files, &includes)?;

    Ok(())
}
