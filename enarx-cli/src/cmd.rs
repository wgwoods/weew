// SPDX-License-Identifier: Apache-2.0

mod noop;
mod run;
mod serve;
mod info;

use anyhow::Result;

// Built-in subcommands need to implement this trait.
pub trait SubCommand {
    fn execute(self) -> Result<()>;
}

pub use {
    noop::NoopOptions,
    run::RunOptions,
    serve::ServeOptions,
    info::InfoOptions,
};
