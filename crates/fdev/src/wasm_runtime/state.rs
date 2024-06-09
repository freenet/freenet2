use std::{fs::File, io::Write, net::SocketAddr, sync::Arc};

use freenet_stdlib::{client_api::WebApi, prelude::*};
use tokio::sync::RwLock;

use crate::wasm_runtime::DeserializationFmt;

use super::ExecutorConfig;

mod v1;

#[derive(Clone)]
pub(super) struct AppState {
    pub(crate) local_node: Arc<RwLock<WebApi>>,
    config: ExecutorConfig,
}

impl AppState {
    pub async fn new(config: &ExecutorConfig) -> Result<Self, anyhow::Error> {
        AppState::new_v1(config).await
    }

    pub fn printout_deser<R: AsRef<[u8]> + ?Sized>(&self, data: &R) -> Result<(), std::io::Error> {
        fn write_res(config: &ExecutorConfig, pprinted: &str) -> Result<(), std::io::Error> {
            if let Some(p) = &config.output_file {
                let mut f = File::create(p)?;
                f.write_all(pprinted.as_bytes())?;
            } else if config.terminal_output {
                tracing::debug!("{pprinted}");
            }
            Ok(())
        }
        if let Some(DeserializationFmt::Json) = self.config.ser_format {
            let deser: serde_json::Value = serde_json::from_slice(data.as_ref())?;
            let pp = serde_json::to_string_pretty(&deser)?;
            write_res(&self.config, &pp)?;
        }
        Ok(())
    }
}
