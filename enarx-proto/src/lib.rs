// SPDX-License-Identifier: Apache-2.0

/* If we specify out_dir("src/") in build.rs then do this */
#[path = "enarx.v0.rs"]
pub mod v0;

/* If we're using OUT_DIR in build.rs, then this works */
//pub mod v0 { tonic::include_proto!("enarx.v0"); }

#[cfg(test)]
mod tests {
    // Check for expected public struct names / behaviors
    #[test]
    fn pub_names() {
        use crate::v0::{BackendInfo, BootRequest, Code, InfoRequest, KeepldrInfo, Result};
        let r = Result {
            code: Code::Ok as i32,
            message: "it worked! here, have a hot dog: 🌭".to_string(),
            details: vec![],
        };
        assert_eq!(r.code(), Code::Ok);
        assert_eq!(Code::from_i32(0), Some(Code::Ok));
        assert_eq!(Code::Ok as i32, 0);
    }
}
