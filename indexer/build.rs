use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let status = Command::new("protoc")
        .args([
            "-Iproto",                     // локальная папка с api.proto
            "-I../core/proto/third_party", // путь к google/api + google/protobuf
            "--openapiv2_out",
            &out_dir, // куда положить swagger.json
            "--openapiv2_opt",
            "logtostderr=true", // логгировать в stderr
            "proto/api.proto",
        ])
        .status()
        .expect("failed to run protoc for swagger generation");

    if !status.success() {
        panic!("protoc failed with status: {:?}", status);
    }

    // если изменится — перегенерировать
    println!("cargo:rerun-if-changed=proto/api.proto");
    println!("cargo:rerun-if-changed=../core/proto/third_party/google/api/annotations.proto");
    println!("cargo:rerun-if-changed=../core/proto/third_party/google/api/http.proto");
    println!("cargo:rerun-if-changed=../core/proto/third_party/google/protobuf/descriptor.proto");
}
