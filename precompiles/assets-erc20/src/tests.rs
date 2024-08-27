// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompileset-assets-erc20 package, originally developed by
// Purestake Inc. pallet-evm-precompileset-assets-erc20 package used in Tangle Network in terms of
// GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{eip2612::Eip2612, mock::*, *};
use frame_support::assert_ok;
use hex_literal::hex;
use libsecp256k1::{sign, Message, SecretKey};
use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};
use sp_core::H256;
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
	assert!(ForeignPCall::eip2612_nonces_selectors().contains(&0x7ecebe00));
	assert!(ForeignPCall::eip2612_permit_selectors().contains(&0xd505accf));
	assert!(ForeignPCall::eip2612_domain_separator_selectors().contains(&0x3644e515));

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
			tester.test_view_modifier(ForeignPCall::eip2612_nonces_selectors());
			tester.test_default_modifier(ForeignPCall::eip2612_permit_selectors());
			tester.test_view_modifier(ForeignPCall::eip2612_domain_separator_selectors());
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
					from_utf8(output).unwrap().contains("Dispatched call failed with error: ")
						&& from_utf8(output).unwrap().contains("BalanceLow")
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
					CryptoAlith, /* CryptoAlith sending transferFrom herself, no need for
					              * allowance. */
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
fn permit_valid() {
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

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into(); // todo: proper timestamp

			let permit = Eip2612::<Runtime, pallet_assets::Instance1>::generate_permit(
				ForeignAssetId(0u128).into(),
				0u128,
				owner,
				spender,
				value,
				0u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.expect_cost(34533000)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(500)),
				))
				.execute_returns(());

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
				.execute_returns(U256::from(500u16));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(1u8));
		});
}

#[test]
fn permit_valid_named_asset() {
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
			assert_ok!(Assets::set_metadata(
				RuntimeOrigin::signed(CryptoAlith.into()),
				0u128,
				b"Test token".to_vec(),
				b"TEST".to_vec(),
				18
			));

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into(); // todo: proper timestamp

			let permit = Eip2612::<Runtime, pallet_assets::Instance1>::generate_permit(
				ForeignAssetId(0u128).into(),
				0u128,
				owner,
				spender,
				value,
				0u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.expect_cost(34533000)
				.expect_log(log3(
					ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(500)),
				))
				.execute_returns(());

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
				.execute_returns(U256::from(500u16));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(1u8));
		});
}

#[test]
fn permit_invalid_nonce() {
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

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			let permit = Eip2612::<Runtime, pallet_assets::Instance1>::generate_permit(
				ForeignAssetId(0u128).into(),
				0u128,
				owner,
				spender,
				value,
				1u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.execute_reverts(|output| output == b"Invalid permit");

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
				.execute_returns(U256::from(0u16));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));
		});
}

#[test]
fn permit_invalid_signature() {
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

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: 0,
						r: H256::random(),
						s: H256::random(),
					},
				)
				.execute_reverts(|output| output == b"Invalid permit");

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
				.execute_returns(U256::from(0u16));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));
		});
}

#[test]
fn permit_invalid_deadline() {
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

			pallet_timestamp::Pallet::<Runtime>::set_timestamp(10_000);

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 5u8.into(); // deadline < timestamp => expired

			let permit = Eip2612::<Runtime, pallet_assets::Instance1>::generate_permit(
				ForeignAssetId(0u128).into(),
				0u128,
				owner,
				spender,
				value,
				0u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.execute_reverts(|output| output == b"Permit expired");

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
				.execute_returns(U256::from(0u16));

			precompiles()
				.prepare_test(
					CryptoAlith,
					ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces { owner: Address(CryptoAlith.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(0u8));
		});
}

#[test]
fn permit_valid_with_metamask_signed_data() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			// assetId 1
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				1u128,
				CryptoAlith.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(CryptoAlith.into()),
				1u128,
				CryptoAlith.into(),
				1000
			));

			let owner: H160 = CryptoAlith.into();
			let spender: H160 = Bob.into();
			let value: U256 = 1000u16.into();
			let deadline: U256 = 1u16.into(); // todo: proper timestamp

			let rsv = hex!(
				"3aac886f06729d76067b6b0dbae23978fe48224b10b5648265b8f0e8c4cf25ff7625965d64bf9a6069d
				b00ef5771b65fd24dd118531fc6e86b61a238ca76b9a11c"
			)
			.as_slice();
			let (r, sv) = rsv.split_at(32);
			let (s, v) = sv.split_at(32);
			let v_real = v[0];
			let r_real: [u8; 32] = r.try_into().unwrap();
			let s_real: [u8; 32] = s.try_into().unwrap();

			precompiles()
				.prepare_test(
					Charlie,
					ForeignAssetId(1u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v_real,
						r: H256::from(r_real),
						s: H256::from(s_real),
					},
				)
				.expect_cost(34533000)
				.expect_log(log3(
					ForeignAssetId(1u128),
					SELECTOR_LOG_APPROVAL,
					CryptoAlith,
					Bob,
					solidity::encode_event_data(U256::from(1000)),
				))
				.execute_returns(());
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["ERC20.sol"], ForeignPCall::supports_selector)
}
