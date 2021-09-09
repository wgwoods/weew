// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::os::unix::io::{RawFd, AsRawFd};
use wasmparser::WasmFeatures;

/// Options for setting up TLS connections
#[derive(StructOpt, Debug)]
pub struct TLSOptions {
    /// PEM-encoded certificate chain
    #[structopt(long)]
    pub cert: Option<PathBuf>,
    
    /// PEM-encoded private key
    #[structopt(long)]
    pub key: Option<PathBuf>,

    /// File containing trusted CA certificates
    #[structopt(long)]
    pub cacert: Option<PathBuf>,

    /// Directory containing trusted CA certificates
    #[structopt(long)]
    pub capath: Option<PathBuf>,
}


/// Settings for the workload's runtime environment
#[derive(Debug)]
pub struct EnvConfig {
    pub envs: Vec<(String, String)>,
    pub args: Vec<String>,
    pub stdin: Option<ReadHandle>,
    pub stdout: Option<WriteHandle>,
    pub stderr: Option<WriteHandle>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            envs: Default::default(),
            args: Default::default(),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }
}

impl EnvConfig {
    pub fn inherit_stdin(mut self) -> Self {
        self.stdin = Some(ReadHandle::Inherit(std::io::stdin().as_raw_fd()));
        self
    }

    pub fn inherit_stdout(mut self) -> Self {
        self.stdout = Some(WriteHandle::Inherit(std::io::stdout().as_raw_fd()));
        self
    }
    pub fn inherit_stderr(mut self) -> Self {
        self.stderr = Some(WriteHandle::Inherit(std::io::stderr().as_raw_fd()));
        self
    }
    pub fn inherit_stdio(self) -> Self {
        self.inherit_stdin().inherit_stdout().inherit_stderr()
    }

}

/// Options for 
#[derive(Debug)]
pub enum ReadHandle {
    Null,
    Inherit(RawFd),
    PlaintextSocket(SocketAddr),
}

#[derive(Debug)]
pub enum WriteHandle {
    Null,
    Inherit(RawFd),
    PlaintextSocket(SocketAddr),
}


pub struct WasmConfig {
    pub features: WasmFeatures,
}