// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), std::io::Error> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        //.out_dir("src/")
        .compile(
            &["src/v0.proto"],
            &["src"]
        )
}