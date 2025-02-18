use crate::{mock::*, mock_evm::*};
use frame_support::{assert_ok, traits::Currency};
use pallet_multi_asset_delegation::{CurrentRound, Delegators};
use precompile_utils::testing::*;
use sp_core::{H160, H256, U256};
use sp_keyring::AccountKeyring;

#[test]
fn test_delegate_nomination_through_precompile() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator = TestAccount::Alex;
		let operator: AccountId = AccountKeyring::Alice.into();
		let validator = Staking::invulnerables()[0].clone();
		let amount = 100_000;
		let delegate_amount = amount / 2;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Bond and nominate through staking precompile
		Balances::make_free_balance_be(&delegator.into(), amount);

		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(2), // Staking precompile address
				SCall::bond {
					value: U256::from(amount),
					payee: H256([0; 32]), // Stash
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(2),
				SCall::nominate { targets: vec![H256::from(validator.as_ref())] },
			)
			.execute_returns(());

		// Delegate nomination through multi-asset delegation precompile
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::delegate_nomination {
					operator: H256::from(operator.as_ref()),
					amount: U256::from(delegate_amount),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		// Verify delegation
		let delegator_account_id: AccountId = delegator.into();
		let delegator = MultiAssetDelegation::delegators(delegator_account_id).unwrap();
		assert_eq!(delegator.total_nomination_delegations(), delegate_amount);
	});
}

#[test]
fn test_delegate_nomination_invalid_operator() {
	ExtBuilder::default().build().execute_with(|| {
        let delegator = TestAccount::Alex;
        let invalid_operator: AccountId = AccountKeyring::Bob.into();
        let validator = Staking::invulnerables()[0].clone();
        let amount = 100_000;
        let delegate_amount = amount / 2;

        // Bond and nominate through staking precompile
        Balances::make_free_balance_be(&delegator.into(), amount);

        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(2),
                SCall::bond {
                    value: U256::from(amount),
                    payee: H256([0; 32]),
                },
            )
            .execute_returns(());

        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(2),
                SCall::nominate {
                    targets: vec![H256::from(validator.as_ref())],
                },
            )
            .execute_returns(());

        // Try to delegate to invalid operator
        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(1),
                PCall::delegate_nomination {
                    operator: H256::from(invalid_operator.as_ref()),
                    amount: U256::from(delegate_amount),
                    blueprint_selection: Default::default(),
                },
            )
            .execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 9, error: [3, 0, 0, 0], message: Some(\"NotAnOperator\") })");
    });
}

#[test]
fn test_delegate_nomination_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
        let delegator = TestAccount::Alex;
        let operator: AccountId = AccountKeyring::Alice.into();
        let validator = Staking::invulnerables()[0].clone();
        let amount = 100_000;
        let delegate_amount = amount * 2; // More than bonded

        // Setup operator
        assert_ok!(MultiAssetDelegation::join_operators(
            RuntimeOrigin::signed(operator.clone()),
            10_000
        ));

        // Bond and nominate through staking precompile
        Balances::make_free_balance_be(&delegator.into(), amount);

        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(2),
                SCall::bond {
                    value: U256::from(amount),
                    payee: H256([0; 32]),
                },
            )
            .execute_returns(());

        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(2),
                SCall::nominate {
                    targets: vec![H256::from(validator.as_ref())],
                },
            )
            .execute_returns(());

        // Try to delegate more than bonded
        PrecompilesValue::get()
            .prepare_test(
                delegator,
                H160::from_low_u64_be(1),
                PCall::delegate_nomination {
                    operator: H256::from(operator.as_ref()),
                    amount: U256::from(delegate_amount),
                    blueprint_selection: Default::default(),
                },
            )
            .execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 9, error: [15, 0, 0, 0], message: Some(\"InsufficientBalance\") })");
    });
}

#[test]
fn test_delegate_nomination_unstake_lifecycle() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator = TestAccount::Alex;
		let operator: AccountId = AccountKeyring::Alice.into();
		let validator = Staking::invulnerables()[0].clone();
		let amount = 100_000;
		let delegate_amount = amount / 2;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Bond and nominate through staking precompile
		Balances::make_free_balance_be(&delegator.into(), amount);

		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(2),
				SCall::bond { value: U256::from(amount), payee: H256([0; 32]) },
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(2),
				SCall::nominate { targets: vec![H256::from(validator.as_ref())] },
			)
			.execute_returns(());

		// Delegate nomination
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::delegate_nomination {
					operator: H256::from(operator.as_ref()),
					amount: U256::from(delegate_amount),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		// Schedule unstake
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::schedule_delegator_nomination_unstake {
					operator: H256::from(operator.as_ref()),
					amount: U256::from(delegate_amount),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		// Verify unstake request exists
		let delegator_account: AccountId = delegator.into();
		let metadata = Delegators::<Runtime>::get(&delegator_account).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);

		// Cancel unstake
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::cancel_delegator_nomination_unstake {
					operator: H256::from(operator.as_ref()),
				},
			)
			.execute_returns(());

		// Verify unstake request was cancelled
		let metadata = Delegators::<Runtime>::get(&delegator_account).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 0);

		// Schedule unstake again
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::schedule_delegator_nomination_unstake {
					operator: H256::from(operator.as_ref()),
					amount: U256::from(delegate_amount),
					blueprint_selection: Default::default(),
				},
			)
			.execute_returns(());

		// Advance rounds to make unstake executable
		CurrentRound::<Runtime>::put(100);

		// Execute unstake
		PrecompilesValue::get()
			.prepare_test(
				delegator,
				H160::from_low_u64_be(1),
				PCall::execute_delegator_nomination_unstake {
					operator: H256::from(operator.as_ref()),
				},
			)
			.execute_returns(());

		// Verify unstake was executed
		let metadata = Delegators::<Runtime>::get(&delegator_account).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 0);
		assert_eq!(metadata.total_nomination_delegations(), 0);
	});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(
		&["MultiAssetDelegation.sol"],
		PCall::supports_selector,
	)
}
