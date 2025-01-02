use super::*;
use crate::services::Asset;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::Config;
use sp_runtime::Saturating;

/// Represents different types of rewards a user can earn
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserRewards<Balance, BlockNumber, AssetId, MaxServiceRewards: Get<u32>> {
	/// Rewards earned from restaking (in TNT)
	pub restaking_rewards: Balance,
	/// Boost rewards information
	pub boost_rewards: BoostInfo<Balance, BlockNumber>,
	/// Service rewards in their respective assets
	pub service_rewards: BoundedVec<ServiceRewards<AssetId, Balance>, MaxServiceRewards>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserRestakeUpdate<Balance, AssetId> {
	pub asset: Asset<AssetId>,
	pub amount: Balance,
	pub multiplier: LockMultiplier,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ServiceRewards<AssetId, Balance> {
	asset_id: Asset<AssetId>,
	amount: Balance,
}

/// Information about a user's boost rewards
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct BoostInfo<Balance, BlockNumber> {
	/// Amount of boost rewards
	pub amount: Balance,
	/// Multiplier for the boost (e.g. OneMonth = 1x, TwoMonths = 2x)
	pub multiplier: LockMultiplier,
	/// Block number when the boost expires
	pub expiry: BlockNumber,
}

impl<Balance: Default, BlockNumber: Default> Default for BoostInfo<Balance, BlockNumber> {
	fn default() -> Self {
		Self {
			amount: Balance::default(),
			multiplier: LockMultiplier::OneMonth,
			expiry: BlockNumber::default(),
		}
	}
}

impl<Balance: Default, BlockNumber: Default, AssetId, MaxServiceRewards: Get<u32>> Default
	for UserRewards<Balance, BlockNumber, AssetId, MaxServiceRewards>
{
	fn default() -> Self {
		Self {
			restaking_rewards: Balance::default(),
			boost_rewards: BoostInfo::default(),
			service_rewards: BoundedVec::default(),
		}
	}
}

/// Lock multiplier for rewards, representing months of lock period
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum LockMultiplier {
	/// One month lock period (1x multiplier)
	OneMonth = 1,
	/// Two months lock period (2x multiplier)
	TwoMonths = 2,
	/// Three months lock period (3x multiplier)
	ThreeMonths = 3,
	/// Six months lock period (6x multiplier)
	SixMonths = 6,
}

impl Default for LockMultiplier {
	fn default() -> Self {
		Self::OneMonth
	}
}

impl LockMultiplier {
	/// Get the multiplier value
	pub fn value(&self) -> u32 {
		*self as u32
	}

	/// Get the block number for each multiplier
	pub fn get_blocks(&self) -> u32 {
		// assuming block time of 6 seconds
		match self {
			LockMultiplier::OneMonth => 432000,
			LockMultiplier::TwoMonths => 864000,
			LockMultiplier::ThreeMonths => 1296000,
			LockMultiplier::SixMonths => 2592000,
		}
	}

	/// Calculate the expiry block number based on the current block number and multiplier
	pub fn expiry_block_number<T: Config>(&self, current_block: u32) -> u32 {
		current_block.saturating_add(self.get_blocks())
	}
}
