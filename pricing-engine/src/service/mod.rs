//! Service module for the Tangle Cloud Pricing Engine
//!
//! This module provides the main service orchestration for the pricing engine,
//! including blockchain integration and RPC services for a single operator.

pub mod blockchain;
pub mod rpc;

use std::{net::SocketAddr, sync::Arc};

use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use crate::{
	error::{Error, Result},
	models::PricingModel,
	types::ServiceCategory,
};
use blockchain::{event::BlockchainEvent, listener::EventListener};
use rpc::server::{OperatorInfo, RpcServer};

/// Service state enum for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
	/// Service is initializing
	Initializing,
	/// Service is running
	Running,
	/// Service is shutting down
	ShuttingDown,
	/// Service has shut down
	ShutDown,
}

/// Command enum for the service control channel
enum ServiceCommand {
	/// Stop the service
	Stop(oneshot::Sender<()>),
}

/// Configuration for the pricing engine service
pub struct ServiceConfig {
	/// RPC server address
	pub rpc_addr: SocketAddr,
	/// Substrate node websocket URL
	pub node_url: String,
	/// Path to the keystore for signing transactions
	pub keystore_path: Option<String>,
	/// Operator name
	pub operator_name: String,
	/// Operator description
	pub operator_description: Option<String>,
	/// Operator public key (on-chain identity)
	pub operator_public_key: String,
}

/// The main pricing engine service
pub struct Service {
	/// Current state of the service
	state: ServiceState,

	/// The operator information
	operator_info: OperatorInfo,

	/// Available pricing models
	pricing_models: Vec<PricingModel>,

	/// The blockchain event listener
	event_listener: Option<Arc<EventListener>>,

	/// Command channel for service control
	command_tx: mpsc::Sender<ServiceCommand>,
	command_rx: Option<mpsc::Receiver<ServiceCommand>>,

	/// Channel for blockchain events
	event_tx: mpsc::Sender<BlockchainEvent>,
	event_rx: Option<mpsc::Receiver<BlockchainEvent>>,
}

impl Service {
	/// Create a new pricing engine service
	pub fn new(initial_models: Vec<PricingModel>) -> Self {
		let (command_tx, command_rx) = mpsc::channel(32);
		let (event_tx, event_rx) = mpsc::channel(128);

		// Create a default operator info that will be updated during start
		let operator_info = OperatorInfo {
			id: "".to_string(),
			name: "".to_string(),
			description: None,
			supported_categories: vec![],
		};

		Self {
			state: ServiceState::Initializing,
			operator_info,
			pricing_models: initial_models,
			event_listener: None,
			command_tx,
			command_rx: Some(command_rx),
			event_tx,
			event_rx: Some(event_rx),
		}
	}

	/// Start the pricing engine service
	pub async fn start(&mut self, config: ServiceConfig) -> Result<()> {
		info!("Starting Tangle Cloud Pricing Engine");

		// Set up operator info
		let supported_categories = self
			.pricing_models
			.iter()
			.map(|m| m.category)
			.collect::<std::collections::HashSet<_>>()
			.into_iter()
			.collect();

		self.operator_info = OperatorInfo {
			id: config.operator_public_key.clone(),
			name: config.operator_name.clone(),
			description: config.operator_description.clone(),
			supported_categories,
		};

		// Start the blockchain event listener
		info!("Starting blockchain event listener");
		let event_listener = EventListener::new(config.node_url.clone(), self.event_tx.clone())
			.await
			.map_err(|e| Error::ChainConnection(e.to_string()))?;

		// Start listening for blockchain events
		let event_listener = Arc::new(event_listener);
		let listener_clone = event_listener.clone();
		tokio::spawn(async move {
			if let Err(e) = listener_clone.run().await {
				error!("Blockchain event listener error: {}", e);
			}
		});

		self.event_listener = Some(event_listener);

		// Start the RPC server
		info!("Starting RPC server at {}", config.rpc_addr);
		let rpc_server = RpcServer::new(self.operator_info.clone(), self.pricing_models.clone());

		tokio::spawn(async move {
			match rpc_server.start(config.rpc_addr).await {
				Ok(handle) => {
					info!("RPC server started");
					let _ = handle.stopped().await;
					info!("RPC server stopped");
				},
				Err(e) => {
					error!("Failed to start RPC server: {}", e);
				},
			}
		});

		// Start the event handler
		let event_rx = self.event_rx.take().unwrap();
		tokio::spawn(async move {
			Self::handle_events(event_rx).await;
		});

		// Mark the service as running
		self.state = ServiceState::Running;
		info!("Tangle Cloud Pricing Engine started");

		Ok(())
	}

	/// Run the service until it is stopped
	pub async fn run_until_stopped(&mut self) -> Result<()> {
		// Wait for a stop command
		if let Some(mut command_rx) = self.command_rx.take() {
			while let Some(command) = command_rx.recv().await {
				match command {
					ServiceCommand::Stop(sender) => {
						info!("Stopping service");
						self.state = ServiceState::ShuttingDown;

						// Clean up resources

						self.state = ServiceState::ShutDown;
						let _ = sender.send(());
						break;
					},
				}
			}
		}

		Ok(())
	}

	/// Stop the service
	pub async fn stop(&self) -> Result<()> {
		let (tx, rx) = oneshot::channel();
		self.command_tx
			.send(ServiceCommand::Stop(tx))
			.await
			.map_err(|_| Error::ServiceShutdown("Failed to send stop command".to_string()))?;

		rx.await.map_err(|_| {
			Error::ServiceShutdown("Failed to receive stop confirmation".to_string())
		})?;

		info!("Service stopped");
		Ok(())
	}

	/// Handle blockchain events
	async fn handle_events(mut event_rx: mpsc::Receiver<BlockchainEvent>) {
		while let Some(event) = event_rx.recv().await {
			debug!("Received blockchain event: {:?}", event);

			// Process events based on their type
			match &event {
				BlockchainEvent::PricingUpdated { provider_id, models } => {
					info!("Pricing models updated for provider {}", provider_id);
					// Update pricing models in the storage if this is for our operator
				},
				BlockchainEvent::ServiceRequested {
					request_id,
					user_id,
					category,
					requirements,
					max_price,
				} => {
					info!("Service request received: {} from user {}", request_id, user_id);
					// Process the service request:
					// 1. Check if we support this service category
					// 2. Calculate price based on requirements
					// 3. If it's below max_price (if specified), approve the request
					// 4. Otherwise, reject or ignore
				},
				BlockchainEvent::ServiceStarted { service_id, user_id, provider_id, .. } => {
					info!(
						"Service started: {} for user {} by provider {}",
						service_id, user_id, provider_id
					);
					// Track active services
				},
				BlockchainEvent::ServiceTerminated { service_id, reason, final_cost } => {
					info!(
						"Service terminated: {} with reason: {}, final cost: {}",
						service_id, reason, final_cost
					);
					// Clean up any tracking data for this service
				},
				// Handle other events as needed
				_ => {},
			}
		}
	}

	/// Get the current service state
	pub fn state(&self) -> ServiceState {
		self.state
	}

	/// Add or update a pricing model
	pub fn update_pricing_model(&mut self, model: PricingModel) {
		// Check if we already have a model for this category
		let existing_index = self.pricing_models.iter().position(|m| m.name == model.name);

		if let Some(index) = existing_index {
			// Update existing model
			self.pricing_models[index] = model;
		} else {
			// Add new model
			self.pricing_models.push(model);
		}

		// Update supported categories in operator info
		let supported_categories = self
			.pricing_models
			.iter()
			.map(|m| m.category)
			.collect::<std::collections::HashSet<_>>()
			.into_iter()
			.collect();

		self.operator_info.supported_categories = supported_categories;
	}
}
