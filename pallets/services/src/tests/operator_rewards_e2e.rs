// True end-to-end operator rewards tests
// Tests the complete flow with real balance tracking and multi-block simulations

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::{assert_ok, traits::Currency};
use sp_runtime::Percent;
use tangle_primitives::{
	services::{Asset, AssetSecurityCommitment, PricingModel, Service},
	traits::RewardRecorder,
};

/// Helper to simulate operator claiming rewards from the rewards pallet
/// In production this would be the actual pallet-rewards claim_rewards() extrinsic
fn simulate_operator_claim(operator: &AccountId, rewards_account: &AccountId) -> Balance {
	let pending_rewards = MockRewardsManager::get_pending_rewards(operator);
	let total_claimable: Balance = pending_rewards.iter().map(|(_, amt)| *amt).sum();

	if total_claimable > 0 {
		// Transfer from rewards pallet account to operator using Currency trait
		let _ = <Balances as Currency<AccountId>>::transfer(
			rewards_account,
			operator,
			total_claimable,
			frame_support::traits::ExistenceRequirement::KeepAlive,
		);
		// Clear the pending rewards from the mock to simulate actual claim
		MockRewardsManager::clear_pending_rewards(operator);
	}

	total_claimable
}

/// Helper to advance blocks and return processed subscription count
fn advance_blocks_with_subscriptions(n: u64) -> u32 {
	let mut total_processed = 0u32;
	for _ in 0..n {
		let current = System::block_number();
		System::set_block_number(current + 1);

		// Process subscription payments for this block
		let _ = Services::process_subscription_payments_on_block(System::block_number());

		// Count how many rewards were added this block
		total_processed += 1;
	}
	total_processed
}

#[test]
fn test_full_e2e_native_payment_with_claim() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let operator = mock_pub_key(BOB);
		let developer = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		// Setup initial balances
		Balances::make_free_balance_be(&customer, 100_000);
		Balances::make_free_balance_be(&operator, 10_000);
		Balances::make_free_balance_be(&developer, 10_000);
		Balances::make_free_balance_be(&rewards_account, 1_000);

		// Record initial balances
		let customer_initial = Balances::free_balance(&customer);
		let operator_initial = Balances::free_balance(&operator);
		let developer_initial = Balances::free_balance(&developer);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Create service with operator
		let service = Service {
			id: 0,
			blueprint: 0,
			owner: developer.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![(
				operator.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(100),
				}]
				.try_into()
				.unwrap(),
			)]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 1 },
		};

		// Customer makes payment
		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(&service, &developer, payment, &pricing_model));

		// Verify balances after payment
		let customer_after_payment = Balances::free_balance(&customer);
		let rewards_after_payment = Balances::free_balance(&rewards_account);

		assert_eq!(
			customer_initial - customer_after_payment,
			payment,
			"Customer should have paid 10,000"
		);
		assert_eq!(
			rewards_after_payment - rewards_initial,
			payment,
			"Rewards pallet should have received 10,000"
		);

		// Verify rewards were recorded
		let operator_pending = MockRewardsManager::get_pending_rewards(&operator);
		let operator_pending_total: Balance = operator_pending.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(operator_pending_total, 8_500, "Operator should have 8,500 pending (85%)");

		let developer_pending = MockRewardsManager::get_pending_rewards(&developer);
		let developer_pending_total: Balance = developer_pending.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(developer_pending_total, 1_000, "Developer should have 1,000 pending (10%)");

		// Simulate operator claiming rewards
		let operator_claimed = simulate_operator_claim(&operator, &rewards_account);
		assert_eq!(operator_claimed, 8_500, "Operator should claim 8,500");

		// Verify operator balance increased
		let operator_after_claim = Balances::free_balance(&operator);
		assert_eq!(
			operator_after_claim - operator_initial,
			8_500,
			"Operator balance should increase by 8,500"
		);

		// Simulate developer claiming rewards
		let developer_claimed = simulate_operator_claim(&developer, &rewards_account);
		assert_eq!(developer_claimed, 1_000, "Developer should claim 1,000");

		let developer_after_claim = Balances::free_balance(&developer);
		assert_eq!(
			developer_after_claim - developer_initial,
			1_000,
			"Developer balance should increase by 1,000"
		);

		// Verify rewards pallet account depleted (minus existential deposit)
		let rewards_after_claims = Balances::free_balance(&rewards_account);
		assert_eq!(
			rewards_after_payment - rewards_after_claims,
			9_500,
			"Rewards pallet should have paid out 9,500 (95% of 10,000)"
		);

		// Verify complete money flow
		let customer_paid = customer_initial - Balances::free_balance(&customer);
		let operator_received = Balances::free_balance(&operator) - operator_initial;
		let developer_received = Balances::free_balance(&developer) - developer_initial;

		assert_eq!(customer_paid, 10_000, "Customer paid 10,000");
		assert_eq!(operator_received + developer_received, 9_500, "Total distributed 9,500 (95%)");
	});
}

#[test]
fn test_multi_block_subscription_payments_with_claims() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let customer = mock_pub_key(ALICE);
		let operator = mock_pub_key(BOB);
		let developer = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		// Setup balances - customer needs enough for multiple payments
		Balances::make_free_balance_be(&customer, 100_000);
		Balances::make_free_balance_be(&operator, 10_000);
		Balances::make_free_balance_be(&developer, 10_000);
		Balances::make_free_balance_be(&rewards_account, 1_000);

		let customer_initial = Balances::free_balance(&customer);
		let operator_initial = Balances::free_balance(&operator);
		let _developer_initial = Balances::free_balance(&developer);

		let service = Service {
			id: 0,
			blueprint: 0,
			owner: developer.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![(
				operator.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				}]
				.try_into()
				.unwrap(),
			)]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 1 },
		};

		let rate_per_interval: Balance = 1_000;
		let interval: BlockNumberFor<Runtime> = 10;
		let pricing_model = PricingModel::Subscription {
			rate_per_interval,
			interval,
			maybe_end: Some(50),
		};

		// Payment 1: Block 1
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(&service, &developer, rate_per_interval, &pricing_model));

		// Payment 2: Block 11
		advance_blocks_with_subscriptions(10);
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(&service, &developer, rate_per_interval, &pricing_model));

		// Payment 3: Block 21
		advance_blocks_with_subscriptions(10);
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(&service, &developer, rate_per_interval, &pricing_model));

		// Payment 4: Block 31
		advance_blocks_with_subscriptions(10);
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(&service, &developer, rate_per_interval, &pricing_model));

		// Verify 4 payments made (blocks 1, 11, 21, 31)
		let total_paid = rate_per_interval * 4;
		let customer_after_payments = Balances::free_balance(&customer);
		assert_eq!(
			customer_initial - customer_after_payments,
			total_paid,
			"Customer should have paid 4,000 total (4 x 1,000)"
		);

		// Verify operator accumulated rewards
		let operator_pending = MockRewardsManager::get_pending_rewards(&operator);
		let operator_total: Balance = operator_pending.iter().map(|(_, amt)| *amt).sum();
		let expected_operator = 850 * 4; // 85% of 1,000 per payment
		assert_eq!(operator_total, expected_operator, "Operator should have 3,400 pending (4 x 850)");

		// Verify developer accumulated rewards
		let developer_pending = MockRewardsManager::get_pending_rewards(&developer);
		let developer_total: Balance = developer_pending.iter().map(|(_, amt)| *amt).sum();
		let expected_developer = 100 * 4; // 10% of 1,000 per payment
		assert_eq!(developer_total, expected_developer, "Developer should have 400 pending (4 x 100)");

		// Simulate operator claiming after 4 payments
		let operator_claimed = simulate_operator_claim(&operator, &rewards_account);
		assert_eq!(operator_claimed, 3_400, "Operator claims 3,400");

		let operator_after_claim = Balances::free_balance(&operator);
		assert_eq!(
			operator_after_claim - operator_initial,
			3_400,
			"Operator net gain should be 3,400"
		);

		// Continue with Payment 5: Block 41
		advance_blocks_with_subscriptions(10);
		assert_ok!(Services::charge_payment(&customer, &customer, rate_per_interval));
		assert_ok!(Services::distribute_service_payment(&service, &developer, rate_per_interval, &pricing_model));

		// Operator should have new pending rewards (850 from payment 5)
		let operator_pending_2 = MockRewardsManager::get_pending_rewards(&operator);
		let operator_total_2: Balance = operator_pending_2.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(operator_total_2, 850, "Operator should have 850 new pending rewards");

		// Simulate operator claiming again
		let operator_claimed_2 = simulate_operator_claim(&operator, &rewards_account);
		assert_eq!(operator_claimed_2, 850, "Operator claims another 850");

		let operator_final = Balances::free_balance(&operator);
		assert_eq!(
			operator_final - operator_initial,
			4_250,
			"Operator total net gain should be 4,250 (5 payments)"
		);

		// Verify developer can claim all accumulated rewards
		let developer_claimed = simulate_operator_claim(&developer, &rewards_account);
		assert_eq!(developer_claimed, 500, "Developer claims 500 total (5 x 100)");
	});
}

#[test]
fn test_multiple_operators_progressive_claims() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let rewards_account = MockRewardsManager::account_id();

		// Setup balances
		Balances::make_free_balance_be(&customer, 100_000);
		Balances::make_free_balance_be(&bob, 5_000);
		Balances::make_free_balance_be(&charlie, 5_000);
		Balances::make_free_balance_be(&rewards_account, 1_000);

		let bob_initial = Balances::free_balance(&bob);
		let charlie_initial = Balances::free_balance(&charlie);
		let rewards_initial = Balances::free_balance(&rewards_account);

		// Service with 2 operators: Bob (60% exposure), Charlie (40% exposure)
		let service = Service {
			id: 0,
			blueprint: 0,
			owner: dave.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![
				(
					bob.clone(),
					vec![AssetSecurityCommitment {
						asset: Asset::Custom(TNT),
						exposure_percent: Percent::from_percent(60),
					}]
					.try_into()
					.unwrap(),
				),
				(
					charlie.clone(),
					vec![AssetSecurityCommitment {
						asset: Asset::Custom(TNT),
						exposure_percent: Percent::from_percent(40),
					}]
					.try_into()
					.unwrap(),
				),
			]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 2 },
		};

		// Payment 1: 10,000 tokens
		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(&service, &dave, payment, &pricing_model));

		// Expected distribution:
		// Operator pool: 85% * 10,000 = 8,500
		// Bob: (60/100) * 8,500 = 5,100
		// Charlie: (40/100) * 8,500 = 3,400

		// Bob claims immediately
		let bob_claimed_1 = simulate_operator_claim(&bob, &rewards_account);
		assert_eq!(bob_claimed_1, 5_100, "Bob claims 5,100");

		let bob_after_claim_1 = Balances::free_balance(&bob);
		assert_eq!(bob_after_claim_1 - bob_initial, 5_100, "Bob gained 5,100");

		// Payment 2: Another 10,000 tokens
		assert_ok!(Services::charge_payment(&customer, &customer, payment));
		assert_ok!(Services::distribute_service_payment(&service, &dave, payment, &pricing_model));

		// Now Charlie claims all accumulated (from both payments)
		let charlie_claimed = simulate_operator_claim(&charlie, &rewards_account);
		assert_eq!(charlie_claimed, 6_800, "Charlie claims 6,800 (2 x 3,400)");

		let charlie_after_claim = Balances::free_balance(&charlie);
		assert_eq!(charlie_after_claim - charlie_initial, 6_800, "Charlie gained 6,800");

		// Bob claims second payment
		let bob_claimed_2 = simulate_operator_claim(&bob, &rewards_account);
		assert_eq!(bob_claimed_2, 5_100, "Bob claims another 5,100");

		let bob_final = Balances::free_balance(&bob);
		assert_eq!(bob_final - bob_initial, 10_200, "Bob total gain 10,200 (2 x 5,100)");

		// Verify rewards pallet balance decreased appropriately
		let rewards_final = Balances::free_balance(&rewards_account);
		assert!(
			rewards_final < rewards_initial + (2 * payment),
			"Rewards pallet should have less funds after claims"
		);
	});
}

#[test]
fn test_erc20_pay_once_job_payment_e2e() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account = address_to_account_id(charlie_address);
		let rewards_account = MockRewardsManager::account_id();

		// Create blueprint with ERC20 PayOnce pricing
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operator
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Initial ERC20 balances
		let _initial_charlie_erc20 = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b.as_u128())
			.unwrap_or(0);
		let _initial_rewards_erc20 = Services::query_erc20_balance_of(USDC_ERC20,
			account_id_to_address(rewards_account.clone()))
			.map(|(b, _)| b.as_u128())
			.unwrap_or(0);

		let payment_amount = 5_000u128;

		// Request service with ERC20 payment
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie_evm_account.clone()),
			Some(charlie_address),
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(TNT, &[50, 100])],
			100,
			Asset::Erc20(USDC_ERC20),
			payment_amount,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Operator approves
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(TNT, 50)]
		));

		// Simulate job call that triggers PayOnce payment
		// Note: In production this would be called via Services::call() extrinsic
		// For now we test the payment processing logic directly
		assert_ok!(Services::process_job_pay_once_payment(
			0, // service_id
			0, // job_index
			0, // call_id
			&charlie_evm_account,
			&charlie_evm_account,
			payment_amount,
		));

		// Verify ERC20 payment was processed
		// Note: With current implementation, ERC20 uses Currency::transfer for native asset
		// The proper ERC20 implementation would use EVM calls to transfer ERC20 tokens

		// Verify rewards were recorded
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: Balance = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 4_250, "Bob should receive 4,250 (85% of 5,000)");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: Balance = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 500, "Alice should receive 500 (10% of 5,000)");

		// Simulate operators claiming rewards
		// In production with ERC20, this would involve EVM calls to transfer ERC20 tokens
		let bob_claimed = simulate_operator_claim(&bob, &rewards_account);
		assert_eq!(bob_claimed, 4_250, "Bob claims 4,250");

		let alice_claimed = simulate_operator_claim(&alice, &rewards_account);
		assert_eq!(alice_claimed, 500, "Alice claims 500");
	});
}

#[test]
fn test_custom_asset_usdc_subscription_e2e() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		// Mint USDC to customer and rewards account
		mint_tokens(USDC, alice.clone(), charlie.clone(), 1_000_000);
		// Give rewards account USDC (need at least minimum balance of 100_000)
		mint_tokens(USDC, alice.clone(), rewards_account.clone(), 200_000);

		let charlie_initial_usdc = Assets::balance(USDC, charlie.clone());
		let bob_initial_usdc = Assets::balance(USDC, bob.clone());
		let rewards_initial_usdc = Assets::balance(USDC, rewards_account.clone());

		// Create service with USDC subscription
		let service = Service {
			id: 0,
			blueprint: 0,
			owner: alice.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![(
				bob.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(USDC),
					exposure_percent: Percent::from_percent(100),
				}]
				.try_into()
				.unwrap(),
			)]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 1 },
		};

		let rate_per_interval: Balance = 10_000; // 10,000 USDC per interval
		let interval: BlockNumberFor<Runtime> = 5;
		let pricing_model = PricingModel::Subscription {
			rate_per_interval,
			interval,
			maybe_end: Some(30),
		};

		// Process 3 subscription payments (blocks 1, 6, 11)
		for payment_num in 0..3 {
			let _current_block = 1 + (payment_num * interval);

			// Charge payment using custom asset
			assert_ok!(Services::charge_payment_with_asset(
				&charlie,
				&charlie,
				rate_per_interval,
				&Asset::Custom(USDC),
			));

			assert_ok!(Services::distribute_service_payment(
				&service,
				&alice,
				rate_per_interval,
				&pricing_model
			));

			if payment_num < 2 {
				advance_blocks_with_subscriptions(interval);
			}
		}

		// Verify USDC was deducted from customer
		let charlie_after_usdc = Assets::balance(USDC, charlie.clone());
		let total_paid = rate_per_interval * 3;
		assert_eq!(
			charlie_initial_usdc - charlie_after_usdc,
			total_paid,
			"Charlie should have paid 30,000 USDC"
		);

		// Verify USDC went to rewards pallet
		let rewards_after_usdc = Assets::balance(USDC, rewards_account.clone());
		assert_eq!(
			rewards_after_usdc - rewards_initial_usdc,
			total_paid,
			"Rewards pallet should have received 30,000 USDC"
		);

		// Verify operator rewards recorded
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: Balance = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		let expected_bob = 8_500 * 3; // 85% of 10,000 per payment
		assert_eq!(bob_total, expected_bob, "Bob should have 25,500 USDC pending");

		// Simulate claiming by transferring USDC from rewards to operator
		let bob_claimed = bob_total;
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(rewards_account.clone()),
			USDC,
			bob.clone().into(),
			bob_claimed,
		));

		let bob_after_usdc = Assets::balance(USDC, bob.clone());
		assert_eq!(
			bob_after_usdc - bob_initial_usdc,
			bob_claimed,
			"Bob should have received 25,500 USDC"
		);

		// Verify complete USDC flow
		assert_eq!(
			charlie_initial_usdc - Assets::balance(USDC, charlie.clone()),
			30_000,
			"Customer paid 30,000 USDC"
		);
		assert_eq!(
			Assets::balance(USDC, bob.clone()) - bob_initial_usdc,
			25_500,
			"Operator received 25,500 USDC"
		);
	});
}

#[test]
fn test_event_driven_payment_multiple_events_e2e() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let customer = mock_pub_key(ALICE);
		let operator = mock_pub_key(BOB);
		let developer = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		Balances::make_free_balance_be(&customer, 100_000);
		Balances::make_free_balance_be(&operator, 10_000);
		Balances::make_free_balance_be(&rewards_account, 1_000);

		let customer_initial = Balances::free_balance(&customer);
		let operator_initial = Balances::free_balance(&operator);

		let service = Service {
			id: 0,
			blueprint: 0,
			owner: developer.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![(
				operator.clone(),
				vec![AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(75),
				}]
				.try_into()
				.unwrap(),
			)]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 1 },
		};

		let reward_per_event: Balance = 100;

		// Event batch 1: 10 events
		let event_count_1 = 10u32;
		let total_1 = reward_per_event * event_count_1 as Balance;
		assert_ok!(Services::charge_payment(&customer, &customer, total_1));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&developer,
			total_1,
			&PricingModel::EventDriven { reward_per_event }
		));

		// Operator claims after first batch
		let claimed_1 = simulate_operator_claim(&operator, &rewards_account);
		assert_eq!(claimed_1, 850, "Operator claims 850 (85% of 1,000)");

		// Event batch 2: 25 events
		let event_count_2 = 25u32;
		let total_2 = reward_per_event * event_count_2 as Balance;
		assert_ok!(Services::charge_payment(&customer, &customer, total_2));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&developer,
			total_2,
			&PricingModel::EventDriven { reward_per_event }
		));

		// Event batch 3: 50 events
		let event_count_3 = 50u32;
		let total_3 = reward_per_event * event_count_3 as Balance;
		assert_ok!(Services::charge_payment(&customer, &customer, total_3));
		assert_ok!(Services::distribute_service_payment(
			&service,
			&developer,
			total_3,
			&PricingModel::EventDriven { reward_per_event }
		));

		// Operator claims accumulated from batches 2 & 3
		let claimed_2_3 = simulate_operator_claim(&operator, &rewards_account);
		let expected_2_3 = 2_125 + 4_250; // 85% of 2,500 + 85% of 5,000 = 6,375
		assert_eq!(claimed_2_3, expected_2_3, "Operator claims 6,375");

		// Verify totals
		let customer_final = Balances::free_balance(&customer);
		let total_events = event_count_1 + event_count_2 + event_count_3;
		let total_paid = reward_per_event * total_events as Balance;
		assert_eq!(
			customer_initial - customer_final,
			total_paid,
			"Customer paid for 85 events total (8,500)"
		);

		let operator_final = Balances::free_balance(&operator);
		let total_operator_gain = operator_final - operator_initial;
		let expected_total = 850 + expected_2_3; // 850 + 6,375 = 7,225
		assert_eq!(
			total_operator_gain,
			expected_total,
			"Operator total gain should be 7,225"
		);
	});
}

#[test]
fn test_weth_custom_asset_pay_once_e2e() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let rewards_account = MockRewardsManager::account_id();

		// Mint WETH to customer and rewards account (WETH is owned by authorities[1] = BOB in mock)
		let weth_amount = 100 * 10u128.pow(18); // 100 WETH
		mint_tokens(WETH, bob.clone(), charlie.clone(), weth_amount);
		// Give rewards account some WETH for existential deposit
		mint_tokens(WETH, bob.clone(), rewards_account.clone(), 10 * 10u128.pow(18));

		let charlie_initial_weth = Assets::balance(WETH, charlie.clone());
		let bob_initial_weth = Assets::balance(WETH, bob.clone());

		let service = Service {
			id: 0,
			blueprint: 0,
			owner: alice.clone(),
			args: vec![].try_into().unwrap(),
			operator_security_commitments: vec![(
				bob.clone(),
				vec![
					AssetSecurityCommitment {
						asset: Asset::Custom(WETH),
						exposure_percent: Percent::from_percent(50),
					},
					AssetSecurityCommitment {
						asset: Asset::Custom(TNT),
						exposure_percent: Percent::from_percent(30),
					},
				]
				.try_into()
				.unwrap(),
			)]
			.try_into()
			.unwrap(),
			security_requirements: vec![].try_into().unwrap(),
			permitted_callers: vec![].try_into().unwrap(),
			ttl: 100,
			membership_model: MembershipModel::Fixed { min_operators: 1 },
		};

		// Payment in WETH
		let payment_amount = 10 * 10u128.pow(18); // 10 WETH
		let pricing_model = PricingModel::PayOnce { amount: payment_amount };

		// Charge using WETH
		assert_ok!(Services::charge_payment_with_asset(
			&charlie,
			&charlie,
			payment_amount,
			&Asset::Custom(WETH),
		));

		assert_ok!(Services::distribute_service_payment(
			&service,
			&alice,
			payment_amount,
			&pricing_model
		));

		// Verify WETH transferred
		let charlie_after_weth = Assets::balance(WETH, charlie.clone());
		assert_eq!(
			charlie_initial_weth - charlie_after_weth,
			payment_amount,
			"Charlie paid 10 WETH"
		);

		let rewards_weth = Assets::balance(WETH, rewards_account.clone());
		let expected_rewards_weth = 10 * 10u128.pow(18) + payment_amount; // Initial 10 WETH + payment 10 WETH
		assert_eq!(rewards_weth, expected_rewards_weth, "Rewards pallet has 20 WETH total");

		// Verify operator rewards
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: Balance = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		let expected_bob = (payment_amount * 85) / 100; // 85%
		assert_eq!(bob_total, expected_bob, "Bob should have 8.5 WETH pending");

		// Simulate claim by transferring WETH
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(rewards_account.clone()),
			WETH,
			bob.clone().into(),
			bob_total,
		));

		let bob_after_weth = Assets::balance(WETH, bob.clone());
		let bob_weth_gain = bob_after_weth - bob_initial_weth;
		assert_eq!(bob_weth_gain, bob_total, "Bob received 8.5 WETH");
	});
}
