use super::*;

#[test]
fn test_unbond_deposit() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let pool_id = 0;

			let lst_collection_owner = <Runtime as Config>::LstCollectionOwner::get();

			// deposit is stored at the pallet account
			assert_eq!(
				Pools::member_points(pool_id, Pallet::<Runtime>::deposit_account_id(pool_id)),
				10
			);

			// Bond some extra funds for depositor
			bond_extra(10, pool_id, 20);

			// deposit is not ready for unbonding yet
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id),
				Error::<Runtime>::DepositNotReadyForUnbonding
			);

			// set state to destroying
			unsafe_set_state(pool_id, PoolState::Destroying);

			// deposit is not ready for unbonding yet, still
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id),
				Error::<Runtime>::DepositNotReadyForUnbonding
			);

			// fully unbond the depositor
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(10), pool_id, 10));

			// deposit is still not ready for unbonding, since there are still funds in the pool
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id),
				Error::<Runtime>::DepositNotReadyForUnbonding
			);

			// now fully unbond only one of the members
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(40), pool_id, 40));

			// deposit is still not ready for unbonding, since there are still funds in the pool
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id),
				Error::<Runtime>::DepositNotReadyForUnbonding
			);

			// now fully unbond the last member
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(550), pool_id, 550));

			// deposit is not yet ready for unbonding, we wait for the last member to withdraw
			// unbonded funds
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id),
				Error::<T>::PoolMembersRemaining
			);

			// let all the members withdraw unbonded
			CurrentEra::set(3);
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(550), pool_id, 550, 550));
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(40), pool_id, 40, 40));
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 10));

			// deposit is now ready for unbonding
			assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));

			// checks
			assert_eq!(Pools::member_points(pool_id, 10), 0);
			assert_eq!(Pools::member_points(pool_id, 40), 0);
			assert_eq!(Pools::member_points(pool_id, 550), 0);
			assert_eq!(Pools::member_points(pool_id, lst_collection_owner), 0);

			// check events
			assert_event_deposited!(Event::Unbonded {
				member: Pallet::<Runtime>::deposit_account_id(pool_id),
				pool_id,
				balance: 10,
				points: 10,
				era: 6
			});
		});
}

#[test]
fn test_withdraw_deposit() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;

		// prepare for unbonding
		unsafe_set_state(pool_id, PoolState::Destroying);
		let _ = pool_events_since_last_call();
		let _ = balances_events_since_last_call();

		// do the unbond
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));

		// skip eras
		CurrentEra::set(3);

		// unbonding entire balance kills the token storage
		assert!(Fungibles::token_of(staked_collection_id(), pool_id as AssetId).is_none());

		// withdraw the unbonded deposit
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), pool_id));
		assert!(Fungibles::token_of(staked_collection_id(), pool_id as AssetId).is_none());

		// check events
		let depositor = Pallet::<Runtime>::deposit_account_id(pool_id);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Unbonded { member: depositor, pool_id, balance: 10, points: 10, era: 3 },
				Event::Withdrawn { member: 10, pool_id, balance: 10, points: 10 },
				Event::Destroyed { pool_id }
			]
		);

		assert_eq!(
			balances_events_since_last_call()[0..2],
			vec![
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
					to: 10,
					amount: 10
				},
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Reward),
					to: 10,
					amount: 5
				}
			]
		);
	})
}

#[test]
fn test_unbond_deposit_owner_changed() {
	ExtBuilder::default().build_and_execute(|| {
		// record balances before
		let starting_balance = 10_000_000 * UNIT;

		Balances::make_free_balance_be(&421, starting_balance);

		let pool_token_id = DEFAULT_TOKEN_ID;

		let pool_id = 0;

		// now send token to 421
		assert_ok!(<Runtime as Config>::Fungibles::transfer(
			RuntimeOrigin::signed(10),
			421,
			<<Runtime as Config>::PoolCollectionId as Get<_>>::get(),
			<Fungibles as Fungibles>::TransferParams::try_from(
				ConsolidatedTransferParams {
					token_id: pool_token_id,
					amount: 1,
					source: None,
					depositor: None
				}
			)
			.unwrap()
		));

		// now check that pool's deposit owner is 421

		// first set state of pools to destroying
		unsafe_set_state(pool_id, PoolState::Destroying);

		// reset events
		let _ = pool_events_since_last_call();
		let _ = balances_events_since_last_call();

		// unbond the deposit
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));

		// skip eras
		CurrentEra::set(3);

		// withdraw unbonded deposit
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), pool_id));

		// check events
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Unbonded {
					member: Pallet::<Runtime>::deposit_account_id(pool_id),
					pool_id,
					balance: 10,
					points: 10,
					era: 3
				},
				Event::Withdrawn { member: 421, pool_id, balance: 10, points: 10 },
				Event::Destroyed { pool_id },
			]
		);

		// check balances events
		assert_eq!(
			balances_events_since_last_call()[0..2],
			vec![
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
					to: 421,
					amount: 10
				},
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Reward),
					to: 421,
					amount: 5
				},
			]
		);
	});
}

#[test]
fn test_withdraw_deposit_owner_changed() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		Balances::make_free_balance_be(&421, 10_000_000 * UNIT);

		// first set state of pools to destroying
		unsafe_set_state(pool_id, PoolState::Destroying);

		// reset events
		let _ = pool_events_since_last_call();
		let _ = balances_events_since_last_call();

		// unbond the deposit
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));

		// skip eras
		CurrentEra::set(3);

		// now change the owner to 421
		assert_ok!(<Runtime as Config>::Fungibles::transfer(
			RuntimeOrigin::signed(10),
			421,
			<<Runtime as Config>::PoolCollectionId as Get<_>>::get(),
			<Fungibles as Fungibles>::TransferParams::try_from(
				ConsolidatedTransferParams {
					token_id: DEFAULT_TOKEN_ID,
					amount: 1,
					source: None,
					depositor: None
				}
			)
			.unwrap()
		));

		// withdraw unbonded deposit
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), pool_id));

		// check events
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Unbonded {
					member: Pallet::<Runtime>::deposit_account_id(pool_id),
					pool_id,
					balance: 10,
					points: 10,
					era: 3
				},
				Event::Withdrawn { member: 421, pool_id, balance: 10, points: 10 },
				Event::Destroyed { pool_id },
			]
		);

		// check balances events
		assert_eq!(
			balances_events_since_last_call()[0..2],
			vec![
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
					to: 421,
					amount: 10
				},
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Reward),
					to: 421,
					amount: 5
				},
			]
		);
	})
}

#[test]
fn test_unbond_deposit_burned_pool_token() {
	// in an extremely rare case, the pool token can be burned before the deposit is unbonded.
	// this test ensures that in this case, the deposit is sent to the treasury address.
	ExtBuilder::default().build_and_execute(|| {
		// send some funds to treasury account
		let orphan_account = <Runtime as Config>::UnclaimedBalanceReceiver::get();

		Balances::make_free_balance_be(&orphan_account, 5);

		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// burn the pool token
		assert_ok!(<Runtime as Config>::Fungibles::burn(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<_>>::get(),
		));

		// prepare for unbonding
		unsafe_set_state(pool_id, PoolState::Destroying);
		let _ = pool_events_since_last_call();
		let _ = balances_events_since_last_call();

		// unbonding the deposit should work fine
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));

		// skip eras
		CurrentEra::set(3);

		// withdraw unbonded deposit
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), pool_id));

		// check that the deposit is sent to the treasury address
		assert_eq!(
			balances_events_since_last_call()[0..2],
			vec![
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
					to: orphan_account,
					amount: 10
				},
				BEvent::Transfer {
					from: Pools::compute_pool_account_id(pool_id, AccountType::Reward),
					to: orphan_account,
					amount: 5
				}
			]
		);
	});
}
