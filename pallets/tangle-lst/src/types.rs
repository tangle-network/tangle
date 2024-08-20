use super::*;

/// The balance type used by the currency system.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Type used for unique identifier of each pool.
pub type PoolId = u32;

pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub const POINTS_TO_BALANCE_INIT_RATIO: u32 = 1;

pub mod bonded_pool;
pub mod commission;
pub mod pools;
pub mod sub_pools;

pub use bonded_pool::*;
pub use commission::*;
pub use pools::*;
pub use sub_pools::*;
