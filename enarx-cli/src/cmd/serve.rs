// SPDX-License-Identifier: Apache-2.0

use crate::cmd::SubCommand;
use crate::util::ListenFds;

use anyhow::{bail, Result};
use log::{debug, info};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;
use tokio::net::UnixListener;

use structopt::StructOpt;

use futures_util::TryFutureExt;
use tonic::transport::server::Connected;
use tonic::{transport::Server, Request, Response, Status};

use enarx_proto::v0;
use v0::keepldr_server::{Keepldr, KeepldrServer};
use v0::{BackendInfo, InfoRequest, KeepldrInfo};

#[cfg(unix)]
use std::os::unix::{io::AsRawFd, io::FromRawFd};

type TonicResult<T> = std::result::Result<Response<T>, Status>;

#[derive(Debug, Default)]
struct KeepldrState {}

#[tonic::async_trait]
impl Keepldr for KeepldrState {
    async fn info(&self, _req: Request<InfoRequest>) -> TonicResult<KeepldrInfo> {
        let keepldrinfo = KeepldrInfo {
            name: "enarx serve".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            sallyport_version: "0.1.0".to_string(), // FIXME
            backend: Some(BackendInfo {
                sgx: None,
                kvm: None,
                sev: None,
            }),
        };
        Ok(Response::new(keepldrinfo))
    }

    async fn boot(&self, request: Request<v0::BootRequest>) -> TonicResult<v0::Result> {
        let boot = request.get_ref();

        let result = v0::Result {
            code: v0::Code::Unknown as i32,
            message: format!("shim: {:?} exec: {:?}", boot.shim, boot.exec),
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

pub struct TonicUnixStream(pub tokio::net::UnixStream);

impl FromRawFd for TonicUnixStream {
    unsafe fn from_raw_fd(fd: std::os::unix::prelude::RawFd) -> Self {
        let std = std::os::unix::net::UnixStream::from_raw_fd(fd);
        Self(tokio::net::UnixStream::from_std(std).unwrap())
    }
}

impl AsRawFd for TonicUnixStream {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}

use std::sync::Arc;
impl Connected for TonicUnixStream {
    type ConnectInfo = (
        Option<Arc<tokio::net::unix::SocketAddr>>,
        Option<tokio::net::unix::UCred>,
    );
    fn connect_info(&self) -> Self::ConnectInfo {
        (
            self.0.peer_addr().ok().map(Arc::new),
            self.0.peer_cred().ok(),
        )
    }
}

impl TonicUnixStream {
    fn local_addr(&self) -> std::io::Result<tokio::net::unix::SocketAddr> {
        self.0.local_addr()
    }

    fn from_std(std: std::os::unix::net::UnixStream) -> std::io::Result<Self> {
        tokio::net::UnixStream::from_std(std).map(Self)
    }
}

use tokio::io::{AsyncRead, AsyncWrite};

impl AsyncRead for TonicUnixStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TonicUnixStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl ServeOptions {
    /// Handle an already-accepted connection on an already-opened socket
    fn serve(&self, sock: UnixStream) -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        rt.block_on(async {
            Server::builder()
                .timeout(Duration::from_millis(self.idle_timeout))
                .add_service(KeepldrServer::new(KeepldrState::default()))
                .serve_with_incoming(
                    async_stream::stream! { yield TonicUnixStream::from_std(sock) },
                )
                .await
        })?;
        Ok(())
    }

    /// Listen for & handle connections on the given socket
    #[tokio::main]
    async fn listen(&self, socket_path: &Path) -> Result<()> {
        // Build an incoming connection Stream that binds to the socket and
        // yields a new TonicUnixStream for each accepted connection.
        let incoming = {
            debug!("binding to socket {:?}", socket_path);
            let sock = UnixListener::bind(socket_path)?;
            async_stream::stream! {
                while let conn = sock.accept().map_ok(|(sock, _addr)| TonicUnixStream(sock)).await {
                    debug!("new connection on {:?}", socket_path);
                    yield conn;
                }
            }
        };

        // Fire up a tonic Server that implements the Keepldr service and
        // asynchronously handles incoming connections
        Server::builder()
            .timeout(Duration::from_millis(self.idle_timeout))
            .add_service(KeepldrServer::new(KeepldrState::default()))
            .serve_with_incoming(incoming)
            .await?;

        // We're done!
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
        debug!(
            "fd {} local_addr {:?}",
            sock.as_raw_fd(),
            sock.local_addr()?
        );
        debug!("INSTANCE_ID: {:?}", std::env::var("INSTANCE_ID"));
        // If provided, check CLI-provided path against actual socket path
        if let Some(ref expect_path) = self.socket_path {
            let addr = sock.local_addr()?;
            let socket_path = addr.as_pathname();
            if socket_path != Some(expect_path) {
                bail!(
                    "socket path {:?} does not match expected path {:?}",
                    socket_path,
                    expect_path
                );
            }
        }
        Ok(sock)
    }
}

impl SubCommand for ServeOptions {
    fn execute(self) -> Result<()> {
        if self.systemd_socket_accept {
            info!("looking for a systemd-passed socket");
            match self.accept_from_systemd() {
                Err(e) => bail!("Failed to get socket from systemd: {}", e),
                Ok(sock) => self.serve(sock),
            }
        } else {
            info!("looking for socket path to listen on");
            match &self.socket_path {
                None => bail!("missing required 'socket_path' arg"),
                Some(path) => self.listen(path),
            }
        }
    }
}
