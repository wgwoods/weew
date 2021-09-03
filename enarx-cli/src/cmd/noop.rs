// SPDX-License-Identifier: Apache-2.0

use crate::cmd::{Result, SubCommand};
use log::info;
///
use structopt::{clap::AppSettings, StructOpt};

/// Noop command. Really just a template for adding new commands.
#[derive(StructOpt, Debug)]
#[structopt(
    setting = AppSettings::TrailingVarArg,
)]
pub struct NoopOptions {
    /// Arguments, which will all be ignored. What fun!
    pub args: Vec<String>,
}

impl SubCommand for NoopOptions {
    fn execute(self) -> Result<()> {
        Ok(info!("it works! great job! here, have a hot dog: 🌭"))
    }
}
