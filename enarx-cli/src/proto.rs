// SPDX-License-Identifier: Apache-2.0

#[path = "proto/enarx.v0.rs"]
pub mod v0;

impl std::fmt::Display for v0::boot_request::boot_item::From {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blob(v) => write!(f, "Blob([{} bytes...])", v.len()),
        }
    }
}
