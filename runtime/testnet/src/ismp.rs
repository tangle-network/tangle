use frame_support::pallet_prelude::Weight;
use frame_support::parameter_types;
use frame_system::EnsureRoot;
use ismp::router::Timeout;
use pallet_ismp::{Config as IsmpConfig, NoOpMmrTree};
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_std::boxed::Box;
use sp_std::vec::Vec;

// Import consensus clients
use ismp::{
	error::Error,
	host::StateMachine,
	module::IsmpModule,
	router::{IsmpRouter, PostRequest, Request, Response},
};
use ismp_grandpa::consensus::GrandpaConsensusClient;
use pallet_ismp::dispatcher::FeeMetadata;
use pallet_ismp::{
	mmr::{Leaf, ProofKeys},
	ModuleId,
};
use tangle_primitives::{AccountId, Balance};

// Import other necessary components
use crate::{Balances, Ismp, Runtime, RuntimeEvent, Timestamp};

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Polkadot(3367));
	// The host state machine of this pallet
	pub const HostStateMachine: StateMachine = StateMachine::Polkadot(1000); // your paraId here
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AdminOrigin = EnsureRoot<AccountId>;
	type HostStateMachine = HostStateMachine;
	type Coprocessor = Coprocessor;
	type TimestampProvider = Timestamp;
	type Balance = Balance;
	type Currency = Balances;
	type Router = Router;
	type ConsensusClients = ();
	type Mmr = NoOpMmrTree<Runtime>;
	type WeightProvider = ();
}

#[derive(Default)]
pub struct ProxyModule;

impl IsmpModule for ProxyModule {
	fn on_accept(&self, request: PostRequest) -> Result<(), Error> {
		if request.dest != HostStateMachine::get() {
			Ismp::dispatch_request(
				Request::Post(request),
				FeeMetadata::<Runtime> { payer: [0u8; 32].into(), fee: Default::default() },
			)?;
			return Ok(());
		}

		let pallet_id =
			ModuleId::from_bytes(&request.to).map_err(|err| Error::Custom(err.to_string()))?;

		match pallet_id {
			// TODO: Fill in here
			_ => Err(Error::Custom("Destination module not found".to_string())),
		}
	}

	fn on_response(&self, response: Response) -> Result<(), Error> {
		if response.dest_chain() != HostStateMachine::get() {
			Ismp::dispatch_response(
				response,
				FeeMetadata::<Runtime> { payer: [0u8; 32].into(), fee: Default::default() },
			)?;
			return Ok(());
		}

		let request = &response.request();
		let from = match &request {
			Request::Post(post) => &post.from,
			Request::Get(get) => &get.from,
		};

		let pallet_id = ModuleId::from_bytes(from).map_err(|err| Error::Custom(err.to_string()))?;

		match pallet_id {
			// TODO: Fill in here.
			_ => Err(Error::Custom("Destination module not found".to_string())),
		}
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), Error> {
		let (from, source) = match &timeout {
			Timeout::Request(Request::Post(post)) => (&post.from, &post.source),
			Timeout::Request(Request::Get(get)) => (&get.from, &get.source),
			Timeout::Response(res) => (&res.post.to, &res.post.dest),
		};

		let pallet_id = ModuleId::from_bytes(from).map_err(|err| Error::Custom(err.to_string()))?;
		match pallet_id {
			// TODO: Fill in here.
			// instead of returning an error, do nothing. The timeout is for a connected chain.
			_ => Ok(()),
		}
	}
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, _bytes: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		Ok(Box::new(ProxyModule::default()))
	}
}
