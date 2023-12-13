//! Webb Relayer Gadget
//!
//! Integrates the Webb Relayer into the Substrate Node.

use ethereum_types::Secret;
use sc_keystore::LocalKeystore;
use sp_application_crypto::{ecdsa as crypto, ByteArray, Pair};
use sp_keystore::Keystore;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use webb_relayer::service;
use webb_relayer_context::RelayerContext;

/// Webb Relayer gadget initialization parameters.
pub struct RelayerParams {
	/// Concrete local key store
	pub local_keystore: Arc<LocalKeystore>,
	/// Configuration directory
	pub config_dir: Option<PathBuf>,
	/// Database path
	pub database_path: Option<PathBuf>,
	/// RPC address, `None` if disabled.
	pub rpc_addr: Option<SocketAddr>,
}

pub async fn start_relayer_gadget(
	relayer_params: RelayerParams,
	key_type: sp_application_crypto::KeyTypeId,
) {
	let mut config = match relayer_params.config_dir.as_ref() {
		Some(p) => load_config(p).expect("failed to load relayer config"),
		None => {
			tracing::warn!(
				target: "relayer-gadget",
				"Error: Not Starting Webb Relayer Gadget. No Config Directory Specified"
			);
			return
		},
	};

	post_process_config(&mut config, &relayer_params, key_type)
		.expect("failed to post process relayer config");

	let store = create_store(relayer_params.database_path).expect("failed to create relayer store");
	let ctx = RelayerContext::new(config, store.clone())
		.await
		.expect("failed to build relayer context");

	// Start the web server:
	service::build_web_services(ctx.clone())
		.await
		.expect("failed to build relayer web services");
	service::ignite(ctx, Arc::new(store))
		.await
		.expect("failed to ignite relayer services");
}

/// Loads the configuration from the given directory.
fn load_config(
	config_dir: &PathBuf,
) -> Result<webb_relayer_config::WebbRelayerConfig, Box<dyn std::error::Error>> {
	if !config_dir.is_dir() {
		return Err("Config path is not a directory".into())
	}

	Ok(webb_relayer_config::utils::load(config_dir)?)
}

/// Creates a database store for the relayer based on the configuration passed in.
pub fn create_store(
	database_path: Option<PathBuf>,
) -> Result<webb_relayer_store::SledStore, Box<dyn std::error::Error>> {
	let db_path = match database_path {
		Some(p) => p.join("relayerdb"),
		None => {
			tracing::debug!("Using temp dir for store");
			return webb_relayer_store::SledStore::temporary().map_err(Into::into)
		},
	};

	webb_relayer_store::SledStore::open(db_path).map_err(Into::into)
}

/// Post process the relayer configuration.
///
/// - if there is no signer for any EVM chain, set the signer to the ecdsa key from the
/// keystore.
/// - Ensures that governance relayer is always enabled.
fn post_process_config(
	config: &mut webb_relayer_config::WebbRelayerConfig,
	params: &RelayerParams,
	key_type: sp_application_crypto::KeyTypeId,
) -> Result<(), Box<dyn std::error::Error>> {
	// Make sure governance relayer is always enabled
	config.features.governance_relay = true;
	let ecdsa_pair =
		get_ecdsa_pair(params.local_keystore.clone(), key_type)?.ok_or("no ecdsa key found")?;
	let ecdsa_secret = ecdsa_pair.to_raw_vec();
	// for each evm chain, if there is no signer, set the signer to the ecdsa key
	for chain in config.evm.values_mut() {
		if chain.private_key.is_none() {
			chain.private_key = Some(Secret::from_slice(&ecdsa_secret).into())
		}
	}
	Ok(())
}

fn get_ecdsa_pair(
	local_keystore: Arc<LocalKeystore>,
	key_type: sp_application_crypto::KeyTypeId,
) -> Result<Option<crypto::AppPair>, Box<dyn std::error::Error>> {
	let ecdsa_public = local_keystore
		.ecdsa_public_keys(key_type)
		.into_iter()
		.find_map(|public_key| crypto::Public::from_slice(&public_key.0).ok())
		.ok_or("failed to get ecdsa public key")?;

	local_keystore
		.key_pair::<crypto::AppPair>(&ecdsa_public.into())
		.map_err(Into::into)
}
