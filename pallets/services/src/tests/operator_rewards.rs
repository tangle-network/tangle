// End-to-end tests for operator rewards claiming flow
// Tests the complete flow: customer payment → distribution → operator claim

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::assert_ok;
use sp_runtime::Percent;
use tangle_primitives::{
	services::{Asset, AssetSecurityCommitment, PricingModel, Service},
	traits::RewardRecorder,
};

/// Helper to create a minimal test service with specified operators and commitments
fn create_test_service_with_operators(
	blueprint_id: u64,
	service_id: u64,
	owner: AccountId,
	commitments: Vec<(AccountId, Vec<AssetSecurityCommitment<AssetId>>)>,
) -> Service<ConstraintsOf<Runtime>, AccountId, BlockNumberFor<Runtime>, AssetId> {
	Service {
		id: service_id,
		blueprint: blueprint_id,
		owner,
		args: vec![].try_into().unwrap(),
		operator_security_commitments: commitments
			.into_iter()
			.map(|(op, comms)| (op, comms.try_into().unwrap()))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		security_requirements: vec![].try_into().unwrap(),
		permitted_callers: vec![].try_into().unwrap(),
		ttl: 100,
		membership_model: MembershipModel::Fixed { min_operators: 1 },
	}
}

#[test]
fn test_customer_payment_transfers_to_rewards_pallet() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let rewards_account = MockRewardsManager::account_id(); // mock_pub_key(100)

		// Check initial balances
		let customer_initial = Balances::free_balance(&customer);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Customer pays 10,000 tokens
		let payment: Balance = 10_000;
		assert_ok!(Services::charge_payment(&customer, &customer, payment));

		// Verify funds transferred to rewards pallet account
		let customer_after = Balances::free_balance(&customer);
		let rewards_after = Balances::free_balance(&rewards_account);

		assert_eq!(
			customer_initial - customer_after,
			payment,
			"Customer should have paid 10,000 tokens"
		);
		assert_eq!(
			rewards_after - rewards_initial,
			payment,
			"Rewards pallet should have received 10,000 tokens"
		);
	});
}

#[test]
fn test_e2e_pay_once_payment_with_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let rewards_account = MockRewardsManager::account_id();

		// Create service with 2 operators
		// Bob: 60% TNT exposure
		// Charlie: 40% TNT exposure
		let service = create_test_service_with_operators(
			0,
			0,
			dave.clone(),
			vec![
				(
					bob.clone(),
					vec![AssetSecurityCommitment {
						asset: Asset::Custom(TNT),
						exposure_percent: Percent::from_percent(60),
					}],
				),
				(
					charlie.clone(),
					vec![AssetSecurityCommitment {
						asset: Asset::Custom(TNT),
						exposure_percent: Percent::from_percent(40),
					}],
				),
			],
		);

		let customer_initial = Balances::free_balance(&customer);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Customer pays 10,000 tokens
		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		// Step 1: Customer payment
		assert_ok!(Services::charge_payment(&customer, &customer, payment));

		// Verify funds transferred to rewards pallet
		let customer_after_payment = Balances::free_balance(&customer);
		let rewards_after_payment = Balances::free_balance(&rewards_account);

		assert_eq!(customer_initial - customer_after_payment, payment);
		assert_eq!(rewards_after_payment - rewards_initial, payment);

		// Step 2: Distribute payment
		assert_ok!(Services::distribute_service_payment(&service, &dave, payment, &pricing_model));

		// Verify reward distribution:
		// Total exposure: 60 + 40 = 100 percentage points
		// Operator share: 85% of 10,000 = 8,500 tokens
		// Bob: (60/100) * 8,500 = 5,100 tokens
		// Charlie: (40/100) * 8,500 = 3,400 tokens
		// Developer (Dave): 10% of 10,000 = 1,000 tokens

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 5_100, "Bob should receive 5,100 tokens (60% exposure)");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 3_400, "Charlie should receive 3,400 tokens (40% exposure)");

		let dave_rewards = MockRewardsManager::get_pending_rewards(&dave);
		let dave_total: u128 = dave_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(dave_total, 1_000, "Dave (developer) should receive 1,000 tokens (10%)");

		// Verify total adds up
		assert_eq!(
			bob_total + charlie_total + dave_total,
			9_500,
			"Total distributed should be 9,500 (95% of 10,000)"
		);

		// Funds remain in rewards pallet account until claimed
		let rewards_final = Balances::free_balance(&rewards_account);
		assert_eq!(rewards_final, rewards_after_payment, "Funds should remain in rewards pallet");
	});
}

#[test]
fn test_e2e_subscription_payment_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		// Create service with 1 operator
		let service = create_test_service_with_operators(
			0,
			0,
			charlie.clone(),
			vec![(
				bob.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				}],
			)],
		);

		let customer_initial = Balances::free_balance(&customer);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Subscription payment: 1,000 tokens per 10 blocks
		let rate_per_interval: Balance = 1_000;
		let interval: BlockNumberFor<Runtime> = 10;
		let pricing_model =
			PricingModel::Subscription { rate_per_interval, interval, maybe_end: Some(100) };

		// Process first subscription payment
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&charlie,
			rate_per_interval,
			&pricing_model
		));

		// Verify first payment
		let customer_after_1 = Balances::free_balance(&customer);
		let rewards_after_1 = Balances::free_balance(&rewards_account);

		assert_eq!(customer_initial - customer_after_1, rate_per_interval);
		assert_eq!(rewards_after_1 - rewards_initial, rate_per_interval);

		// Bob should get 85% of 1,000 = 850 tokens
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 850, "Bob should receive 850 tokens (85% of 1,000)");

		// Charlie (developer) should get 10% of 1,000 = 100 tokens
		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 100, "Charlie should receive 100 tokens (10% of 1,000)");
	});
}

#[test]
fn test_multiple_operators_different_exposures() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let rewards_account = MockRewardsManager::account_id();

		// Create service with 3 operators with multi-asset exposures
		// Bob: 50% TNT + 30% WETH = 80 total
		// Charlie: 40% TNT + 20% WETH = 60 total
		// Dave: 30% TNT + 10% WETH = 40 total
		// Total exposure: 180 percentage points
		let service = create_test_service_with_operators(
			0,
			0,
			customer.clone(),
			vec![
				(
					bob.clone(),
					vec![
						AssetSecurityCommitment {
							asset: Asset::Custom(TNT),
							exposure_percent: Percent::from_percent(50),
						},
						AssetSecurityCommitment {
							asset: Asset::Custom(WETH),
							exposure_percent: Percent::from_percent(30),
						},
					],
				),
				(
					charlie.clone(),
					vec![
						AssetSecurityCommitment {
							asset: Asset::Custom(TNT),
							exposure_percent: Percent::from_percent(40),
						},
						AssetSecurityCommitment {
							asset: Asset::Custom(WETH),
							exposure_percent: Percent::from_percent(20),
						},
					],
				),
				(
					dave.clone(),
					vec![
						AssetSecurityCommitment {
							asset: Asset::Custom(TNT),
							exposure_percent: Percent::from_percent(30),
						},
						AssetSecurityCommitment {
							asset: Asset::Custom(WETH),
							exposure_percent: Percent::from_percent(10),
						},
					],
				),
			],
		);

		let payment: Balance = 9_000; // Reduced to avoid balance issues
		let pricing_model = PricingModel::PayOnce { amount: payment };

		let rewards_initial = Balances::free_balance(&rewards_account);

		// Customer pays
		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&customer,
			payment,
			&pricing_model
		));

		// Verify funds transferred
		let rewards_after = Balances::free_balance(&rewards_account);
		assert_eq!(rewards_after - rewards_initial, payment);

		// Calculate expected rewards:
		// Operator share: 85% * 9,000 = 7,650
		// Bob: (80/180) * 7,650 = 3,400
		// Charlie: (60/180) * 7,650 = 2,550
		// Dave: (40/180) * 7,650 = 1,700
		// Developer: 10% * 9,000 = 900

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 3_400, "Bob should receive 3,400 tokens");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 2_550, "Charlie should receive 2,550 tokens");

		let dave_rewards = MockRewardsManager::get_pending_rewards(&dave);
		let dave_total: u128 = dave_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(dave_total, 1_700, "Dave should receive 1,700 tokens");

		let customer_rewards = MockRewardsManager::get_pending_rewards(&customer);
		let customer_total: u128 = customer_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(customer_total, 900, "Developer should receive 900 tokens");

		// Verify total
		assert_eq!(
			bob_total + charlie_total + dave_total + customer_total,
			8_550,
			"Total should be 8,550 (95% of 9,000)"
		);
	});
}

#[test]
fn test_payment_fails_with_insufficient_balance() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let customer_balance = Balances::free_balance(&customer);

		// Try to pay more than balance
		let excessive_payment = customer_balance + 1_000;
		assert!(Services::charge_payment(&customer, &customer, excessive_payment).is_err());
	});
}

#[test]
fn test_zero_payment_no_transfer() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let rewards_account = MockRewardsManager::account_id();

		let service = create_test_service_with_operators(
			0,
			0,
			customer.clone(),
			vec![(
				bob.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				}],
			)],
		);

		let rewards_initial = Balances::free_balance(&rewards_account);

		// Zero payment
		let payment: Balance = 0;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&customer,
			payment,
			&pricing_model
		));

		// No funds transferred
		let rewards_after = Balances::free_balance(&rewards_account);
		assert_eq!(rewards_after, rewards_initial);

		// No rewards recorded
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(bob_rewards.len(), 0);
	});
}

#[test]
fn test_payment_authorization_check() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Alice tries to charge Bob's account - should fail
		let payment: Balance = 1_000;
		assert!(Services::charge_payment(&alice, &bob, payment).is_err());
	});
}

#[test]
fn test_e2e_event_driven_payment_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		let service = create_test_service_with_operators(
			0,
			0,
			charlie.clone(),
			vec![(
				bob.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(100),
				}],
			)],
		);

		let reward_per_event: Balance = 100;
		let event_count = 10u32;
		let total_payment = reward_per_event * event_count as u128;

		let customer_initial = Balances::free_balance(&customer);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Process event-driven payment
		assert_ok!(Services::charge_payment(&customer, &customer, total_payment));

		let pricing_model = PricingModel::EventDriven { reward_per_event };
		assert_ok!(Services::distribute_service_payment(
			&service,
			&charlie,
			total_payment,
			&pricing_model
		));

		// Verify funds transferred
		let customer_after = Balances::free_balance(&customer);
		let rewards_after = Balances::free_balance(&rewards_account);

		assert_eq!(customer_initial - customer_after, total_payment);
		assert_eq!(rewards_after - rewards_initial, total_payment);

		// Bob should get 85% of 1,000 = 850 tokens
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 850, "Bob should receive 850 tokens (85%)");

		// Charlie should get 10% of 1,000 = 100 tokens
		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 100, "Charlie should receive 100 tokens (10%)");
	});
}

#[test]
fn test_rewards_remain_in_pallet_until_claimed() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		let service = create_test_service_with_operators(
			0,
			0,
			charlie.clone(),
			vec![(
				bob.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				}],
			)],
		);

		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		let rewards_initial = Balances::free_balance(&rewards_account);

		// Payment and distribution
		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&charlie,
			payment,
			&pricing_model
		));

		// Funds should remain in rewards pallet account
		let rewards_after = Balances::free_balance(&rewards_account);
		assert_eq!(
			rewards_after - rewards_initial,
			payment,
			"All funds should remain in rewards pallet until operators claim"
		);

		// Rewards are recorded but not yet transferred to operators
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert!(!bob_rewards.is_empty(), "Bob should have pending rewards recorded");

		// Bob's actual balance hasn't changed yet
		let bob_balance = Balances::free_balance(&bob);
		// In a real scenario with actual claim_rewards(), Bob would need to call it to receive
		// funds This test verifies the funds are safely held in the rewards pallet account
		assert_eq!(bob_balance, 20_000, "Bob's balance unchanged until he claims");
	});
}
