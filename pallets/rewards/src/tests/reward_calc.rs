use super::*;

#[test]
fn test_claim_rewards_with_invalid_asset() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Try to claim rewards for an asset that doesn't exist in the vault
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::AssetNotInVault
		);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1000,
				incentive_cap: 1000,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_claim_rewards_with_no_deposit() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1000,
				incentive_cap: 1000,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}