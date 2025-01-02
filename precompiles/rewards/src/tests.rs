use super::*;
use frame_support::assert_ok;
use mock::*;
use precompile_utils::testing::*;
use sp_core::{H160, U256};
use sp_runtime::traits::Zero;
use tangle_primitives::services::Asset;

#[test]
fn test_whitelist_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let precompiles = precompiles();

        // Test native asset
        let asset_id = U256::from(1);
        let token_address = Address::zero();

        let input = EvmDataWriter::new_with_selector(Action::WhitelistAsset)
            .write(asset_id)
            .write(token_address)
            .build();

        let caller = H160::from_low_u64_be(1);
        let context = Context {
            address: Default::default(),
            caller,
            apparent_value: U256::zero(),
        };

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Test ERC20 asset
        let token_address = Address(H160::from_low_u64_be(2));
        let input = EvmDataWriter::new_with_selector(Action::WhitelistAsset)
            .write(U256::zero())
            .write(token_address)
            .build();

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));
    });
}

#[test]
fn test_set_asset_apy() {
    ExtBuilder::default().build().execute_with(|| {
        let precompiles = precompiles();

        // Whitelist asset first
        let asset_id = U256::from(1);
        let token_address = Address::zero();
        let apy = 500u32; // 5% APY

        // Whitelist the asset
        let input = EvmDataWriter::new_with_selector(Action::WhitelistAsset)
            .write(asset_id)
            .write(token_address)
            .build();

        let caller = H160::from_low_u64_be(1);
        let context = Context {
            address: Default::default(),
            caller,
            apparent_value: U256::zero(),
        };

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Set APY
        let input = EvmDataWriter::new_with_selector(Action::SetAssetApy)
            .write(asset_id)
            .write(token_address)
            .write(apy)
            .build();

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Verify APY was set
        let input = EvmDataWriter::new_with_selector(Action::GetAssetInfo)
            .write(asset_id)
            .write(token_address)
            .build();

        let mut handle = MockHandle::new(context, input);
        let result = precompiles.execute(&mut handle).unwrap();
        let (actual_apy, _) = EvmDataReader::new(&result)
            .read::<(U256, U256)>()
            .unwrap();
        assert_eq!(actual_apy, apy.into());
    });
}

#[test]
fn test_update_asset_apy() {
    ExtBuilder::default().build().execute_with(|| {
        let precompiles = precompiles();

        // Whitelist asset first
        let asset_id = U256::from(1);
        let token_address = Address::zero();
        let initial_apy = 500u32; // 5% APY
        let updated_apy = 1000u32; // 10% APY

        // Whitelist the asset
        let input = EvmDataWriter::new_with_selector(Action::WhitelistAsset)
            .write(asset_id)
            .write(token_address)
            .build();

        let caller = H160::from_low_u64_be(1);
        let context = Context {
            address: Default::default(),
            caller,
            apparent_value: U256::zero(),
        };

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Set initial APY
        let input = EvmDataWriter::new_with_selector(Action::SetAssetApy)
            .write(asset_id)
            .write(token_address)
            .write(initial_apy)
            .build();

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Update APY
        let input = EvmDataWriter::new_with_selector(Action::UpdateAssetApy)
            .write(asset_id)
            .write(token_address)
            .write(updated_apy)
            .build();

        let mut handle = MockHandle::new(context, input);
        assert_ok!(precompiles.execute(&mut handle));

        // Verify APY was updated
        let input = EvmDataWriter::new_with_selector(Action::GetAssetInfo)
            .write(asset_id)
            .write(token_address)
            .build();

        let mut handle = MockHandle::new(context, input);
        let result = precompiles.execute(&mut handle).unwrap();
        let (actual_apy, _) = EvmDataReader::new(&result)
            .read::<(U256, U256)>()
            .unwrap();
        assert_eq!(actual_apy, updated_apy.into());
    });
}

#[test]
fn test_get_user_rewards() {
    ExtBuilder::default().build().execute_with(|| {
        let precompiles = precompiles();

        // Setup test data
        let user = Address(H160::from_low_u64_be(2));
        let asset_id = U256::from(1);
        let token_address = Address::zero();

        // Query rewards
        let input = EvmDataWriter::new_with_selector(Action::GetUserRewards)
            .write(user)
            .write(asset_id)
            .write(token_address)
            .build();

        let context = Context {
            address: Default::default(),
            caller: H160::from_low_u64_be(1),
            apparent_value: U256::zero(),
        };

        let mut handle = MockHandle::new(context, input);
        let result = precompiles.execute(&mut handle).unwrap();
        
        // Parse result
        let (boost_amount, boost_expiry, service_rewards, restaking_rewards) = 
            EvmDataReader::new(&result)
                .read::<(U256, U256, U256, U256)>()
                .unwrap();

        // Initially all rewards should be zero
        assert!(boost_amount.is_zero());
        assert!(boost_expiry.is_zero());
        assert!(service_rewards.is_zero());
        assert!(restaking_rewards.is_zero());
    });
}
