// This file is part of Bifrost.

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

use crate::governance::TechAdminOrCouncil;
use crate::{Balances, Ismp, IsmpParachain, NativeCurrencyId, Runtime, RuntimeEvent, Timestamp};
use crate::{BncDecimals, Currencies};
use crate::{TokenGateway, Treasury};
use bifrost_asset_registry::AssetIdMaps;
use bifrost_primitives::{AccountId, Balance};
use frame_support::parameter_types;
use ismp::{host::StateMachine, module::IsmpModule, router::IsmpRouter};
use sp_core::Get;
use sp_std::boxed::Box;
use sp_std::vec::Vec;

impl pallet_hyperbridge::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

parameter_types! {
	// The hyperbridge parachain on Polkadot
	pub const Coprocessor: Option<StateMachine> = Some(StateMachine::Polkadot(3367));
	 // The host state machine of this pallet, your state machine id goes here
	pub const HostStateMachine: StateMachine = StateMachine::Polkadot(2030); // polkadot
}

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// Modify the consensus client's permissions, for example, TechAdmin
	type AdminOrigin = TechAdminOrCouncil;
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
	// A tuple of types implementing the ConsensusClient interface, which defines all consensus algorithms supported by this protocol deployment
	type ConsensusClients = (ismp_parachain::ParachainConsensusClient<Runtime, IsmpParachain>,);
	type WeightProvider = ();
	type OffchainDB = ();
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// pallet-ismp implements the IsmpHost
	type IsmpHost = Ismp;
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		match id.as_slice() {
			pallet_hyperbridge::PALLET_HYPERBRIDGE_ID => {
				Ok(Box::new(pallet_hyperbridge::Pallet::<Runtime>::default()))
			},
			id if TokenGateway::is_token_gateway(&id) => {
				Ok(Box::new(pallet_token_gateway::Pallet::<Runtime>::default()))
			},
			_ => Err(ismp::Error::ModuleNotFound(id))?,
		}
	}
}

/// Should provide an account that is funded and can be used to pay for asset creation
pub struct AssetAdmin;
impl Get<AccountId> for AssetAdmin {
	fn get() -> AccountId {
		Treasury::account_id()
	}
}

impl pallet_token_gateway::Config for Runtime {
	// configure the runtime event
	type RuntimeEvent = RuntimeEvent;
	// Configured as Pallet Ismp
	type Dispatcher = Ismp;
	// Configured as Pallet Assets
	type Assets = Currencies;
	// Configured as Pallet balances
	type NativeCurrency = Balances;
	// AssetAdmin account
	type AssetAdmin = AssetAdmin;
	// The Native asset Id
	type NativeAssetId = NativeCurrencyId;
	// A type that provides a function for creating unique asset ids
	// A concrete implementation for your specific runtime is required
	type AssetIdFactory = ();
	// The precision of the native asset
	type Decimals = BncDecimals;
	type ControlOrigin = TechAdminOrCouncil;
	type CurrencyIdConvert = AssetIdMaps<Runtime>;
}
