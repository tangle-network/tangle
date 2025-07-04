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

//! Runtime API definition for the credits pallet.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]
use parity_scale_codec::Codec;
use sp_runtime::{Serialize, traits::MaybeDisplay};

pub type BlockNumberOf<Block> =
	<<Block as sp_runtime::traits::HeaderProvider>::HeaderT as sp_runtime::traits::Header>::Number;

sp_api::decl_runtime_apis! {
	pub trait CreditsApi<AccountId, Balance, AssetId>
	where
		AccountId: Codec + MaybeDisplay + Serialize,
		Balance: Codec + MaybeDisplay + Serialize,
		AssetId: Codec + MaybeDisplay + Serialize,
	{
		/// Query all the rewards that this operator is providing along with their blueprints.
		///
		/// ## Arguments
		/// - `account_id`: The account id.
		/// ## Return
		/// - [`Balance`]: The credits of the account.
		fn query_user_credits(
			account_id: AccountId,
		) -> Result<
			Balance,
			sp_runtime::DispatchError,
		>;

		/// Query credits for a specific asset.
		///
		/// ## Arguments
		/// - `account_id`: The account id.
		/// - `asset_id`: The asset id to query credits for.
		/// ## Return
		/// - [`Balance`]: The credits of the account for the specified asset.
		fn query_user_credits_with_asset(
			account_id: AccountId,
			asset_id: AssetId,
		) -> Result<
			Balance,
			sp_runtime::DispatchError,
		>;
	}
}
