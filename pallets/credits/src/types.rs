use frame_support::{pallet_prelude::*, Deserialize, Serialize};
use scale_info::TypeInfo;
/// Defines a staking tier with its minimum stake threshold and credit emission rate per block.
/// The rates are applied based on the stake amount reported by the `StakingProvider`.
#[derive(
	Encode,
	Decode,
	Clone,
	Eq,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	PartialOrd,
	Ord,
	Serialize,
	Deserialize,
)]
pub struct StakeTier<
	Balance: MaxEncodedLen + Encode + Decode + Clone + Eq + PartialEq + TypeInfo + Serialize,
> {
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
