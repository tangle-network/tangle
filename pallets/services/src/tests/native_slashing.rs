// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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

use super::*;
use frame_support::{assert_err, assert_ok};
use sp_runtime::Percent;
use sp_staking::StakingAccount;

#[test]
fn test_basic_native_restaking_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, blueprint_id, security_commitments, .. } = deploy();

		// Setup native restaking
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 10_000;

		// Delegate via native restaking
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount / 2, // Delegate half the stake
			vec![blueprint_id].into(),
		));

		// Verify initial state
		let staking_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert_eq!(staking_ledger.active, stake_amount);

		// Create and apply slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		let native_exposure = security_commitments
			.iter()
			.find(|(asset, _)| asset.is_native())
			.map(|(_, commitment)| commitment.exposure_percent)
			.unwrap();

		// Verify unapplied slash storage values
		let unapplied_slash_index = Services::next_unapplied_slash_index() - 1;
		let unapplied_slash = UnappliedSlashes::<Runtime>::get(0, unapplied_slash_index).unwrap();

		assert_eq!(unapplied_slash.era, 0);
		assert_eq!(unapplied_slash.operator, operator);
		assert_eq!(unapplied_slash.service_id, service_id);
		assert_eq!(unapplied_slash.blueprint_id, blueprint_id);
		assert_eq!(unapplied_slash.slash_percent, slash_percent);

		// TODO: Verify final state after applying slashes
		assert_ok!(MultiAssetDelegation::slash_operator(&unapplied_slash));
	});
}

#[test]
fn test_mixed_native_and_regular_delegation_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, blueprint_id, security_commitments } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let native_stake = 10_000;
		let regular_stake = 100_000;

		// Setup regular delegation
		mint_tokens(USDC, mock_pub_key(ALICE), delegator.clone(), regular_stake * 10u128.pow(3));
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(USDC),
			regular_stake,
			None,
			None,
		));

		// Delegate both native and regular stakes
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			native_stake / 2,
			vec![blueprint_id].into(),
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			regular_stake,
			vec![blueprint_id].into(),
		));

		// Apply slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify the unapplied slash is stored correctly
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		assert_eq!(slashes.len(), 1, "Should have one unapplied slash");

		let (_, slash) = &slashes[0];
		assert_eq!(slash.service_id, service_id);
		assert_eq!(slash.operator, operator);
		assert_eq!(slash.blueprint_id, blueprint_id);
		assert_eq!(slash.era, 0);
		assert_eq!(slash.slash_percent, slash_percent);

		// TODO: Verify final state after applying slashes
		assert_ok!(MultiAssetDelegation::slash_operator(slash));
	});
}

#[test]
fn test_native_restaking_slash_with_invalid_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let invalid_operator = mock_pub_key(99); // Non-existent operator

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Try to slash an invalid operator
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				invalid_operator.clone(),
				service_id,
				Percent::from_percent(50)
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn test_native_restaking_slash_with_multiple_services() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);

		// Deploy first service
		let Deployment { service_id: service_id1, blueprint_id, security_commitments } = deploy();

		// Deploy second service
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			1,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// Request second service
		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			1,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Approve second service
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 1000;

		// Delegate to both services
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			bob.clone(),
			stake_amount,
			vec![blueprint_id, blueprint_id + 1].into(),
		));

		// Slash in first service
		let service1 = Services::services(service_id1).unwrap();
		let slashing_origin1 =
			Services::query_slashing_origin(&service1).map(|(o, _)| o.unwrap()).unwrap();

		let first_slash_percent = Percent::from_percent(50);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin1.clone()),
			bob.clone(),
			service_id1,
			first_slash_percent
		));

		// Verify first slash was recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Slash in second service
		let service2 = Services::services(1).unwrap();
		let slashing_origin2 =
			Services::query_slashing_origin(&service2).map(|(o, _)| o.unwrap()).unwrap();

		let second_slash_percent = Percent::from_percent(25);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin2.clone()),
			bob.clone(),
			1,
			second_slash_percent
		));

		// Verify both slashes are recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 2);
		// Verify slash data
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		assert_eq!(slashes.len(), 2);

		// TODO: Verify final state after applying slashes
		assert_ok!(MultiAssetDelegation::slash_operator(&slashes[0].1));
		assert_ok!(MultiAssetDelegation::slash_operator(&slashes[1].1));
	});
}
