// SPDX-License-Identifier: Apache-2.0

// systemd socket activation helpers

use std::env::{var, VarError};
use std::num::ParseIntError;
use std::os::unix::io::RawFd;

const LISTEN_FDS_START: RawFd = 3;

type Pid = i32;
type FdCount = usize;

type Result<T> = std::result::Result<T, ListenFdError>;


/// The error enum for ListenFd.
#[derive(Debug, PartialEq)]
pub enum ListenFdError {
    NotPresent,
    ParseError,
    CountError,
}

impl std::error::Error for ListenFdError {}

impl std::fmt::Display for ListenFdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListenFdError::NotPresent => write!(f, "not present"),
            ListenFdError::ParseError => write!(f, "parse error"),
            ListenFdError::CountError => write!(f, "fd overflow/mismatch"),
        }
    }

}

impl From<VarError> for ListenFdError {
    fn from(error: VarError) -> Self {
        match error {
            VarError::NotPresent => ListenFdError::NotPresent,
            VarError::NotUnicode(_) => ListenFdError::ParseError,
        }
    }
}

impl From<ParseIntError> for ListenFdError {
    fn from(_error: ParseIntError) -> Self {
        Self::ParseError
    }
}


// Convenience wrapper for getting environment variables where NotPresent
// doesn't constitute an error
fn optvar(key: &str) -> std::result::Result<Option<String>, VarError> {
    match std::env::var(key) {
        Err(VarError::NotPresent) => Ok(None),
        Ok(val) => Ok(Some(val)),
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
pub struct ListenFds {
    pub fds: FdCount,
    pub fdnames: Option<Vec<String>>,
}

impl ListenFds {    
    fn get_listen_pid() -> Result<Pid> {
        Ok(var("LISTEN_PID")?.parse()?)
    }

    fn get_listen_fds() -> Result<FdCount> {
        Ok(var("LISTEN_FDS")?.parse()?)
    }

    fn get_listen_fdnames() -> Result<Option<Vec<String>>> {
        Ok(optvar("LISTEN_FDNAMES")?.map(|v| v.split(':').map(str::to_owned).collect()))
    }

    pub fn from_env() -> Result<Self> {
        let pid = Self::get_listen_pid()?;
        if pid != std::process::id() as i32 {
            return Err(ListenFdError::NotPresent);
        }

        let fds = Self::get_listen_fds()?;
        if fds <= 0 || fds > (RawFd::MAX - LISTEN_FDS_START) as usize {
            return Err(ListenFdError::CountError);
        }

        let fdnames = Self::get_listen_fdnames()?;
        if fdnames.is_some() && fdnames.as_ref().unwrap().len() != fds {
            return Err(ListenFdError::CountError);
        }

        Ok(Self { fds, fdnames })
    }

    pub fn unset_env() {
        std::env::remove_var("LISTEN_PID");
        std::env::remove_var("LISTEN_FDS");
        std::env::remove_var("LISTEN_FDNAMES");
    }

    pub fn count(&self) -> FdCount {
        self.fds
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = RawFd> {
        let start = LISTEN_FDS_START;
        let end = LISTEN_FDS_START.saturating_add(self.fds as i32);
        (start..end).map(RawFd::from)
    }

    pub fn iter_names(&self) -> impl ExactSizeIterator<Item = &str> {
        ListenFdNamesIter { cur: 0, lfd: self }
    }

    pub fn iter_with_names(&self) -> impl ExactSizeIterator<Item = (RawFd, &str)> {
        self.iter().zip(self.iter_names())
    }

    /// Get the first FD labeled "connection", which is how systemd indicates
    /// the activating socket for services with `Accept=yes` in the socket
    /// unit file. See sd_listen_fds(3) for details.
    pub fn get_connection_fd(&self) -> Option<RawFd> {
        if self.fdnames.is_some() {
            for (fd, name) in self.iter_with_names() {
                if name == "connection" {
                    return Some(fd)
                }
            }
        }
        None
    }
}


pub struct ListenFdNamesIter<'a> {
    cur: usize,
    lfd: &'a ListenFds,
}

impl<'a> Iterator for ListenFdNamesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.lfd.fds {
            return None;
        }
        let name = match &self.lfd.fdnames {
            Some(v) => &v[self.cur],
            None => "unknown",
        };
        self.cur += 1;
        Some(name)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.lfd.fds - self.cur;
        (remaining, Some(remaining))
    }

}

impl<'a> ExactSizeIterator for ListenFdNamesIter<'a> {}
 

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env::{remove_var, set_var};

    #[test]
    #[serial]
    fn not_present() {
        ListenFds::unset_env();
        match ListenFds::from_env() {
            Ok(_) => panic!("unexpected Ok()"),
            Err(ListenFdError::NotPresent) => {}
            Err(e) => panic!("unexpected Err({:?})", e),
        }
    }

    #[test]
    #[serial]
    fn no_names() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "4");
        remove_var("LISTEN_FDNAMES");
        let lfd = ListenFds::from_env().unwrap();
        assert_eq!(lfd.count(), 4);
    }

    #[test]
    #[serial]
    fn bad_fds() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "0");
        remove_var("LISTEN_FDNAMES");
        assert_eq!(
            ListenFds::from_env().unwrap_err(),
            ListenFdError::CountError
        );
        set_var("LISTEN_FDS", (RawFd::MAX - 2).to_string());
        assert_eq!(
            ListenFds::from_env().unwrap_err(),
            ListenFdError::CountError
        );
    }

    #[test]
    #[serial]
    fn with_names() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "4");
        set_var("LISTEN_FDNAMES", "one:two:three:four");
        let lfd = ListenFds::from_env().unwrap();
        assert_eq!(lfd.count(), 4);
    }

    #[test]
    #[serial]
    fn iter_names_from_env() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "4");
        set_var("LISTEN_FDNAMES", "one:two:three:four");
        let lfd = ListenFds::from_env().unwrap();
        assert_eq!(
            lfd.iter_names().collect::<Vec<&str>>(),
            vec!["one", "two", "three", "four"]
        )
    }

    #[test]
    #[serial]
    fn iter_names_no_env() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "4");
        remove_var("LISTEN_FDNAMES");
        let lfd = ListenFds::from_env().unwrap();
        assert_eq!(
            lfd.iter_names().collect::<Vec<&str>>(),
            vec!["unknown", "unknown", "unknown", "unknown"]
        );
    }

    #[test]
    #[serial]
    fn iter_with_names() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "4");
        set_var("LISTEN_FDNAMES", "fd3:fd4:fd5:fd6");
        let lfd = ListenFds::from_env().unwrap();
        let mut pair_iter = lfd.iter_with_names();
        assert_eq!(pair_iter.len(), 4);
        assert_eq!(pair_iter.next(), Some((3 as RawFd, "fd3")));
        assert_eq!(pair_iter.next(), Some((4 as RawFd, "fd4")));
        assert_eq!(pair_iter.next(), Some((5 as RawFd, "fd5")));
        assert_eq!(pair_iter.next(), Some((6 as RawFd, "fd6")));
    }

    #[test]
    #[serial]
    fn wrong_number_of_names() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "3");
        set_var("LISTEN_FDNAMES", "one");
        assert_eq!(
            ListenFds::from_env().unwrap_err(),
            ListenFdError::CountError
        );
        set_var("LISTEN_FDNAMES", "one:two:three:four");
        assert_eq!(
            ListenFds::from_env().unwrap_err(),
            ListenFdError::CountError
        );
    }

    #[test]
    #[serial]
    fn incorrect_pid() {
        set_var("LISTEN_PID", (std::process::id() + 1).to_string());
        set_var("LISTEN_FDS", "3");
        remove_var("LISTEN_FDNAMES");
        match ListenFds::from_env() {
            Ok(_) => panic!("unexpected Ok()"),
            Err(ListenFdError::NotPresent) => {}
            Err(e) => panic!("unexpected Err({:?})", e),
        }
    }

    #[test]
    #[serial]
    fn get_connection_fd() {
        set_var("LISTEN_PID", std::process::id().to_string());
        set_var("LISTEN_FDS", "2");
        set_var("LISTEN_FDNAMES", "connection:other");
        assert_eq!(
            ListenFds::from_env().unwrap().get_connection_fd(),
            Some(3)
        );
    }
}
