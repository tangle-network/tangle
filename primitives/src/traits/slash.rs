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

use crate::services::UnappliedSlash;
use frame_support::weights::Weight;
use sp_runtime::DispatchError;

/// Trait for managing slashing in the Tangle network.
/// This trait provides functionality to slash operators and delegators.
pub trait SlashManager<AccountId, Balance, AssetId> {
	/// Slash a delegator's stake for an offense.
	///
	/// # Parameters
	/// * `unapplied_slash` - The unapplied slash record containing slash details
	/// * `delegator` - The account of the delegator being slashed
	fn slash_delegator(
		unapplied_slash: &UnappliedSlash<AccountId, Balance, AssetId>,
		delegator: &AccountId,
	) -> Result<Weight, DispatchError>;

	/// Slash an operator's stake for an offense.
	///
	/// # Parameters
	/// * `unapplied_slash` - The unapplied slash record containing slash details
	fn slash_operator(
		unapplied_slash: &UnappliedSlash<AccountId, Balance, AssetId>,
	) -> Result<Weight, DispatchError>;
}

impl<AccountId, Balance, AssetId> SlashManager<AccountId, Balance, AssetId> for () {
	fn slash_delegator(
		_unapplied_slash: &UnappliedSlash<AccountId, Balance, AssetId>,
		_delegator: &AccountId,
	) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}

	fn slash_operator(
		_unapplied_slash: &UnappliedSlash<AccountId, Balance, AssetId>,
	) -> Result<Weight, DispatchError> {
		Ok(Weight::zero())
	}
}
