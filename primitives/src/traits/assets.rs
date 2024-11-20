/// Trait for reading the next available asset ID from pallet-assets
pub trait NextAssetId<AssetId> {
	/// Get the next available asset ID
	fn next_asset_id() -> Option<AssetId>;
}
