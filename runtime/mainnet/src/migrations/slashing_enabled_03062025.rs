use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_runtime::traits::Header;

/// Migration to ensure slashing is not enabled
pub struct EnsureSlashingNotEnabled<T>(sp_std::marker::PhantomData<T>);

pub type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as Header>::Number;

impl<T: pallet_services::Config> OnRuntimeUpgrade for EnsureSlashingNotEnabled<T> {
	fn on_runtime_upgrade() -> Weight {
		let reads = 0u64;
		let writes = 1u64;

		pallet_services::SlashingEnabled::<T>::put(false);

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		// Verify slashing is not enabled
		ensure!(!pallet_services::SlashingEnabled::<T>::get(), "Slashing should be disabled");
		Ok(())
	}
}
