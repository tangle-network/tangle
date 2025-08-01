// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
		constants::{ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_MILLIS},
	},
};
use smallvec::smallvec;
use sp_runtime::{
	MultiAddress, MultiSignature, Perbill,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
};

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod chain_identifier;
pub mod impls;
pub mod services;
pub mod types;
pub use types::*;
pub mod precompiles_constants;
pub mod traits;

#[cfg(feature = "verifying")]
pub mod verifier;

/// Tangle parachain time-related
pub mod time {
	use crate::types::{BlockNumber, Moment};
	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	#[cfg(not(feature = "manual-seal"))]
	pub const SECONDS_PER_BLOCK: Moment = 6;
	#[cfg(feature = "manual-seal")]
	pub const SECONDS_PER_BLOCK: Moment = 1;

	pub const MILLISECS_PER_BLOCK: Moment = SECONDS_PER_BLOCK * 1000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	#[cfg(feature = "fast-runtime")]
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10; // 10 blocks for fast tests

	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	#[cfg(not(feature = "fast-runtime"))]
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};
}

/// Money matters.
pub mod currency {
	use crate::types::Balance;

	// Supply units
	// =============
	/// The base unit, since we use 18 decimal places (10^18)
	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const MILLIUNIT: Balance = UNIT / 1000;
	pub const MICROUNIT: Balance = MILLIUNIT / 1000;

	// Monetary value
	// =============
	/// Lets assume 100 TNT = 1USD
	/// This assumption forms the base of all fee calculations, revisit this
	/// if the assumption is no longer true.
	pub const DOLLAR: Balance = UNIT * 10;
	pub const CENT: Balance = DOLLAR / 100;
	pub const MILLICENT: Balance = CENT / 1000;
	/// The existential deposit.
	pub const EXISTENTIAL_DEPOSIT: Balance = UNIT;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	// 0.1 GWei
	pub const WEIGHT_FEE: Balance = 100 * MEGAWEI;
	/// Return the cost to add an item to storage based on size
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * DOLLAR + (bytes as Balance) * 10 * MILLICENT
	}
}

/// Fee config for tangle parachain
pub mod fee {
	use super::*;
	use crate::{currency::*, types::Balance};
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
			let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

pub mod evm {
	/// Current approximation of the gas/s consumption considering
	/// EVM execution over compiled WASM (on 4.4Ghz CPU).
	/// Given the 500ms Weight, from which 75% only are used for transactions,
	/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
	pub const GAS_PER_SECOND: u64 = 40_000_000;

	/// Approximate ratio of the amount of Weight per Gas.
	/// u64 works for approximations because Weight is a very small unit compared to gas.
	pub const WEIGHT_PER_GAS: u64 = frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND
		.saturating_div(GAS_PER_SECOND);

	/// The amount of gas per pov. A ratio of 4 if we convert ref_time to gas and we compare
	/// it with the pov_size for a block. E.g.
	/// ceil(
	///     (max_extrinsic.ref_time() / max_extrinsic.proof_size()) / WEIGHT_PER_GAS
	/// )
	pub const GAS_LIMIT_POV_SIZE_RATIO: u64 = 4;

	#[macro_export]
	macro_rules! impl_proxy_type {
		() => {
			#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
			#[derive(
				Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
			)]
			pub enum ProxyType {
				/// All calls can be proxied. This is the trivial/most permissive filter.
				Any = 0,
				/// Only extrinsics related to governance (democracy and collectives).
				Governance = 1,
				/// Allow to veto an announced proxy call.
				CancelProxy = 2,
				/// Allow extrinsic related to Balances.
				Balances = 3,
			}

			impl Default for ProxyType {
				fn default() -> Self {
					Self::Any
				}
			}

			fn is_governance_precompile(address: H160) -> bool {
				// Check if the precompile address matches Democracy (2051) or Preimage (2054) precompiles
				let democracy_address = precompile_utils::precompile_set::AddressU64::<{ tangle_primitives::precompiles_constants::PRECOMPILE_DEMOCRACY }>::get();
				let preimage_address = precompile_utils::precompile_set::AddressU64::<{ tangle_primitives::precompiles_constants::PRECOMPILE_PREIMAGE }>::get();
				address == democracy_address || address == preimage_address
			}

			// Be careful: Each time this filter is modified, the substrate filter must also be modified
			// consistently.
			impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {
				fn is_evm_proxy_call_allowed(
					&self,
					call: &pallet_evm_precompile_proxy::EvmSubCall,
					recipient_has_code: bool,
					gas: u64,
				) -> precompile_utils::EvmResult<bool> {
					Ok(match self {
						ProxyType::Any => true,
						ProxyType::Governance =>
							call.value == U256::zero() &&
								matches!(
									PrecompileName::from_address(call.to.0),
									Some(ref precompile) if is_governance_precompile(call.to.0)
								),
						// The proxy precompile does not contain method cancel_proxy
						ProxyType::CancelProxy => false,
						ProxyType::Balances => {
							// Allow only "simple" accounts as recipient (no code nor precompile).
							// Note: Checking the presence of the code is not enough because some precompiles
							// have no code.
							!recipient_has_code &&
								!precompile_utils::precompile_set::is_precompile_or_fail::<Runtime>(
									call.to.0, gas,
								)?
						},
					})
				}
			}
		}
	}
}

pub mod democracy {
	use crate::{Balance, BlockNumber, currency::UNIT, time::MINUTES};
	pub const LAUNCH_PERIOD: BlockNumber = 12 * 60 * MINUTES; // 12 hours
	pub const VOTING_PERIOD: BlockNumber = 7 * 24 * 60 * MINUTES; // 7 days
	pub const FASTTRACK_VOTING_PERIOD: BlockNumber = 2 * 24 * 60 * MINUTES; // 2 days
	pub const MINIMUM_DEPOSIT: Balance = 1000 * UNIT; // 1000 TNT
	pub const ENACTMENT_PERIOD: BlockNumber = 3 * 24 * 60 * MINUTES; // 3 days
	pub const COOLOFF_PERIOD: BlockNumber = 3 * 24 * 60 * MINUTES; // 3 days
	pub const MAX_PROPOSALS: u32 = 100;
}

pub mod elections {
	use crate::{Balance, BlockNumber, currency::UNIT, time::DAYS};

	pub const CANDIDACY_BOND: Balance = 1_000 * UNIT;
	pub const TERM_DURATION: BlockNumber = 7 * DAYS;
	pub const DESIRED_MEMBERS: u32 = 5;
	pub const DESIRED_RUNNERS_UP: u32 = 3;
	pub const MAX_CANDIDATES: u32 = 64;
	pub const MAX_VOTERS: u32 = 512;
	pub const MAX_VOTES_PER_VOTER: u32 = 32;
	pub const ELECTIONS_PHRAGMEN_PALLET_ID: frame_support::traits::LockIdentifier = *b"phrelect";
}

pub mod treasury {
	use crate::{
		Balance, BlockNumber,
		currency::{CENT, UNIT},
		time::DAYS,
	};
	use frame_support::PalletId;
	use sp_runtime::{Percent, Permill};

	pub const PROPOSAL_BOND: Permill = Permill::from_percent(5);
	pub const PROPOSAL_BOND_MINIMUM: Balance = 1000 * UNIT;
	pub const SPEND_PERIOD: BlockNumber = DAYS;
	pub const BURN: Permill = Permill::from_percent(0);
	pub const TIP_COUNTDOWN: BlockNumber = DAYS;
	pub const TIP_FINDERS_FEE: Percent = Percent::from_percent(20);
	pub const TIP_REPORT_DEPOSIT_BASE: Balance = UNIT;
	pub const DATA_DEPOSIT_PER_BYTE: Balance = CENT;
	pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");
	pub const MAXIMUM_REASON_LENGTH: u32 = 300;
	pub const MAX_APPROVALS: u32 = 100;
}

#[cfg(not(feature = "fast-runtime"))]
pub mod staking {
	// Six sessions in an era (24 hours).
	pub const SESSIONS_PER_ERA: sp_staking::SessionIndex = 6;
	// 14 eras for unbonding (14 days).
	pub const BONDING_DURATION: sp_staking::EraIndex = 14;
	// 27 eras for slash defer duration (10 days).
	pub const SLASH_DEFER_DURATION: sp_staking::EraIndex = 10;
	pub const MAX_NOMINATOR_REWARDED_PER_VALIDATOR: u32 = 256;
	pub const OFFENDING_VALIDATOR_THRESHOLD: sp_arithmetic::Perbill =
		sp_arithmetic::Perbill::from_percent(17);
	pub const OFFCHAIN_REPEAT: crate::BlockNumber = 5;
	pub const HISTORY_DEPTH: u32 = 80;
}

#[cfg(feature = "fast-runtime")]
pub mod staking {
	// 1 sessions in an era (10 blocks).
	pub const SESSIONS_PER_ERA: sp_staking::SessionIndex = 1;
	// 2 eras for unbonding (20 blocks).
	pub const BONDING_DURATION: sp_staking::EraIndex = 2;
	// 1 eras for slash defer (10 blocks).
	pub const SLASH_DEFER_DURATION: sp_staking::EraIndex = 1;
	pub const MAX_NOMINATOR_REWARDED_PER_VALIDATOR: u32 = 256;
	pub const OFFENDING_VALIDATOR_THRESHOLD: sp_arithmetic::Perbill =
		sp_arithmetic::Perbill::from_percent(17);
	pub const OFFCHAIN_REPEAT: crate::BlockNumber = 5;
	pub const HISTORY_DEPTH: u32 = 80;
}

pub mod credits {
	use crate::{currency::UNIT, time::DAYS, types::Balance};

	/// The maximum rate per block for stake tiers
	/// 1 TNT per block = 14,400 TNT per day
	pub const MAX_RATE_PER_BLOCK: Balance = UNIT;

	/// The maximum number of stake tiers allowed in storage
	pub const MAX_STAKE_TIERS: u32 = 20;

	/// The maximum accrual window duration in blocks
	pub const CLAIM_WINDOW_BLOCKS: u64 = DAYS * 7;
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2000ms of compute with a 6 second average block time.
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 2000;
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_MILLISECS_PER_BLOCK * WEIGHT_REF_TIME_PER_MILLIS, u64::MAX);

pub use sp_consensus_babe::AuthorityId as BabeId;

// 5845 this would give us addresses with tg prefix for mainnet like
// tgGmBRR5yM53bvq8tTzgsUirpPtfCXngYYU7uiihmWFJhmYGM
pub const MAINNET_SS58_PREFIX: u16 = 5845;
pub const MAINNET_CHAIN_ID: u64 = MAINNET_SS58_PREFIX as u64;

// 3799 this would give us addresses with tt prefix for testnet like
// ttFELSU4MTyzpfsgZ9tFinrmox7pV7nF1BLbfYjsu4rfDYM74
pub const TESTNET_SS58_PREFIX: u16 = 3799;
pub const TESTNET_CHAIN_ID: u64 = TESTNET_SS58_PREFIX as u64;

// 3287 this would give us addresses with tt prefix for testnet like
// ttFELSU4MTyzpfsgZ9tFinrmox7pV7nF1BLbfYjsu4rfDYM74
pub const TESTNET_LOCAL_SS58_PREFIX: u16 = 3287;
pub const TESTNET_LOCAL_CHAIN_ID: u64 = TESTNET_LOCAL_SS58_PREFIX as u64;
