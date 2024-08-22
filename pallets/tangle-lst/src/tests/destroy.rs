use super::*;

#[test]
fn test_destroy_works() {
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// only the pool token holder can call destroy
		assert_noop!(
			Pools::destroy(RuntimeOrigin::signed(11), pool_id),
			Error::<Runtime>::DoesNotHavePermission
		);

		// call the destroy extrinsic
		assert_ok!(Pools::destroy(RuntimeOrigin::signed(10), pool_id));
		assert_last_event!(Event::StateChanged { pool_id, new_state: PoolState::Destroying });

		assert_eq!(
			BondedPool::<Runtime>::get(pool_id).unwrap(),
			BondedPool {
				id: pool_id,
				inner: BondedPoolInner {
					token_id: DEFAULT_TOKEN_ID,
					state: PoolState::Destroying,
					capacity: 1_000,
					commission: Commission::default(),
				}
			}
		);

		// sanity check, no new members should be able to join
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(100), pool_id, 1000.into()),
			Error::<Runtime>::NotOpen
		);
	})
}

#[test]
fn test_destroy_burned_pool_token() {
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// call the destroy extrinsis with non-owner should fail
		assert_noop!(
			Pools::destroy(RuntimeOrigin::signed(12345), pool_id),
			Error::<Runtime>::DoesNotHavePermission
		);

		// burn the pool token
		assert_ok!(<Runtime as Config>::FungibleHandler::burn(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<_>>::get(),
		));

		// `destroy` should be permissionless now
		assert_ok!(Pools::destroy(RuntimeOrigin::signed(12345), pool_id));

		assert_eq!(
			BondedPool::<Runtime>::get(pool_id).unwrap(),
			BondedPool {
				id: pool_id,
				inner: BondedPoolInner {
					token_id: DEFAULT_TOKEN_ID,
					state: PoolState::Destroying,
					capacity: 1_000,
					commission: Commission::default(),
				}
			}
		);
	});
}
