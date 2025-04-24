use std::path::Path;
use std::sync::Arc;

use blueprint_keystore::{Keystore, KeystoreConfig};
use blueprint_manager::config::BlueprintManagerConfig;
use blueprint_manager::config::DEFAULT_DOCKER_HOST;
use blueprint_manager::config::SourceType;
use blueprint_manager::executor::{BlueprintManagerHandle, run_blueprint_manager_with_keystore};
use blueprint_runner::config::BlueprintEnvironment;
use sc_keystore::LocalKeystore;
use sc_service::error::Error as ServiceError;

/// Runs the blueprint manager service.
pub async fn create_blueprint_manager_service<P: AsRef<Path>>(
	rpc_port: u16,
	data_dir: P,
	local_keystore: Arc<LocalKeystore>,
	test_mode: bool,
) -> Result<BlueprintManagerHandle, ServiceError> {
	let data_dir = data_dir.as_ref().to_path_buf();
	let base_dir = data_dir.parent().ok_or_else(|| {
		ServiceError::Application("Failed to get parent directory for keystore".into())
	})?;
	let config = BlueprintManagerConfig {
		gadget_config: None,
		keystore_uri: base_dir.join("keystore").to_path_buf().to_string_lossy().into(),
		data_dir,
		verbose: 2,
		pretty: false,
		instance_id: None,
		test_mode,
		podman_host: DEFAULT_DOCKER_HOST.clone(),
		preferred_source: SourceType::default(),
	};
	let mut env = BlueprintEnvironment::default();

	env.http_rpc_endpoint = format!("http://127.0.0.1:{}", rpc_port);
	env.ws_rpc_endpoint = format!("ws://127.0.0.1:{}", rpc_port);
	env.keystore_uri = config.keystore_uri.clone();
	env.data_dir = Some(config.data_dir.clone());
	env.protocol_settings = blueprint_runner::config::ProtocolSettings::None;
	env.test_mode = config.test_mode;

	let keystore = Keystore::new(KeystoreConfig::new().substrate(local_keystore))
		.map_err(|e| ServiceError::Application(e.into()))?;

	let shutdown_cmd = futures::future::pending();
	let mut handle =
		match run_blueprint_manager_with_keystore(config, keystore, env, shutdown_cmd).await {
			Ok(handle) => handle,
			Err(e) => {
				log::error!("Failed to start blueprint manager: {}", e);
				return Err(ServiceError::Application(e.into()));
			},
		};
	handle.start().map_err(|e| ServiceError::Application(e.into()))?;
	log::info!("Blueprint manager started successfully.");
	Ok(handle)
}
