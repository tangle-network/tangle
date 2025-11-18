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

use super::*;
use tangle_primitives::services::Constraints;
#[cfg(feature = "runtime-benchmarks")]
use tangle_primitives::traits::{
	MultiAssetDelegationBenchmarkingHelperDelegation,
	MultiAssetDelegationBenchmarkingHelperOperator,
};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type ConstraintsFor<T> = <T as Config>::Constraints;

pub type MaxPermittedCallersOf<T> = <ConstraintsFor<T> as Constraints>::MaxPermittedCallers;

pub type MaxServicesPerUserOf<T> = <ConstraintsFor<T> as Constraints>::MaxServicesPerUser;

pub type MaxFieldsOf<T> = <ConstraintsFor<T> as Constraints>::MaxFields;

pub type MaxOperatorsPerServiceOf<T> = <ConstraintsFor<T> as Constraints>::MaxOperatorsPerService;

pub type MaxAssetsPerServiceOf<T> = <ConstraintsFor<T> as Constraints>::MaxAssetsPerService;

/// Extract the constraints from the runtime.
#[derive(RuntimeDebugNoBound, CloneNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(encode_bound(skip_type_params(T)))]
#[codec(decode_bound(skip_type_params(T)))]
#[codec(mel_bound(skip_type_params(T)))]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintsOf<T>(sp_std::marker::PhantomData<T>);

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkingHelper<AccountId, Balance, AssetId>:
	MultiAssetDelegationBenchmarkingHelperDelegation<AccountId, Balance, AssetId>
	+ MultiAssetDelegationBenchmarkingHelperOperator<AccountId, Balance>
{
	// Take function from `use frame_support::traits::tokens::fungibles::Inspect;`
	fn asset_exists(_asset: AssetId) -> bool;
	fn balance(_asset: AssetId, _who: &AccountId) -> Balance;

	// Take function from `use frame_support::traits::tokens::fungibles::Mutate;`
	fn mint_into(
		_asset: AssetId,
		_who: &AccountId,
		_amount: Balance,
	) -> Result<Balance, DispatchError>;

	// Take function from `use frame_support::traits::tokens::fungibles::Create;`
	fn create(
		_id: AssetId,
		_admin: AccountId,
		_is_sufficient: bool,
		_min_balance: Balance,
	) -> DispatchResult;
}
