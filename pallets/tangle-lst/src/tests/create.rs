use super::*;

#[test]
fn test_create() {
	ExtBuilder::default().build_and_execute(|| {
		let first_pool_id = NextPoolId::<Runtime>::get() - 1;
		let token_id = DEFAULT_TOKEN_ID + 1;
		let minimum_bond = StakingMock::minimum_nominator_bond();

		// next pool id is 2.
		let next_pool_stash = Pools::compute_pool_account_id(1, AccountType::Bonded);
		let minimum_balance = Balances::minimum_balance();

		assert!(!BondedPools::<Runtime>::contains_key(1));
		assert!(!UnbondingMembers::<Runtime>::contains_key(first_pool_id, 11));
		assert_err!(
			StakingMock::active_stake(&next_pool_stash),
			DispatchError::Other("balance not found")
		);

		Balances::make_free_balance_be(&11, minimum_bond + (minimum_balance * 2));

		let min_duration = <<Runtime as Config>::MinDuration as Get<EraIndex>>::get();
		let max_duration = <<Runtime as Config>::MaxDuration as Get<EraIndex>>::get();

		// cannot create with duration above MaxDuration
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				2,
				1_000,
				max_duration + 1,
				Default::default()
			),
			Error::<Runtime>::DurationOutOfBounds
		);

		// cannot create with duration below MinDuration
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				2,
				1_000,
				min_duration - 1,
				Default::default()
			),
			Error::<Runtime>::DurationOutOfBounds
		);

		// cannot create if capacity is less than amount
		Balances::make_free_balance_be(&999, 2_000);
		mint_pool_token(57, 999);
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(999),
				57,
				minimum_bond,
				minimum_bond - 1,
				min_duration,
				Default::default(),
			),
			Error::<Runtime>::CapacityExceeded
		);

		// cannot create pool if doesnt hold pool token
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				minimum_bond,
				1_000,
				min_duration,
				Default::default()
			),
			Error::<Runtime>::TokenRequired
		);

		// give account (11) NFT allowing to create pools
		mint_pool_token(token_id, 11);

		// cannot create with name length above maximum
		let mut name = String::new();
		for _ in 0..<Runtime as Config>::MaxPoolNameLength::get() + 1 {
			name.push('0');
		}
		let name: Option<PoolNameOf<Runtime>> = name.as_bytes().to_vec().try_into().ok();
		assert!(name.is_none());

		assert_ok!(Pools::create(
			RuntimeOrigin::signed(11),
			token_id,
			minimum_bond,
			1_000,
			min_duration,
			b"test".to_vec().try_into().unwrap(),
		));

		// cannot create another pool with same token_id
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				minimum_bond,
				1_000,
				min_duration,
				Default::default(),
			),
			Error::<Runtime>::PoolTokenAlreadyInUse
		);

		let second_pool_id = NextPoolId::<Runtime>::get() - 1;

		// make sure reverse token_id -> pool_id lookup was stored
		assert_eq!(UsedPoolTokenIds::<Runtime>::get(token_id), Some(second_pool_id));

		// make sure the funds where bonded and recored
		assert_eq!(Balances::free_balance(11), 0);
		assert_eq!(
			Pools::member_points(second_pool_id, Pools::deposit_account_id(second_pool_id)),
			minimum_bond
		);
		// bonus account should have minimum balance
		assert_eq!(
			Balances::free_balance(Pools::compute_pool_account_id(
				second_pool_id,
				AccountType::Bonus
			)),
			minimum_balance
		);
		let pool = BondedPool::<Runtime>::get(1).unwrap();
		assert_eq!(
			pool,
			BondedPool {
				id: 1,
				inner: BondedPoolInner {
					token_id: DEFAULT_TOKEN_ID + 1,
					state: PoolState::Open,
					capacity: 1_000,
					commission: Commission::default(),
				}
			}
		);
		assert_eq!(pool.points(), minimum_bond);
		assert_eq!(StakingMock::active_stake(&next_pool_stash).unwrap(), minimum_bond);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id: 0, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(0), pool_id: 0, bonded: 10 },
				Event::Created { creator: 11, pool_id: 1, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(1), pool_id: 1, bonded: 10 }
			]
		);
	});
}

#[test]
fn create_errors_correctly() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		// Given
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);
		assert_eq!(StakingMock::minimum_nominator_bond(), 10);

		// Then
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				DEFAULT_TOKEN_ID,
				9,
				1_000,
				30,
				Default::default(),
			),
			Error::<Runtime>::MinimumBondNotMet
		);

		// Given
		MinCreateBond::<Runtime>::put(20);

		// Then
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				DEFAULT_TOKEN_ID,
				19,
				1_000,
				30,
				Default::default(),
			),
			Error::<Runtime>::MinimumBondNotMet
		);

		// Given
		BondedPool::<Runtime> {
			id: 2,
			inner: BondedPoolInner {
				token_id: DEFAULT_TOKEN_ID,
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		}
		.put();
		assert_eq!(BondedPools::<Runtime>::count(), 2);
	});
}

#[test]
fn test_create_with_capacity() {
	ExtBuilder::default().build_and_execute(|| {
		let token_id = DEFAULT_TOKEN_ID + 1;
		mint_pool_token(token_id, 11);
		Balances::make_free_balance_be(&11, 21_000_000 * UNIT);

		// given global max is set at 20M TNT
		// and default capacity is 500K TNT

		// creating a pool without attribute must fallback to default 500K max capacity
		// and if we try to create a pool with capacity above 500K TNT, it should fail
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				StakingMock::minimum_nominator_bond(),
				500_001 * UNIT,
				30,
				Default::default(),
			),
			Error::<Runtime>::CapacityExceeded
		);

		// non number capacity attribute should fail
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			b"not-capacity".to_vec().try_into().unwrap(),
			None
		));

		// creating a pool with non number capacity attribute should fail
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				StakingMock::minimum_nominator_bond(),
				1_000 * UNIT,
				30,
				Default::default(),
			),
			Error::<Runtime>::AttributeValueDecodeFailed
		);

		// set max_pool_capacity attribute to exceed global 20M TNT
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			(20_000_001 * UNIT).to_string().as_bytes().to_vec().try_into().unwrap(),
			None
		));

		// creating a pool with attribute capacity exceeding global capacity should fail
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				StakingMock::minimum_nominator_bond(),
				1_000 * UNIT,
				30,
				Default::default(),
			),
			Error::<Runtime>::AttributeCapacityExceedsGlobalCapacity
		);

		// set max_pool_capacity attribute to decimal value
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			b"100.50".to_vec().try_into().unwrap(),
			None
		));

		// should not be able to decode attribute
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				StakingMock::minimum_nominator_bond(),
				1_000 * UNIT,
				30,
				Default::default(),
			),
			Error::<Runtime>::AttributeValueDecodeFailed
		);

		// set max_pool_capacity attribute to 1M TNT
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			(1_000_000 * UNIT).to_string().as_bytes().to_vec().try_into().unwrap(),
			None
		));

		// creating a pool with capacity exceeding attribute capacity should fail
		assert_noop!(
			Pools::create(
				RuntimeOrigin::signed(11),
				token_id,
				StakingMock::minimum_nominator_bond(),
				1_000_001 * UNIT,
				30,
				Default::default(),
			),
			Error::<Runtime>::CapacityExceeded
		);

		// creating a pool with capacity below attribute capacity should succeed
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(11),
			token_id,
			StakingMock::minimum_nominator_bond(),
			1_000_000 * UNIT,
			30,
			Default::default(),
		));
	})
}

#[test]
fn create_lst_token_works() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		let ed = Balances::minimum_balance();

		mint_pool_token(DEFAULT_TOKEN_ID + 1, 11);

		Balances::make_free_balance_be(&11, StakingMock::minimum_nominator_bond() * 2 + ed * 3);

		// next pool id is 2
		let second_pool_id = NextPoolId::<Runtime>::get();
		let pool_capacity = 1000;

		assert_ok!(Pools::create(
			RuntimeOrigin::signed(11),
			DEFAULT_TOKEN_ID + 1,
			StakingMock::minimum_nominator_bond(),
			pool_capacity,
			50,
			Default::default(),
		));

		bond_extra(11, second_pool_id, StakingMock::minimum_nominator_bond());

		// new token is created with `token_id` of `pool_id`
		let token = FungibleHandler::token_of(LST_COLLECTION_ID, second_pool_id as u128).unwrap();

		assert_eq!(token.supply, StakingMock::minimum_nominator_bond() * 2);
		assert_eq!(token.cap, None);

		assert_eq!(Pools::member_points(second_pool_id, 11), StakingMock::minimum_nominator_bond());

		// check `FungibleHandlerEvent::TokenCreated`
		assert_event_deposited!(FungibleHandlerEvent::TokenCreated {
			collection_id: LST_COLLECTION_ID,
			token_id: second_pool_id as u128,
			issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
			initial_supply: StakingMock::minimum_nominator_bond(),
		});

		// mint another token for new pool
		mint_pool_token(DEFAULT_TOKEN_ID + 2, 12);

		Balances::make_free_balance_be(&12, StakingMock::minimum_nominator_bond() + ed * 2);

		// next pool id is 3
		let third_pool_id = NextPoolId::<Runtime>::get();

		// create another pool
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(12),
			DEFAULT_TOKEN_ID + 2,
			StakingMock::minimum_nominator_bond(),
			pool_capacity,
			50,
			Default::default(),
		));

		// check `FungibleHandlerEvent::TokenCreated`
		assert_event_deposited!(FungibleHandlerEvent::TokenCreated {
			collection_id: LST_COLLECTION_ID,
			token_id: third_pool_id as u128,
			issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
			initial_supply: StakingMock::minimum_nominator_bond(),
		});

		// new token is created with `token_id` of `pool_id`
		let token = FungibleHandler::token_of(LST_COLLECTION_ID, third_pool_id as u128).unwrap();

		assert_eq!(token.supply, StakingMock::minimum_nominator_bond());
		assert_eq!(token.cap, None);

		assert_eq!(
			Pools::member_points(third_pool_id, 12),
			0, // deposit is stored in `Pools::deposit_account_id`
		);
		assert_eq!(
			Pools::member_points(third_pool_id, Pools::deposit_account_id(third_pool_id)),
			StakingMock::minimum_nominator_bond()
		);
	});
}
