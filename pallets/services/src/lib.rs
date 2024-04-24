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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::crypto::ByteArray;
use sp_runtime::{
	traits::{AccountIdConversion, Get, Zero},
	DispatchResult,
};
use sp_std::{prelude::*, vec::Vec};
use tangle_primitives::{
	jobs::{
		traits::{JobToFee, MPCHandler},
		DKGTSSKeySubmissionResult, JobId, JobInfo, JobResult, PhaseResult, ValidatorOffenceType,
	},
	misbehavior::{traits::MisbehaviorHandler, MisbehaviorSubmission},
	roles::traits::RolesHandler,
};

mod functions;
mod impls;
mod rpc;
mod types;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod mock_evm;
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::types::BalanceOf;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet(dev_mode)]
pub mod module {
	use super::*;
	use scale_info::prelude::fmt::Debug;
	use sp_runtime::Saturating;
	use tangle_primitives::roles::RoleType;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// `PalletId` for the jobs pallet.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
