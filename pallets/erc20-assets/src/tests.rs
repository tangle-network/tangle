use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, U256};
use pallet_evm_precompileset_assets_erc20::AddressToAssetId;

#[test]
fn create_asset_works() {
    new_test_ext().execute_with(|| {
        let erc20_address = H160::repeat_byte(1);
        let admin = 1;

        // Create a new asset
        assert_ok!(Erc20Assets::create_asset(
            RuntimeOrigin::signed(1),
            erc20_address,
            admin
        ));

        // The asset should exist in the Assets pallet
        let expected_asset_id = Erc20Assets::address_to_asset_id(erc20_address).unwrap();
        assert!(Assets::asset_exists(expected_asset_id));
    });
}

#[test]
fn address_to_asset_id_is_deterministic() {
    new_test_ext().execute_with(|| {
        let erc20_address = H160::repeat_byte(1);
        
        // Converting the same address multiple times should yield the same asset ID
        let asset_id1 = Erc20Assets::address_to_asset_id(erc20_address).unwrap();
        let asset_id2 = Erc20Assets::address_to_asset_id(erc20_address).unwrap();
        assert_eq!(asset_id1, asset_id2);

        // Different addresses should yield different asset IDs
        let other_address = H160::repeat_byte(2);
        let other_asset_id = Erc20Assets::address_to_asset_id(other_address).unwrap();
        assert_ne!(asset_id1, other_asset_id);
    });
}

#[test]
fn asset_id_to_address_is_reversible() {
    new_test_ext().execute_with(|| {
        let original_address = H160::repeat_byte(1);
        
        // Convert address to asset ID
        let asset_id = Erc20Assets::address_to_asset_id(original_address).unwrap();
        
        // Convert asset ID back to address
        let recovered_address = Erc20Assets::asset_id_to_address(asset_id);
        
        // The recovered address should match the original
        assert_eq!(original_address, recovered_address);
    });
}
