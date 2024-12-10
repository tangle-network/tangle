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
use parity_scale_codec::HasCompact;
use sp_std::prelude::*;
use tangle_primitives::services::Constraints;

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

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct UnappliedSlash<AccountId, Balance: HasCompact> {
	/// The Service Instance Id on which the slash is applied.
	pub service_id: u64,
	/// The account ID of the offending operator.
	pub operator: AccountId,
	/// The operator's own slash.
	pub own: Balance,
	/// All other slashed restakers and amounts.
	pub others: Vec<(AccountId, Balance)>,
	/// Reporters of the offence; bounty payout recipients.
	pub reporters: Vec<AccountId>,
	/// The amount of payout.
	pub payout: Balance,
}
