use crate::BalanceOf;
use frame_support::{pallet_prelude::*, traits::fungibles::Inspect};
use scale_info::TypeInfo;
use tangle_primitives::Balance; // Import BalanceOf from crate itself

/// Defines a staking tier with its minimum stake threshold and credit emission rate per block.
/// The rates are applied based on the stake amount reported by the `StakingProvider`.
#[derive(
	Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialOrd, Ord,
)]
pub struct StakeTier<Balance> {
	/// The minimum stake amount required to qualify for this tier.
	#[codec(compact)]
	pub threshold: Balance,
	/// The number of credits emitted per block for stakes within this tier.
	#[codec(compact)]
	pub rate_per_block: Balance,
}

/// Type alias for the block number type from the frame_system configuration.
pub type BlockNumberOf<T> = frame_system::pallet_prelude::BlockNumberFor<T>;

/// Type alias for the bounded vector representing the off-chain account ID.
pub type OffchainAccountIdOf<T> = BoundedVec<u8, <T as crate::Config>::MaxOffchainAccountIdLength>;
