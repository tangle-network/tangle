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
use super::*;
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type FeeInfoOf<T> = FeeInfo<BalanceOf<T>>;

/// This struct represents the preset fee to charge for different DKG job types
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FeeInfo<Balance: MaxEncodedLen> {
	/// The base fee for all jobs.
	pub base_fee: Balance,

	/// The fee for Distributed Key Generation (DKG) job.
	pub dkg_validator_fee: Balance,

	/// The fee for signature generation.
	pub sig_validator_fee: Balance,

	/// The fee for refresh existing DKG.
	pub refresh_validator_fee: Balance,

	/// The storage fee per byts
	pub storage_fee_per_byte: Balance,

	// The storage fee per block
	pub storage_fee_per_block: Balance,
}

impl<Balance: MaxEncodedLen> FeeInfo<Balance> {
	/// Get the base fee.
	pub fn get_base_fee(self) -> Balance {
		self.base_fee
	}

	/// Get the DKG validator fee.
	pub fn get_dkg_validator_fee(self) -> Balance {
		self.dkg_validator_fee
	}

	/// Get the signature validator fee.
	pub fn get_sig_validator_fee(self) -> Balance {
		self.sig_validator_fee
	}

	/// Get the storage fee per byte
	pub fn get_storage_fee_per_byte(self) -> Balance {
		self.storage_fee_per_byte
	}
}
