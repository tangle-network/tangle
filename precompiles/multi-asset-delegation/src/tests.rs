use crate::{mock::*, mock_evm::*, U256};
use frame_support::{assert_ok, traits::Currency};
use pallet_multi_asset_delegation::{CurrentRound, Delegators, Operators};
use precompile_utils::prelude::*;
use precompile_utils::testing::*;
use sp_core::{H160, H256};
use tangle_primitives::services::Asset;

// Helper function for creating and minting tokens
pub fn create_and_mint_tokens(
	asset_id: u128,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset_id, recipient.clone(), false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(recipient.clone()), asset_id, recipient, amount));
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
fn test_delegate_assets_invalid_operator() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();

		Balances::make_free_balance_be(&delegator_account, 500);
		create_and_mint_tokens(1, delegator_account.clone(), 500);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator_account.clone()), 
			Asset::Custom(1), 
			200, 
			Some(TestAccount::Alex.into()), 
			None
		));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(sp_runtime::AccountId32::from(TestAccount::Eve).as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
				.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 9, error: [3, 0, 0, 0], message: Some(\"NotAnOperator\") })");

		assert_eq!(Balances::free_balance(delegator_account), 500);
	});
}

#[test]
fn test_deposit_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();
		Balances::make_free_balance_be(&delegator_account, 500);

		create_and_mint_tokens(1, delegator_account.clone(), 500);
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

		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // should lose deposit

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());
	});
}

#[test]
fn test_deposit_assets_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();
		Balances::make_free_balance_be(&delegator_account, 500);

		create_and_mint_tokens(1, delegator_account.clone(), 200);
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

		assert_eq!(Assets::balance(1, &delegator_account), 200); // should not lose deposit

		assert!(Delegators::<Runtime>::get(&delegator_account).is_none());
	});
}

#[test]
fn test_deposit_assets_erc20() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();
		Balances::make_free_balance_be(&delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
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
		let delegator_account: AccountId = TestAccount::Alex.into();
		Balances::make_free_balance_be(&delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
					lock_multiplier: 0,
				},
			)
			.with_subcall_handle(|_subcall| {
				// Simulate a failed ERC20 transfer
				let mut out = SubcallOutput::succeed();
				out.output = ethabi::encode(&[ethabi::Token::Bool(false)]).to_vec();
				out
			})
			.execute_reverts(|output| output == b"Failed to transfer ERC20 tokens: false");

		assert!(Delegators::<Runtime>::get(delegator_account).is_none());

		// Delegate
	});
}

#[test]
fn test_delegate_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account: AccountId = TestAccount::Bobo.into();
		let delegator_account: AccountId = TestAccount::Alex.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator_account.clone()),
			Asset::Custom(1),
			200,
			Some(TestAccount::Alex.into()),
			None
		));
		assert_eq!(Assets::balance(1, &delegator_account), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert_eq!(Assets::balance(1, &delegator_account), 500 - 200); // no change when delegating
		assert!(Operators::<Runtime>::get(operator_account)
			.unwrap()
			.delegations
			.iter()
			.any(|x| x.delegator == delegator_account
				&& x.asset == Asset::Custom(1)
				&& x.amount == 100));
	});
}

#[test]
fn test_delegate_assets_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account: AccountId = TestAccount::Bobo.into();
		let delegator_account: AccountId = TestAccount::Eve.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);

		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account.clone()), Asset::Custom(1), 200, None, None));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(300),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 9, error: [15, 0, 0, 0], message: Some(\"InsufficientBalance\") })");

		assert_eq!(Balances::free_balance(delegator_account), 500);
	});
}

#[test]
fn test_unstake_assets_erc20() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();
		let operator_account: AccountId = TestAccount::Bobo.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);
		Balances::make_free_balance_be(&delegator_account, 500);

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
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

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());

		// Unstake

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_delegator_unstake {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(());

		let d = Delegators::<Runtime>::get(delegator_account).unwrap();
		assert!(d
			.delegator_unstake_requests
			.iter()
			.any(|x| x.amount == 200 && x.asset == Asset::Erc20(USDC_ERC20)));
	});
}

#[test]
fn test_schedule_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account: AccountId = TestAccount::Bobo.into();
		let delegator_account: AccountId = TestAccount::Alex.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);

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

		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());

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
		let delegator_account: AccountId = TestAccount::Alex.into();
		let operator_account: AccountId = TestAccount::Bobo.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);

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
		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());

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

		let metadata = MultiAssetDelegation::delegators(&delegator_account).unwrap();
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
		let delegator_account: AccountId = TestAccount::Alex.into();
		let operator_account: AccountId = TestAccount::Bobo.into();

		Balances::make_free_balance_be(&delegator_account, 10_000);
		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);

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
		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());
		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // delegate should not change balance

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
		let delegator_account: AccountId = TestAccount::Alex.into();
		let operator_account: AccountId = TestAccount::Bobo.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);

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
		assert_eq!(Assets::balance(1, delegator_account.clone()), 500 - 200); // should lose deposit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::from(1),
					amount: U256::from(100),
					blueprint_selection: Default::default(),
					token_address: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(&delegator_account).is_some());

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

		let metadata = MultiAssetDelegation::delegators(&delegator_account).unwrap();
		assert!(metadata.deposits.contains_key(&Asset::Custom(1)));
		assert!(metadata.withdraw_requests.is_empty());

		assert_eq!(Assets::balance(1, delegator_account), 500 - 200); // no change
	});
}

#[test]
fn balance_of_works() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account: AccountId = TestAccount::Alex.into();
		let operator_account: AccountId = TestAccount::Bobo.into();

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account.clone()),
			10_000
		));

		create_and_mint_tokens(1, delegator_account.clone(), 500);
		Balances::make_free_balance_be(&delegator_account, 500);

		// Not a delegator yet.
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::zero());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::zero());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::deposit {
					asset_id: U256::zero(),
					amount: U256::from(200),
					token_address: Address(USDC_ERC20),
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

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());

		// Deposit successful, now check balance

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(200));

		// This should still zero, since it's not delegated yet.
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::zero());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::zero(),
					amount: U256::from(100),
					token_address: Address(USDC_ERC20),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account.clone()).is_some());
		// Delegated balance should now be 100
		// Deposit balance should be the same as before.
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(200));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(100));

		// Unstake
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_delegator_unstake {
					operator: H256::from(operator_account.as_ref()),
					asset_id: U256::zero(),
					amount: U256::from(50),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(());

		let d = Delegators::<Runtime>::get(delegator_account.clone()).unwrap();
		assert!(d
			.delegator_unstake_requests
			.iter()
			.any(|x| x.amount == 50 && x.asset == Asset::Erc20(USDC_ERC20)));

		// Now check balance again

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(200));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(100));

		MultiAssetDelegation::handle_round_change(5);
		// Execute unstake
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::execute_delegator_unstake {},
			)
			.execute_returns(());

		// Check balance again

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(200));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(50));

		// Schedule withdraw
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw {
					asset_id: U256::zero(),
					amount: U256::from(100),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(());

		// Check balance again
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(100));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(50));

		MultiAssetDelegation::handle_round_change(6);

		// Execute withdraw
		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::execute_withdraw {})
			.with_subcall_handle(|subcall| {
				// Intercept the call
				assert!(!subcall.is_static);
				assert_eq!(subcall.address, USDC_ERC20);
				assert_eq!(subcall.context.caller, MultiAssetDelegation::pallet_evm_account());
				assert_eq!(subcall.context.apparent_value, U256::zero());
				assert_eq!(subcall.context.address, USDC_ERC20);
				assert_eq!(subcall.input[0..4], keccak256!("transfer(address,uint256)")[0..4]);
				// if all of the above passed, then it is okay.

				let mut out = SubcallOutput::succeed();
				out.output = ethabi::encode(&[ethabi::Token::Bool(true)]).to_vec();
				out
			})
			.execute_returns(());

		// Check balance again

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(100));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegated_balance_of {
					who: TestAccount::Alex.into(),
					asset_id: U256::zero(),
					token_address: Address(USDC_ERC20),
				},
			)
			.execute_returns(U256::from(50));
	});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(
		&["MultiAssetDelegation.sol"],
		PCall::supports_selector,
	)
}
