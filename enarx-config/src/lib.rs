// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;
use std::os::unix::io::RawFd;
use std::os::unix::prelude::AsRawFd;
use wasmparser::WasmFeatures;

// TODO: TLS stuff, keymgr URI, etc.
//pub struct KeepConfig { }

/// Settings for the runtime environment
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