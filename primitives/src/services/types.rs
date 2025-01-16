// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use crate::{Account, Weight};
use educe::Educe;
use fp_evm::CallInfo;
use frame_support::pallet_prelude::*;
use serde::Deserializer;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ByteArray, RuntimeDebug, H160, U256};
use sp_runtime::Percent;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use super::field::FieldType;

/// An error that can occur during type checking.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TypeCheckError {
	/// The argument type does not match the expected type.
	ArgumentTypeMismatch {
		/// The index of the argument.
		index: u8,
		/// The expected type.
		expected: FieldType,
		/// The actual type.
		actual: FieldType,
	},
	/// Not enough arguments were supplied.
	NotEnoughArguments {
		/// The number of arguments that were expected.
		expected: u8,
		/// The number of arguments that were supplied.
		actual: u8,
	},
	/// The result type does not match the expected type.
	ResultTypeMismatch {
		/// The index of the argument.
		index: u8,
		/// The expected type.
		expected: FieldType,
		/// The actual type.
		actual: FieldType,
	},
}

impl frame_support::traits::PalletError for TypeCheckError {
	const MAX_ENCODED_SIZE: usize = 2;
}

/// Different types of assets that can be used.
#[derive(
	PartialEq,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Copy,
	Clone,
	MaxEncodedLen,
	Ord,
	PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Asset<AssetId> {
	/// Use the specified AssetId.
	#[codec(index = 0)]
	Custom(AssetId),

	/// Use an ERC20-like token with the specified contract address.
	#[codec(index = 1)]
	Erc20(H160),
}

impl<AssetId: sp_runtime::traits::Zero> Default for Asset<AssetId> {
	fn default() -> Self {
		Asset::Custom(sp_runtime::traits::Zero::zero())
	}
}

impl<AssetId: Encode + Decode> Asset<AssetId> {
	pub fn to_ethabi_param_type() -> ethabi::ParamType {
		ethabi::ParamType::Tuple(vec![
			// Kind of the Asset
			ethabi::ParamType::Uint(8),
			// Data of the Asset (Contract Address or AssetId)
			ethabi::ParamType::FixedBytes(32),
		])
	}

	pub fn to_ethabi_param() -> ethabi::Param {
		ethabi::Param {
			name: String::from("asset"),
			kind: Self::to_ethabi_param_type(),
			internal_type: Some(String::from("struct ServiceOperators.Asset")),
		}
	}

	pub fn to_ethabi(&self) -> ethabi::Token {
		match self {
			Asset::Custom(asset_id) => {
				let asset_id = asset_id.using_encoded(ethabi::Uint::from_little_endian);
				let mut asset_id_bytes = [0u8; core::mem::size_of::<ethabi::Uint>()];
				asset_id.to_big_endian(&mut asset_id_bytes);
				ethabi::Token::Tuple(vec![
					ethabi::Token::Uint(0.into()),
					ethabi::Token::FixedBytes(asset_id_bytes.into()),
				])
			},
			Asset::Erc20(addr) => ethabi::Token::Tuple(vec![
				ethabi::Token::Uint(1.into()),
				ethabi::Token::FixedBytes(addr.to_fixed_bytes().into()),
			]),
		}
	}
}

/// Represents the pricing structure for various hardware resources.
/// All prices are specified in USD/hr, calculated based on the average block time.
#[derive(
	PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceTargets {
	/// Price per vCPU per hour
	pub cpu: u64,
	/// Price per MB of memory per hour
	pub mem: u64,
	/// Price per GB of HDD storage per hour
	pub storage_hdd: u64,
	/// Price per GB of SSD storage per hour
	pub storage_ssd: u64,
	/// Price per GB of NVMe storage per hour
	pub storage_nvme: u64,
}

impl PriceTargets {
	/// Converts the struct to ethabi ParamType.
	pub fn to_ethabi_param_type() -> ethabi::ParamType {
		ethabi::ParamType::Tuple(vec![
			// Price per vCPU per hour
			ethabi::ParamType::Uint(64),
			// Price per MB of memory per hour
			ethabi::ParamType::Uint(64),
			// Price per GB of HDD storage per hour
			ethabi::ParamType::Uint(64),
			// Price per GB of SSD storage per hour
			ethabi::ParamType::Uint(64),
			// Price per GB of NVMe storage per hour
			ethabi::ParamType::Uint(64),
		])
	}

	/// Converts the struct to ethabi Param.
	pub fn to_ethabi_param() -> ethabi::Param {
		ethabi::Param {
			name: String::from("priceTargets"),
			kind: Self::to_ethabi_param_type(),
			internal_type: Some(String::from("struct ServiceOperators.PriceTargets")),
		}
	}

	/// Converts the struct to ethabi Token.
	pub fn to_ethabi(&self) -> ethabi::Token {
		ethabi::Token::Tuple(vec![
			ethabi::Token::Uint(self.cpu.into()),
			ethabi::Token::Uint(self.mem.into()),
			ethabi::Token::Uint(self.storage_hdd.into()),
			ethabi::Token::Uint(self.storage_ssd.into()),
			ethabi::Token::Uint(self.storage_nvme.into()),
		])
	}
}

/// Trait for asset identifiers
pub trait AssetIdT:
	Default
	+ Clone
	+ PartialEq
	+ Eq
	+ PartialOrd
	+ Ord
	+ parity_scale_codec::Codec
	+ MaxEncodedLen
	+ TypeInfo
	+ core::fmt::Debug
{
}

#[cfg(feature = "std")]
impl<T> AssetIdT for T where
	T: Default
		+ Clone
		+ PartialEq
		+ Eq
		+ PartialOrd
		+ Ord
		+ parity_scale_codec::Codec
		+ MaxEncodedLen
		+ TypeInfo
		+ core::fmt::Debug
		+ serde::Serialize
		+ for<'de> serde::Deserialize<'de>
{
}

#[cfg(not(feature = "std"))]
impl<T> AssetIdT for T where
	T: Default
		+ Clone
		+ PartialEq
		+ Eq
		+ PartialOrd
		+ Ord
		+ parity_scale_codec::Codec
		+ MaxEncodedLen
		+ TypeInfo
		+ core::fmt::Debug
{
}

/// The approval state of an operator for a service request
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Clone(bound()), PartialEq(bound()), Eq, PartialOrd, Ord(bound()))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(serde::Serialize, serde::Deserialize),
	serde(bound = ""),
	educe(Debug(bound()))
)]
pub enum ApprovalState<AssetId: AssetIdT> {
	/// The operator has not yet responded to the request
	Pending,
	/// The operator has approved the request with specific asset commitments
	Approved {
		/// The percentage of native currency stake to expose
		native_exposure_percent: Percent,
		/// Asset-specific exposure commitments
		asset_exposure: Vec<AssetSecurityCommitment<AssetId>>,
	},
	/// The operator has rejected the request
	Rejected,
}

/// Asset-specific security requirements for a service request
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(serde::Serialize, serde::Deserialize),
	serde(bound = ""),
	educe(Debug(bound()))
)]
pub struct AssetSecurityRequirement<AssetId: AssetIdT> {
	/// The asset that needs to be secured
	pub asset: Asset<AssetId>,
	/// The minimum percentage of the asset that needs to be exposed for slashing
	pub min_exposure_percent: Percent,
	/// The maximum percentage of the asset that can be exposed for slashing
	pub max_exposure_percent: Percent,
}

/// Asset-specific security commitment from an operator
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Clone(bound()), PartialEq(bound()), Eq, PartialOrd, Ord(bound()))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(serde::Serialize, serde::Deserialize),
	serde(bound = ""),
	educe(Debug(bound()))
)]
pub struct AssetSecurityCommitment<AssetId: AssetIdT> {
	/// The asset being secured
	pub asset: Asset<AssetId>,
	/// The percentage of the asset exposed for slashing
	pub exposure_percent: Percent,
}
