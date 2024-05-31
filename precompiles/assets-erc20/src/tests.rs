// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use crate::{mock::*, *};
use frame_support::assert_ok;
use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};
use std::str::from_utf8;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0u128, CryptoAlith.into(), true, 1));
		// This selector is only three bytes long when four are required.
		precompiles()
			.prepare_test(CryptoAlith, ForeignAssetId(0u128), vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0u128, CryptoAlith.into(), true, 1));

		precompiles()
			.prepare_test(CryptoAlith, ForeignAssetId(0u128), vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn selectors() {
	assert!(ForeignPCall::balance_of_selectors().contains(&0x70a08231));
	assert!(ForeignPCall::total_supply_selectors().contains(&0x18160ddd));
	assert!(ForeignPCall::approve_selectors().contains(&0x095ea7b3));
	assert!(ForeignPCall::allowance_selectors().contains(&0xdd62ed3e));
	assert!(ForeignPCall::transfer_selectors().contains(&0xa9059cbb));
	assert!(ForeignPCall::transfer_from_selectors().contains(&0x23b872dd));
	assert!(ForeignPCall::name_selectors().contains(&0x06fdde03));
	assert!(ForeignPCall::symbol_selectors().contains(&0x95d89b41));
	assert!(ForeignPCall::decimals_selectors().contains(&0x313ce567));

	assert_eq!(
		crate::SELECTOR_LOG_TRANSFER,
		&Keccak256::digest(b"Transfer(address,address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_APPROVAL,
		&Keccak256::digest(b"Approval(address,address,uint256)")[..]
	);
}

#[test]
fn modifiers() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			let mut tester =
				PrecompilesModifierTester::new(precompiles(), CryptoAlith, ForeignAssetId(0u128));

			tester.test_view_modifier(ForeignPCall::balance_of_selectors());
			tester.test_view_modifier(ForeignPCall::total_supply_selectors());
			tester.test_default_modifier(ForeignPCall::approve_selectors());
			tester.test_view_modifier(ForeignPCall::allowance_selectors());
			tester.test_default_modifier(ForeignPCall::transfer_selectors());
			tester.test_default_modifier(ForeignPCall::transfer_from_selectors());
			tester.test_view_modifier(ForeignPCall::name_selectors());
			tester.test_view_modifier(ForeignPCall::symbol_selectors());
			tester.test_view_modifier(ForeignPCall::decimals_selectors());
		});
}

#[test]
fn get_total_supply() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(CryptoAlith, ForeignAssetId(0u128), ForeignPCall::total_supply {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(1000u64));
		});
}

#[test]
fn get_balances_known_user() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(1000u64));
		});
}

#[test]
fn get_balances_unknown_user() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u64));
		});
}

#[test]
fn approve() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.expect_cost(34534756)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(500)),
				))
				.execute_returns(true);
		});
}

#[test]
fn approve_saturating() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: U256::MAX },
				)
				.expect_cost(34534756)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::MAX),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(CryptoAlith.into()),
						spender: Address(Bob.into()),
					},
				)
				.expect_cost(0u64)
				.expect_no_logs()
				.execute_returns(U256::from(u128::MAX));
		});
}

#[test]
fn check_allowance_existing() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.execute_some();

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(CryptoAlith.into()),
						spender: Address(Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(500u64));
		});
}

#[test]
fn check_allowance_not_existing() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(CryptoAlith.into()),
						spender: Address(Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u64));
		});
}

#[test]
fn transfer() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::transfer { to: Address(Bob.into()), value: 400.into() },
				)
				.expect_cost(48477756) // 1 weight => 1 gas in mock
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Bob,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(400));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(600));
		});
}

#[test]
fn transfer_not_enough_founds() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::transfer { to: Address(Charlie.into()), value: 50.into() },
				)
				.execute_reverts(|output| {
					from_utf8(&output).unwrap().contains("Dispatched call failed with error: ")
						&& from_utf8(&output).unwrap().contains("BalanceLow")
				});
		});
}

#[test]
fn transfer_from() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.execute_some();

			// TODO: Duplicate approve (noop)?
			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Bob, // Bob is the one sending transferFrom!
					ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(CryptoAlith.into()),
						to: Address(Charlie.into()),
						value: 400.into(),
					},
				)
				.expect_cost(69947756) // 1 weight => 1 gas in mock
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					CryptoAlith,
					Charlie,
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(600));

			precompiles()
				.prepare_test(
					Bob,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Charlie.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(400));
		});
}

#[test]
fn transfer_from_non_incremental_approval() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			// We first approve 500
			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.expect_cost(34534756)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(500)),
				))
				.execute_returns(true);

			// We then approve 300. Non-incremental, so this is
			// the approved new value
			// Additionally, the gas used in this approval is higher because we
			// need to clear the previous one
			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 300.into() },
				)
				.expect_cost(72171756)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(300)),
				))
				.execute_returns(true);

			// This should fail, as now the new approved quantity is 300
			precompiles()
				.prepare_test(
					Bob, // Bob is the one sending transferFrom!
					ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(CryptoAlith.into()),
						to: Address(Bob.into()),
						value: 500.into(),
					},
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
				});
		});
}

#[test]
fn transfer_from_above_allowance() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 300.into() },
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Bob, // Bob is the one sending transferFrom!
					ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(CryptoAlith.into()),
						to: Address(Bob.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
				});
		});
}

#[test]
fn transfer_from_self() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith, // CryptoAlith sending transferFrom herself, no need for allowance.
					ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(CryptoAlith.into()),
						to: Address(Bob.into()),
						value: 400.into(),
					},
				)
				.expect_cost(48477756) // 1 weight => 1 gas in mock
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(600));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(400));
		});
}

#[test]
fn get_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));

			precompiles()
				.prepare_test(CryptoAlith, ForeignAssetId(0u128), ForeignPCall::name {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from("TestToken"));

			precompiles()
				.prepare_test(CryptoAlith, ForeignAssetId(0u128), ForeignPCall::symbol {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from("Test"));

			precompiles()
				.prepare_test(CryptoAlith, ForeignAssetId(0u128), ForeignPCall::decimals {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(12u8);
		});
}

#[test]
fn transfer_amount_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::transfer {
						to: Address(Bob.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");

			precompiles()
				.prepare_test(
					Bob,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::balance_of { who: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(1000));
		});
}

#[test]
fn transfer_from_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				0u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				CryptoAlith.into(),
				1000
			));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.execute_some();

			// TODO: Duplicate approve of same value (noop?)
			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::approve { spender: Address(Bob.into()), value: 500.into() },
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Bob, // Bob is the one sending transferFrom!
					ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(CryptoAlith.into()),
						to: Address(Charlie.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["ERC20.sol"], ForeignPCall::supports_selector)
}
