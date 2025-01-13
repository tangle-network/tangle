use crate::{mock::*, mock_evm::*, U256};
use frame_support::{assert_ok, traits::Currency};
use pallet_multi_asset_delegation::{types::OperatorStatus, CurrentRound, Delegators, Operators};
use precompile_utils::prelude::*;
use precompile_utils::testing::*;
use sp_core::H160;
use tangle_primitives::services::Asset;

// Helper function for creating and minting tokens
pub fn create_and_mint_tokens(
	asset_id: u128,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset_id, recipient, false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(recipient), asset_id, recipient, amount));
}

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_join_operators() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let initial_balance = Balances::free_balance(account);
		assert!(Operators::<Runtime>::get(account).is_none());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::join_operators { bond_amount: U256::from(10_000) },
			)
			.execute_returns(());

		assert!(Operators::<Runtime>::get(account).is_some());
		let expected_balance = initial_balance - 10_000;
		assert_eq!(Balances::free_balance(account), expected_balance);
	});
}

#[test]
fn test_join_operators_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Eve);
		Balances::make_free_balance_be(&account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::join_operators { bond_amount: U256::from(10_000) },
			)
			.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 1, error: [2, 0, 0, 0], message: Some(\"InsufficientBalance\") })");

		assert_eq!(Balances::free_balance(account), 500);
	});
}

#[test]
fn test_delegate_assets_invalid_operator() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);

		Balances::make_free_balance_be(&delegator_account, 500);
		create_and_mint_tokens(1, delegator_account, 500);

		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account), Asset::Custom(1), 200, Some(TestAccount::Alex.into()), None));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: sp_core::sr25519::Public::from(TestAccount::Eve).into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 6, error: [2, 0, 0, 0], message: Some(\"NotAnOperator\") })");

		assert_eq!(Balances::free_balance(delegator_account), 500);
	});
}

#[test]
fn test_deposit_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		Balances::make_free_balance_be(&delegator_account, 500);

		create_and_mint_tokens(1, delegator_account, 500);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(200),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_returns(());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());
	});
}

#[test]
fn test_deposit_assets_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		Balances::make_free_balance_be(&delegator_account, 500);

		create_and_mint_tokens(1, delegator_account, 200);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(500),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_reverts(|output| {
				output == b"Dispatched call failed with error: Arithmetic(Underflow)"
			});

		assert_eq!(Assets::balance(1, delegator_account), 200); // should not lose deposit

		assert!(Delegators::<Runtime>::get(delegator_account).is_none());
	});
}

#[test]
fn test_deposit_assets_erc20() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		Balances::make_free_balance_be(&delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: USDC_ERC20,
					lock_multiplier: 0,
				},
			)
			.with_subcall_handle(|subcall| {
				// Intercept the call
				assert!(!subcall.is_static);
				assert_eq!(subcall.address, USDC_ERC20);
				assert_eq!(subcall.context.caller, TestAccount::Alex.into());
				assert_eq!(subcall.context.apparent_value, U256::zero());
				assert_eq!(subcall.context.address, USDC_ERC20);
				assert_eq!(subcall.input[0..4], keccak256!("transfer(address,uint256)")[0..4]);
				// if all of the above passed, then it is okay.

				let mut out = SubcallOutput::succeed();
				out.output = ethabi::encode(&[ethabi::Token::Bool(true)]).to_vec();
				out
			})
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());
	});
}

#[test]
fn test_deposit_assets_insufficient_balance_erc20() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		Balances::make_free_balance_be(&delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: USDC_ERC20,
					lock_multiplier: 0,
				},
			)
			.with_subcall_handle(|_subcall| {
				// Simulate a faild ERC20 transfer
				let mut out = SubcallOutput::succeed();
				out.output = ethabi::encode(&[ethabi::Token::Bool(false)]).to_vec();
				out
			})
			.execute_reverts(|output| output == b"Failed to transfer ERC20 tokens: false");

		assert!(Delegators::<Runtime>::get(delegator_account).is_none());
	});
}

#[test]
fn test_delegate_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator_account),
			Asset::Custom(1),
			200,
			Some(TestAccount::Alex.into()),
			None
		));
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // no change when delegating
	});
}

#[test]
fn test_delegate_assets_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Eve);

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account), Asset::Custom(1), 200, None, None));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(300),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 6, error: [15, 0, 0, 0], message: Some(\"InsufficientBalance\") })");

		assert_eq!(Balances::free_balance(delegator_account), 500);
	});
}

#[test]
fn test_schedule_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(200),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_returns(());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw {
					asset_id: U256::from(1),
					amount: U256::from(100),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // no change
	});
}

#[test]
fn test_execute_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(200),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_returns(());
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw {
					asset_id: U256::from(1),
					amount: U256::from(100),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		let metadata = MultiAssetDelegation::delegators(delegator_account).unwrap();
		assert!(!metadata.withdraw_requests.is_empty());

		<CurrentRound<Runtime>>::put(5);

		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::execute_withdraw {})
			.execute_returns(());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 100); // deposited 200, withdrew 100
	});
}

#[test]
fn test_execute_withdraw_before_due() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);

		Balances::make_free_balance_be(&delegator_account, 10_000);
		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(200),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_returns(());
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // delegate should not change balance

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw {
					asset_id: U256::from(1),
					amount: U256::from(100),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::execute_withdraw {})
			.execute_returns(()); // should not fail

		// not expired so should not change balance
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200);
	});
}

#[test]
fn test_cancel_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::from(1),
					amount: U256::from(200),
					token_address: Default::default(),
					lock_multiplier: 0,
				},
			)
			.execute_returns(());
		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw {
					asset_id: U256::from(1),
					amount: U256::from(100),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::cancel_withdraw {
					asset_id: U256::from(1),
					amount: U256::from(100),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		let metadata = MultiAssetDelegation::delegators(delegator_account).unwrap();
		assert!(metadata.deposits.contains_key(&Asset::Custom(1)));
		assert!(metadata.withdraw_requests.is_empty());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // no change
	});
}

#[test]
fn test_operator_go_offline_and_online() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		PrecompilesValue::get()
			.prepare_test(TestAccount::Bobo, H160::from_low_u64_be(1), PCall::go_offline {})
			.execute_returns(());

		assert!(
			MultiAssetDelegation::operator_info(operator_account).unwrap().status
				== OperatorStatus::Inactive
		);

		PrecompilesValue::get()
			.prepare_test(TestAccount::Bobo, H160::from_low_u64_be(1), PCall::go_online {})
			.execute_returns(());

		assert!(
			MultiAssetDelegation::operator_info(operator_account).unwrap().status
				== OperatorStatus::Active
		);

		assert_eq!(Balances::free_balance(operator_account), 20_000 - 10_000);
	});
}
