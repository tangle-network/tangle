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

//! # Pallet-Dfns-CGGMP21
//!
//! A Substrate pallet for verifying submitted misbehavior of Distributed Key Generation (DKG)
//! protocol using the CGGMP21 scheme developed by dfns.
pub use pallet::*;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(test)]
mod mock;

mod constants;
mod functions;
#[cfg(test)]
mod tests;
mod types;
mod utils;
mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Invalid Misbehavior Role type.
		InvalidRoleType,
		/// Invalid Justification type.
		InvalidJustification,
		/// Unexpected job type
		InvalidJobType,
		/// Invalid signature submitted
		InvalidSignature,
		/// Could not deserialize the round message.
		MalformedRoundMessage,
		/// Signed Round Message not signed by the offender.
		NotSignedByOffender,
		/// The submitted decommitment is valid.
		///
		/// This error is returned when the decommitment is valid
		/// but the caller claims it is invalid!
		ValidDecommitment,
		/// The submitted decommitment data size is valid.
		///
		/// This error is returned when the decommitment data size is valid
		/// but the caller claims it is invalid!
		ValidDataSize,
		/// The submitted messages passed Feldman verification.
		///
		/// This error is returned when the messages passed Feldman verification
		/// but the caller claims it is invalid!
		ValidFeldmanVerification,
		/// The submitted Schnorr Proof is valid.
		///
		/// This error is returned when the decommitment and its
		/// Schnorr are valid. but the caller
		/// claims it is invalid.
		ValidSchnorrProof,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
