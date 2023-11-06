// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
//! Runtime API definition for jobs pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::Codec;
use sp_runtime::{traits::MaybeDisplay, Serialize};

pub use tangle_primitives::jobs::RpcResponseJobsData;

sp_api::decl_runtime_apis! {
	pub trait JobsApi<AccountId> where
		AccountId: Codec + MaybeDisplay + Serialize,
	{
		/// Query jobs associated with a specific validator.
		///
		/// This function takes a `validator` parameter of type `AccountId` and attempts
		/// to retrieve a list of jobs associated with the provided validator. If successful,
		/// it constructs a vector of `RpcResponseJobsData<AccountId>` containing information
		/// about the jobs and returns it as a `Result`.
		///
		/// # Arguments
		///
		/// * `validator` - The account ID of the validator whose jobs are to be queried.
		///
		/// # Returns
		///
		/// Returns a `Result` containing a vector of `RpcResponseJobsData<AccountId>` if the
		/// operation is successful
		fn query_jobs_by_validator(validator: AccountId) -> Result<Vec<RpcResponseJobsData<AccountId>>, String>;
	}
}
