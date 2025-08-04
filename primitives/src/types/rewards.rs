use super::*;
use crate::services::Asset;
use frame_system::Config;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use services::AssetIdT;
use sp_std::vec::Vec;

// Rewards Pallet Constants
use crate::{currency::UNIT, types::Balance};
use frame_support::PalletId;
use sp_runtime::Perbill;

/// Pallet ID for rewards
pub const PALLET_ID: PalletId = PalletId(*b"py/tnrew");

/// Minimum deposit cap (same for both mainnet and testnet)
pub const MIN_DEPOSIT_CAP: Balance = 0;

/// Minimum incentive cap (same for both mainnet and testnet)
pub const MIN_INCENTIVE_CAP: Balance = 0;

/// Maximum vault name length
pub const MAX_VAULT_NAME_LENGTH: u32 = 64;

/// Maximum vault logo length
pub const MAX_VAULT_LOGO_LENGTH: u32 = 256;

/// Maximum pending rewards per operator
pub const MAX_PENDING_REWARDS_PER_OPERATOR: u32 = 100;

/// Mainnet-specific constants
pub mod mainnet {
	use super::*;

	/// Maximum deposit cap for mainnet
	pub const MAX_DEPOSIT_CAP: Balance = UNIT * 100_000_000;

	/// Maximum incentive cap for mainnet
	pub const MAX_INCENTIVE_CAP: Balance = UNIT * 100_000_000;

	/// Maximum APY for mainnet (2%)
	pub const MAX_APY: Perbill = Perbill::from_percent(2);
}

/// Testnet-specific constants
pub mod testnet {
	use super::*;

	/// Maximum deposit cap for testnet
	pub const MAX_DEPOSIT_CAP: Balance = UNIT * 1_000_000_000_000;

	/// Maximum incentive cap for testnet
	pub const MAX_INCENTIVE_CAP: Balance = UNIT * 1_000_000_000_000;

	/// Maximum APY for testnet (20%)
	pub const MAX_APY: Perbill = Perbill::from_percent(20);
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum AssetType<AssetId> {
	/// This includes all lstTNT assets
	Tnt,
	/// This includes all EVM assets
	Evm(AssetId),
	/// This includes all native assets
	Native(AssetId),
}

/// Represents different types of rewards a user can earn
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserRewards<Balance, BlockNumber, AssetId: AssetIdT, MaxServiceRewards: Get<u32>> {
	/// Rewards earned from restaking (in TNT)
	pub restaking_rewards: Balance,
	/// Boost rewards information
	pub boost_rewards: BoostInfo<Balance, BlockNumber>,
	/// Service rewards in their respective assets
	pub service_rewards: BoundedVec<ServiceRewards<AssetId, Balance>, MaxServiceRewards>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserRestakeUpdate<Balance, AssetId: AssetIdT> {
	pub asset: Asset<AssetId>,
	pub amount: Balance,
	pub multiplier: LockMultiplier,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ServiceRewards<AssetId: AssetIdT, Balance> {
	asset: Asset<AssetId>,
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

impl<Balance: Default, BlockNumber: Default, AssetId: AssetIdT, MaxServiceRewards: Get<u32>> Default
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

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserDepositWithLocks<Balance, BlockNumber> {
	pub unlocked_amount: Balance,
	pub amount_with_locks: Option<Vec<LockInfo<Balance, BlockNumber>>>,
}

/// Struct to store the lock info
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct LockInfo<Balance, BlockNumber> {
	pub amount: Balance,
	pub lock_multiplier: LockMultiplier,
	pub expiry_block: BlockNumber,
}
