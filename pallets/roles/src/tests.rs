// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
use frame_support::assert_ok;
use mock::*;
use tangle_primitives::jobs::ValidatorOffence;

#[test]
fn test_assign_role() {
	new_test_ext().execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens
		assert_ok!(Roles::assign_role(RuntimeOrigin::signed(10), 5000, RoleType::Tss));

		assert_events(vec![
			RuntimeEvent::Roles(crate::Event::Bonded { account: 10, amount: 5000 }),
			RuntimeEvent::Roles(crate::Event::RoleAssigned { account: 10, role: RoleType::Tss }),
		]);

		// Lets verify role assigned to account.
		assert_eq!(Roles::account_role(10), Some(RoleType::Tss));
		// Verify ledger mapping
		assert_eq!(Roles::ledger(10), Some(RoleStakingLedger { stash: 10, total_locked: 5000 }));
		// Verify total usable balance of the account. Since we have bonded 5000 tokens, we should
		// have 5000 tokens usable.
		assert_eq!(Balances::usable_balance(10), 5000);
	});
}

#[test]
fn test_clear_role() {
	new_test_ext().execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens
		assert_ok!(Roles::assign_role(RuntimeOrigin::signed(10), 5000, RoleType::Tss));
		// Verify total usable balance of the account. Since we have bonded 5000 tokens, we should
		// have 5000 tokens usable.
		assert_eq!(Balances::usable_balance(10), 5000);

		// Now lets clear the role
		assert_ok!(Roles::clear_role(RuntimeOrigin::signed(10), RoleType::Tss));

		assert_events(vec![
			RuntimeEvent::Roles(crate::Event::Unbonded { account: 10, amount: 5000 }),
			RuntimeEvent::Roles(crate::Event::RoleRemoved { account: 10, role: RoleType::Tss }),
		]);

		// Role should be removed from  account role mappings.
		assert_eq!(Roles::account_role(10), None);

		// Ledger should be removed from ledger mappings.
		assert_eq!(Roles::ledger(10), None);

		// Verify total usable balance of the account. Since we have cleared the role, we should
		// have 10000 tokens usable.
		assert_eq!(Balances::usable_balance(10), 10000);
	});
}

// test slashing
#[test]
fn test_slash_validator() {
	new_test_ext().execute_with(|| {
		// Initially account if funded with 10000 tokens and we are trying to bond 5000 tokens
		assert_ok!(Roles::assign_role(RuntimeOrigin::signed(10), 5000, RoleType::Tss));
		// Verify total usable balance of the account. Since we have bonded 5000 tokens, we should
		// have 5000 tokens usable.
		assert_eq!(Balances::usable_balance(10), 5000);

		// Now lets slash the account for being Inactive.
		assert_ok!(Roles::slash_validator(10, ValidatorOffence::Inactivity));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::Slashed {
			account: 10,
			amount: 1000,
		})]);
		// should be updated in ledger
		assert_eq!(Roles::ledger(10), Some(RoleStakingLedger { stash: 10, total_locked: 4000 }));
	});
}
