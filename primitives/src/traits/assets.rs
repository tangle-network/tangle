/// Trait for reading the next available asset ID from pallet-assets
pub trait NextAssetId<AssetId> {
	/// Get the next available asset ID
	fn next_asset_id() -> Option<AssetId>;
}

/// A mapping between CurrencyId and AssetMetadata.
pub trait AssetIdMapping<AssetId, AssetMetadata> {
	/// Returns the AssetMetadata associated with a given `AssetIds`.
	fn get_asset_metadata(asset_id: AssetId) -> Option<AssetMetadata>;
}
