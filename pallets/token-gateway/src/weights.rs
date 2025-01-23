use frame_support::weights::Weight;

// ============================== INTERFACE ============================================ //
/// Weight functions needed for `pallet_token_gateway.
pub trait WeightInfo {
	fn create_erc6160_asset(x: u32) -> Weight;
	fn teleport() -> Weight;
	fn set_token_gateway_addresses(x: u32) -> Weight;
	fn update_erc6160_asset() -> Weight;
	fn update_asset_precision(x: u32) -> Weight;
}

impl WeightInfo for () {
	fn create_erc6160_asset(_x: u32) -> Weight {
		Weight::zero()
	}

	fn teleport() -> Weight {
		Weight::zero()
	}

	fn set_token_gateway_addresses(_x: u32) -> Weight {
		Weight::zero()
	}

	fn update_erc6160_asset() -> Weight {
		Weight::zero()
	}

	fn update_asset_precision(_x: u32) -> Weight {
		Weight::zero()
	}
}
