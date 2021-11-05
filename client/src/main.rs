// SPDX-License-Identifier: Apache-2.0

//! This crate provides the `enarx` executable, which is a tool for running
//! code inside an Enarx Keep - that is a hardware isolated environment using
//! technologies such as Intel SGX or AMD SEV.
//!
//! For more information about the project and the technology used
//! visit the [Enarx Project home page](https://enarx.dev/).
//!
//! # SGX and SEV machine setup
//!
//! Please see
//! [this wiki page](https://github.com/enarx/enarx/wiki/Reproducible-builds-and-Machine-setup)
//! for instructions.
//!
//! # Building and Testing Enarx
//!
//! Please see [BUILD.md](https://github.com/enarx/enarx/blob/main/BUILD.md) for instructions.
//!
//! # Installing Enarx
//!
//! Please see
//! [this wiki page](https://github.com/enarx/enarx/wiki/Install-Enarx)
//! for instructions.
//!
//! # Build and run a WebAssembly module
//!
//! Install the Webassembly rust toolchain:
//!
//!     $ rustup target install wasm32-wasi
//!
//! Create simple rust program:
//!
//!     $ cargo init --bin hello-world
//!     $ cd hello-world
//!     $ echo 'fn main() { println!("Hello, Enarx!"); }' > src/main.rs
//!     $ cargo build --release --target=wasm32-wasi
//!
//! Assuming you did install the `enarx` binary and have it in your `$PATH`, you can
//! now run the Webassembly program in an Enarx keep.
//!
//!     $ enarx run target/wasm32-wasi/release/hello-world.wasm
//!     […]
//!     Hello, Enarx!
//!
//! If you want to suppress the debug output, add `2>/dev/null`.
//!
//! # Select a Different Backend
//!
//! `enarx` will probe the machine it is running on in an attempt to deduce an
//! appropriate deployment backend. To see what backends are supported on your
//! system, run:
//!
//!     $ enarx info
//!
//! You can manually select a backend with the `--backend` option, or by
//! setting the `ENARX_BACKEND` environment variable:
//!
//!     $ enarx run --backend=sgx target/wasm32-wasi/release/hello-world.wasm
//!     $ ENARX_BACKEND=sgx enarx run target/wasm32-wasi/release/hello-world.wasm

#![deny(clippy::all)]
#![deny(missing_docs)]

mod cli;

use log::info;
use anyhow::Result;
use structopt::StructOpt;

// This defines the toplevel `enarx` CLI
#[derive(StructOpt, Debug)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
)]
struct Options {
    /// Logging options
    #[structopt(flatten)]
    log: cli::LogOptions,

    /// Enarx host service to connect to.
    #[structopt(env = "ENARX_HOST", long_help = r"
Example URIs:
    unix:/path/to/enarx.socket
    tcp://enarx.host:port
    ssh://[user@]enarx.host[:port]/path/to/enarx.socket
"
    )]
    host: cli::EnarxHost,

    /// Subcommands (with their own options)
    #[structopt(flatten)]
    cmd: cli::Command,
}

fn main() -> Result<()> {
    let opts = Options::from_args();
    opts.log.init_logger();

    info!("logging initialized!");
    info!("CLI opts: {:?}", &opts);

    {
        use cli::Command::*;
        match opts.cmd {
            Noop(noop) => { println!("yayyyy, did nothing") },
            Run(run) => { println!("stub") },
        }
    }

    Ok(())
}
