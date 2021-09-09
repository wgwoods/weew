// SPDX-License-Identifier: Apache-2.0

use crate::cmd::SubCommand;
use anyhow::{bail, Context, Result};
use log::{debug, info};
use structopt::StructOpt;

use std::{fmt::Debug, path::PathBuf};

use std::fs::File;
//use std::net::Shutdown;

#[cfg(unix)]
use std::os::unix::{io::AsRawFd, net::UnixStream};
use std::io::Read;
use enarx_config::EnvConfig;

/// Run a WebAssembly module inside an Enarx Keep.
#[derive(StructOpt, Debug)]
pub struct RunOptions {
    /// Set an environment variable for the program
    #[structopt(
        short = "e",
        long = "env",
        number_of_values = 1,
        value_name = "NAME=VAL",
        parse(try_from_str=parse_env_var),
    )]
    pub envs: Vec<(String, String)>,

    // TODO: --inherit-env
    /// Name of the function to invoke
    #[structopt(long, value_name = "FUNCTION")]
    pub invoke: Option<String>,

    // TODO: --stdin, --stdout, --stderr
    /// Path of the WebAssembly module to run
    #[structopt(index = 1, value_name = "MODULE", parse(from_os_str))]
    pub module: PathBuf,
    
    /// Arguments to pass to the WebAssembly module
    #[structopt(value_name = "ARGS", last = true)]
    pub args: Vec<String>,
}

fn parse_env_var(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("must be of the form `NAME=VAL`");
    }
    Ok((parts[0].to_owned(), parts[1].to_owned()))
}


impl RunOptions {
    // The general idea here is something like this:
    // 1. Open a socketpair
    //    Add the "remote" side to the list of FDs to inherit
    // 2. Tell local keepldr to load wasmldr in a keep (inheriting socket)
    // 3. Send config over socket to wasmldr
    // 4. Send module over socket to wasmldr
    // 5. Wait for wasmldr to ack / close socket

    fn get_module_reader(&self) -> Result<File> {
        // TODO: self.module_on_fd
        File::open(&self.module).with_context(|| format!("could not open {:?}", self.module))
    }
    
    #[cfg(unix)]
    fn local_keepmgr(&self) -> Result<()> {
        let (sock_l, sock_r) = UnixStream::pair()?;
        debug!(
            "created unix socket pair: fd{}<->fd{}",
            sock_l.as_raw_fd(),
            sock_r.as_raw_fd()
        );
        bail!("Not implemented yet!");
    }
}


#[derive(Debug)]
struct KeepBuilder {
    env_config: EnvConfig,
}
impl KeepBuilder {
    fn new() -> Self {
        Self {
            env_config: Default::default(),
        }
    }

    fn inherit_stdio(mut self, inherit: bool) -> Self {
        if inherit {
            self.env_config = self.env_config.inherit_stdio();
        }
        self
    }

    fn default_loader(self) -> Self {
        // TODO/FUTURE
        self
    }
    
    fn build(self) -> Result<KeepConn> {
        Ok(KeepConn {})
    }
}

#[derive(Debug)]
struct KeepConn {}

#[derive(Debug)]
struct Report {}

impl KeepConn {
    fn config(self) -> Result<Self> {
        Ok(self)
    }

    fn envs<K, V>(self, envs: impl IntoIterator<Item = (K, V)>) -> Result<Self>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        Ok(self)
    }

    fn args<A>(self, args: impl IntoIterator<Item = A>) -> Result<Self>
    where
        A: AsRef<str>,
    {
        Ok(self)
    }

    fn module(self, module: impl Read + Debug) -> Result<Self> {
        debug!("loading module from {:?}", module);
        Ok(self)
    }

    fn function(self, func: Option<String>) -> Result<Self> {
        match func {
            Some(name) => {
                debug!("will invoke function {:?}", name);
                // TODO
            }
            None => {
                debug!("will invoke default function");
                // TODO
            }
        }
        Ok(self)
    }

    fn run(self) -> Result<Report> {
        Ok(Report {})
    }
}


impl SubCommand for RunOptions {
    /// Run a WebAssembly workload.
    fn execute(self) -> Result<()> {
        let module = self.get_module_reader()?;
        debug!("module open on fd{}", module.as_raw_fd());

        // Build a new, empty keep
        let keep = KeepBuilder::new()
            .default_loader()
            .inherit_stdio(true) // TODO: get from CLI
            .build()?;
        debug!("built keep: {:?}", keep);

        // Configure wasmldr, load code into keep, and run it
        let report = keep
            // Configure wasmldr/wasmtime
            .config(/*self.loader_config*/)?
            // Configure the WASI environment
            .envs(self.envs)?.args(self.args)?
            // Load the module into the keep
            .module(module)?
            // Look up the function we want to run
            .function(self.invoke)?
            // And run it!
            .run()?;
        debug!("report: {:?}", report);

        // Tada!
        Ok(())
    }
}
