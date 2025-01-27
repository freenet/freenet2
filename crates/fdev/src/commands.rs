use std::{fs::File, io::Read, net::SocketAddr, path::PathBuf};

use freenet::dev_tool::OperationMode;
use freenet_stdlib::{
    client_api::{ClientRequest, ContractRequest, DelegateRequest, WebApi},
    prelude::*,
    versioning::Version,
};
use freenet::server::WebApp;
use xz2::read::XzDecoder;
use tar::Builder;

use crate::config::{BaseConfig, PutConfig, UpdateConfig};

mod v1;

#[derive(Debug, Clone, clap::Subcommand)]
pub(crate) enum PutType {
    /// Publish a new contract to the network
    Contract(PutContract),
    /// Publish a new delegate to the network
    Delegate(PutDelegate),
}

#[derive(clap::Parser, Clone, Debug)]
pub(crate) struct PutContract {
    /// Path to a file listing the related contracts
    #[arg(long)]
    pub(crate) related_contracts: Option<PathBuf>,
    /// Path to the initial state for the contract (typically binary format)
    #[arg(long)]
    pub(crate) state: Option<PathBuf>,
    /// Path to a pre-compressed tar.xz webapp archive containing the webapp files (must include index.html at root)
    #[arg(long)]
    pub(crate) webapp_archive: Option<PathBuf>,
    /// Path to the metadata file to include with the webapp (can be any binary format)
    #[arg(long)]
    pub(crate) webapp_metadata: Option<PathBuf>,
}

#[derive(clap::Parser, Clone, Debug)]
pub(crate) struct PutDelegate {
    /// Base58 encoded nonce for delegate encryption. If empty the default value will be used (only allowed in local mode)
    #[arg(long, env = "DELEGATE_NONCE", default_value_t = String::new())]
    pub(crate) nonce: String,
    /// Base58 encoded cipher for delegate encryption. If empty the default value will be used (only allowed in local mode)
    #[arg(long, env = "DELEGATE_CIPHER", default_value_t = String::new())]
    pub(crate) cipher: String,
}

pub async fn put(config: PutConfig, other: BaseConfig) -> anyhow::Result<()> {
    if config.release {
        anyhow::bail!("Cannot publish contracts in the network yet");
    }
    let params = if let Some(params) = &config.parameters {
        let mut buf = vec![];
        File::open(params)?.read_to_end(&mut buf)?;
        Parameters::from(buf)
    } else {
        Parameters::from(&[] as &[u8])
    };
    match &config.package_type {
        PutType::Contract(contract) => put_contract(&config, contract, other, params).await,
        PutType::Delegate(delegate) => put_delegate(&config, delegate, other, params).await,
    }
}

async fn put_contract(
    config: &PutConfig,
    contract_config: &PutContract,
    other: BaseConfig,
    params: Parameters<'static>,
) -> anyhow::Result<()> {
    // Try to load as raw WASM first
    let contract = if let Ok(code) = ContractCode::load_raw(&config.code) {
        // Add version wrapper
        let version = Version::new(0, 0, 1);
        ContractContainer::from_code_versioned(code, version.into())?
    } else {
        // Fall back to trying as already versioned
        ContractContainer::try_from((config.code.as_path(), params))?
    };
    let state = if let Some(ref webapp_archive) = contract_config.webapp_archive {
        // Read webapp archive
        let mut archive = vec![];
        File::open(webapp_archive)?.read_to_end(&mut archive)?;
        
        // Read optional metadata
        let metadata = if let Some(ref metadata_path) = contract_config.webapp_metadata {
            let mut buf = vec![];
            File::open(metadata_path)?.read_to_end(&mut buf)?;
            buf
        } else {
            vec![]
        };

        // Validate archive has index.html (warning only)
        use tar::Archive;
        use std::io::Cursor;
        let mut found_index = false;
        let mut tar = Archive::new(XzDecoder::new(Cursor::new(&archive)));
        for entry in tar.entries()? {
            if let Ok(entry) = entry {
                if entry.path()?.to_string_lossy() == "index.html" {
                    found_index = true;
                    break;
                }
            }
        }
        if !found_index {
            tracing::warn!("Warning: No index.html found at root of webapp archive");
        }

        // Create WebApp state
        let mut archive_builder = Builder::new(Cursor::new(Vec::new()));
        archive_builder.append_data(&mut tar::Header::new_gnu(), "archive.tar.xz", &*archive)?;
        let webapp = WebApp::nfrom_data(metadata, archive_builder)?;
        webapp.pack()?.into()
    } else if let Some(ref state_path) = contract_config.state {
        let mut buf = vec![];
        File::open(state_path)?.read_to_end(&mut buf)?;
        buf.into()
    } else {
        tracing::warn!("no state provided for contract, if your contract cannot handle empty state correctly, this will always cause an error.");
        freenet_stdlib::prelude::State::from(vec![])
    };
    let related_contracts = if let Some(_related) = &contract_config.related_contracts {
        todo!("use `related` contracts")
    } else {
        Default::default()
    };

    println!("Putting contract {}", contract.key());
    let request = ContractRequest::Put {
        contract,
        state: state.to_vec().into(),
        related_contracts,
    }
    .into();
    let mut client = start_api_client(other).await?;
    execute_command(request, &mut client).await
}

async fn put_delegate(
    config: &PutConfig,
    delegate_config: &PutDelegate,
    other: BaseConfig,
    params: Parameters<'static>,
) -> anyhow::Result<()> {
    let delegate = DelegateContainer::try_from((config.code.as_path(), params))?;

    let (cipher, nonce) = if delegate_config.cipher.is_empty() && delegate_config.nonce.is_empty() {
        println!(
"Using default cipher and nonce. 
For additional hardening is recommended to use a different cipher and nonce to encrypt secrets in storage.");
        (
            ::freenet_stdlib::client_api::DelegateRequest::DEFAULT_CIPHER,
            ::freenet_stdlib::client_api::DelegateRequest::DEFAULT_NONCE,
        )
    } else {
        let mut cipher = [0; 32];
        bs58::decode(delegate_config.cipher.as_bytes())
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .onto(&mut cipher)?;

        let mut nonce = [0; 24];
        bs58::decode(delegate_config.nonce.as_bytes())
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .onto(&mut nonce)?;
        (cipher, nonce)
    };

    println!("Putting delegate {} ", delegate.key().encode());
    let request = DelegateRequest::RegisterDelegate {
        delegate,
        cipher,
        nonce,
    }
    .into();
    let mut client = start_api_client(other).await?;
    execute_command(request, &mut client).await
}

pub async fn update(config: UpdateConfig, other: BaseConfig) -> anyhow::Result<()> {
    if config.release {
        anyhow::bail!("Cannot publish contracts in the network yet");
    }
    let key = ContractInstanceId::try_from(config.key)?.into();
    println!("Updating contract {key}");
    let data = {
        let mut buf = vec![];
        File::open(&config.delta)?.read_to_end(&mut buf)?;
        StateDelta::from(buf).into()
    };
    let request = ContractRequest::Update { key, data }.into();
    let mut client = start_api_client(other).await?;
    execute_command(request, &mut client).await
}

pub(crate) async fn start_api_client(cfg: BaseConfig) -> anyhow::Result<WebApi> {
    v1::start_api_client(cfg).await
}

pub(crate) async fn execute_command(
    request: ClientRequest<'static>,
    api_client: &mut WebApi,
) -> anyhow::Result<()> {
    v1::execute_command(request, api_client).await
}
