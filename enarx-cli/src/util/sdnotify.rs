
#[derive(Debug)]
pub struct SdNotify {
    path: PathBuf,
}

impl SdNotify {
    fn get_notify_socket() -> std::result::Result<PathBuf, VarError> {
        Ok(var("NOTIFY_SOCKET")?.into())
    }

    fn from_env() -> std::result::Result<Self, VarError> {
        Ok(Self { path: Self::get_notify_socket()? })
    }

    fn notify(&self, state: &[u8]) -> std::io::Result<usize> {
        UnixDatagram::unbound()?.send_to(state, self.path)
    }

    fn unset_env() {
        std::env::remove_var("NOTIFY_SOCKET")
    }
}
