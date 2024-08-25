use super::*;
use crate::{mock::*, Event};
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop, traits::Get};
use pallet_balances::Event as BEvent;
use sp_runtime::bounded_btree_map;

use sp_runtime::TokenError;
use sp_std::collections::btree_map::BTreeMap;

pub mod bond;
pub mod bonded_pool;
pub mod create;
pub mod deposit;
pub mod destroy;
pub mod mutate;
pub mod nominate;
pub mod payout_rewards;
pub mod pool_withdraw_unbonded;
pub mod process_payouts;
pub mod set_configs;
pub mod sub_pools;
pub mod unbond;
pub mod unbond_pool;
pub mod withdraw_unbonded;

type FungiblesTrait = <Runtime as Config>::Fungibles;

#[macro_export]
macro_rules! unbonding_pools_with_era {
	($($k:expr => $v:expr),* $(,)?) => {{
		use sp_std::iter::{Iterator, IntoIterator};
		let not_bounded: BTreeMap<_, _> = Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]));
		BoundedBTreeMap::<EraIndex, UnbondPool<T>, TotalUnbondingPools<T>>::try_from(not_bounded).unwrap()
	}};
}

#[macro_export]
macro_rules! member_unbonding_eras {
	($( $any:tt )*) => {{
		let x: BoundedBTreeMap<EraIndex, Balance, MaxUnbonding> = bounded_btree_map!($( $any )*);
		x
	}};
}

pub const DEFAULT_TOKEN_ID: AssetId = 14;
/// The manager for the default pool that is created
pub const DEFAULT_MANAGER: AccountId = 10;
pub const DEFAULT_DURATION: EraIndex = 100;

fn deposit_rewards(r: u128) {
	let b = Balances::free_balance(default_reward_account()).checked_add(r).unwrap();
	Balances::make_free_balance_be(&default_reward_account(), b);
}

/// Get bonded1 account for `pool_id`
fn pool_bonded_account(pool_id: PoolId) -> AccountId {
	Pools::compute_pool_account_id(pool_id, AccountType::Bonded)
}

/// Get reward account for `pool_id`
fn pool_reward_account(pool_id: PoolId) -> AccountId {
	Pools::compute_pool_account_id(pool_id, AccountType::Reward)
}

/// Get bonus account for `pool_id`
fn pool_bonus_account(pool_id: PoolId) -> AccountId {
	Pools::compute_pool_account_id(pool_id, AccountType::Bonus)
}

/// Bond extra
fn bond_extra(account_id: AccountId, pool_id: PoolId, amount: Balance) {
	assert_ok!(Pools::bond(RuntimeOrigin::signed(account_id), pool_id, amount.into()));
}

/// Returns the collection id that holds the pool tokens
fn pool_token_collection_id() -> CollectionId {
	<<Runtime as Config>::PoolCollectionId as Get<_>>::get()
}

/// The collection id of the staked token
fn staked_collection_id() -> CollectionId {
	<<Runtime as Config>::LstCollectionId as Get<_>>::get()
}

/// Shorter way to get a pool
fn get_pool(pool_id: PoolId) -> Option<BondedPool<Runtime>> {
	BondedPool::<Runtime>::get(pool_id)
}

/// Mints lst for a pool that does not exist in storage.
fn mint_points(pool_id: PoolId, amount: Balance) {
	let token_id = pool_id as AssetId;
	let collection_id = staked_collection_id();
	let owner = <Runtime as Config>::LstCollectionOwner::get();

	let mint_params = if Fungibles::token_of(collection_id, token_id).is_none() {
		DefaultMintParams::CreateToken {
			token_id,
			initial_supply: amount,
			account_deposit_count: 0,
			behavior: None,
			cap: None,
			listing_forbidden: true,
			freeze_state: None,
			attributes: Default::default(),
			infusion: Default::default(),
			anyone_can_infuse: false,
			metadata: Default::default(),
			privileged_params: Some(PrivilegedCreateTokenParams {
				requires_deposit: false,
				foreign_params: None,
				depositor: None,
			}),
		}
	} else {
		DefaultMintParams::Mint { token_id, amount, depositor: None }
	};
	<<Runtime as Config>::Fungibles as Fungibles>::mint(
		owner,
		owner,
		collection_id,
		mint_params,
	)
	.unwrap();
}

/// Burns lst for a pool that does not exist in storage.
fn burn_points(pool_id: PoolId, amount: Balance) {
	<<Runtime as Config>::Fungibles as Fungibles>::burn(
		<Runtime as Config>::LstCollectionOwner::get(),
		staked_collection_id(),
		false,
	)
	.unwrap();
}

#[test]
fn test_setup() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(BondedPools::<Runtime>::count(), 1);
		assert_eq!(SubPoolsStorage::<Runtime>::count(), 0);
		assert_eq!(StakingMock::bonding_duration(), 3);
		assert_eq!(pallet_staking::MinCommission::<T>::get(), Perbill::from_percent(1));

		let pool_id = NextPoolId::<Runtime>::get() - 1;
		assert_eq!(pool_id, 0);
		let pool = get_pool(pool_id).unwrap();
		assert_eq!(
			pool,
			BondedPool::<Runtime> {
				id: pool_id,
				inner: BondedPoolInner {
					token_id: DEFAULT_TOKEN_ID,
					state: PoolState::Open,
					capacity: 1_000,
					commission: Commission::default(),
				},
			}
		);
		assert_eq!(pool.points(), 10);
		assert!(UnbondingMembers::<Runtime>::get(pool_id, 10).is_none());
		assert_eq!(
			<Runtime as Config>::Fungibles::owner_of(
				&pool_token_collection_id(),
				&pool.token_id
			),
			Some(10)
		);

		let bonded_account = Pools::compute_pool_account_id(pool_id, AccountType::Bonded);
		let reward_account = Pools::compute_pool_account_id(pool_id, AccountType::Reward);
		let bonus_account = Pools::compute_pool_account_id(pool_id, AccountType::Bonus);

		// the bonded_account should be bonded by the depositor's funds.
		assert_eq!(StakingMock::active_stake(&bonded_account).unwrap(), 10);
		assert_eq!(StakingMock::total_stake(&bonded_account).unwrap(), 10);

		// but not nominating yet.
		assert!(Nominations::get().is_none());

		// reward accounts should have an initial ED in it.
		assert_eq!(Balances::free_balance(reward_account), Balances::minimum_balance());
		assert_eq!(Balances::free_balance(bonus_account), Balances::minimum_balance());
	})
}

#[test]
fn test_setup_without_default_pool() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		// there are no pools
		assert_eq!(BondedPools::<Runtime>::count(), 0);
		let next_pool_id = NextPoolId::<Runtime>::get();
		assert_eq!(next_pool_id, 0);
		assert!(!BondedPools::<Runtime>::contains_key(next_pool_id));

		// pool token is not minted, but collection exists
		assert!(Fungibles::collection_owner_of(pool_token_collection_id()).is_some());
		assert_eq!(
			Fungibles::balance_of(pool_token_collection_id(), DEFAULT_TOKEN_ID, 10),
			0
		);
	})
}

/// Tests that [`Pallet::compute_pool_account_id`] doesn't have collisions with other accounts
#[test]
fn test_compute_pool_account_id_collisions() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let mut values = std::collections::HashSet::new();
		values.insert(<Runtime as Config>::LstCollectionOwner::get());
		for pool_id in 0..512 {
			let account_id = Pools::compute_pool_account_id(pool_id, AccountType::Bonded);
			assert!(!values.contains(&account_id));
			values.insert(account_id);

			let account_id = Pools::compute_pool_account_id(pool_id, AccountType::Reward);
			assert!(!values.contains(&account_id));
			values.insert(account_id);

			let account_id = Pools::compute_pool_account_id(pool_id, AccountType::Bonus);
			assert!(!values.contains(&account_id));
			values.insert(account_id);
		}
	})
}

#[test]
fn test_balance_to_point() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		// Set it according to current minimum config values
		let current_balance = 2500 * UNIT;
		let current_points = 2500 * UNIT;
		let new_funds = UNIT;

		let final_points =
			Pallet::<Runtime>::balance_to_point(current_balance, current_points, new_funds);
		assert_eq!(final_points, new_funds);

		// Use closer to real world values

		// assume total issuance is 1 billion TNT
		let current_balance = (10 ^ 9) * UNIT;
		// cap maximum number of points at 1 billion
		let current_points = (10 ^ 9) * UNIT;
		// assume member points is 100 million lst
		let new_funds = (10 ^ 8) * UNIT;

		let final_points =
			Pallet::<Runtime>::balance_to_point(current_balance, current_points, new_funds);
		assert_eq!(final_points, new_funds);
	})
}

#[test]
fn test_point_to_balance() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		// Set it according to current minimum config values
		let tnt_pool_balance: u128 = 2500 * UNIT;
		let tnt_pool_points: u128 = 2500 * UNIT;
		let tnt_member_points: u128 = UNIT;

		let tnt_final_balance = Pallet::<Runtime>::point_to_balance(
			tnt_pool_balance,
			tnt_pool_points,
			tnt_member_points,
		);
		assert_eq!(tnt_final_balance, tnt_member_points);

		// Use closer to real world values

		// assume total issuance is 1 billion TNT
		let tnt_pool_balance: u128 = (10 ^ 9) * UNIT;
		// cap maximum number of points at 1 billion
		let tnt_pool_points: u128 = (10 ^ 9) * UNIT;
		// assume member points is 100 million lst
		let tnt_member_points: u128 = (10 ^ 8) * UNIT;

		let tnt_final_balance = Pallet::<Runtime>::point_to_balance(
			tnt_pool_balance,
			tnt_pool_points,
			tnt_member_points,
		);
		assert_eq!(tnt_final_balance, tnt_member_points);
	})
}

#[test]
fn test_withdraw_free_balance() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let destination = 200;
		let amount = 10 * UNIT;

		// regular accounts can't call it
		assert_noop!(
			Pools::withdraw_free_balance(RuntimeOrigin::signed(1), pool_id, destination, amount),
			frame_support::error::BadOrigin
		);

		// it doesn't work without free balance
		assert_noop!(
			Pools::withdraw_free_balance(RuntimeOrigin::root(), pool_id, destination, amount),
			DispatchError::Token(TokenError::FundsUnavailable)
		);

		// add free balance and it works
		let pool_account = Pools::compute_pool_account_id(pool_id, AccountType::Bonded);
		Balances::make_free_balance_be(&pool_account, amount + UNIT);
		assert_eq!(Balances::total_balance(&destination), 0);
		assert_eq!(Balances::free_balance(pool_account), amount + UNIT);
		Pools::withdraw_free_balance(RuntimeOrigin::root(), pool_id, destination, amount).unwrap();
		assert_eq!(Balances::free_balance(pool_account), UNIT);
		assert_eq!(Balances::total_balance(&destination), amount);
	})
}
