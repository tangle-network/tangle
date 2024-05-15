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

//! Runtime API definition for services pallet.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]
use parity_scale_codec::Codec;
use sp_runtime::{
	traits::{MaybeDisplay},
	Serialize,
};
use sp_std::vec::Vec;
use tangle_primitives;

pub type BlockNumberOf<Block> =
	<<Block as sp_runtime::traits::HeaderProvider>::HeaderT as sp_runtime::traits::Header>::Number;

sp_api::decl_runtime_apis! {
	pub trait ServicesApi<AccountId> where AccountId: Codec + MaybeDisplay + Serialize {
		/// Query jobs associated with a specific validator.
		///
		/// This function takes a `validator` parameter of type `AccountId` and attempts
		/// to retrieve a list of jobs associated with the provided validator. If successful,
		/// it constructs a vector of `RpcResponseJobsData` containing information
		/// about the jobs and returns it as a `Result`.
		///
		/// # Arguments
		///
		/// * `validator` - The account ID of the validator whose jobs are to be queried.
		///
		/// # Returns
		///
		/// An optional vec of `RpcResponseJobsData` of jobs assigned to validator
		fn query_jobs_by_validator(validator: AccountId) -> Option<Vec<RpcResponseJobsData<AccountId, BlockNumberOf<Block>, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>>>;
	}
}
