use super::*;
pub mod bonded_pool;
pub mod commission;
pub mod pools;
pub mod sub_pools;

pub use bonded_pool::*;
pub use commission::*;
pub use pools::*;
pub use sub_pools::*;

/// The balance type used by the currency system.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Type used for unique identifier of each pool.
pub type PoolId = u32;

pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub const POINTS_TO_BALANCE_INIT_RATIO: u32 = 1;
/// Possible operations on the configuration values of this pallet.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound, PartialEq, Clone)]
pub enum ConfigOp<T: Codec + Debug> {
	/// Don't change.
	Noop,
	/// Set the given value.
	Set(T),
	/// Remove from storage.
	Remove,
}

/// The type of bonding that can happen to a pool.
pub enum BondType {
	/// Someone is bonding into the pool upon creation.
	Create,
	/// Someone is adding more funds later to this pool.
	Later,
}

/// How to increase the bond of a member.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum BondExtra<Balance> {
	/// Take from the free balance.
	FreeBalance(Balance),
}

/// The type of account being created.
#[derive(Encode, Decode)]
pub enum AccountType {
	Bonded,
	Reward,
}

/// The permission a pool member can set for other accounts to claim rewards on their behalf.
#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum ClaimPermission {
	/// Only the pool member themself can claim their rewards.
	Permissioned,
	/// Anyone can compound rewards on a pool member's behalf.
	PermissionlessCompound,
	/// Anyone can withdraw rewards on a pool member's behalf.
	PermissionlessWithdraw,
	/// Anyone can withdraw and compound rewards on a pool member's behalf.
	PermissionlessAll,
}

impl ClaimPermission {
	pub fn can_bond_extra(&self) -> bool {
		matches!(self, ClaimPermission::PermissionlessAll | ClaimPermission::PermissionlessCompound)
	}
}

impl Default for ClaimPermission {
	fn default() -> Self {
		Self::Permissioned
	}
}
