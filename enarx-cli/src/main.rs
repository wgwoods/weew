// SPDX-License-Identifier: Apache-2.0

/// enarx-cli - the command-line frontend for running code in an Enarx Keep.
pub mod cmd;
pub mod proto;
mod util;

use anyhow::{bail, Result};
use log::{debug, info};
use structopt::{clap::AppSettings, StructOpt};

use cmd::{NoopOptions, RunOptions, ServeOptions, SubCommand};

/// Logging options
#[derive(StructOpt, Debug)]
struct LogOpts {
    /// Pass many times for more log output.
    ///
    /// By default we only show error messages. Passing `-v` will show warnings,
    /// `-vv` adds info, `-vvv` for debug, and `-vvvv` for trace.
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: u8,

    /// Set logging filters
    #[structopt(long = "log-filter", env = "ENARX_LOG")]
    filter: Option<String>,
    // TODO: log_style, log_target, syslog..?
}

impl LogOpts {
    fn verbosity_level(&self) -> log::LevelFilter {
        match self.verbosity {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    }

    fn init_logger(&self) {
        let mut builder = env_logger::Builder::from_default_env();
        if let Some(ref filter) = self.filter {
            builder.parse_filters(filter);
        }
        builder.filter_level(self.verbosity_level());
        // TODO: style, target
        builder.init();
    }
}

/// Subcommands
#[derive(StructOpt, Debug)]
enum EnarxCommand {
    Run(RunOptions),
    Noop(NoopOptions),
    Serve(ServeOptions),
}

// FUTURE: handle external subcommands
impl EnarxCommand {
    fn execute(self) -> Result<()> {
        match self {
            Self::Run(c) => c.execute(),
            Self::Noop(c) => c.execute(),
            Self::Serve(c) => c.execute(),
        }
    }
}

/// The Enarx CLI
#[derive(StructOpt, Debug)]
#[structopt(
    name = "enarx",
    // Don't split flags and options into different groups
    setting = AppSettings::UnifiedHelpMessage,
    // List the options in the same order as they appear in the struct
    setting = AppSettings::DeriveDisplayOrder,
)]
struct EnarxApp {
    #[structopt(flatten)]
    log_opts: LogOpts,

    #[structopt(subcommand)]
    cmd: EnarxCommand,
}

fn main() -> Result<()> {
    let opts = EnarxApp::from_args();
    opts.log_opts.init_logger();

    info!("enarx version {}", env!("CARGO_PKG_VERSION"));
    debug!("opts: {:#?}", opts);

    opts.cmd.execute()
}
