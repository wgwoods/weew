use crate::cmd::SubCommand;
use structopt::StructOpt;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use std::{convert::TryFrom, path::PathBuf, path::Path};
use anyhow::{bail, Result};
use tower::service_fn;

use enarx_proto::v0::{InfoRequest, keepldr_client::KeepldrClient};

// TODO rename to InfoCommandOptions or something..?
#[derive(StructOpt, Debug)]
pub struct InfoOptions {
    //#[structopt()]
    pub socket_path: PathBuf,
}

impl SubCommand for InfoOptions {
    #[tokio::main]
    async fn execute(self) -> Result<()> {
        let uri = Uri::builder()
                    .scheme("unix")
                    .authority("enarx.dev")
                    .path_and_query(self.socket_path.to_str().unwrap_or_default())
                    .build()
                    .unwrap();
        let channel = Endpoint::try_from(uri)?
            .connect_with_connector(
                service_fn(|u: Uri| { UnixStream::connect(u.path().to_string()) })
            ).await?;

        let mut client = KeepldrClient::new(channel);

        let request = tonic::Request::new(InfoRequest {});

        let response = client.info(request).await?;

        println!("RESPONSE: {:?}", response);
        
        Ok(())
    }
}