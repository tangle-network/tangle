use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Header, Perbill};

/// Migration to set default values for rewards pallet storage items
pub struct SetRewardsDefaultValues<T>(sp_std::marker::PhantomData<T>);

pub type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as Header>::Number;

impl<T: pallet_rewards::Config> OnRuntimeUpgrade for SetRewardsDefaultValues<T> {
	fn on_runtime_upgrade() -> Weight {
		let reads = 0u64;
		let writes = 3u64;

		// Set the number of blocks used for APY calculation
		// This is approximately 1 year worth of blocks with 6s block time (5,256,000 blocks)
		let apy_blocks: BlockNumberFor<T> = 5_256_000u32.into();
		pallet_rewards::ApyBlocks::<T>::put(apy_blocks);

		// Set the number of blocks after which decay starts (30 days with 6s blocks = 432,000 blocks)
		let decay_period: BlockNumberFor<T> = 432_000u32.into();
		pallet_rewards::DecayStartPeriod::<T>::put(decay_period);

		// Set the decay rate to 0.01% per block (1 basis point)
		pallet_rewards::DecayRate::<T>::put(Perbill::from_parts(1));

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		// Verify default values are set
		let expected_apy_blocks: BlockNumberFor<T> = 5_256_000u32.into();
		let expected_decay_period: BlockNumberFor<T> = 432_000u32.into();

		ensure!(
			pallet_rewards::ApyBlocks::<T>::get() == expected_apy_blocks,
			"APY blocks should be set to 5,256,000"
		);
		ensure!(
			pallet_rewards::DecayStartPeriod::<T>::get() == expected_decay_period,
			"Decay start period should be set to 432,000"
		);
		ensure!(
			pallet_rewards::DecayRate::<T>::get() == Perbill::from_parts(1),
			"Decay rate should be set to 0.01%"
		);
		Ok(())
	}
}
