use crate::{mock::*, U256};
use frame_support::{assert_ok, traits::Currency};
use precompile_utils::testing::*;
use sp_core::H160;

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
fn test_join() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let initial_balance = Balances::free_balance(account);

		// First create a pool
		let root = sp_core::sr25519::Public::from(TestAccount::Bob).into();
		let nominator = sp_core::sr25519::Public::from(TestAccount::Charlie).into();
		let bouncer = sp_core::sr25519::Public::from(TestAccount::Dave).into();
		let name = b"Test Pool".to_vec();
		let icon = b"icon_data".to_vec();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					amount: U256::from(10_000),
					root,
					nominator,
					bouncer,
					name: name.clone(),
					icon: icon.clone(),
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::join { amount: U256::from(10_000), pool_id: U256::from(1) },
			)
			.execute_returns(());

		let expected_balance = initial_balance - 20_000;
		assert_eq!(Balances::free_balance(account), expected_balance - 1);
	});
}

#[test]
fn test_bond_extra() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let initial_balance = Balances::free_balance(account);

		// First create a pool
		let root = sp_core::sr25519::Public::from(TestAccount::Bob).into();
		let nominator = sp_core::sr25519::Public::from(TestAccount::Charlie).into();
		let bouncer = sp_core::sr25519::Public::from(TestAccount::Dave).into();
		let name = b"Test Pool".to_vec();
		let icon = b"icon_data".to_vec();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					amount: U256::from(10_000),
					root,
					nominator,
					bouncer,
					name: name.clone(),
					icon: icon.clone(),
				},
			)
			.execute_returns(());

		// then join the pool
		assert_ok!(Lst::join(RuntimeOrigin::signed(account), 10_000, 1));

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::bond_extra {
					pool_id: U256::from(1),
					extra_type: 0,
					extra: U256::from(5_000),
				},
			)
			.execute_returns(());
	});
}

#[test]
fn test_create_pool() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let initial_balance = Balances::free_balance(account);

		let root = sp_core::sr25519::Public::from(TestAccount::Bob).into();
		let nominator = sp_core::sr25519::Public::from(TestAccount::Charlie).into();
		let bouncer = sp_core::sr25519::Public::from(TestAccount::Dave).into();
		let name = b"Test Pool".to_vec();
		let icon = b"icon_data".to_vec();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					amount: U256::from(10_000),
					root,
					nominator,
					bouncer,
					name: name.clone(),
					icon: icon.clone(),
				},
			)
			.execute_returns(());

		let expected_balance = initial_balance - 10_000;
		assert_eq!(Balances::free_balance(account), expected_balance - 1);
	});
}

#[test]
fn test_nominate() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);

		// First create a pool
		let root = sp_core::sr25519::Public::from(TestAccount::Alex);
		let nominator = sp_core::sr25519::Public::from(TestAccount::Bob);
		let bouncer = sp_core::sr25519::Public::from(TestAccount::Charlie);
		assert_ok!(Lst::create(
			RuntimeOrigin::signed(account),
			10_000,
			root,
			nominator,
			bouncer,
			Some(b"Test Pool".to_vec().try_into().unwrap()),
			Some(b"icon_data".to_vec().try_into().unwrap()),
		));

		let validators = vec![
			sp_core::sr25519::Public::from(TestAccount::Dave).into(),
			sp_core::sr25519::Public::from(TestAccount::Eve).into(),
		];

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob, // Using nominator account
				H160::from_low_u64_be(1),
				PCall::nominate { pool_id: U256::from(1), validators },
			)
			.execute_returns(());
	});
}
