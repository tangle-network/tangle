//! Implementations for this pallet

use crate::{
	BalanceOf, BondedPool, Config, Event, Pallet, Perbill, ReversePoolIdLookup, SubPoolsStorage,
	AssetIdOf,
};
use frame_support::traits::Defensive;
use sp_runtime::{traits::Zero, SaturatedConversion, Saturating};
use sp_staking::{EraIndex, OnStakingUpdate, StakingInterface};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

impl<T: Config> OnStakingUpdate<T::AccountId, BalanceOf<T>> for Pallet<T> {
	fn on_slash(
		pool_account: &T::AccountId,
		// Bonded balance is always read directly from staking, therefore we don't need to update
		// anything here.
		slashed_bonded: BalanceOf<T>,
		slashed_unlocking: &BTreeMap<EraIndex, BalanceOf<T>>,
		_total_slashed: BalanceOf<T>,
	) {
		if let Some(pool_id) = ReversePoolIdLookup::<T>::get(pool_account) {
			let mut sub_pools = match SubPoolsStorage::<T>::get(pool_id).defensive() {
				Some(sub_pools) => sub_pools,
				None => return,
			};
			for (era, slashed_balance) in slashed_unlocking.iter() {
				if let Some(pool) = sub_pools.with_era.get_mut(era) {
					pool.balance = *slashed_balance;
					Self::deposit_event(Event::<T>::UnbondingPoolSlashed {
						era: *era,
						pool_id,
						balance: *slashed_balance,
					});
				}
			}

			Self::deposit_event(Event::<T>::PoolSlashed { pool_id, balance: slashed_bonded });
			SubPoolsStorage::<T>::insert(pool_id, sub_pools);
		}
	}
}
