use super::*;
use frame_support::{traits::Currency, RuntimeDebug};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

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
}

impl<Balance: MaxEncodedLen> FeeInfo<Balance> {
	pub fn get_base_fee(self) -> Balance {
		self.base_fee
	}

	pub fn get_dkg_validator_fee(self) -> Balance {
		self.dkg_validator_fee
	}

	pub fn get_sig_validator_fee(self) -> Balance {
		self.sig_validator_fee
	}
}
