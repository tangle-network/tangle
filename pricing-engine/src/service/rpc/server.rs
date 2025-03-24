//! RPC server implementation for the Tangle Cloud Pricing Engine
//!
//! This module provides a JSON-RPC server that allows users to query
//! pricing information and obtain signed price quotes from the operator.

use std::{net::SocketAddr, sync::Arc};

use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	server::{RpcModule, ServerBuilder, ServerHandle},
};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{
	calculation::calculate_service_price,
	error::Result,
	models::PricingModel,
	types::{PricingContext, ResourceRequirement, ServiceCategory},
};

/// RPC API trait for the pricing engine
#[jsonrpsee::proc_macros::rpc(server)]
pub trait PricingApi {
	/// Get operator information
	#[method(name = "pricing_getOperatorInfo")]
	fn get_operator_info(&self) -> RpcResult<OperatorInfo>;

	/// Get available pricing models for the operator
	#[method(name = "pricing_getPricingModels")]
	fn get_pricing_models(&self) -> RpcResult<Vec<PricingModelInfo>>;

	/// Calculate price for a service with specified requirements
	#[method(name = "pricing_calculatePrice")]
	fn calculate_price(&self, request: PriceCalculationRequest) -> RpcResult<PriceQuote>;
}

/// Operator information returned by the RPC API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorInfo {
	/// Operator identifier (public key)
	pub id: String,
	/// Operator name
	pub name: String,
	/// Operator description
	pub description: Option<String>,
	/// Supported service categories
	pub supported_categories: Vec<ServiceCategory>,
}

/// Pricing model information returned by the RPC API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModelInfo {
	/// Model identifier
	pub id: String,
	/// Model name
	pub name: String,
	/// Model description
	pub description: Option<String>,
	/// Service category this model applies to
	pub category: ServiceCategory,
	/// Whether this model is currently active
	pub active: bool,
}

/// Request to calculate the price for a service
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PriceCalculationRequest {
	/// Blueprint ID for the service
	pub blueprint_id: String,
	/// Requirements for the service
	pub requirements: ResourceRequirement,
	/// Service category
	pub category: ServiceCategory,
	/// Duration of the service in seconds (optional)
	pub duration: Option<u64>,
}

/// Price quote response from the operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
	/// The calculated price
	pub price: u64,
	/// The currency of the price (e.g., "TNGL")
	pub currency: String,
	/// The pricing model used
	pub model_id: String,
	/// Expiration timestamp for this quote
	pub expires_at: u64,
	/// Operator signature of the quote (can be verified on-chain)
	pub signature: Option<String>,
}

/// RPC server for the pricing engine
pub struct RpcServer {
	/// Operator information
	operator_info: OperatorInfo,
	/// Available pricing models
	pricing_models: Vec<PricingModel>,
}

impl RpcServer {
	/// Create a new RPC server
	pub fn new(operator_info: OperatorInfo, pricing_models: Vec<PricingModel>) -> Self {
		Self { operator_info, pricing_models }
	}

	/// Start the RPC server
	pub async fn start(self, addr: SocketAddr) -> Result<ServerHandle, JsonRpseeError> {
		let server = ServerBuilder::default().build(addr).await?;
		let mut module = RpcModule::new(());

		// Register the RPC methods
		let operator_info = self.operator_info.clone();
		module.register_async_method("pricing_getOperatorInfo", move |_, _, _| {
			let info = operator_info.clone();
			async move { Ok::<_, JsonRpseeError>(info) }
		})?;

		let pricing_models = self.pricing_models.clone();
		module.register_async_method("pricing_getPricingModels", move |_, _, _| {
			let models = pricing_models.clone();
			async move {
				let model_infos = models
					.iter()
					.map(|model| PricingModelInfo {
						id: format!("model_{}", model.name.to_lowercase().replace(" ", "_")),
						name: model.name.clone(),
						description: model.description.clone(),
						category: model.category,
						active: true,
					})
					.collect();

				Ok::<_, JsonRpseeError>(model_infos)
			}
		})?;

		let pricing_models_for_calc = self.pricing_models.clone();
		let operator_id = self.operator_info.id.clone();
		module.register_async_method("pricing_calculatePrice", move |params, _, _| {
			let models = pricing_models_for_calc.clone();
			let provider_id = operator_id.clone();

			async move {
				let request: PriceCalculationRequest = params.parse()?;

				// Find models that match the category
				let matching_models =
					models.iter().filter(|m| m.category == request.category).collect::<Vec<_>>();

				if matching_models.is_empty() {
					return Err(JsonRpseeError::Custom(format!(
						"No pricing models available for category {:?}",
						request.category
					)));
				}

				// Find the best price
				let mut best_price = u64::MAX;
				let mut best_model_id = None;

				// Context for price calculation
				let context = PricingContext { provider_id: provider_id.clone() };

				// Calculate price for each matching model
				for model in matching_models {
					match calculate_service_price(&request.requirements, model, &context) {
						Ok(price) => {
							if price < best_price {
								best_price = price;
								best_model_id = Some(format!(
									"model_{}",
									model.name.to_lowercase().replace(" ", "_")
								));
							}
						},
						Err(e) => {
							debug!("Error calculating price with model {}: {}", model.name, e);
						},
					}
				}

				// Return the price quote
				if let Some(model_id) = best_model_id {
					// Current timestamp plus 10 minutes (example expiration)
					let now = std::time::SystemTime::now()
						.duration_since(std::time::UNIX_EPOCH)
						.unwrap_or_default()
						.as_secs();

					let expires_at = now + 10 * 60; // 10 minutes from now

					// In a real implementation, we would sign the price quote here
					// using the operator's key
					let signature = None; // Placeholder for actual signature

					Ok(PriceQuote {
						price: best_price,
						currency: "TNGL".to_string(),
						model_id,
						expires_at,
						signature,
					})
				} else {
					Err(JsonRpseeError::Custom("Failed to calculate price".to_string()))
				}
			}
		})?;

		info!("Starting RPC server at {}", addr);
		let server_handle = server.start(module)?;

		Ok(server_handle)
	}
}

/// Service request handler to process signed price quotes and handle on-chain submission
pub struct ServiceRequestHandler {}
