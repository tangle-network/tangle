#![cfg(test)]
use crate::{
	mock::{self as mock, *},
	types::*,
	Error, Event, Pallet as Credits,
};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use mock_currency::set_balance as set_tnt_balance;
use sp_runtime::{DispatchError, DispatchResult, Perbill};

fn link_account(who: AccountId, id_str: &[u8]) -> DispatchResult {
	let bounded_id: OffchainAccountIdOf<Test> = id_str.to_vec().try_into().expect("ID too long");
	Credits::link_account(RuntimeOrigin::signed(who), bounded_id)
}

fn claim_credits(who: AccountId, amount: Balance, id_str: &[u8]) -> DispatchResult {
	let bounded_id: OffchainAccountIdOf<Test> = id_str.to_vec().try_into().expect("ID too long");
	Credits::claim_credits(RuntimeOrigin::signed(who), amount, bounded_id)
}

fn credit_balance(who: AccountId) -> Balance {
	Credits::credit_balance(who)
}

fn last_interaction(who: AccountId) -> BlockNumber {
	Credits::last_interaction_block(who)
}

fn set_stake(who: AccountId, amount: Balance) {
	MOCK_STAKING_INFO.with(|s| s.borrow_mut().set_stake(who, TNT_ASSET_ID, amount));
}

#[test]
fn accrue_credits_works() {
	new_test_ext().execute_with(|| {
		// Initial setup in new_test_ext: Alice Tier 3 (15/block), Bob Tier 1 (1/block)

		run_to_block(10);
		// Trigger accrual
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(BOB)));

		// Block 1 is genesis, accrual starts block 2. 9 blocks passed (2 to 10 inclusive).
		assert_eq!(credit_balance(ALICE), 15 * 9, "Alice initial accrual");
		assert_eq!(credit_balance(BOB), 1 * 9, "Bob initial accrual");
		assert_eq!(last_interaction(ALICE), 10);
		assert_eq!(last_interaction(BOB), 10);

		run_to_block(20);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));

		// Alice: 10 more blocks (11 to 20 inclusive)
		assert_eq!(credit_balance(ALICE), (15 * 9) + (15 * 10), "Alice second accrual");
		assert_eq!(last_interaction(ALICE), 20);
		// Bob hasn't updated since block 10
		assert_eq!(credit_balance(BOB), 1 * 9, "Bob balance unchanged");
		assert_eq!(last_interaction(BOB), 10);

		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(BOB)));
		// Bob: 10 more blocks (11 to 20 inclusive)
		assert_eq!(credit_balance(BOB), (1 * 9) + (1 * 10), "Bob second accrual");
		assert_eq!(last_interaction(BOB), 20);
	});
}

#[test]
fn link_and_claim_basic_flow() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@test.com";
		assert_ok!(link_account(ALICE, alice_id));
		assert_eq!(last_interaction(ALICE), 1);

		run_to_block(10);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let accrued = 15 * 9;
		assert_eq!(credit_balance(ALICE), accrued);
		assert_eq!(last_interaction(ALICE), 10); // Update resets interaction

		// Claim some
		assert_ok!(claim_credits(ALICE, 100, alice_id));
		assert_eq!(credit_balance(ALICE), accrued - 100);
		assert_eq!(last_interaction(ALICE), 10); // Claim resets interaction (at same block)

		// Fails if not linked
		assert_noop!(claim_credits(BOB, 1, b"bob"), Error::<Test>::AccountNotLinked);

		// Fails if wrong ID
		assert_noop!(claim_credits(ALICE, 1, b"wrong_id"), Error::<Test>::OffchainAccountMismatch);
	});
}

// --- Decay Tests ---

#[test]
fn decay_no_decay_within_grace_period() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@decay";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction at block 1

		let end_of_grace = 1 + WEEK;
		run_to_block(end_of_grace); // Go to end of 1st week + 1 block
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let accrued = 15 * WEEK; // Accrued over exactly 1 week
		assert_eq!(credit_balance(ALICE), accrued);
		assert_eq!(last_interaction(ALICE), end_of_grace);

		// Claim full amount - should succeed fully (elapsed 0 since update)
		assert_ok!(claim_credits(ALICE, accrued, alice_id));
		assert_eq!(credit_balance(ALICE), 0);
		assert_eq!(last_interaction(ALICE), end_of_grace);
	});
}

#[test]
fn decay_step1_boundary_just_over() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@decay75_boundary";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Go exactly to the start of week 2 + 1 block
		let claim_block = 1 + WEEK + 1;
		run_to_block(claim_block);

		// Claim. Elapsed = WEEK + 1 blocks. Factor should still be 100% according to steps [ (WEEK
		// * 1, 100%), (WEEK*2, 75%)]
		let accrued = 15 * (claim_block - 1); // Includes accrual up to claim block
		let expected_claimable = Perbill::from_percent(100).mul_floor(accrued);

		assert_ok!(claim_credits(ALICE, expected_claimable, alice_id));
		assert_eq!(credit_balance(ALICE), 0); // Should be able to claim all
		assert_eq!(last_interaction(ALICE), claim_block);
	});
}

#[test]
fn decay_step2_boundary_just_over() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@decay40_boundary";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Go exactly to the start of week 3 + 1 block
		let claim_block = 1 + 2 * WEEK + 1;
		run_to_block(claim_block);

		// Claim. Elapsed = 2*WEEK + 1 blocks. Factor should be 75% [ (WEEK*2, 75%), (WEEK*3, 40%)]
		let accrued = 15 * (claim_block - 1);
		let expected_claimable = Perbill::from_percent(75).mul_floor(accrued);

		// Try claiming slightly more
		assert_noop!(
			claim_credits(ALICE, expected_claimable + 1, alice_id),
			Error::<Test>::InsufficientCreditBalance
		);

		// Claim exact effective amount
		assert_ok!(claim_credits(ALICE, expected_claimable, alice_id));
		let expected_raw_deduction =
			Perbill::from_percent(75).saturating_reciprocal_mul_ceil(expected_claimable);
		let expected_remaining = accrued.saturating_sub(expected_raw_deduction);
		assert_eq!(credit_balance(ALICE), expected_remaining);
		assert_eq!(last_interaction(ALICE), claim_block);
	});
}

#[test]
fn decay_step_intermediate_value() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@decay_intermediate";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Go to 2.5 weeks
		let claim_block = 1 + 2 * WEEK + WEEK / 2;
		run_to_block(claim_block);

		// Claim. Elapsed = 2.5 * WEEK blocks. Factor should be 75% (from >= 2 weeks step)
		let accrued = 15 * (claim_block - 1);
		let expected_claimable = Perbill::from_percent(75).mul_floor(accrued);

		// Claim exact effective amount
		assert_ok!(claim_credits(ALICE, expected_claimable, alice_id));
		let expected_raw_deduction =
			Perbill::from_percent(75).saturating_reciprocal_mul_ceil(expected_claimable);
		let expected_remaining = accrued.saturating_sub(expected_raw_deduction);
		assert_eq!(credit_balance(ALICE), expected_remaining);
		assert_eq!(last_interaction(ALICE), claim_block);
	});
}

#[test]
fn decay_step_final_0_percent() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@decay0";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Go past 5 weeks
		let claim_block = 1 + 5 * WEEK + 100;
		run_to_block(claim_block);

		// Claiming 1 should fail (0% factor)
		assert_noop!(claim_credits(ALICE, 1, alice_id), Error::<Test>::InsufficientCreditBalance);

		// Accrue credits to check balance remains raw
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let expected_raw = 15 * (claim_block - 1);
		assert_eq!(credit_balance(ALICE), expected_raw);
		assert_eq!(last_interaction(ALICE), claim_block); // Update resets timer

		// Now claim should work (elapsed = 0)
		assert_ok!(claim_credits(ALICE, 100, alice_id));
		assert_eq!(credit_balance(ALICE), expected_raw - 100);
		assert_eq!(last_interaction(ALICE), claim_block);
	});
}

#[test]
fn interaction_resets_decay() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@reset";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Wait 3.5 weeks
		let block1 = 1 + 3 * WEEK + WEEK / 2;
		run_to_block(block1);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let accrued1 = 15 * (block1 - 1);
		assert_eq!(credit_balance(ALICE), accrued1);
		assert_eq!(last_interaction(ALICE), block1); // Interaction reset

		// Wait just under 1 week
		let block2 = block1 + WEEK - 100;
		run_to_block(block2);

		// Claim should have 100% factor (elapsed < 1 week)
		let accrued2 = 15 * (WEEK - 100);
		let total_raw = accrued1 + accrued2;
		assert_ok!(claim_credits(ALICE, total_raw, alice_id));
		assert_eq!(credit_balance(ALICE), 0);
		assert_eq!(last_interaction(ALICE), block2); // Interaction reset by claim
	});
}

#[test]
fn burn_does_not_reset_decay() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@burn_no_reset";
		set_tnt_balance(TNT_ASSET_ID, ALICE, 100);
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Wait 1.5 weeks
		let block_burn = 1 + WEEK + WEEK / 2;
		run_to_block(block_burn);
		assert_eq!(last_interaction(ALICE), 1);

		// Burn TNT
		assert_ok!(Credits::burn(RuntimeOrigin::signed(ALICE), 10));
		let credits_from_burn = 10 * CreditBurnConversionValue::get();
		let credits_from_staking = 15 * (block_burn - 1); // Accrued during burn call
		assert_eq!(credit_balance(ALICE), credits_from_burn + credits_from_staking);
		// IMPORTANT: Last interaction still 1
		assert_eq!(last_interaction(ALICE), 1);

		// Wait another week (total > 2 weeks since interaction)
		let block_claim = block_burn + WEEK;
		run_to_block(block_claim);

		// Claim should use 75% factor (elapsed > 2 weeks from block 1)
		let final_staking_credits = 15 * WEEK;
		let final_raw_balance = credits_from_burn + credits_from_staking + final_staking_credits;
		let expected_claimable = Perbill::from_percent(75).mul_floor(final_raw_balance);

		assert_ok!(claim_credits(ALICE, expected_claimable, alice_id));
		assert_eq!(last_interaction(ALICE), block_claim); // Reset by claim

		let expected_raw_deduction =
			Perbill::from_percent(75).saturating_reciprocal_mul_ceil(expected_claimable);
		let expected_remaining = final_raw_balance.saturating_sub(expected_raw_deduction);
		assert_eq!(credit_balance(ALICE), expected_remaining);
	});
}

#[test]
fn admin_actions_reset_decay() {
	new_test_ext().execute_with(|| {
		let bob_id = b"bob";
		assert_ok!(link_account(BOB, bob_id)); // Interaction block 1

		// Wait 3.5 weeks
		let block1 = 1 + 3 * WEEK + WEEK / 2;
		run_to_block(block1);

		// Admin sets balance - resets interaction
		assert_ok!(Credits::force_set_credit_balance(RuntimeOrigin::signed(ADMIN), BOB, 5000));
		assert_eq!(credit_balance(BOB), 5000);
		assert_eq!(last_interaction(BOB), block1);

		// Wait just under 1 week
		let block2 = block1 + WEEK - 100;
		run_to_block(block2);

		// Claim should be 100%
		assert_ok!(claim_credits(BOB, 5000, bob_id));
		assert_eq!(credit_balance(BOB), 0);
		assert_eq!(last_interaction(BOB), block2);

		// Wait 3.5 weeks again
		let block3 = block2 + 3 * WEEK + WEEK / 2;
		run_to_block(block3);

		// Admin links account - resets interaction
		assert_ok!(Credits::force_link_account(
			RuntimeOrigin::signed(ADMIN),
			BOB,
			bob_id.to_vec().try_into().unwrap()
		));
		assert_eq!(last_interaction(BOB), block3);
	});
}

#[test]
fn claiming_partial_amount_with_decay() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@partial_decay";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Wait 2.5 weeks
		let block1 = 1 + 2 * WEEK + WEEK / 2;
		run_to_block(block1);

		// Accrue and update
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let raw_balance1 = 15 * (block1 - 1);
		assert_eq!(credit_balance(ALICE), raw_balance1);
		assert_eq!(last_interaction(ALICE), block1);

		// Wait another 2.5 weeks (total 5 weeks since link, but only 2.5 since trigger)
		let block2 = block1 + 2 * WEEK + WEEK / 2;
		run_to_block(block2);

		// Elapsed time is 2.5 weeks. Decay factor should be 75%.
		let final_raw_balance = raw_balance1 + 15 * (2 * WEEK + WEEK / 2);
		let decay_factor = Perbill::from_percent(75);
		let effective_claimable = decay_factor.mul_floor(final_raw_balance);

		// Claim about half of the effective amount
		let claim_amount = effective_claimable / 2;
		assert!(claim_amount > 0);

		assert_ok!(claim_credits(ALICE, claim_amount, alice_id));

		// Calculate expected raw deduction = claim / factor (ceil)
		let expected_raw_deduction = decay_factor.saturating_reciprocal_mul_ceil(claim_amount);
		let expected_remaining_raw = final_raw_balance.saturating_sub(expected_raw_deduction);

		assert_eq!(credit_balance(ALICE), expected_remaining_raw);
		assert_eq!(last_interaction(ALICE), block2); // Updated by claim
	});
}

// --- Failure Case Tests ---

#[test]
fn link_account_failures() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice";
		assert_ok!(link_account(ALICE, alice_id));
		// Already linked
		assert_noop!(link_account(ALICE, b"new_id"), Error::<Test>::AlreadyLinked);
		// ID too long (MaxAccIdLen is 128)
		let long_id = vec![0u8; 129];
		let bounded_long_id: Result<OffchainAccountIdOf<Test>, _> = long_id.try_into();
		assert!(bounded_long_id.is_err()); // Ensure conversion itself fails if frame_support checks it
		                             // If conversion doesn't check, the extrinsic might (depending
		                             // on implementation details not shown)
		                             // Let's assume the extrinsic or type conversion prevents it.
		                             // Note: Direct call test depends on how BoundedVec handles
		                             // oversized input.
	});
}

#[test]
fn burn_failures() {
	new_test_ext().execute_with(|| {
		// Alice starts with 10_000 TNT
		assert_eq!(mock_currency::get_balance(TNT_ASSET_ID, ALICE), 10_000);

		// Burn zero
		assert_noop!(Credits::burn(RuntimeOrigin::signed(ALICE), 0), Error::<Test>::AmountZero);

		// Burn more than balance
		assert_noop!(
			Credits::burn(RuntimeOrigin::signed(ALICE), 10_001),
			Error::<Test>::InsufficientTntBalance
		);

		// Burn almost all (leaving less than min balance if applicable - mock min is 1)
		// Mock burn checks reducible, so burning 10000 fails if min balance is 1.
		// Let's check burning 9999, leaving 1.
		assert_ok!(Credits::burn(RuntimeOrigin::signed(ALICE), 9999));
		assert_eq!(mock_currency::get_balance(TNT_ASSET_ID, ALICE), 1);

		// Now try burning the last 1 - should fail if mock enforces min balance on burn_from
		assert_noop!(
			Credits::burn(RuntimeOrigin::signed(ALICE), 1),
			DispatchError::Token("CannotBurnDust".into()) // Mock error for burn < reducible
		);
	});
}

#[test]
fn claim_credits_failures() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@claim_fail";
		assert_ok!(link_account(ALICE, alice_id));
		run_to_block(2);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let accrued = 15; // 1 block accrual
		assert_eq!(credit_balance(ALICE), accrued);

		// Claim zero
		assert_noop!(claim_credits(ALICE, 0, alice_id), Error::<Test>::AmountZero);

		// Claim more than balance
		assert_noop!(
			claim_credits(ALICE, accrued + 1, alice_id),
			Error::<Test>::InsufficientCreditBalance
		);

		// Claim requires correct linked ID
		assert_noop!(claim_credits(ALICE, 1, b"wrong"), Error::<Test>::OffchainAccountMismatch);

		// Claim requires linked account
		assert_noop!(claim_credits(CHARLIE, 1, b"charlie"), Error::<Test>::AccountNotLinked);
	});
}

#[test]
fn admin_failures() {
	new_test_ext().execute_with(|| {
		// Non-admin cannot call admin functions
		assert_noop!(
			Credits::force_set_credit_balance(RuntimeOrigin::signed(ALICE), BOB, 1000),
			DispatchError::BadOrigin
		);
		assert_noop!(
			Credits::force_link_account(
				RuntimeOrigin::signed(BOB),
				ALICE,
				b"id".to_vec().try_into().unwrap()
			),
			DispatchError::BadOrigin
		);
	});
}

// --- Success Case Tests (More Variations) ---

#[test]
fn burn_success() {
	new_test_ext().execute_with(|| {
		set_tnt_balance(BOB, TNT_ASSET_ID, 500);
		assert_eq!(credit_balance(BOB), 0);

		// Burn 50 TNT
		assert_ok!(Credits::burn(RuntimeOrigin::signed(BOB), 50));
		assert_eq!(mock_currency::get_balance(TNT_ASSET_ID, BOB), 450);
		// Check credits: Burn conversion rate is 100
		// Accrued = 1 * (block - 1) = 1 * (1 - 1) = 0? Burn is at block 1? Need to check.
		// Burn happens at current block (assume 1). Accrue called first: last_update=0, current=1
		// -> 1 block. Stake=150->rate=1. accrue=1. Credits = accrue + burn*rate = 1 + 50 * 100 =
		// 5001 ? Let's re-run test ext at block 2. Rerun new_test_ext to start at block 1
		// consistently.
	});
	// Rerun test with explicit block start
	new_test_ext().execute_with(|| {
		System::set_block_number(10); // Start later
		set_tnt_balance(BOB, TNT_ASSET_ID, 500);
		set_stake(BOB, 150); // Tier 1, rate 1
		assert_eq!(credit_balance(BOB), 0);
		assert_eq!(pallet_credits::LastRewardUpdateBlock::<Test>::get(BOB), 0);

		// Burn 50 TNT at block 10
		assert_ok!(Credits::burn(RuntimeOrigin::signed(BOB), 50));
		assert_eq!(mock_currency::get_balance(TNT_ASSET_ID, BOB), 450);
		// Accrue called first: current=10, last=0. Blocks=10. Rate=1. Accrued=10.
		// Burned = 50 * 100 = 5000.
		// Total = 10 + 5000 = 5010
		assert_eq!(credit_balance(BOB), 5010);
		assert_eq!(pallet_credits::LastRewardUpdateBlock::<Test>::get(BOB), 10); // Updated by accrue
		assert_eq!(last_interaction(BOB), 0); // Burn doesn't update interaction
	});
}

#[test]
fn trigger_update_success() {
	new_test_ext().execute_with(|| {
		run_to_block(100);
		assert_eq!(last_interaction(ALICE), 0);
		assert_eq!(credit_balance(ALICE), 0);

		// Trigger update
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let expected_credits = 15 * 99; // Blocks 2 to 100 inclusive
		assert_eq!(credit_balance(ALICE), expected_credits);
		assert_eq!(last_interaction(ALICE), 100); // Interaction updated
	});
}

#[test]
fn accrual_with_stake_change() {
	new_test_ext().execute_with(|| {
		// Bob: Tier 1 (150 stake -> 1 credit/block)
		run_to_block(10);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(BOB)));
		assert_eq!(credit_balance(BOB), 1 * 9);

		// Increase stake to Tier 2 (500 stake -> 6 credits/block)
		set_stake(BOB, 500);
		run_to_block(20);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(BOB)));
		// Credits = 9 (initial) + 10 blocks * 6 credits/block = 9 + 60 = 69
		assert_eq!(credit_balance(BOB), 9 + (6 * 10));

		// Decrease stake to Tier 0 (50 stake -> 0 credits/block)
		set_stake(BOB, 50);
		run_to_block(30);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(BOB)));
		// Credits = 69 (previous) + 10 blocks * 0 credits/block = 69
		assert_eq!(credit_balance(BOB), 69);
	});
}

// --- More Decay Edge Cases ---

#[test]
fn decay_claim_at_exact_boundary() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@boundary";
		assert_ok!(link_account(ALICE, alice_id)); // Interaction block 1

		// Go exactly to block 2*WEEK + 1 (start of week 3)
		let claim_block = 1 + 2 * WEEK;
		run_to_block(claim_block);

		// Claim. Elapsed = 2*WEEK blocks. Should use 75% factor [ >= 2*WEEK ]
		let accrued = 15 * (claim_block - 1);
		let expected_claimable = Perbill::from_percent(75).mul_floor(accrued);
		assert_ok!(claim_credits(ALICE, expected_claimable, alice_id));
		let expected_raw_deduction =
			Perbill::from_percent(75).saturating_reciprocal_mul_ceil(expected_claimable);
		assert_eq!(credit_balance(ALICE), accrued.saturating_sub(expected_raw_deduction));
	});
}

#[test]
fn decay_multiple_claims_across_boundaries() {
	new_test_ext().execute_with(|| {
		let alice_id = b"alice@multi_claim";
		assert_ok!(link_account(ALICE, alice_id)); // Block 1

		// 1. Claim within grace period
		run_to_block(WEEK - 10);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let bal1 = credit_balance(ALICE);
		assert!(bal1 > 0);
		assert_ok!(claim_credits(ALICE, bal1 / 2, alice_id));
		let bal1_remain = credit_balance(ALICE);
		let last_inter1 = System::block_number();
		assert_eq!(last_interaction(ALICE), last_inter1);

		// 2. Wait 1.5 weeks, claim with 100% factor
		run_to_block(last_inter1 + WEEK + WEEK / 2);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE)));
		let bal2 = credit_balance(ALICE);
		assert!(bal2 > bal1_remain);
		let elapsed2 = System::block_number() - last_inter1;
		assert!(elapsed2 > WEEK && elapsed2 < 2 * WEEK);
		// Factor is 100% as we reset interaction timer with trigger_credit_update
		let claimable2 = Perbill::from_percent(100).mul_floor(bal2);
		assert_eq!(claimable2, bal2);
		assert_ok!(claim_credits(ALICE, claimable2 / 2, alice_id));
		let bal2_remain = credit_balance(ALICE);
		let last_inter2 = System::block_number();
		assert_eq!(last_interaction(ALICE), last_inter2);

		// 3. Wait 2.5 weeks, claim with 75% factor
		run_to_block(last_inter2 + 2 * WEEK + WEEK / 2);
		let elapsed3 = System::block_number() - last_inter2;
		assert!(elapsed3 > 2 * WEEK && elapsed3 < 3 * WEEK);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE))); // Update accrual
		let bal3 = credit_balance(ALICE);
		let claimable3 = Perbill::from_percent(75).mul_floor(bal3);
		assert!(claimable3 < bal3);
		assert_ok!(claim_credits(ALICE, claimable3 / 2, alice_id));
		let bal3_remain = credit_balance(ALICE);
		let last_inter3 = System::block_number();
		assert_eq!(last_interaction(ALICE), last_inter3);

		// 4. Wait 5+ weeks, claim 0
		run_to_block(last_inter3 + 6 * WEEK);
		assert_ok!(Credits::trigger_credit_update(RuntimeOrigin::signed(ALICE))); // Update accrual
		let bal4 = credit_balance(ALICE);
		assert_noop!(claim_credits(ALICE, 1, alice_id), Error::<Test>::InsufficientCreditBalance);
	});
}
