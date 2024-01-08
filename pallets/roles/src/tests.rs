// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use frame_support::{assert_err, assert_ok, BoundedVec};
use mock::*;
use profile::{IndependentRestakeProfile, Record, SharedRestakeProfile};
use sp_std::{default::Default, vec};
use tangle_primitives::{
	jobs::ReportValidatorOffence,
	roles::{ThresholdSignatureRoleType, ZeroKnowledgeRoleType},
};

pub fn independent_profile() -> Profile<Runtime> {
	let profile = IndependentRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record { role: RoleType::Tss(ThresholdSignatureRoleType::TssGG20), amount: Some(2500) },
			Record {
				role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
				amount: Some(2500),
			},
		])
		.unwrap(),
	};
	Profile::Independent(profile)
}

pub fn shared_profile() -> Profile<Runtime> {
	let profile = SharedRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record { role: RoleType::Tss(ThresholdSignatureRoleType::TssGG20), amount: None },
			Record { role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16), amount: None },
		])
		.unwrap(),
		amount: 5000,
	};
	Profile::Shared(profile)
}

// Test create independent profile.
#[test]
fn test_create_independent_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = independent_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::ProfileCreated {
			account: mock_pub_key(1),
			total_profile_restake: profile.get_total_profile_restake().into(),
			roles: profile.get_roles(),
		})]);
		// Get the ledger to check if the profile is created.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert_eq!(ledger.profile, profile);
		assert!(ledger.profile.is_independent());
	});
}

// Test create shared profile.
#[test]
fn test_create_shared_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::ProfileCreated {
			account: mock_pub_key(1),
			total_profile_restake: profile.get_total_profile_restake().into(),
			roles: profile.get_roles(),
		})]);

		// Get the ledger to check if the profile is created.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert_eq!(ledger.profile, profile);
		assert!(ledger.profile.is_shared());
	});
}

// Test create profile should fail if user is not a validator.
#[test]
fn test_create_profile_should_fail_if_user_is_not_a_validator() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(5)), profile.clone()),
			Error::<Runtime>::NotValidator
		);
	});
}

// Test create profile should fail if user already has a profile.
#[test]
fn test_create_profile_should_fail_if_user_already_has_a_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));
		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()),
			Error::<Runtime>::ProfileAlreadyExists
		);
	});
}

// Test create profile should fail if min required restake condition is not met.
// Min restake required is 2500.
#[test]
fn test_create_profile_should_fail_if_min_required_restake_condition_is_not_met() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		pallet::MinRestakingBond::<Runtime>::put(2500);

		let profile = Profile::Shared(SharedRestakeProfile {
			records: BoundedVec::try_from(vec![
				Record { role: RoleType::Tss(ThresholdSignatureRoleType::TssGG20), amount: None },
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: None,
				},
			])
			.unwrap(),
			amount: 1000,
		});

		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()),
			Error::<Runtime>::InsufficientRestakingBond
		);
	});
}

// Test create profile should fail if min required restake condition is not met.
// In case of independent profile, each role should meet the min required restake condition.
// Min restake required is 2500.
#[test]
fn test_create_profile_should_fail_if_min_required_restake_condition_is_not_met_for_independent_profile(
) {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		pallet::MinRestakingBond::<Runtime>::put(2500);

		let profile = Profile::Independent(IndependentRestakeProfile {
			records: BoundedVec::try_from(vec![
				Record {
					role: RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
					amount: Some(1000),
				},
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: Some(1000),
				},
			])
			.unwrap(),
		});

		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()),
			Error::<Runtime>::InsufficientRestakingBond
		);
	});
}

// Update profile from independent to shared.
#[test]
fn test_update_profile_from_independent_to_shared() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create independent profile.
		let profile = independent_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// Get the ledger to check if the profile is created.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert!(ledger.profile.is_independent());
		assert_eq!(ledger.total_restake(), 5000);

		let updated_profile = shared_profile();

		assert_ok!(Roles::update_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			updated_profile.clone()
		));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::ProfileUpdated {
			account: mock_pub_key(1),
			total_profile_restake: profile.get_total_profile_restake().into(),
			roles: profile.get_roles(),
		})]);
		// Get updated ledger and check if the profile is updated.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert_eq!(ledger.profile, updated_profile);
		assert!(ledger.profile.is_shared());
	});
}

// Update profile from shared to independent.
#[test]
fn test_update_profile_from_shared_to_independent() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create shared profile.
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// Get the ledger to check if the profile is created.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert!(ledger.profile.is_shared());
		assert_eq!(ledger.total_restake(), 5000);

		let updated_profile = independent_profile();
		assert_ok!(Roles::update_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			updated_profile.clone()
		));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::ProfileUpdated {
			account: mock_pub_key(1),
			total_profile_restake: profile.get_total_profile_restake().into(),
			roles: profile.get_roles(),
		})]);
		// Get updated ledger and check if the profile is updated.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert_eq!(ledger.profile, updated_profile);
		assert!(ledger.profile.is_independent());
		assert_eq!(ledger.total_restake(), 5000);
	});
}

// Test delete profile.
#[test]
fn test_delete_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create shared profile.
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// Get the ledger to check if the profile is created.
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert!(ledger.profile.is_shared());
		assert_eq!(ledger.total_restake(), 5000);

		assert_ok!(Roles::delete_profile(RuntimeOrigin::signed(mock_pub_key(1))));

		assert_events(vec![RuntimeEvent::Roles(crate::Event::ProfileDeleted {
			account: mock_pub_key(1),
		})]);
		assert_eq!(Roles::ledger(mock_pub_key(1)), None);
	});
}

#[test]
fn test_unbound_funds_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create shared profile.
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// Lets delete profile by opting out of all services.
		assert_ok!(Roles::delete_profile(RuntimeOrigin::signed(mock_pub_key(1))));

		assert_eq!(Roles::ledger(mock_pub_key(1)), None);

		// unbound funds.
		assert_ok!(Roles::unbound_funds(RuntimeOrigin::signed(mock_pub_key(1)), 5000));

		assert_events(vec![RuntimeEvent::Staking(pallet_staking::Event::Unbonded {
			stash: mock_pub_key(1),
			amount: 5000,
		})]);

		// Get  pallet staking ledger mapping.
		let staking_ledger = pallet_staking::Ledger::<Runtime>::get(mock_pub_key(1)).unwrap();
		// Since we we have unbounded 5000 tokens, we should have 5000 tokens in staking ledger.
		assert_eq!(staking_ledger.active, 5000);
	});
}

#[test]
fn test_reward_dist_works_as_expected_with_one_validator() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		assert_eq!(Balances::free_balance(mock_pub_key(1)), 20_000);

		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// The reward is 100, we have 5 authorities
		assert_ok!(Roles::distribute_rewards());

		// ensure the distribution is correct
		assert_eq!(Balances::free_balance(mock_pub_key(1)), 20_000 + 10_000);
	});
}

#[test]
fn test_reward_dist_works_as_expected_with_multiple_validator() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let _reward_amount = 10_000;
		assert_eq!(Balances::free_balance(mock_pub_key(1)), 20_000);

		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(2)), profile.clone()));

		// The reward is 100, we have 5 authorities
		assert_ok!(Roles::distribute_rewards());

		// ensure the distribution is correct
		assert_eq!(Balances::free_balance(mock_pub_key(1)), 20_000 + 5000);
		assert_eq!(Balances::free_balance(mock_pub_key(2)), 20_000 + 5000);
	});
}

// Test report offence should work.
#[test]
fn test_report_offence_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone()));

		// Get current session index.
		let session_index = pallet_session::CurrentIndex::<Runtime>::get();

		// Create offence report.
		let offence_report = ReportValidatorOffence {
			session_index,
			validator_set_count: 4,
			role_type: RoleType::Tss(Default::default()),
			offence_type: tangle_primitives::jobs::ValidatorOffenceType::Inactivity,
			offenders: vec![mock_pub_key(1)],
		};
		// Lets report offence.
		assert_ok!(Roles::report_offence(offence_report));
		// Should slash 700 tokens
		let ledger = Roles::ledger(mock_pub_key(1)).unwrap();
		assert_eq!(ledger.total, 4300);
	});
}
