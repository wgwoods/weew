// SPDX-License-Identifier: Apache-2.0

mod log;
mod noop;
mod run;

use std::{path::PathBuf, str::FromStr};

use url::Url;
use anyhow::{anyhow, bail};
use structopt::{clap::AppSettings, StructOpt};

pub use self::log::LogOptions;

/// `enarx` client subcommands and their options/arguments.
#[derive(StructOpt, Debug)]
pub enum Command {
    Run(run::Options),
    #[structopt(setting(AppSettings::Hidden))]
    Noop(noop::Options),
}

/// The Enarx host to connect to.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum EnarxHost {
    /// Local host, via a unix socket.
    /// URI format: `unix:/path/to/enarx.socket`
    Local { path: PathBuf },

    /// Remote host, via TCP
    /// URI format: `tcp://enarx.host:port`
    // FUTURE: if we had a well-known port number, port could be Option<u16>
    TCP { host: String, port: u16 },

    /// Remote host, via ssh.
    /// URI format: `ssh://[user@]enarx.host[:port]/path/to/enarx.socket`
    SSH {
        host: String,
        path: PathBuf,
        user: Option<String>,
        port: Option<u16>,
    },
}

#[inline]
fn check_unix_path(s: &str) -> anyhow::Result<&str> {
    if !s.starts_with(&['/', '@'][..]) {
        bail!("invalid unix socket path")
    }
    Ok(s)
}

impl FromStr for EnarxHost {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        match url.scheme() {
            "unix" => Ok(EnarxHost::Local {
                path: check_unix_path(url.path())?.into(),
            }),
            "tcp" => Ok(EnarxHost::TCP {
                host: url.host_str().ok_or(anyhow!("missing host"))?.into(),
                port: url.port().ok_or(anyhow!("missing port"))?,
            }),
            "ssh" => Ok(EnarxHost::SSH {
                host: url.host_str().ok_or(anyhow!("missing host"))?.into(),
                port: url.port(),
                user: Some(url.username().to_string()).filter(|u| !u.is_empty()),
                path: url.path().into(),
            }),
            other => Err(anyhow!("unknown URL scheme {:?}", other)),
        }
    }
}

macro_rules! format_some {
    ($opt:expr, $some_fmt:literal) => {
        if let Some(v) = $opt {
            format!($some_fmt, v)
        } else {
            format!("")
        }
    };
}

impl ToString for EnarxHost {
    fn to_string(&self) -> String {
        match self {
            EnarxHost::Local { path } => {
                format!("unix:{}", path.to_string_lossy())
            },
            EnarxHost::TCP { host, port } => {
                format!("tcp://{}:{}", host, port)
            },
            EnarxHost::SSH { host, port, user, path } => {
                format!("ssh://{user}{host}{port}{path}",
                    user = format_some!(user, "{}@"),
                    host = host,
                    port = format_some!(port, ":{}"),
                    path = path.to_string_lossy()
                )
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_url {
        ($str:literal => Local { $path:literal }) => {
            {
                let h = EnarxHost::from_str($str).unwrap();
                assert_eq!(h, EnarxHost::from_str(&h.to_string()).unwrap());
                assert_eq!(h, EnarxHost::Local { path: $path.into() });
            }
        };
        ($str:literal => TCP { $host:literal, $port:literal }) => {
            {
                let h = EnarxHost::from_str($str).unwrap();
                assert_eq!(h, EnarxHost::from_str(&h.to_string()).unwrap());
                assert_eq!(h, EnarxHost::TCP { host: $host.into(), port: $port });
            }
        };
        ($str:literal => SSH { $host:literal, $path:literal }) => {
            assert_url!($str => SSH { $host, $path, None, None })
        };
        ($str:literal => SSH { $host:literal, $path:literal, @$user:literal }) => {
            assert_url!($str => SSH { $host, $path, Some($user.to_string()), None })
        };
        ($str:literal => SSH { $host:literal, $path:literal, :$port:literal }) => {
            assert_url!($str => SSH { $host, $path, None, Some($port) })
        };
        ($str:literal => SSH { $host:literal, $path:literal, @$user:literal, :$port:literal }) => {
            assert_url!($str => SSH { $host, $path, Some($user.to_string()), Some($port) })
        };
        ($str:literal => SSH { $host:literal, $path:literal, $user:expr, $port:expr }) => {
            {
                let h = EnarxHost::from_str($str).unwrap();
                assert_eq!(h, EnarxHost::from_str(&h.to_string()).unwrap());
                assert_eq!(h, EnarxHost::SSH {
                    host: $host.into(),
                    path: $path.into(),
                    port: $port,
                    user: $user,
                });
            }
        };
        ($str:literal => Err) => {
            assert!(EnarxHost::from_str($str).is_err())
        };
    }

    #[test]
    fn parse_host_url_local() {
        assert_url!("unix:/run/enarx/enarx.socket" => Local { "/run/enarx/enarx.socket" });
        assert_url!("unix:@wow, abstract" => Local { "@wow, abstract" });
        // NOTE: URL parsing can give unexpected results without a hostname...
        assert_url!("unix://run/enarx/enarx.socket" => Local { "/enarx/enarx.socket" });
        assert_url!("unix://localhost/run/enarx/enarx.socket" => Local { "/run/enarx/enarx.socket" });
        // These are definitely no good tho
        assert_url!("unix:" => Err);
        assert_url!("unix:.." => Err);
    }

    #[test]
    fn parse_host_url_tcp() {
        assert_url!("tcp://localhost:2903" => TCP { "localhost", 2903 });
        assert_url!("tcp://240.159.140.173:2903" => TCP { "240.159.140.173", 2903 });
        assert_url!("tcp://[f09f:8cad::]:999" => TCP { "[f09f:8cad::]", 999 });
        // Kinda weird that this is valid but OK. FIXME, maybe?
        assert_url!("tcp://localhost:000/" => TCP { "localhost", 0 });
        assert_url!("tcp://:2903" => Err);
        assert_url!("tcp://localhost/" => Err);
        assert_url!("tcp://localhost:/" => Err);
    }

    #[test]
    fn parse_host_url_ssh() {
        assert_url!("ssh://example.com/run/enarx/enarx.socket"
            => SSH { "example.com", "/run/enarx/enarx.socket" }
        );
        // These paths are useless since there's no filename. TODO: Reject?
        assert_url!("ssh://example.com" => SSH { "example.com", "" });
        assert_url!("ssh://localhost:2903" => SSH { "localhost", "", :2903 });
        assert_url!("ssh://localhost:2903/" => SSH { "localhost", "/", :2903 });
        assert_url!("ssh://beefy@example.com/enarx" => SSH { "example.com", "/enarx", @"beefy" });
    }
}
