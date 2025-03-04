// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use crate::{Config, RewardConfigStorage, RewardConfigForAssetVault, BalanceOf};
use frame_support::{
    pallet_prelude::*,
    traits::OnRuntimeUpgrade,
    weights::{Weight, RuntimeDbWeight},
    migration::storage_key_iter,
};
use sp_runtime::{Perbill, Percent};
use sp_std::marker::PhantomData;

/// Migration to convert APY from percentage to Perbill in `RewardConfigForAssetVault`
pub struct PercentageToPerbillMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for PercentageToPerbillMigration<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = Weight::from_parts(0, 0);
        let db_weights = T::DbWeight::get();

        // Define the old version of the structure
        #[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
        pub struct OldRewardConfigForAssetVault<Balance> {
            // The annual percentage yield (APY) for the asset, represented as a percentage
            pub apy: Percent, // Percentage value
            pub incentive_cap: Balance,
            pub deposit_cap: Balance,
            pub boost_multiplier: Option<u32>,
        }

        // Iterate through all entries in the RewardConfigStorage
        let iter = storage_key_iter::<T::VaultId, OldRewardConfigForAssetVault<BalanceOf<T>>, Blake2_128Concat>(
            &RewardConfigStorage::<T>::prefix_hash(),
        );

        let mut migrated_count = 0;

        for (vault_id, old_config) in iter {
            // Read operation
            weight = weight.saturating_add(db_weights.read);
            let new_apy = Perbill::from_percent(old_config.apy);

            // Create new config with converted APY
            let new_config = RewardConfigForAssetVault {
                apy: new_apy,
                incentive_cap: old_config.incentive_cap,
                deposit_cap: old_config.deposit_cap,
                boost_multiplier: old_config.boost_multiplier,
            };

            // Update the storage with the new config
            RewardConfigStorage::<T>::insert(&vault_id, new_config);

            // Write operation
            weight = weight.saturating_add(db_weights.write);

            migrated_count += 1;
        }

        log::info!(
            "PercentageToPerbillMigration: Migrated {} reward configurations",
            migrated_count
        );

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Count how many entries we have pre-migration
        let count = RewardConfigStorage::<T>::iter().count() as u32;
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        // Ensure we have the same number of entries post-migration
        let pre_count = u32::decode(&mut &state[..]).expect("pre_upgrade should have encoded a u32");
        let post_count = RewardConfigStorage::<T>::iter().count() as u32;

        assert_eq!(
            pre_count, post_count,
            "Number of reward configurations changed during migration"
        );

        // Validate all APY values are now proper Perbill values
        for (_vault_id, config) in RewardConfigStorage::<T>::iter() {
            // Ensure APY is within Perbill range (0..=1_000_000_000)
            assert!(
                config.apy.deconstruct() <= 1_000_000_000,
                "APY value exceeds Perbill maximum"
            );
        }

        log::info!(
            "PercentageToPerbillMigration: Successfully migrated {} reward configurations",
            post_count
        );

        Ok(())
    }
}