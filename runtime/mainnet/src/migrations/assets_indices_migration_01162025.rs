use frame_support::traits::Incrementable;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use pallet_assets::NextAssetId;

/// Set [`NextAssetId`] to the value of `ID` if [`NextAssetId`] does not exist yet.
pub struct SetNextAssetId<ID, T: pallet_assets::Config>(core::marker::PhantomData<(ID, T)>);
impl<ID, T: pallet_assets::Config> OnRuntimeUpgrade for SetNextAssetId<ID, T>
where
	T::AssetId: Incrementable,
	ID: Get<T::AssetId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if !NextAssetId::<T>::exists() {
			NextAssetId::<T>::put(ID::get());
			T::DbWeight::get().reads_writes(1, 1)
		} else {
			T::DbWeight::get().reads(1)
		}
	}
}
