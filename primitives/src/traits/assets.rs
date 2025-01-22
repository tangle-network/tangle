/// Trait for reading the next available asset ID from pallet-assets
pub trait NextAssetId<AssetId> {
	/// Get the next available asset ID
	fn next_asset_id() -> Option<AssetId>;
}

/// A mapping between CurrencyId and AssetMetadata.
pub trait CurrencyIdMapping<CurrencyId, AssetMetadata> {
	/// Returns the AssetMetadata associated with a given `AssetIds`.
	fn get_asset_metadata(asset_ids: AssetIds) -> Option<AssetMetadata>;
	/// Returns the AssetMetadata associated with a given `CurrencyId`.
	fn get_currency_metadata(currency_id: CurrencyId) -> Option<AssetMetadata>;
	/// Returns the Location associated with a given CurrencyId.
	fn get_location(currency_id: &CurrencyId) -> Option<Location>;
	/// Returns the CurrencyId associated with a given Location.
	fn get_currency_id(location: &Location) -> Option<CurrencyId>;
	/// Returns all currencies in currencyMetadata.
	fn get_all_currency() -> Vec<CurrencyId>;
}
