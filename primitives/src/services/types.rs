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

use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize};
use sp_core::{RuntimeDebug, H160};
use sp_runtime::{traits::AtLeast32BitUnsigned, Percent};
use sp_staking::EraIndex;
use sp_std::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use super::{field::FieldType, Constraints};

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

impl<AssetId: AssetIdT> Default for Asset<AssetId> {
	fn default() -> Self {
		Asset::Custom(AssetId::default())
	}
}

impl<AssetId: AssetIdT> Asset<AssetId> {
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
	+ Parameter
	+ Member
	+ PartialEq
	+ Eq
	+ PartialOrd
	+ Ord
	+ AtLeast32BitUnsigned
	+ parity_scale_codec::FullCodec
	+ MaxEncodedLen
	+ TypeInfo
	+ core::fmt::Debug
	+ MaybeSerializeDeserialize
	+ Display
{
}

impl<T> AssetIdT for T where
	T: Default
		+ Clone
		+ Parameter
		+ Member
		+ PartialEq
		+ Eq
		+ PartialOrd
		+ Ord
		+ AtLeast32BitUnsigned
		+ parity_scale_codec::FullCodec
		+ MaxEncodedLen
		+ TypeInfo
		+ core::fmt::Debug
		+ MaybeSerializeDeserialize
		+ Display
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

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen)]
pub struct OperatorPreferences {
	/// The operator ECDSA public key.
	pub key: [u8; 65],
	/// The pricing targets for the operator's resources.
	pub price_targets: PriceTargets,
}

#[cfg(feature = "std")]
impl Serialize for OperatorPreferences {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeTuple;
		let mut tup = serializer.serialize_tuple(2)?;
		tup.serialize_element(&self.key[..])?;
		tup.serialize_element(&self.price_targets)?;
		tup.end()
	}
}

#[cfg(feature = "std")]
struct OperatorPreferencesVisitor;

#[cfg(feature = "std")]
impl<'de> serde::de::Visitor<'de> for OperatorPreferencesVisitor {
	type Value = OperatorPreferences;

	fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
		formatter.write_str("a tuple of 2 elements")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::SeqAccess<'de>,
	{
		let key = seq
			.next_element::<Vec<u8>>()?
			.ok_or_else(|| serde::de::Error::custom("key is missing"))?;
		let price_targets = seq
			.next_element::<PriceTargets>()?
			.ok_or_else(|| serde::de::Error::custom("price_targets is missing"))?;
		let key_arr: [u8; 65] = key.try_into().map_err(|_| {
			serde::de::Error::custom(
				"key must be in the uncompressed format with length of 65 bytes",
			)
		})?;
		Ok(OperatorPreferences { key: key_arr, price_targets })
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for OperatorPreferences {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_tuple(2, OperatorPreferencesVisitor)
	}
}

impl OperatorPreferences {
	/// Returns the ethabi ParamType for OperatorPreferences.
	pub fn to_ethabi_param_type() -> ethabi::ParamType {
		ethabi::ParamType::Tuple(vec![
			// Operator's ECDSA Public Key (33 bytes)
			ethabi::ParamType::Bytes,
			// Operator's price targets
			PriceTargets::to_ethabi_param_type(),
		])
	}
	/// Returns the ethabi Param for OperatorPreferences.
	pub fn to_ethabi_param() -> ethabi::Param {
		ethabi::Param {
			name: String::from("operatorPreferences"),
			kind: Self::to_ethabi_param_type(),
			internal_type: Some(String::from(
				"struct IBlueprintServiceManager.OperatorPreferences",
			)),
		}
	}

	/// Encode the fields to ethabi bytes.
	pub fn to_ethabi(&self) -> ethabi::Token {
		ethabi::Token::Tuple(vec![
			// operator public key
			ethabi::Token::Bytes(self.key.to_vec()),
			// price targets
			self.price_targets.to_ethabi(),
		])
	}
}

/// Operator Profile is a profile of an operator that
/// contains metadata about the services that the operator is providing.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct OperatorProfile<C: Constraints> {
	/// The Service IDs that I'm currently providing.
	pub services: BoundedBTreeSet<u64, C::MaxServicesPerOperator>,
	/// The Blueprint IDs that I'm currently registered for.
	pub blueprints: BoundedBTreeSet<u64, C::MaxBlueprintsPerOperator>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub enum MembershipModel {
	/// Fixed set of operators defined at service creation
	Fixed { min_operators: u32 },
	/// Operators can join/leave subject to blueprint rules
	Dynamic { min_operators: u32, max_operators: Option<u32> },
}

impl Default for MembershipModel {
	fn default() -> Self {
		MembershipModel::Fixed { min_operators: 1 }
	}
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(Balance))]
pub struct UnappliedSlash<AccountId, Balance, AssetId> {
	/// The era the slash was reported.
	pub era: EraIndex,
	/// The Blueprint Id of the service being slashed.
	pub blueprint_id: u64,
	/// The Service Instance Id on which the slash is applied.
	pub service_id: u64,
	/// The account ID of the offending operator.
	pub operator: AccountId,
	/// The operator's own slash in native currency
	pub own: Balance,
	/// All other slashed restakers and amounts per asset.
	/// (delegator, asset, amount)
	pub others: Vec<(AccountId, Asset<AssetId>, Balance)>,
	/// Reporters of the offence; bounty payout recipients.
	pub reporters: Vec<AccountId>,
}
