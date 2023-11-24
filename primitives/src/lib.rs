// Copyright 2022 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	pallet_prelude::Weight,
	weights::{
		constants::{ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_SECOND},
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use smallvec::smallvec;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiAddress, MultiSignature, Perbill,
};

pub mod types;
pub use types::*;
pub mod traits;

/// Tangle parachain time-related
pub mod time {
	use crate::types::{BlockNumber, Moment};
	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	#[cfg(not(feature = "integration-tests"))]
	pub const SECONDS_PER_BLOCK: Moment = 6;

	#[allow(clippy::identity_op)]
	#[cfg(feature = "integration-tests")]
	pub const SECONDS_PER_BLOCK: Moment = 3;

	pub const MILLISECS_PER_BLOCK: Moment = SECONDS_PER_BLOCK * 1000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

/// Money matters.
pub mod currency {
	use crate::Balance;

	// Supply units
	// =============
	/// The base unit, since we use 18 decimal places (10^18)
	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const MILLIUNIT: Balance = UNIT / 1000;
	pub const MICROUNIT: Balance = MILLIUNIT / 1000;

	// Monetary value
	// =============
	/// Lets assume 10 TNT = 1USD
	/// This assumption forms the base of all fee calculations, revisit this
	/// if the assumption is no longer true.
	pub const DOLLAR: Balance = UNIT * 10;
	pub const CENT: Balance = DOLLAR / 100;
	pub const MILLICENT: Balance = CENT / 1000;
	/// The existential deposit.
	#[allow(clippy::identity_op)]
	#[cfg(feature = "integration-tests")]
	pub const EXISTENTIAL_DEPOSIT: Balance = 1000;

	#[cfg(not(feature = "integration-tests"))]
	pub const EXISTENTIAL_DEPOSIT: Balance = MICROUNIT;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	// 0.1 GWei
	pub const WEIGHT_FEE: Balance = 100 * MEGAWEI;
	/// Return the cost to add an item to storage based on size
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLAR + (bytes as Balance) * 100 * MILLICENT
	}
}

/// Fee config for tangle parachain
pub mod fee {
	use super::*;
	use crate::currency::*;
	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
	///   - `[Balance::min, Balance::max]`
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
			// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
			let p = CENT;
			let q = 100 * crate::Balance::from(ExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

/// The number of blocks in one session
#[allow(clippy::identity_op)]
#[cfg(feature = "integration-tests")]
pub const SESSION_PERIOD_BLOCKS: BlockNumber = 20 * crate::time::MINUTES;

#[cfg(not(feature = "integration-tests"))]
pub const SESSION_PERIOD_BLOCKS: BlockNumber = 6 * crate::time::HOURS;

#[cfg(not(feature = "integration-tests"))]
pub const UNSIGNED_PROPOSAL_EXPIRY: BlockNumber = SESSION_PERIOD_BLOCKS / 4;

#[cfg(feature = "integration-tests")]
pub const UNSIGNED_PROPOSAL_EXPIRY: BlockNumber = SESSION_PERIOD_BLOCKS;

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 1 of a second of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND,
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);
