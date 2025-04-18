// This file is part of Tangle.

// Copyright (C) Tangle Foundation
// Copyright (C) Liebi Technologies PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{
	AccountId, Assets, Balances, EnsureRoot, EnsureRootOrHalfCouncil, Get, H160, Ismp, Runtime,
	RuntimeEvent, Timestamp, TokenGateway, Treasury,
};
use frame_support::parameter_types;
use ismp::{host::StateMachine, module::IsmpModule, router::IsmpRouter};
use pallet_token_gateway::types::EvmToSubstrate;
use sp_std::{boxed::Box, vec::Vec};
use tangle_primitives::Balance;

impl pallet_hyperbridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Kusama(4009));
	 // The host state machine of this pallet, your state machine id goes here
	pub const HostStateMachine: StateMachine = StateMachine::Substrate(*b"TNGL");
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Modify the consensus client's permissions, for example, TechAdmin
	type AdminOrigin = EnsureRootOrHalfCouncil;
	// The state machine identifier of the chain -- parachain id
	type HostStateMachine = HostStateMachine;
	type TimestampProvider = Timestamp;
	// The router provides the implementation for the IsmpModule as the module id.
	type Router = Router;
	type Balance = Balance;
	// The token used to collect fees, only stablecoins are supported
	type Currency = Balances;
	// Co-processor
	type Coprocessor = Coprocessor;
	// A tuple of types implementing the ConsensusClient interface, which defines all consensus
	// algorithms supported by this protocol deployment
	type ConsensusClients = (::ismp_grandpa::consensus::GrandpaConsensusClient<Runtime>,);
	type WeightProvider = ();
	type OffchainDB = ();
}

impl ::ismp_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IsmpHost = pallet_ismp::Pallet<Runtime>;
	type WeightInfo = crate::weights::ismp_grandpa::WeightInfo<Runtime>;
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		match id.as_slice() {
			pallet_hyperbridge::PALLET_HYPERBRIDGE_ID => {
				Ok(Box::new(pallet_hyperbridge::Pallet::<Runtime>::default()))
			},
			id if TokenGateway::is_token_gateway(id) => Ok(Box::new(TokenGateway::default())),
			_ => Err(ismp::Error::ModuleNotFound(id))?,
		}
	}
}

pub struct EvmToSubstrateFactory;

impl EvmToSubstrate<Runtime> for EvmToSubstrateFactory {
	fn convert(addr: H160) -> AccountId {
		let mut account = [0u8; 32];
		account[12..].copy_from_slice(&addr.0);
		account.into()
	}
}

/// Should provide an account that is funded and can be used to pay for asset creation
pub struct AssetAdmin;
impl Get<AccountId> for AssetAdmin {
	fn get() -> AccountId {
		Treasury::account_id()
	}
}

parameter_types! {
	// A constant that should represent the native asset id, this id must be unique to the native currency
	pub const NativeAssetId: u32 = 0;
	// Set the correct decimals for the native currency
	pub const Decimals: u8 = 18;
}

impl ::pallet_token_gateway::Config for Runtime {
	// configure the runtime event
	type RuntimeEvent = RuntimeEvent;
	// Configured as Pallet Ismp
	type Dispatcher = pallet_hyperbridge::Pallet<Runtime>;
	// Configured as Pallet Assets
	type Assets = Assets;
	// Configured as Pallet balances
	type NativeCurrency = Balances;
	// AssetAdmin account
	type AssetAdmin = AssetAdmin;
	// The Native asset Id
	type NativeAssetId = NativeAssetId;
	// The precision of the native asset
	type Decimals = Decimals;
	type EvmToSubstrate = EvmToSubstrateFactory;
	type WeightInfo = crate::weights::pallet_token_gateway::SubstrateWeight<Runtime>;
	type CreateOrigin = EnsureRoot<AccountId>;
}
