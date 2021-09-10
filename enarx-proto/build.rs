// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), std::io::Error> {
    // NOTE: as of tonic-build 0.5 / prost-build 0.8, the .compile() function
    // doesn't emit "rerun-if-changed=PATH" directives for the .proto files
    // (see https://docs.rs/prost-build/0.8.0/src/prost_build/lib.rs.html#714)
    // so we have to do that ourselves.
    let proto_files = vec![
        "proto/v0.proto",
    ];
    let proto_include_path = vec![
        "proto/",
    ];
    for file in &proto_files {
        println!("cargo:rerun-if-changed={}", file)
    }
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir("src/")
        .compile(&proto_files, &proto_include_path)
}