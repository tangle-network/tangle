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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

use alloy_primitives::hex;
use ismp::host::StateMachine;
use sp_core::{ConstU32, H160, H256};
use sp_runtime::BoundedVec;

extern crate alloc;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

/// Pallet Token Governor's module ID for ISMP requests
pub const TOKEN_GOVERNOR_ID: [u8; 8] = *b"registry";

/// Pallet Token Gateway's module ID for ISMP requests
///
/// Derived from `keccak_256(b"tokengty")[12..]`
pub const PALLET_TOKEN_GATEWAY_ID: [u8; 20] = hex!("a09b1c60e8650245f92518c8a17314878c4043ed");

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
	/// List of asset ids to deregister
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
	/// Asset creation message
	CreateAsset(GatewayAssetRegistration),
	/// Asset modification message
	UpdateAsset(GatewayAssetUpdate),
}
