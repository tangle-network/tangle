use super::*;
use parity_scale_codec::HasCompact;
use sp_runtime::RuntimeDebug;

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UnlockChunk<Balance: HasCompact + MaxEncodedLen> {
	/// Amount of funds to be unlocked.
	#[codec(compact)]
	pub value: Balance,
	/// Era number at which point it'll be unlocked.
	#[codec(compact)]
	pub era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct RestakingLedger<T: Config> {
	/// The stash account whose balance is actually locked and at stake.
	pub stash: T::AccountId,
	/// The total amount of the stash's balance that is restaked for all selected roles.
	/// This restaked balance we are currently accounting for new slashing conditions.
	#[codec(compact)]
	pub total: BalanceOf<T>,
	/// Restaking Profile
	pub profile: Profile<T>,
	/// Roles map with their respective records.
	pub roles: BoundedBTreeMap<RoleType, Record<T>, T::MaxRolesPerValidator>,
	/// Role key
	pub role_key: BoundedVec<u8, T::MaxKeyLen>,
	/// Any balance that is becoming free, which may eventually be transferred out of the stash
	/// (assuming it doesn't get slashed first). It is assumed that this will be treated as a first
	/// in, first out queue where the new (higher value) eras get pushed on the back.
	pub unlocking: BoundedVec<UnlockChunk<BalanceOf<T>>, T::MaxUnlockingChunks>,
	/// List of eras for which the stakers behind a validator have claimed rewards. Only updated
	/// for validators.
	pub claimed_rewards: BoundedVec<EraIndex, T::HistoryDepth>,
	/// Max active services
	pub max_active_services: u32,
}

impl<T: Config> RestakingLedger<T> {
	/// New staking ledger for a stash account.
	pub fn try_new(
		stash: T::AccountId,
		profile: Profile<T>,
		role_key: Vec<u8>,
		max_active_services: Option<u32>,
	) -> Result<Self, DispatchError> {
		let total_restake = profile.get_total_profile_restake();
		let mut roles: BoundedBTreeMap<_, _, _> = Default::default();
		for record in profile.get_records().into_iter() {
			roles.try_insert(record.role, record).map_err(|_| Error::<T>::KeySizeExceeded)?;
		}
		let bounded_role_key: BoundedVec<u8, T::MaxKeyLen> =
			role_key.try_into().map_err(|_| Error::<T>::KeySizeExceeded)?;
		Ok(Self {
			stash,
			total: total_restake.into(),
			profile,
			roles,
			role_key: bounded_role_key,
			unlocking: Default::default(),
			claimed_rewards: Default::default(),
			max_active_services: max_active_services
				.unwrap_or_else(|| T::MaxActiveJobsPerValidator::get()),
		})
	}

	/// Returns the total amount of the stash's balance that is restaked for all selected roles.
	pub fn total_restake(&self) -> BalanceOf<T> {
		self.total
	}

	/// Returns the amount of the stash's balance that is restaked for the given role.
	/// If the role is not found, returns zero.
	pub fn restake_for(&self, role: &RoleType) -> BalanceOf<T> {
		self.roles
			.get(role)
			.map_or_else(Zero::zero, |record| record.amount.unwrap_or_default())
	}
}

pub type CurrencyOf<T> = <T as pallet_staking::Config>::Currency;
pub type BalanceOf<T> =
	<CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type PositiveImbalanceOf<T> =
	<CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;
pub type NegativeImbalanceOf<T> =
	<CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
