use crate::Config;
use tangle_primitives::traits::assets::NextAssetId;

impl<T: Config> NextAssetId<T::AssetId> for crate::Pallet<T> {
	fn next_asset_id() -> Option<T::AssetId> {
		crate::NextAssetId::<T>::get()
	}
}
