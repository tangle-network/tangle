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

use super::BoundedString;
use educe::Educe;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize};
use sp_core::{H160, RuntimeDebug, Get};
use sp_runtime::{Percent, traits::AtLeast32BitUnsigned};
use sp_staking::EraIndex;
use sp_std::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::{Constraints, field::FieldType, ServiceBlueprint};
use crate::{BlueprintId};

/// Maximum length for metadata fields
pub const MAX_METADATA_LENGTH: u32 = 1024;

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
	pub fn is_erc20(&self) -> bool {
		matches!(self, Asset::Erc20(_))
	}

	pub fn is_native(&self) -> bool {
		if let Asset::Custom(asset_id) = self { asset_id == &AssetId::default() } else { false }
	}

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
			Asset::Erc20(addr) => {
				let mut addr_bytes = [0u8; 32];
				addr_bytes[12..].copy_from_slice(addr.as_fixed_bytes());
				ethabi::Token::Tuple(vec![
					ethabi::Token::Uint(1.into()),
					ethabi::Token::FixedBytes(addr_bytes.into()),
				])
			},
		}
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
		/// Asset-specific exposure commitments
		security_commitments: Vec<AssetSecurityCommitment<AssetId>>,
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

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
pub struct OperatorPreferences<C: Constraints> {
	/// The operator ECDSA public key.
	pub key: [u8; 65],
	/// The address of the RPC server the operator is running.
	pub rpc_address: BoundedString<C::MaxRpcAddressLength>,
}

impl<C: Constraints> Default for OperatorPreferences<C> {
	fn default() -> Self {
		Self { key: [0u8; 65], rpc_address: BoundedString::default() }
	}
}

#[cfg(feature = "std")]
impl<C: Constraints> Serialize for OperatorPreferences<C> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeTuple;
		let mut tup = serializer.serialize_tuple(3)?;
		tup.serialize_element(&self.key[..])?;
		tup.serialize_element(&self.rpc_address)?;
		tup.end()
	}
}

#[cfg(feature = "std")]
struct OperatorPreferencesVisitor<C: Constraints> {
	_phantom: std::marker::PhantomData<C>,
}

#[cfg(feature = "std")]
impl<'de, C: Constraints> serde::de::Visitor<'de> for OperatorPreferencesVisitor<C> {
	type Value = OperatorPreferences<C>;

	fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
		formatter.write_str("a tuple of 3 elements (key_bytes, price_targets, rpc_address_string)")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::SeqAccess<'de>,
	{
		let key = seq
			.next_element::<Vec<u8>>()?
			.ok_or_else(|| serde::de::Error::custom("key is missing"))?;
		let rpc_address = seq
			.next_element::<BoundedString<C::MaxRpcAddressLength>>()?
			.ok_or_else(|| serde::de::Error::custom("rpc_address is missing"))?;
		let key_arr: [u8; 65] = key.try_into().map_err(|_| {
			serde::de::Error::custom(
				"key must be in the uncompressed format with length of 65 bytes",
			)
		})?;
		Ok(OperatorPreferences { key: key_arr, rpc_address })
	}
}

#[cfg(feature = "std")]
impl<'de, C: Constraints> Deserialize<'de> for OperatorPreferences<C> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_tuple(3, OperatorPreferencesVisitor {
			_phantom: std::marker::PhantomData::<C>,
		})
	}
}

impl<C: Constraints> OperatorPreferences<C> {
	/// Returns the ethabi ParamType for OperatorPreferences.
	pub fn to_ethabi_param_type() -> ethabi::ParamType {
		ethabi::ParamType::Tuple(vec![
			// Operator's ECDSA Public Key (33 bytes)
			ethabi::ParamType::Bytes,
			// Operator's RPC address - represent as String in ABI
			ethabi::ParamType::String,
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
			// rpc address
			ethabi::Token::String(self.rpc_address.to_string()),
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
pub enum MembershipModelType {
	/// Fixed set of operators defined at service creation
	Fixed,
	/// Operators can join/leave subject to blueprint rules
	Dynamic,
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
pub struct UnappliedSlash<AccountId> {
	/// The era the slash was reported.
	pub era: EraIndex,
	/// The Blueprint Id of the service being slashed.
	pub blueprint_id: u64,
	/// The Service Instance Id on which the slash is applied.
	pub service_id: u64,
	/// The account ID of the offending operator.
	pub operator: AccountId,
	/// The slash percentage
	pub slash_percent: Percent,
}

pub type ServiceId = u64;

/// Defines the different pricing models for services.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PricingModel<BlockNumber, Balance> {
    /// A one-time payment for the service.
    PayOnce {
        /// The total amount to be paid.
        amount: Balance,
    },
    /// A subscription-based model with recurring payments.
    Subscription {
        /// The amount to be paid per interval.
        rate_per_interval: Balance,
        /// The duration of each billing interval.
        interval: BlockNumber,
        /// An optional end block for the subscription.
        maybe_end: Option<BlockNumber>,
    },
    /// An event-driven model where rewards are based on reported events.
    EventDriven {
        /// The reward amount per reported event.
        reward_per_event: Balance,
    },
}

impl<BlockNumber, Balance: Default> Default for PricingModel<BlockNumber, Balance> {
    fn default() -> Self {
        PricingModel::PayOnce {
            amount: Balance::default(),
        }
    }
}

/// Blueprint data.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BlueprintData<AccountId, AssetId: AssetIdT, BlockNumber, Balance, C: Constraints> {
	/// The owner of the service blueprint.
	pub owner: AccountId,
	/// The metadata for the service blueprint.
	pub metadata: BoundedVec<u8, ConstU32<MAX_METADATA_LENGTH>>,
	/// The type definition of the service blueprint.
	pub typedef: ServiceBlueprint<C>,
	/// The membership model for the service blueprint.
	pub membership_model: MembershipModel,
	/// The security requirements for the service blueprint.
	pub security_requirements: Vec<AssetSecurityRequirement<AssetId>>,
	/// The price targets for the service blueprint.
	pub price_targets: Option<PriceTargets>,
	/// The pricing model for services created from this blueprint.
	pub pricing_model: PricingModel<BlockNumber, Balance>,
}

/// Represents an instance of a service.
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Instance<AccountId, AssetId: AssetIdT, MaxPermittedCallers: Get<u32>, MaxOperators: Get<u32>, BlockNumber, Balance> {
	/// The owner of the service instance.
	pub owner: AccountId,
	/// The blueprint ID from which this service instance was created.
	pub blueprint: BlueprintId,
	/// The set of permitted callers for this service instance.
	pub permitted_callers: BoundedBTreeSet<AccountId, MaxPermittedCallers>,
	/// The list of operators currently servicing this instance, along with their security
	/// commitments.
	pub operator_security_commitments:
		BoundedVec<(AccountId, Vec<AssetSecurityCommitment<AssetId>>), MaxOperators>,
	/// The pricing model for this service instance, copied from the blueprint.
	pub pricing_model: PricingModel<BlockNumber, Balance>,
	/// The block number when the last reward was recorded for this service.
	/// Used for PayOnce and Subscription models to prevent double-billing.
	pub last_billed: Option<BlockNumber>,
}

impl<AccountId, AssetId: AssetIdT, MaxPermittedCallers: Get<u32>, MaxOperators: Get<u32>, BlockNumber, Balance>
	Instance<AccountId, AssetId, MaxPermittedCallers, MaxOperators, BlockNumber, Balance>
{
	/// Validates the security commitments against the blueprint's requirements.
	pub fn validate_security_commitments(
		&self,
		_security_commitments: &[AssetSecurityCommitment<AssetId>],
	) -> bool {
		// TODO: Implement actual validation logic based on blueprint requirements.
		// This likely involves fetching the blueprint's `security_requirements`
		// and comparing them against the provided `security_commitments`.
		true
	}
}
