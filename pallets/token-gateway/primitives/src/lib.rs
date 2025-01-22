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

//! This library contains types shared with token gateway and other pallets
#![cfg_attr(not(feature = "std"), no_std)]

use ismp::host::StateMachine;
use sp_core::{ConstU32, H160, H256};
use sp_runtime::BoundedVec;

extern crate alloc;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

/// The token registry Id
pub const REGISTRY: [u8; 8] = *b"registry";

/// Token Gateway Id for substrate chains
/// Module Id is the last 20 bytes of the keccak hash of the pallet id
pub fn token_gateway_id() -> H160 {
	let hash = sp_io::hashing::keccak_256(b"tokengty");
	H160::from_slice(&hash[12..32])
}

pub fn token_governor_id() -> Vec<u8> {
	REGISTRY.to_vec()
}

/// Holds metadata relevant to a multi-chain native asset
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, Default)]
pub struct AssetMetadata {
	/// The asset name
	pub name: BoundedVec<u8, ConstU32<50>>,
	/// The asset symbol
	pub symbol: BoundedVec<u8, ConstU32<20>>,
	/// The asset decimals of the ERC6160 or ERC20 counterpart of this asset
	pub decimals: u8,
	/// Asset's minimum balance, only used by substrate chains
	pub minimum_balance: Option<u128>,
}

/// A struct for deregistering asset id on pallet-token-gateway
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, Default)]
pub struct DeregisterAssets {
	pub asset_ids: Vec<H256>,
}

/// Holds data required for multi-chain native asset registration
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq)]
pub struct GatewayAssetRegistration {
	/// The asset name
	pub name: BoundedVec<u8, ConstU32<50>>,
	/// The asset symbol
	pub symbol: BoundedVec<u8, ConstU32<20>>,
	/// The list of chains to create the asset on
	pub chains: Vec<StateMachine>,
	/// Minimum balance for the asset, for substrate chains,
	pub minimum_balance: Option<u128>,
}

/// Allows a user to update their multi-chain native token potentially on multiple chains
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, Default)]
pub struct GatewayAssetUpdate {
	/// The asset identifier
	pub asset_id: H256,
	/// Chains to add support for the asset on
	pub add_chains: BoundedVec<StateMachine, ConstU32<100>>,
	/// Chains to delist the asset from
	pub remove_chains: BoundedVec<StateMachine, ConstU32<100>>,
	/// Chains to change the asset admin on
	pub new_admins: BoundedVec<(StateMachine, H160), ConstU32<100>>,
}

/// Holds data required for multi-chain native asset registration
#[derive(Debug, Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq)]
pub enum RemoteERC6160AssetRegistration {
	CreateAsset(GatewayAssetRegistration),
	UpdateAsset(GatewayAssetUpdate),
}
