// SPDX-License-Identifier: Apache-2.0

use crate::cmd::SubCommand;
use crate::util::ListenFds;

use std::{convert::TryInto, path::PathBuf};
use structopt::StructOpt;
use std::str::FromStr;
use anyhow::{bail, Context, Result};
use log::info;
use quiche;


#[cfg(unix)]
use std::os::unix::{io::AsRawFd, io::FromRawFd, net::UnixDatagram};

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

#[derive(StructOpt, Debug)]
pub struct ServeOptions {
    /// Idle connection timeout time, in milliseconds (0=forever)
    pub idle_timeout: u64,

    /// Options for setting up TLS certificate chains & keys
    #[structopt(flatten)]
    pub tls: TLSOptions,
}

struct ConnectionIdGenerator {
    //rng: Box<dyn ring::rand::SecureRandom>,
    //seed: ring::hmac::Key,
}

impl ConnectionIdGenerator {
    fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[inline]
fn quiche_path<'a>(p: &'a PathBuf) -> Result<&'a str> {
    match p.to_str() {
        None => bail!("PathBuf {:?} can't be converted to a &str", p),
        Some(s) => Ok(s),
    }
}

const MAX_DATAGRAM_SIZE: usize = 1350;

impl ServeOptions {
    fn quiche_config(self) -> Result<quiche::Config> {
        let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;

        if let Some(cert) = self.tls.cert {
            config.load_cert_chain_from_pem_file(quiche_path(&cert)?)?;
        }

        if let Some(key) = self.tls.key {
            config.load_priv_key_from_pem_file(quiche_path(&key)?)?;
        }

        if let Some(cacert) = self.tls.cacert {
            config.load_verify_locations_from_file(quiche_path(&cacert)?)?;
        }

        if let Some(capath) = self.tls.capath {
            config.load_verify_locations_from_directory(quiche_path(&capath)?)?;
        }

        config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
        
        config.set_max_idle_timeout(self.idle_timeout);

        config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);

        // FUTURE: we could expose these parameters as well...
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_stream_data_bidi_remote(1_000_000);
        config.set_initial_max_stream_data_uni(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_initial_max_streams_uni(100);
        config.set_disable_active_migration(true);
        config.enable_early_data();
    
        Ok(config)
    }

    fn serve(self, sock: UnixDatagram) -> Result<()> {
        let mut buf = [0; 1024*64];
        let mut out = [0; MAX_DATAGRAM_SIZE];
        let config = self.quiche_config();
        let h3conf = quiche::h3::Config::new();
        let cidgen = ConnectionIdGenerator::new();
        Ok(())
    }
}

impl SubCommand for ServeOptions {
    fn execute(self) -> Result<()> {
        let listen_fds = ListenFds::from_env()?;
        ListenFds::unset_env();
        let sock = match listen_fds.get_connection_fd() {
            None => bail!("can't find connection socket fd"),
            // FIXME: should check the socket type...
            // FIXME: UnixDatagram makes no sense - we'd start a new server for
            // every packet! Should just use a regular stream socket for this,
            // then bring up a datagram socket inside the keep and advertise
            // that via ALPN when we get there..
            Some(fd) => unsafe { UnixDatagram::from_raw_fd(fd) },
        };
        // TODO: check the rest of the FDs, if any...
        self.serve(sock)
    }
}