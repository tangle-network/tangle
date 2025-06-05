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
use sp_runtime::traits::{BlakeTwo256, Hash};
use tangle_primitives::services::BoundedString;

#[test]
fn test_hooks() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let alice = mock_pub_key(ALICE);
		let mut blueprint = cggmp21_blueprint();
		blueprint.manager = BlueprintServiceManager::Evm(HOOKS_TEST);

		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]), // Include native asset requirement
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Bob approves the request with security commitments
		let security_commitments =
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)];
		let security_commitment_hash = BlakeTwo256::hash_of(&security_commitments);
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			security_commitment_hash
		));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}
