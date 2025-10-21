// Test file for reward distribution logic
// Verifies that service payments are correctly distributed to operators, developers, and protocol

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::assert_ok;
use sp_runtime::{Perbill, Percent};
use tangle_primitives::services::{Asset, AssetSecurityCommitment, PricingModel, Service};

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
fn test_service_payment_distributes_to_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);

		// Create service with 3 operators with different exposure levels
		// Bob: 50% TNT + 50% WETH = 100 total percentage points
		// Charlie: 30% TNT + 30% WETH = 60 total percentage points
		// Dave: 20% TNT + 20% WETH = 40 total percentage points
		let service = create_test_service_with_operators(0, 0, alice.clone(), vec![
			(bob.clone(), vec![
				AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				},
				AssetSecurityCommitment {
					asset: Asset::Custom(WETH),
					exposure_percent: Percent::from_percent(50),
				},
			]),
			(charlie.clone(), vec![
				AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(30),
				},
				AssetSecurityCommitment {
					asset: Asset::Custom(WETH),
					exposure_percent: Percent::from_percent(30),
				},
			]),
			(dave.clone(), vec![
				AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(20),
				},
				AssetSecurityCommitment {
					asset: Asset::Custom(WETH),
					exposure_percent: Percent::from_percent(20),
				},
			]),
		]);

		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		// Call distribute_service_payment directly
		assert_ok!(Services::distribute_service_payment(&service, &alice, payment, &pricing_model));

		// Verify distribution:
		// Total exposure: 100 + 60 + 40 = 200 percentage points
		// Operator share: 85% of 10,000 = 8,500 tokens
		// Bob should get: (100/200) * 8,500 = 4,250 tokens
		// Charlie should get: (60/200) * 8,500 = 2,550 tokens
		// Dave should get: (40/200) * 8,500 = 1,700 tokens
		// Developer (Alice) should get: 10% of 10,000 = 1,000 tokens

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 4_250, "Bob should receive 4,250 tokens (50% exposure)");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 2_550, "Charlie should receive 2,550 tokens (30% exposure)");

		let dave_rewards = MockRewardsManager::get_pending_rewards(&dave);
		let dave_total: u128 = dave_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(dave_total, 1_700, "Dave should receive 1,700 tokens (20% exposure)");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 1_000, "Alice (developer) should receive 1,000 tokens (10%)");

		// Verify total distribution (95% = 85% operators + 10% developer)
		let total_distributed = bob_total + charlie_total + dave_total + alice_total;
		let expected_distributed = Perbill::from_percent(95) * payment;
		assert_eq!(
			total_distributed, expected_distributed,
			"Total distributed should be 95% of payment"
		);
	});
}

#[test]
fn test_single_operator_gets_full_share() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Single operator with 60% exposure
		let service =
			create_test_service_with_operators(0, 0, alice.clone(), vec![(bob.clone(), vec![
				AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(60),
				},
			])]);

		let payment: Balance = 5_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::distribute_service_payment(&service, &alice, payment, &pricing_model));

		// Bob should get full operator share: 85% of 5,000 = 4,250 tokens
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 4_250, "Single operator should receive full operator share (85%)");

		// Developer still gets 10%
		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 500, "Developer should receive 500 tokens (10%)");
	});
}

#[test]
fn test_zero_payment_handling() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		let service =
			create_test_service_with_operators(0, 0, alice.clone(), vec![(bob.clone(), vec![
				AssetSecurityCommitment {
					asset: Asset::Custom(TNT),
					exposure_percent: Percent::from_percent(50),
				},
			])]);

		let payment: Balance = 0;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		// Zero payment should succeed but not create rewards
		assert_ok!(Services::distribute_service_payment(&service, &alice, payment, &pricing_model));

		// No rewards should be recorded
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(bob_rewards.len(), 0, "Zero payment should not create rewards");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		assert_eq!(alice_rewards.len(), 0, "Zero payment should not create rewards");
	});
}

#[test]
fn test_unequal_exposure_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Bob commits 40% exposure, Charlie commits 10% exposure
		// Total exposure: 50 percentage points
		// Operator share: 85% * 10,000 = 8,500
		// Bob: (40/50) * 8,500 = 6,800
		// Charlie: (10/50) * 8,500 = 1,700
		let service = create_test_service_with_operators(0, 0, alice.clone(), vec![
			(bob.clone(), vec![AssetSecurityCommitment {
				asset: Asset::Custom(TNT),
				exposure_percent: Percent::from_percent(40),
			}]),
			(charlie.clone(), vec![AssetSecurityCommitment {
				asset: Asset::Custom(TNT),
				exposure_percent: Percent::from_percent(10),
			}]),
		]);

		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::distribute_service_payment(&service, &alice, payment, &pricing_model));

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 6_800, "Bob should receive 6,800 tokens (80% of operator share)");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(
			charlie_total, 1_700,
			"Charlie should receive 1,700 tokens (20% of operator share)"
		);

		// Verify Bob gets 4x Charlie's reward (40% vs 10%)
		assert_eq!(bob_total, charlie_total * 4, "Bob should get 4x Charlie's reward");
	});
}

#[test]
fn test_no_operators_fails() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);

		// Service with no operators
		let service = create_test_service_with_operators(0, 0, alice.clone(), vec![]);

		let payment: Balance = 1_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		// Should fail with NoOperatorsAvailable
		assert!(
			Services::distribute_service_payment(&service, &alice, payment, &pricing_model)
				.is_err()
		);
	});
}

#[test]
fn test_zero_exposure_operator_gets_nothing() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Bob has 50% exposure, Charlie has 0% exposure (shouldn't happen but test anyway)
		let service = create_test_service_with_operators(0, 0, alice.clone(), vec![
			(bob.clone(), vec![AssetSecurityCommitment {
				asset: Asset::Custom(TNT),
				exposure_percent: Percent::from_percent(50),
			}]),
			(charlie.clone(), vec![AssetSecurityCommitment {
				asset: Asset::Custom(TNT),
				exposure_percent: Percent::from_percent(0),
			}]),
		]);

		let payment: Balance = 10_000;
		let pricing_model = PricingModel::PayOnce { amount: payment };

		assert_ok!(Services::distribute_service_payment(&service, &alice, payment, &pricing_model));

		// Bob should get full operator share
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 8_500, "Bob should receive full operator share");

		// Charlie should get nothing
		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 0, "Charlie with 0% exposure should get nothing");
	});
}
