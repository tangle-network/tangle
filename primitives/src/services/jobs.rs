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

use crate::services::{constraints::Constraints, types::TypeCheckError};
use educe::{Educe, *};
use frame_support::pallet_prelude::*;
use parity_scale_codec::Encode;
use sp_core::H160;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use super::{
	field::{Field, FieldType},
	BoundedString,
};

/// A Job Definition is a definition of a job that can be called.
/// It contains the input and output fields of the job with the permitted caller.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct JobDefinition<C: Constraints> {
	/// The metadata of the job.
	pub metadata: JobMetadata<C>,
	/// These are parameters that are required for this job.
	/// i.e. the input.
	pub params: BoundedVec<FieldType, C::MaxFields>,
	/// These are the result, the return values of this job.
	/// i.e. the output.
	pub result: BoundedVec<FieldType, C::MaxFields>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct JobMetadata<C: Constraints> {
	/// The Job name.
	pub name: BoundedString<C::MaxMetadataLength>,
	/// The Job description.
	pub description: Option<BoundedString<C::MaxMetadataLength>>,
}

/// A Job Call is a call to execute a job using it's job definition.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default)),
    Clone(bound(AccountId: Clone)),
    PartialEq(bound(AccountId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize),
    serde(bound(serialize = "AccountId: Serialize", deserialize = "AccountId: Deserialize<'de>")),
    educe(Debug(bound(AccountId: core::fmt::Debug)))
)]
pub struct JobCall<C: Constraints, AccountId> {
	/// The Service ID that this call is for.
	pub service_id: u64,
	/// The job definition index in the service that this call is for.
	pub job: u8,
	/// The supplied arguments for this job call.
	pub args: BoundedVec<Field<C, AccountId>, C::MaxFields>,
}

/// A Job Call Result is the result of a job call.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default)),
    Clone(bound(AccountId: Clone)),
    PartialEq(bound(AccountId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize),
    serde(bound(serialize = "AccountId: Serialize", deserialize = "AccountId: Deserialize<'de>")),
    educe(Debug(bound(AccountId: core::fmt::Debug)))
)]
pub struct JobCallResult<C: Constraints, AccountId> {
	/// The id of the service.
	pub service_id: u64,
	/// The id of the job call.
	pub call_id: u64,
	/// The result of the job call.
	pub result: BoundedVec<Field<C, AccountId>, C::MaxFields>,
}

/// A Job Result verifier is a verifier that will verify the result of a job call
/// using different verification methods.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobResultVerifier {
	/// No verification is needed.
	#[default]
	None,
	/// An EVM Contract Address that will verify the result.
	Evm(H160),
	// NOTE(@shekohex): Add more verification methods here.
}

/// Type checks the supplied arguments against the parameters.
pub fn type_checker<C: Constraints, AccountId: Encode + Clone>(
	params: &[FieldType],
	args: &[Field<C, AccountId>],
) -> Result<(), TypeCheckError> {
	if params.len() != args.len() {
		return Err(TypeCheckError::NotEnoughArguments {
			expected: params.len() as u8,
			actual: args.len() as u8,
		});
	}
	for i in 0..args.len() {
		let arg = &args[i];
		let expected = &params[i];
		if arg != expected {
			return Err(TypeCheckError::ArgumentTypeMismatch {
				index: i as u8,
				expected: expected.clone(),
				actual: arg.clone().into(),
			});
		}
	}
	Ok(())
}

impl<C: Constraints, AccountId: Encode + Clone> JobCall<C, AccountId> {
	/// Check if the supplied arguments match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition<C>) -> Result<(), TypeCheckError> {
		type_checker(&job_def.params, &self.args)
	}
}

impl<C: Constraints, AccountId: Encode + Clone> JobCallResult<C, AccountId> {
	/// Check if the supplied result match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition<C>) -> Result<(), TypeCheckError> {
		type_checker(&job_def.result, &self.result)
	}
}
