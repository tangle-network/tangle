// Copyright (C) Polytope Labs Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pallet types

use alloc::{collections::BTreeMap, vec::Vec};
use frame_support::{pallet_prelude::*, traits::fungibles};
use ismp::host::StateMachine;
use primitive_types::H256;
use sp_core::H160;

use crate::Config;

pub type AssetId<T> =
	<<T as Config>::Assets as fungibles::Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

/// Asset teleportation parameters
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq)]
pub struct TeleportParams<AssetId, Balance> {
	/// Asset Id registered on Hyperbridge
	pub asset_id: AssetId,
	/// Destination state machine
	pub destination: StateMachine,
	/// Receiving account on destination
	pub recepient: H256,
	/// Amount to be sent
	pub amount: Balance,
	/// Request timeout
	pub timeout: u64,
	/// Token gateway address
	pub token_gateway: Vec<u8>,
	/// Relayer fee
	pub relayer_fee: Balance,
	/// Optional call data to be executed on the destination chain
	pub call_data: Option<Vec<u8>>,
	/// Redeem native erc20 assets
	pub redeem: bool,
}

/// Local asset Id and its corresponding token gateway asset id
#[derive(Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub struct AssetRegistration<AssetId> {
	/// Local Asset Id should already exist
	pub local_id: AssetId,
	/// MNT Asset registration details
	pub reg: token_gateway_primitives::GatewayAssetRegistration,
	/// Flag that asserts if this asset is custodied and originally minted on this chain
	pub native: bool,
	/// Precision of asset on supported chains
	pub precision: BTreeMap<StateMachine, u8>,
}

/// Update the precision of an asset
#[derive(Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, RuntimeDebug)]
pub struct PrecisionUpdate<AssetId> {
	/// Asset Id
	pub asset_id: AssetId,
	/// New precisions
	pub precisions: BTreeMap<StateMachine, u8>,
}

alloy_sol_macro::sol! {
	#![sol(all_derives)]
	struct Body {
		// Amount of the asset to be sent
		uint256 amount;
		// The asset identifier
		bytes32 asset_id;
		// Flag to redeem the erc20 asset on the destination
		bool redeem;
		// Sender address
		bytes32 from;
		// Recipient address
		bytes32 to;
	}

	struct BodyWithCall {
		// Amount of the asset to be sent
		uint256 amount;
		// The asset identifier
		bytes32 asset_id;
		// Flag to redeem the erc20 asset on the destination
		bool redeem;
		// Sender address
		bytes32 from;
		// Recipient address
		bytes32 to;
		// Calldata to be passed to the asset destination
		bytes data;
	}
}

#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq)]
pub struct SubstrateCalldata {
	/// A scale encoded encoded [MultiSignature](sp_runtime::MultiSignature) of the beneficiary's
	/// account nonce and the encoded runtime call
	pub signature: Vec<u8>,
	/// Encoded Runtime call that should be executed
	pub runtime_call: Vec<u8>,
}

/// Type that encapsulates both types of token gateway request bodies
pub struct RequestBody {
	pub amount: alloy_primitives::U256,
	pub asset_id: alloy_primitives::FixedBytes<32>,
	pub redeem: bool,
	pub from: alloy_primitives::FixedBytes<32>,
	pub to: alloy_primitives::FixedBytes<32>,
	pub data: Option<alloy_primitives::Bytes>,
}

impl From<Body> for RequestBody {
	fn from(value: Body) -> Self {
		RequestBody {
			amount: value.amount,
			asset_id: value.asset_id,
			redeem: value.redeem,
			from: value.from,
			to: value.to,
			data: None,
		}
	}
}

impl From<BodyWithCall> for RequestBody {
	fn from(value: BodyWithCall) -> Self {
		RequestBody {
			amount: value.amount,
			asset_id: value.asset_id,
			redeem: value.redeem,
			from: value.from,
			to: value.to,
			data: Some(value.data),
		}
	}
}

pub trait EvmToSubstrate<T: frame_system::Config> {
	fn convert(addr: H160) -> T::AccountId;
}

impl<T: frame_system::Config> EvmToSubstrate<T> for ()
where
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
{
	fn convert(addr: H160) -> <T as frame_system::Config>::AccountId {
		let mut account = [0u8; 32];
		account[12..].copy_from_slice(&addr.0);
		account.into()
	}
}
