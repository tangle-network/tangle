// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.
#![cfg(test)]
use super::*;
use frame_support::{assert_err, assert_ok};
use mock::*;

#[test]
fn test_assign_roles() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens

		// Roles user is interested in re-staking.
		let role_records = vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }];

		assert_ok!(Roles::assign_roles(RuntimeOrigin::signed(1), role_records));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::RoleAssigned {
			account: 1,
			role: RoleType::Tss,
		})]);

		// Lets verify role assigned to account.
		assert_eq!(Roles::has_role(1, RoleType::Tss), true);
		// Verify ledger mapping
		assert_eq!(
			Roles::ledger(1),
			Some(RoleStakingLedger {
				stash: 1,
				total: 5000,
				roles: vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }]
			})
		);
	});
}

// test assign multiple roles to an account.
#[test]
fn test_assign_multiple_roles() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens

		// Roles user is interested in re-staking.
		let role_records = vec![
			RoleStakingRecord { role: RoleType::Tss, re_staked: 2500 },
			RoleStakingRecord { role: RoleType::ZkSaas, re_staked: 2500 },
		];

		assert_ok!(Roles::assign_roles(RuntimeOrigin::signed(1), role_records));

		// Lets verify role assigned to account.
		assert_eq!(Roles::has_role(1, RoleType::Tss), true);

		// Lets verify role assigned to account.
		assert_eq!(Roles::has_role(1, RoleType::ZkSaas), true);

		assert_eq!(
			Roles::ledger(1),
			Some(RoleStakingLedger {
				stash: 1,
				total: 5000,
				roles: vec![
					RoleStakingRecord { role: RoleType::Tss, re_staked: 2500 },
					RoleStakingRecord { role: RoleType::ZkSaas, re_staked: 2500 },
				]
			})
		);
	});
}

// Test assign roles, should fail if total re-stake value exceeds max re-stake value.
// Max re-stake value is 5000 (50% of total staked value).
#[test]
fn test_assign_roles_should_fail_if_total_re_stake_value_exceeds_max_re_stake_value() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens

		// Roles user is interested in re-staking.
		let role_records = vec![
			RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 },
			RoleStakingRecord { role: RoleType::ZkSaas, re_staked: 5000 },
		];
		// Since max re_stake limit is 5000 it should fail with `ExceedsMaxReStakeValue` error.
		assert_err!(
			Roles::assign_roles(RuntimeOrigin::signed(1), role_records),
			Error::<Runtime>::ExceedsMaxReStakeValue
		);
	});
}

#[test]
fn test_clear_role() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens

		// Roles user is interested in re-staking.
		let role_records = vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }];

		assert_ok!(Roles::assign_roles(RuntimeOrigin::signed(1), role_records));

		// Now lets clear the role
		assert_ok!(Roles::clear_role(RuntimeOrigin::signed(1), RoleType::Tss));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::RoleRemoved {
			account: 1,
			role: RoleType::Tss,
		})]);

		// Role should be removed from  account role mappings.
		assert_eq!(Roles::has_role(1, RoleType::Tss), false);

		// Ledger should be removed from ledger mappings.
		assert_eq!(Roles::ledger(1), None);
	});
}

#[test]
fn test_assign_roles_should_fail_if_not_validator() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// we will use account 5 which is not a validator

		// Roles user is interested in re-staking.
		let role_records = vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }];

		assert_err!(
			Roles::assign_roles(RuntimeOrigin::signed(5), role_records),
			Error::<Runtime>::NotValidator
		);
	});
}

#[test]
fn test_unbound_funds_should_work() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially validator account has staked 10_000 tokens and wants to re-stake 5000 tokens
		// for providing TSS services.

		// Roles user is interested in re-staking.
		let role_records = vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }];

		assert_ok!(Roles::assign_roles(RuntimeOrigin::signed(1), role_records));

		// Lets verify role is assigned to account.
		assert_eq!(Roles::has_role(1, RoleType::Tss), true);

		// Lets clear the role.
		assert_ok!(Roles::clear_role(RuntimeOrigin::signed(1), RoleType::Tss));

		// Role should be removed from  account role mappings.
		assert_eq!(Roles::has_role(1, RoleType::Tss), false);

		// unbound funds.
		assert_ok!(Roles::unbound_funds(RuntimeOrigin::signed(1), 5000));

		assert_events(vec![RuntimeEvent::Staking(pallet_staking::Event::Unbonded {
			stash: 1,
			amount: 5000,
		})]);

		// Get  pallet staking ledger mapping.
		let staking_ledger = pallet_staking::Ledger::<Runtime>::get(1).unwrap();
		// Since we we have unbounded 5000 tokens, we should have 5000 tokens in staking ledger.
		assert_eq!(staking_ledger.active, 5000);
	});
}

// Test unbound should fail if role is assigned to account.
#[test]
fn test_unbound_funds_should_fail_if_role_assigned() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially validator account has staked 10_000 tokens and wants to re-stake 5000 tokens
		// for providing TSS services.

		// Roles user is interested in re-staking.
		let role_records = vec![RoleStakingRecord { role: RoleType::Tss, re_staked: 5000 }];

		assert_ok!(Roles::assign_roles(RuntimeOrigin::signed(1), role_records));

		// Lets verify role is assigned to account.
		assert_eq!(Roles::has_role(1, RoleType::Tss), true);

		// Lets try to unbound funds.
		assert_err!(
			Roles::unbound_funds(RuntimeOrigin::signed(1), 5000),
			Error::<Runtime>::HasRoleAssigned
		);
	});
}

// Test unbound should work if no role assigned to account.
#[test]
fn test_unbound_funds_should_work_if_no_role_assigned() {
	new_test_ext_raw_authorities(vec![1, 2, 3, 4]).execute_with(|| {
		// Initially validator account has staked 10_000 tokens.

		// Since validator has not opted for any roles, he should be able to unbound his funds.
		assert_ok!(Roles::unbound_funds(RuntimeOrigin::signed(1), 5000));

		assert_events(vec![RuntimeEvent::Staking(pallet_staking::Event::Unbonded {
			stash: 1,
			amount: 5000,
		})]);
	});
}
