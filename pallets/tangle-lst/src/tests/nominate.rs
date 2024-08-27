use super::*;
use frame_support::assert_err;
use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_support::traits::fungible::InspectFreeze;


	#[test]
	fn nominate_works() {
		ExtBuilder::default().build_and_execute(|| {
			// Depositor can't nominate
			assert_noop!(
				Lst::nominate(RuntimeOrigin::signed(10), 1, vec![21]),
				Error::<Runtime>::NotNominator
			);

			// bouncer can't nominate
			assert_noop!(
				Lst::nominate(RuntimeOrigin::signed(902), 1, vec![21]),
				Error::<Runtime>::NotNominator
			);

			// Root can nominate
			assert_ok!(Lst::nominate(RuntimeOrigin::signed(900), 1, vec![21]));
			assert_eq!(Nominations::get().unwrap(), vec![21]);

			// Nominator can nominate
			assert_ok!(Lst::nominate(RuntimeOrigin::signed(901), 1, vec![31]));
			assert_eq!(Nominations::get().unwrap(), vec![31]);

			// Can't nominate for a pool that doesn't exist
			assert_noop!(
				Lst::nominate(RuntimeOrigin::signed(902), 123, vec![21]),
				Error::<Runtime>::PoolNotFound
			);
		});
	}


	#[test]
	fn set_state_works() {
		ExtBuilder::default().build_and_execute(|| {
			// Given
			assert_ok!(BondedPool::<Runtime>::get(1).unwrap().ok_to_be_open());

			// Only the root and bouncer can change the state when the pool is ok to be open.
			assert_noop!(
				Lst::set_state(RuntimeOrigin::signed(10), 1, PoolState::Blocked),
				Error::<Runtime>::CanNotChangeState
			);
			assert_noop!(
				Lst::set_state(RuntimeOrigin::signed(901), 1, PoolState::Blocked),
				Error::<Runtime>::CanNotChangeState
			);

			// Root can change state
			assert_ok!(Lst::set_state(RuntimeOrigin::signed(900), 1, PoolState::Blocked));

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::StateChanged { pool_id: 1, new_state: PoolState::Blocked }
				]
			);

			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Blocked);

			// bouncer can change state
			assert_ok!(Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Destroying));
			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

			// If the pool is destroying, then no one can set state
			assert_noop!(
				Lst::set_state(RuntimeOrigin::signed(900), 1, PoolState::Blocked),
				Error::<Runtime>::CanNotChangeState
			);
			assert_noop!(
				Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Blocked),
				Error::<Runtime>::CanNotChangeState
			);

			// If the pool is not ok to be open, then anyone can set it to destroying

			// Given
			unsafe_set_state(1, PoolState::Open);
			// slash the pool to the point that `max_points_to_balance` ratio is
			// surpassed. Making this pool destroyable by anyone.
			StakingMock::slash_by(1, 10);

			// When
			assert_ok!(Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Destroying));
			// Then
			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

			// Given
			Currency::make_free_balance_be(&default_bonded_account(), Balance::MAX / 10);
			unsafe_set_state(1, PoolState::Open);
			// When
			assert_ok!(Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Destroying));
			// Then
			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

			// If the pool is not ok to be open, it cannot be permissionlessly set to a state that
			// isn't destroying
			unsafe_set_state(1, PoolState::Open);
			assert_noop!(
				Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Blocked),
				Error::<Runtime>::CanNotChangeState
			);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying },
					Event::PoolSlashed { pool_id: 1, balance: 0 },
					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying },
					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying }
				]
			);
		});
	}


	#[test]
	fn set_metadata_works() {
		ExtBuilder::default().build_and_execute(|| {
			// Root can set metadata
			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1]));
			assert_eq!(Metadata::<Runtime>::get(1), vec![1, 1]);

			// bouncer can set metadata
			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(902), 1, vec![2, 2]));
			assert_eq!(Metadata::<Runtime>::get(1), vec![2, 2]);

			// Depositor can't set metadata
			assert_noop!(
				Lst::set_metadata(RuntimeOrigin::signed(10), 1, vec![3, 3]),
				Error::<Runtime>::DoesNotHavePermission
			);

			// Nominator can't set metadata
			assert_noop!(
				Lst::set_metadata(RuntimeOrigin::signed(901), 1, vec![3, 3]),
				Error::<Runtime>::DoesNotHavePermission
			);

			// Metadata cannot be longer than `MaxMetadataLen`
			assert_noop!(
				Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1, 1]),
				Error::<Runtime>::MetadataExceedsMaxLen
			);
		});
	}
