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
use sp_runtime::{traits::MaybeDisplay, Serialize};
use sp_std::vec::Vec;
use tangle_primitives::services::{Constraints, RpcServicesWithBlueprint};

pub type BlockNumberOf<Block> =
	<<Block as sp_runtime::traits::HeaderProvider>::HeaderT as sp_runtime::traits::Header>::Number;

sp_api::decl_runtime_apis! {
	pub trait ServicesApi<C, AccountId, AssetId>
	where
		C: Constraints,
		AccountId: Codec + MaybeDisplay + Serialize,
		AssetId: Codec + MaybeDisplay + Serialize,
	{
		/// Query all the services that this operator is providing along with their blueprints.
		///
		/// ## Arguments
		/// - `operator`: The operator account id.
		/// ## Return
		/// - [`RpcServicesWithBlueprint`]: A list of services with their blueprints.
		fn query_services_with_blueprints_by_operator(
			operator: AccountId,
		) -> Result<
			Vec<RpcServicesWithBlueprint<C, AccountId, BlockNumberOf<Block>, AssetId>>,
			sp_runtime::DispatchError,
		>;
	}
}
