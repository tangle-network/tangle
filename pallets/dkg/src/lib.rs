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
//! # Pallet-DKG
//!
//! A Substrate pallet for verifying submitted results of Distributed Key Generation (DKG)
//! protocols.
//!
//! This pallet provides functionality to verify the results of a DKG process. It includes methods
//! to verify DKG signatures, recover ECDSA public keys, and ensure the validity of the generated
//! DKG key.
pub use pallet::*;

#[cfg(test)]
mod mock;

mod functions;
#[cfg(test)]
mod tests;
mod types;
mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::{types::FeeInfoOf, weights::WeightInfo};
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{Get, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use tangle_primitives::jobs::JobId;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The origin which may set filter.
		type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Weight info for pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn fee_info)]
	pub type FeeInfo<T> = StorageValue<_, FeeInfoOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fee has been updated to the new value
		FeeUpdated(FeeInfoOf<T>),
		/// A DKG has been rotated.
		KeyRotated { from_job_id: JobId, to_job_id: JobId, signature: Vec<u8> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Cannot retreive signers from the signature
		CannotRetreiveSigner,
		/// Not enough signers for threshold
		NotEnoughSigners,
		/// The signers have signed different data
		InvalidSignatureData,
		/// No participants found
		NoParticipantsFound,
		/// No signatures found
		NoSignaturesFound,
		/// Unexpected job type
		InvalidJobType,
		/// Duplicate signature found in submission
		DuplicateSignature,
		/// Invalid signature submitted
		InvalidSignature,
		/// Signed with a different key
		SigningKeyMismatch,
		/// Invalid participant public key
		InvalidParticipantPublicKey,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the fee information for the pallet.
		///
		/// This extrinsic allows the designated origin to update the fee information, which
		/// includes parameters such as the base fee and the fee per validator. It updates the
		/// storage with the provided `FeeInfo` and emits an event indicating that the fee has been
		/// updated.
		///
		/// # Arguments
		///
		/// * `origin` - The origin that is permitted to set the fee. It should be authorized by
		///   `UpdateOrigin`.
		/// * `fee_info` - The new fee information to be set for the pallet.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_fee(origin: OriginFor<T>, fee_info: FeeInfoOf<T>) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;

			// Update storage.
			<FeeInfo<T>>::put(fee_info.clone());

			// Emit an event.
			Self::deposit_event(Event::FeeUpdated(fee_info));
			Ok(().into())
		}
	}
}
