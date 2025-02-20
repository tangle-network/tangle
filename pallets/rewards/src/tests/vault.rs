use super::*;

// Access control.
#[test]
fn update_vault_config_non_force_origin() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let apy = Perbill::from_percent(10);
		let deposit_cap = 1000;
		let boost_multiplier = Some(150);
		let incentive_cap = 1000;

		// Configure the reward vault
		assert_err!(
			RewardsPallet::<Runtime>::update_vault_reward_config(
				RuntimeOrigin::signed(mock_pub_key(1)),
				vault_id,
				RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier }
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn add_or_remove_asset_non_force_origin() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Add asset to vault
		assert_err!(
			RewardsPallet::<Runtime>::manage_asset_reward_vault(
				RuntimeOrigin::signed(mock_pub_key(1)),
				vault_id,
				asset,
				AssetAction::Add,
			),
			DispatchError::BadOrigin
		);

		// Remove asset from vault
		assert_err!(
			RewardsPallet::<Runtime>::manage_asset_reward_vault(
				RuntimeOrigin::signed(mock_pub_key(1)),
				vault_id,
				asset,
				AssetAction::Remove,
			),
			DispatchError::BadOrigin
		);
	});
}
