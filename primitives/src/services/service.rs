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

use super::{
	constraints::Constraints,
	jobs::{type_checker, JobDefinition},
	types::{ApprovalState, Asset, MembershipModel},
	AssetIdT, AssetSecurityCommitment, AssetSecurityRequirement, BoundedString, Gadget,
	TypeCheckError,
};
use crate::{Account, BlueprintId};
use educe::Educe;
use frame_support::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::Percent;
use sp_std::{vec, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use super::field::{Field, FieldType};

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize), serde(bound = ""))]
pub struct ServiceMetadata<C: Constraints> {
	/// The Service name.
	pub name: BoundedString<C::MaxMetadataLength>,
	/// The Service description.
	pub description: Option<BoundedString<C::MaxMetadataLength>>,
	/// The Service author.
	/// Could be a company or a person.
	pub author: Option<BoundedString<C::MaxMetadataLength>>,
	/// The Job category.
	pub category: Option<BoundedString<C::MaxMetadataLength>>,
	/// Code Repository URL.
	/// Could be a github, gitlab, or any other code repository.
	pub code_repository: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service Logo URL.
	pub logo: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service Website URL.
	pub website: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service License.
	pub license: Option<BoundedString<C::MaxMetadataLength>>,
}

/// Blueprint Service Manager is a smart contract that will manage the service lifecycle.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BlueprintServiceManager {
	/// A Smart contract that will manage the service lifecycle.
	Evm(H160),
}

impl BlueprintServiceManager {
	pub fn try_into_evm(self) -> Result<H160, Self> {
		match self {
			Self::Evm(addr) => Ok(addr),
		}
	}
}

impl Default for BlueprintServiceManager {
	fn default() -> Self {
		Self::Evm(Default::default())
	}
}

/// Master Blueprint Service Manager Revision.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum MasterBlueprintServiceManagerRevision {
	/// Use Whatever the latest revision available on-chain.
	///
	/// This is the default value.
	#[default]
	#[codec(index = 0)]
	Latest,

	/// Use a specific revision number.
	///
	/// Note: Must be already deployed on-chain.
	#[codec(index = 1)]
	Specific(u32),
}

/// A Service Blueprint is a the main definition of a service.
/// it contains the metadata of the service, the job definitions, and other hooks, along with the
/// gadget that will be executed when one of the jobs is calling this service.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize), serde(bound = ""))]
pub struct ServiceBlueprint<C: Constraints> {
	/// The metadata of the service.
	pub metadata: ServiceMetadata<C>,
	/// The job definitions that are available in this service.
	pub jobs: BoundedVec<JobDefinition<C>, C::MaxJobsPerService>,
	/// The parameters that are required for the service registration.
	pub registration_params: BoundedVec<FieldType, C::MaxFields>,
	/// The request hook that will be called before creating a service from the service blueprint.
	/// The parameters that are required for the service request.
	pub request_params: BoundedVec<FieldType, C::MaxFields>,
	/// A Blueprint Manager is a smart contract that implements the `IBlueprintServiceManager`
	/// interface.
	pub manager: BlueprintServiceManager,
	/// The Revision number of the Master Blueprint Service Manager.
	///
	/// If not sure what to use, use `MasterBlueprintServiceManagerRevision::default()` which will
	/// use the latest revision available.
	pub master_manager_revision: MasterBlueprintServiceManagerRevision,
	/// The gadget that will be executed for the service.
	pub gadget: Gadget<C>,
	/// The membership models supported by this blueprint
	pub supported_membership_models: BoundedVec<MembershipModel, ConstU32<2>>,
}

impl<C: Constraints> ServiceBlueprint<C> {
	/// Check if the supplied arguments match the registration parameters.
	pub fn type_check_registration<AccountId: Encode + Clone>(
		&self,
		args: &[Field<C, AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.registration_params, args)
	}

	/// Check if the supplied arguments match the request parameters.
	pub fn type_check_request<AccountId: Encode + Clone>(
		&self,
		args: &[Field<C, AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.request_params, args)
	}

	/// Converts the struct to ethabi ParamType.
	pub fn to_ethabi_param_type() -> ethabi::ParamType {
		ethabi::ParamType::Tuple(vec![
			// Service Metadata
			ethabi::ParamType::Tuple(vec![
				// Service Name
				ethabi::ParamType::String,
				// Service Description
				ethabi::ParamType::String,
				// Service Author
				ethabi::ParamType::String,
				// Service Category
				ethabi::ParamType::String,
				// Code Repository
				ethabi::ParamType::String,
				// Service Logo
				ethabi::ParamType::String,
				// Service Website
				ethabi::ParamType::String,
				// Service License
				ethabi::ParamType::String,
			]),
			// Job Definitions ?
			// Registration Parameters ?
			// Request Parameters ?
			// Blueprint Manager
			ethabi::ParamType::Address,
			// Master Manager Revision
			ethabi::ParamType::Uint(32),
			// Gadget ?
		])
	}

	/// Converts the struct to ethabi Param.
	pub fn to_ethabi_param() -> ethabi::Param {
		ethabi::Param {
			name: String::from("blueprint"),
			kind: Self::to_ethabi_param_type(),
			internal_type: Some(String::from("struct MasterBlueprintServiceManager.Blueprint")),
		}
	}

	/// Converts the struct to ethabi Token.
	pub fn to_ethabi(&self) -> ethabi::Token {
		ethabi::Token::Tuple(vec![
			// Service Metadata
			ethabi::Token::Tuple(vec![
				// Service Name
				ethabi::Token::String(self.metadata.name.as_str().into()),
				// Service Description
				ethabi::Token::String(
					self.metadata
						.description
						.as_ref()
						.map(|v| v.as_str().into())
						.unwrap_or_default(),
				),
				// Service Author
				ethabi::Token::String(
					self.metadata.author.as_ref().map(|v| v.as_str().into()).unwrap_or_default(),
				),
				// Service Category
				ethabi::Token::String(
					self.metadata.category.as_ref().map(|v| v.as_str().into()).unwrap_or_default(),
				),
				// Code Repository
				ethabi::Token::String(
					self.metadata
						.code_repository
						.as_ref()
						.map(|v| v.as_str().into())
						.unwrap_or_default(),
				),
				// Service Logo
				ethabi::Token::String(
					self.metadata.logo.as_ref().map(|v| v.as_str().into()).unwrap_or_default(),
				),
				// Service Website
				ethabi::Token::String(
					self.metadata.website.as_ref().map(|v| v.as_str().into()).unwrap_or_default(),
				),
				// Service License
				ethabi::Token::String(
					self.metadata.license.as_ref().map(|v| v.as_str().into()).unwrap_or_default(),
				),
			]),
			// Job Definitions ?
			// Registration Parameters ?
			// Request Parameters ?
			// Blueprint Manager
			match self.manager {
				BlueprintServiceManager::Evm(addr) => ethabi::Token::Address(addr),
			},
			// Master Manager Revision
			match self.master_manager_revision {
				MasterBlueprintServiceManagerRevision::Latest => {
					ethabi::Token::Uint(ethabi::Uint::MAX)
				},
				MasterBlueprintServiceManagerRevision::Specific(rev) => {
					ethabi::Token::Uint(rev.into())
				},
			},
			// Gadget ?
		])
	}
}

/// Represents a request for service with specific security requirements for each asset.
/// The security requirements define the minimum and maximum exposure percentages that
/// operators must commit to be eligible for the service.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: AssetIdT"
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: AssetIdT)))
)]

pub struct ServiceRequest<C: Constraints, AccountId, BlockNumber, AssetId: AssetIdT> {
	/// The blueprint ID this request is for
	pub blueprint: BlueprintId,
	/// The account that requested the service
	pub owner: AccountId,
	/// The assets required for this service along with their security requirements.
	/// This defines both which assets are needed and how much security backing is required.
	pub non_native_asset_security:
		BoundedVec<AssetSecurityRequirement<AssetId>, C::MaxAssetsPerService>,
	/// Time-to-live for this request in blocks
	pub ttl: BlockNumber,
	/// Arguments for service initialization
	pub args: BoundedVec<Field<C, AccountId>, C::MaxFields>,
	/// Accounts permitted to call service functions
	pub permitted_callers: BoundedVec<AccountId, C::MaxPermittedCallers>,
	/// Operators and their approval states
	pub operators_with_approval_state:
		BoundedVec<(AccountId, ApprovalState<AssetId>), C::MaxOperatorsPerService>,
	/// The membership model to use for this service instance
	pub membership_model: MembershipModel,
}

impl<C: Constraints, AccountId, BlockNumber, AssetId: AssetIdT>
	ServiceRequest<C, AccountId, BlockNumber, AssetId>
{
	/// Returns true if all the operators are [ApprovalState::Approved].
	pub fn is_approved(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.all(|(_, state)| matches!(state, ApprovalState::Approved { .. }))
	}

	/// Returns true if any the operators are [ApprovalState::Pending].
	pub fn is_pending(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.any(|(_, state)| state == &ApprovalState::Pending)
	}

	/// Returns true if any the operators are [ApprovalState::Rejected].
	pub fn is_rejected(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.any(|(_, state)| state == &ApprovalState::Rejected)
	}

	/// Validates that an operator's security commitments meet the requirements
	pub fn validate_commitments(
		&self,
		asset_commitments: &[AssetSecurityCommitment<AssetId>],
	) -> bool
	where
		AssetId: PartialEq,
	{
		// Ensure commitments exist for all required assets
		self.non_native_asset_security.iter().all(|req| {
			asset_commitments.iter().any(|commit| {
				commit.asset == req.asset
					&& commit.exposure_percent >= req.min_exposure_percent
					&& commit.exposure_percent <= req.max_exposure_percent
			})
		})
	}
}

/// A staging service payment is a payment that is made for a service request
/// but will be paid when the service is created or refunded if the service is rejected.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen, Copy)]
#[educe(
    Default(bound(AccountId: Default, Balance: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, Balance: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, Balance: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "AccountId: Serialize, Balance: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, Balance: Deserialize<'de>, AssetId: AssetIdT",
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, Balance: core::fmt::Debug, AssetId: AssetIdT)))
)]
pub struct StagingServicePayment<AccountId, AssetId: AssetIdT, Balance> {
	/// The service request ID.
	pub request_id: u64,
	/// Where the refund should go.
	pub refund_to: Account<AccountId>,
	/// The Asset used in the payment.
	pub asset: Asset<AssetId>,
	/// The amount of the asset that is paid.
	pub amount: Balance,
}

/// A Service is an instance of a service blueprint.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: AssetIdT",
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: AssetIdT)))
)]
pub struct Service<C: Constraints, AccountId, BlockNumber, AssetId: AssetIdT> {
	/// Unique identifier for this service instance
	pub id: u64,
	/// The blueprint this service was created from
	pub blueprint: BlueprintId,
	/// The account that owns this service
	pub owner: AccountId,
	/// The assets and their security commitments from operators.
	/// This represents the actual security backing the service.
	pub non_native_asset_security: BoundedVec<
		(AccountId, BoundedVec<AssetSecurityCommitment<AssetId>, C::MaxAssetsPerService>),
		C::MaxOperatorsPerService,
	>,
	/// Active operators and their native currency exposure percentages
	pub native_asset_security: BoundedVec<(AccountId, Percent), C::MaxOperatorsPerService>,
	/// Accounts permitted to call service functions
	pub permitted_callers: BoundedVec<AccountId, C::MaxPermittedCallers>,
	/// Time-to-live in blocks
	pub ttl: BlockNumber,
	/// The membership model of the service
	pub membership_model: MembershipModel,
}

/// RPC Response for query the blueprint along with the services instances of that blueprint.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: AssetIdT",
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: core::fmt::Debug)))
)]
pub struct RpcServicesWithBlueprint<C: Constraints, AccountId, BlockNumber, AssetId: AssetIdT> {
	/// The blueprint ID.
	pub blueprint_id: u64,
	/// The service blueprint.
	pub blueprint: ServiceBlueprint<C>,
	/// The services instances of that blueprint.
	pub services: Vec<Service<C, AccountId, BlockNumber, AssetId>>,
}
