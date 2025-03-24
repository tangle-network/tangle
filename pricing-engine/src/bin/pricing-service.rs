//! Tangle Cloud Pricing Engine Service
//!
//! A standalone service for the Tangle Cloud Pricing Engine.

use std::{net::SocketAddr, process::exit};

use clap::Parser;
use pricing_engine::{
	models::{PricingModel, PricingModelType},
	types::{Price, ServiceCategory, TimePeriod},
	Service, ServiceConfig,
};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Parser)]
#[clap(name = "pricing-service", about = "Tangle Cloud Pricing Engine Service", version)]
struct Cli {
	/// WebSocket URL of the Tangle node to connect to
	#[clap(long, default_value = "ws://127.0.0.1:9944")]
	node_url: String,

	/// JSON-RPC server listen address
	#[clap(long, default_value = "127.0.0.1:9955")]
	rpc_addr: String,

	/// Path to the keystore for signing transactions
	#[clap(long)]
	keystore_path: Option<String>,

	/// Operator name
	#[clap(long, default_value = "Tangle Cloud Operator")]
	operator_name: String,

	/// Operator description
	#[clap(long)]
	operator_description: Option<String>,

	/// Operator public key (on-chain identity)
	#[clap(long)]
	operator_public_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Initialize tracing
	tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

	// Parse command line arguments
	let cli = Cli::parse();

	info!("Starting Tangle Cloud Pricing Engine Service");
	info!("Node URL: {}", cli.node_url);
	info!("RPC address: {}", cli.rpc_addr);
	info!("Operator name: {}", cli.operator_name);

	// Parse the RPC address
	let rpc_addr: SocketAddr = cli.rpc_addr.parse()?;

	// Create some example pricing models
	let models = create_example_pricing_models();

	// Create operator key - in a real implementation, this would be loaded from the keystore
	let operator_public_key = cli.operator_public_key.unwrap_or_else(|| {
		// Use a placeholder key if none provided
		"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()
	});

	// Create the service configuration
	let config = ServiceConfig {
		rpc_addr,
		node_url: cli.node_url,
		keystore_path: cli.keystore_path,
		operator_name: cli.operator_name,
		operator_description: cli.operator_description,
		operator_public_key,
	};

	// Create and start the service
	let mut service = Service::new(models);
	if let Err(e) = service.start(config).await {
		error!("Failed to start service: {}", e);
		exit(1);
	}

	// Run the service until it's stopped
	if let Err(e) = service.run_until_stopped().await {
		error!("Service error: {}", e);
		exit(1);
	}

	info!("Service stopped");
	Ok(())
}

/// Create example pricing models for demonstration
fn create_example_pricing_models() -> Vec<PricingModel> {
	// Define pricing periods and token
	let hour = TimePeriod::Hour;
	let month = TimePeriod::Month;
	let token = "TNGL".to_string();

	vec![
		// Basic compute model
		PricingModel {
			model_type: PricingModelType::Fixed,
			name: "Basic Compute".to_string(),
			description: Some("Low-cost compute resources".to_string()),
			category: ServiceCategory::Compute,
			base_price: Some(Price { value: 10.0, token: token.clone() }),
			resource_pricing: Vec::new(),
			billing_period: Some(hour),
		},
		// Premium compute model
		PricingModel {
			model_type: PricingModelType::Fixed,
			name: "Premium Compute".to_string(),
			description: Some("High-performance compute resources".to_string()),
			category: ServiceCategory::Compute,
			base_price: Some(Price { value: 25.0, token: token.clone() }),
			resource_pricing: Vec::new(),
			billing_period: Some(hour),
		},
		// Basic storage model
		PricingModel {
			model_type: PricingModelType::Fixed,
			name: "Basic Storage".to_string(),
			description: Some("Standard storage solution".to_string()),
			category: ServiceCategory::Storage,
			base_price: Some(Price { value: 5.0, token: token.clone() }),
			resource_pricing: Vec::new(),
			billing_period: Some(month),
		},
		// Premium storage model
		PricingModel {
			model_type: PricingModelType::Fixed,
			name: "Premium Storage".to_string(),
			description: Some("High-speed SSD storage".to_string()),
			category: ServiceCategory::Storage,
			base_price: Some(Price { value: 15.0, token: token.clone() }),
			resource_pricing: Vec::new(),
			billing_period: Some(month),
		},
	]
}
