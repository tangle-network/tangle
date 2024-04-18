// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

//! Jobs v2 module.

use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::traits::Get;

mod field;
pub use field::*;

/// A Job Definition is a definition of a job that can be called.
/// It contains the input and output fields of the job with the permitted caller.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobDefinition<AccountId, MaxFields: Get<u32>> {
	/// These are parameters that are required for this job.
	/// i.e. the input.
	pub params: BoundedVec<FieldType, MaxFields>,
	/// These are the result, the return values of this job.
	/// i.e. the output.
	pub result: BoundedVec<FieldType, MaxFields>,
	/// The caller who can trigger this submission of this job type
	pub permitted_caller: Option<AccountId>,
	/// The verifier of the job result.
	pub verifier: JobResultVerifier,
}

/// A Job Call is a call to execute a job using it's job definition.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCall<AccountId, MaxFieldsSize: Get<u32>, MaxFields: Get<u32>> {
	/// The job definition that this call is for.
	pub job_def_id: u64,
	/// The supplied arguments for this job call.
	pub args: BoundedVec<Field<AccountId, MaxFieldsSize>, MaxFields>,
}

impl<AccountId: Clone, MaxFieldsSize: Get<u32> + Clone, MaxFields: Get<u32> + Clone>
	JobCall<AccountId, MaxFieldsSize, MaxFields>
{
	/// Check if the supplied arguments match the job definition types.
	pub fn type_check(
		&self,
		job_def: &JobDefinition<AccountId, MaxFields>,
	) -> Result<(), TypeCheckError> {
		if job_def.params.len() != self.args.len() {
			return Err(TypeCheckError::NotEnoughArguments {
				expected: job_def.params.len() as u8,
				actual: self.args.len() as u8,
			});
		}

		for i in 0..self.args.len() {
			let arg = &self.args[i];
			let expected = &job_def.params[i];
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
}

/// A Job Call Result is the result of a job call.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCallResult<AccountId, MaxFieldsSize: Get<u32>, MaxFields: Get<u32>> {
	/// The job definition that this call is for.
	pub job_def_id: u64,
	/// The id of the job call.
	pub call_id: u64,
	/// The result of the job call.
	pub result: BoundedVec<Field<AccountId, MaxFieldsSize>, MaxFields>,
}

/// A Job Result verifier is a verifier that will verify the result of a job call
/// using different verification methods.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobResultVerifier {
	/// An EVM Contract Address that will verify the result.
	Evm(sp_core::H160),
}

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
}
