// SPDX-License-Identifier: Apache-2.0

use crate::cmd::SubCommand;
use crate::util::ListenFds;

use std::collections::HashMap;
use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::{bail, Context, Result};
use log::{debug, info};

use tonic::{transport::Server, Request, Response, Status};

mod v0 {
    tonic::include_proto!("enarx.v0");
}

use v0::keepldr_server::{Keepldr, KeepldrServer};
use v0::{InfoRequest, KeepldrInfo, BackendInfo};

#[cfg(unix)]
use std::os::unix::{io::AsRawFd, io::FromRawFd, net::UnixStream};

type TonicResult<T> = std::result::Result<Response<T>, Status>;

#[tonic::async_trait]
impl<T> Keepldr for KeepldrServer<T>
where T: Keepldr
{
    async fn info(&self, req: Request<InfoRequest>) -> TonicResult<KeepldrInfo> {
        let keepldrinfo = KeepldrInfo {
            name: "enarx serve".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            sallyport_version: "0.1.0".to_string(), // FIXME
            backend: Some(BackendInfo { sgx: None, kvm: None, sev: None }),    
        };
        Ok(Response::new(keepldrinfo))
    }

    async fn boot(&self, request: Request<v0::BootRequest>) -> TonicResult<v0::Result> {
        let boot = request.get_ref();
        let result = v0::Result {
            code: v0::Code::Unknown as i32,
            message: format!("got shim ({} bytes) and exec ({} bytes)", boot.shim, boot.exec),
            details: vec![],
        };
        Ok(Response::new(result))
    }
}

/// Handle an incoming request as a systemd socket-activated service
#[derive(StructOpt, Debug)]
pub struct ServeOptions {
    /// Handle a connection from a systemd socket unit with "Accept=yes"
    #[structopt(long)]
    pub systemd_socket_accept: bool,

    /// Idle connection timeout time, in milliseconds (0=forever)
    #[structopt(long, default_value = "5000")]
    pub idle_timeout: u64,

    /// Socket path to listen on
    #[structopt(required_unless = "systemd-socket-accept")]
    pub socket_path: Option<PathBuf>,
}

impl ServeOptions {
    fn serve(&self, sock: UnixStream) -> Result<()> {
        // TODO: apply idle_timeout
        // TODO: actually... serve
        info!("you did it!!");
        Ok(())
    }

    fn accept_from_systemd(&self) -> Result<UnixStream> {
        // Get systemd socket info
        let listen_fds = ListenFds::take_from_env()?;
        debug!("got fds: {:?}", listen_fds);
        let sock = match listen_fds.get_connection_fd() {
            None => bail!("can't find fd for incoming socket connection"),
            Some(fd) => unsafe { UnixStream::from_raw_fd(fd) },
        };
        debug!("fd {} local_addr {:?}", sock.as_raw_fd(), sock.local_addr()?);
        debug!("INSTANCE_ID: {:?}", std::env::var("INSTANCE_ID"));
        // If provided, check CLI-provided path against actual socket path
        if let Some(ref expect_path) = self.socket_path {
            let addr = sock.local_addr()?;
            let socket_path = addr.as_pathname();
            if socket_path != Some(expect_path) {
                bail!("socket path {:?} does not match expected path {:?}",
                        socket_path, expect_path);
            }
        }
        Ok(sock)
    }
}

impl SubCommand for ServeOptions {
    fn execute(self) -> Result<()> {
        if self.systemd_socket_accept {
            match self.accept_from_systemd() {
                Err(e) => bail!("Failed to get socket from systemd: {}", e),
                Ok(sock) => self.serve(sock),
            }
        } else {
            bail!("TODO! Use --systemd-socket-accept instead.")
        }
    }
}