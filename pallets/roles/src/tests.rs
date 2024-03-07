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
use pallet_staking::{CurrentEra, ErasTotalStake};
use profile::{IndependentRestakeProfile, Record, SharedRestakeProfile};
use sp_std::{default::Default, vec};
use tangle_primitives::{
	jobs::ReportRestakerOffence,
	roles::{ThresholdSignatureRoleType, ZeroKnowledgeRoleType},
};

pub fn independent_profile() -> Profile<Runtime> {
	let profile = IndependentRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record {
				role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
				amount: Some(2500),
			},
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
			Record {
				role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
				amount: None,
			},
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
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

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
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

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
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(5)), profile.clone(), None),
			Error::<Runtime>::NotValidator
		);
	});
}

// Test create profile should fail if user already has a profile.
#[test]
fn test_create_profile_should_fail_if_user_already_has_a_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));
		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone(), None),
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
				Record {
					role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
					amount: None,
				},
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: None,
				},
			])
			.unwrap(),
			amount: 1000,
		});

		assert_err!(
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone(), None),
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
					role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
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
			Roles::create_profile(RuntimeOrigin::signed(mock_pub_key(1)), profile.clone(), None),
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
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

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
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

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

#[test]
fn test_update_profile_should_fail_if_user_is_not_a_validator() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_err!(
			Roles::update_profile(RuntimeOrigin::signed(mock_pub_key(5)), profile.clone()),
			Error::<Runtime>::NotValidator
		);
	});
}

// Test delete profile.
#[test]
fn test_delete_profile() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create shared profile.
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

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
fn test_unbond_funds_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// Lets create shared profile.
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

		// Lets delete profile by opting out of all services.
		assert_ok!(Roles::delete_profile(RuntimeOrigin::signed(mock_pub_key(1))));

		assert_eq!(Roles::ledger(mock_pub_key(1)), None);

		// unbond funds.
		assert_ok!(Roles::unbond_funds(RuntimeOrigin::signed(mock_pub_key(1)), 5000));

		assert_events(vec![RuntimeEvent::Staking(pallet_staking::Event::Unbonded {
			stash: mock_pub_key(1),
			amount: 5000,
		})]);

		// Get  pallet staking ledger mapping.
		let staking_ledger = pallet_staking::Ledger::<Runtime>::get(mock_pub_key(1)).unwrap();
		// Since we we have unbonded 5000 tokens, we should have 5000 tokens in staking ledger.
		assert_eq!(staking_ledger.active, 5000);
	});
}

#[test]
fn test_reward_dist_works_as_expected_with_multiple_validator() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let _total_inflation_reward = 10_000;
		CurrentEra::<Runtime>::put(1);
		ErasTotalStake::<Runtime>::insert(1, 40_000);

		assert_eq!(Balances::free_balance(mock_pub_key(1)), 20_000);
		assert_eq!(Balances::free_balance(mock_pub_key(2)), 20_000);

		// lets give both validators equal rewards for jobs participation
		let mut validator_rewards: BoundedBTreeMap<_, _, _> = Default::default();
		validator_rewards.try_insert(mock_pub_key(1), 100_u32).unwrap();
		validator_rewards.try_insert(mock_pub_key(2), 100_u32).unwrap();
		ValidatorJobsInEra::<Runtime>::put(validator_rewards);

		let profile = shared_profile();
		for validator in vec![1, 2, 3, 4] {
			assert_ok!(Roles::create_profile(
				RuntimeOrigin::signed(mock_pub_key(validator)),
				profile.clone(),
				None
			));
		}

		// The reward is 1000, we have 5 authorities
		assert_ok!(Roles::compute_rewards(1));
		assert!(ValidatorJobsInEra::<Runtime>::get().is_empty());

		// Rewards math
		// Total rewards : 10_000
		// Active validators : 1&2, will receive 50% each
		// All validators : will receive 50%, everyone receives equally

		// 1 & 2 receives, 5000/2 + 5000/4 + 100 (job reward)
		let reward_points = ErasRestakeRewardPoints::<Runtime>::get(1);
		assert_eq!(
			*reward_points.individual.get(&mock_pub_key(1)).unwrap(),
			2500_u32 + 1250_u32 + 100_u32
		);
		assert_eq!(
			*reward_points.individual.get(&mock_pub_key(2)).unwrap(),
			2500_u32 + 1250_u32 + 100_u32
		);

		// 3 & 4 receives only 5000/4
		assert_eq!(*reward_points.individual.get(&mock_pub_key(3)).unwrap(), 1250_u32);
		assert_eq!(*reward_points.individual.get(&mock_pub_key(4)).unwrap(), 1250_u32);
	});
}

#[test]
fn test_reward_dist_takes_restake_into_account() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let _total_inflation_reward = 10_000;
		CurrentEra::<Runtime>::put(1);
		ErasTotalStake::<Runtime>::insert(1, 24000);

		let profile = SharedRestakeProfile {
			records: BoundedVec::try_from(vec![
				Record {
					role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
					amount: None,
				},
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: None,
				},
			])
			.unwrap(),
			amount: 1000,
		};

		for validator in vec![1, 2] {
			assert_ok!(Roles::create_profile(
				RuntimeOrigin::signed(mock_pub_key(validator)),
				Profile::Shared(profile.clone()),
				None
			));
		}

		let profile = SharedRestakeProfile {
			records: BoundedVec::try_from(vec![
				Record {
					role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
					amount: None,
				},
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: None,
				},
			])
			.unwrap(),
			amount: 5000,
		};
		for validator in vec![3, 4] {
			assert_ok!(Roles::create_profile(
				RuntimeOrigin::signed(mock_pub_key(validator)),
				Profile::Shared(profile.clone()),
				None
			));
		}

		// The reward is 1000, we have 5 authorities
		assert_ok!(Roles::compute_rewards(1));
		assert!(ValidatorJobsInEra::<Runtime>::get().is_empty());

		// Rewards math
		// Total rewards : 10_000
		// Inflation rewards for re-staking : 50% = 5000
		// Stake of 1 = 1000
		// Stake of 2 = 1000
		// Stake of 3 = 5000
		// Stake of 4 = 5000
		// Total stake = 12000

		let reward_points = ErasRestakeRewardPoints::<Runtime>::get(1);
		// rewards should be 1/12, since validator 1 bonded 1000/12000
		assert_eq!(*reward_points.individual.get(&mock_pub_key(1)).unwrap(), 5000 / 12);
		// rewards should be 1/12, since validator 2 bonded 1000/12000
		assert_eq!(*reward_points.individual.get(&mock_pub_key(2)).unwrap(), 5000 / 12);

		// rewards should be 5/12, since validator 3 bonded 5000/12000
		assert_eq!(*reward_points.individual.get(&mock_pub_key(3)).unwrap(), (5000 * 5) / 12);

		// rewards should be 5/12, since validator4 bonded 5000/12000
		assert_eq!(*reward_points.individual.get(&mock_pub_key(4)).unwrap(), (5000 * 5) / 12);
	});
}

#[test]
fn test_reward_dist_handles_less_than_ideal_restake() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let _total_inflation_reward = 10_000;
		CurrentEra::<Runtime>::put(1);
		ErasTotalStake::<Runtime>::insert(1, 100_000);

		let profile = SharedRestakeProfile {
			records: BoundedVec::try_from(vec![
				Record {
					role: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
					amount: None,
				},
				Record {
					role: RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
					amount: None,
				},
			])
			.unwrap(),
			amount: 1000,
		};

		for validator in vec![1] {
			assert_ok!(Roles::create_profile(
				RuntimeOrigin::signed(mock_pub_key(validator)),
				Profile::Shared(profile.clone()),
				None
			));
		}

		// The reward is 10_000, we have 1 authority
		assert_ok!(Roles::compute_rewards(1));
		assert!(ValidatorJobsInEra::<Runtime>::get().is_empty());

		// Rewards math
		// Total rewards : 10_000
		// Inflation rewards for re-staking : 50% = 5000
		// Stake of 1 = 1000
		// Total stake = 1000

		let reward_points = ErasRestakeRewardPoints::<Runtime>::get(1);
		// our ideal re-stake rate is 50%
		// but we only have 1%
		// so reward of 5000 will be reduced to 1% of 5000
		assert_eq!(*reward_points.individual.get(&mock_pub_key(1)).unwrap(), 50);
	});
}

// Test report offence should work.
#[test]
fn test_report_offence_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let profile = shared_profile();
		assert_ok!(Roles::create_profile(
			RuntimeOrigin::signed(mock_pub_key(1)),
			profile.clone(),
			None
		));

		// Get current session index.
		let session_index = pallet_session::CurrentIndex::<Runtime>::get();

		// Create offence report.
		let offence_report = ReportRestakerOffence {
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

#[test]
fn test_set_min_validator_stake() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// should fail with non force origin
		frame_support::assert_noop!(
			Roles::set_min_restaking_bond(RuntimeOrigin::signed(mock_pub_key(1)), 100u32.into()),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_ok!(Roles::set_min_restaking_bond(RuntimeOrigin::root(), 100u32.into()));

		assert_eq!(MinRestakingBond::<Runtime>::get(), 100u32.into());
	})
}
