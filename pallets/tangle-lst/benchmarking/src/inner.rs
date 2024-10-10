//! Benchmarks for the nomination pools coupled with the staking and bags list pallets.

use alloc::{vec, vec::Vec};
use frame_benchmarking::v1::{account, whitelist_account};
use frame_election_provider_support::SortedListProvider;
use frame_support::traits::Currency;
use frame_support::{
	assert_ok, ensure,
	traits::{
		fungible::{Inspect, Mutate, Unbalanced},
		tokens::Preservation,
		Get, Imbalance,
	},
};
use frame_system::RawOrigin as RuntimeOrigin;
use pallet_staking::MaxNominationsOf;
use pallet_tangle_lst::{
	BalanceOf, BondExtra, BondedPoolInner, BondedPools, ClaimPermission, ClaimPermissions,
	Commission, CommissionChangeRate, CommissionClaimPermission, ConfigOp, GlobalMaxCommission,
	MaxPools, Metadata, MinCreateBond, MinJoinBond, Pallet as Pools, PoolId, PoolRoles, PoolState,
	RewardPools, SubPoolsStorage,
};
use sp_runtime::{
	traits::{Bounded, StaticLookup, Zero},
	Perbill,
};
use sp_staking::EraIndex;
// `frame_benchmarking::benchmarks!` macro needs this
use pallet_tangle_lst::Call;

type CurrencyOf<T> = <T as pallet_tangle_lst::Config>::Currency;

const USER_SEED: u32 = 0;
const MAX_SPANS: u32 = 100;

pub(crate) type VoterBagsListInstance = pallet_bags_list::Instance1;
pub trait Config:
	pallet_tangle_lst::Config + pallet_staking::Config + pallet_bags_list::Config<VoterBagsListInstance>
{
}

pub struct Pallet<T: Config>(Pools<T>);

fn create_funded_user_with_balance<T: pallet_tangle_lst::Config>(
	string: &'static str,
	n: u32,
	balance: BalanceOf<T>,
) -> T::AccountId {
	let user = account(string, n, USER_SEED);
	T::Currency::make_free_balance_be(&user, balance);
	user
}

// Create a bonded pool account, bonding `balance` and giving the account `balance * 2` free
// balance.
fn create_pool_account<T: pallet_tangle_lst::Config>(
	n: u32,
	balance: BalanceOf<T>,
	commission: Option<Perbill>,
) -> (T::AccountId, T::AccountId) {
	let ed = CurrencyOf::<T>::minimum_balance();
	let pool_creator: T::AccountId =
		create_funded_user_with_balance::<T>("pool_creator", n, ed + balance * 2u32.into());
	let pool_creator_lookup = T::Lookup::unlookup(pool_creator.clone());

	Pools::<T>::create(
		RuntimeOrigin::Signed(pool_creator.clone()).into(),
		balance,
		pool_creator_lookup.clone(),
		pool_creator_lookup.clone(),
		pool_creator_lookup,
		Default::default(),
	)
	.unwrap();

	if let Some(c) = commission {
		let pool_id = pallet_tangle_lst::LastPoolId::<T>::get();
		Pools::<T>::set_commission(
			RuntimeOrigin::Signed(pool_creator.clone()).into(),
			pool_id,
			Some((c, pool_creator.clone())),
		)
		.expect("pool just created, commission can be set by root; qed");
	}

	let pool_account = pallet_tangle_lst::BondedPools::<T>::iter()
		.find(|(_, bonded_pool)| bonded_pool.roles.depositor == pool_creator)
		.map(|(pool_id, _)| Pools::<T>::create_bonded_account(pool_id))
		.expect("pool_creator created a pool above");

	(pool_creator, pool_account)
}

fn vote_to_balance<T: pallet_tangle_lst::Config>(vote: u64) -> Result<BalanceOf<T>, &'static str> {
	vote.try_into().map_err(|_| "could not convert u64 to Balance")
}

frame_benchmarking::benchmarks! {
	where_clause {
		where
			T: pallet_staking::Config,
			pallet_staking::BalanceOf<T>: From<u128>,
			BalanceOf<T>: Into<u128>,
	}

	join {
		let origin_weight = Pools::<T>::depositor_min_bond() * 2u32.into();

		// setup the worst case list scenario.
		let joiner_free = CurrencyOf::<T>::minimum_balance() * 100u32.into();

		let joiner: T::AccountId
			= create_funded_user_with_balance::<T>("joiner", 0, joiner_free);

		whitelist_account!(joiner);
	}: _(RuntimeOrigin::Signed(joiner.clone()), 100u32.into(), 1)
	verify {
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), joiner_free);
	}

	bond_extra_transfer {
		let origin_weight = Pools::<T>::depositor_min_bond() * 2u32.into();
		let scenario = ListScenario::<T>::new(origin_weight, true)?;
		let extra = scenario.dest_weight - origin_weight;

		// creator of the src pool will bond-extra, bumping itself to dest bag.

	}: bond_extra(RuntimeOrigin::Signed(scenario.creator1.clone()), BondExtra::FreeBalance(extra))
	verify {
		assert!(
			T::StakeAdapter::active_stake(Pool::from(scenario.origin1)) >=
			scenario.dest_weight
		);
	}

	bond_extra_other {
		let claimer: T::AccountId = account("claimer", USER_SEED + 4, 0);

		let origin_weight = Pools::<T>::depositor_min_bond() * 2u32.into();
		let scenario = ListScenario::<T>::new(origin_weight, true)?;
		let extra = (scenario.dest_weight - origin_weight).max(CurrencyOf::<T>::minimum_balance());

		// set claim preferences to `PermissionlessAll` to any account to bond extra on member's behalf.
		let _ = Pools::<T>::set_claim_permission(RuntimeOrigin::Signed(scenario.creator1.clone()).into(), ClaimPermission::PermissionlessAll);

		// transfer exactly `extra` to the depositor of the src pool (1),
		let reward_account1 = Pools::<T>::generate_reward_account(1);
		assert!(extra >= CurrencyOf::<T>::minimum_balance());
		let _ = CurrencyOf::<T>::mint_into(&reward_account1, extra);

	}: _(RuntimeOrigin::Signed(claimer), T::Lookup::unlookup(scenario.creator1.clone()), BondExtra::Rewards)
	verify {
		 // commission of 50% deducted here.
		assert!(
			T::StakeAdapter::active_stake(Pool::from(scenario.origin1)) >=
			scenario.dest_weight / 2u32.into()
		);
	}

	claim_payout {
		let claimer: T::AccountId = account("claimer", USER_SEED + 4, 0);
		let commission = Perbill::from_percent(50);
		let origin_weight = Pools::<T>::depositor_min_bond() * 2u32.into();
		let ed = CurrencyOf::<T>::minimum_balance();
		let (depositor, pool_account) = create_pool_account::<T>(0, origin_weight, Some(commission));
		let reward_account = Pools::<T>::generate_reward_account(1);

		// Send funds to the reward account of the pool
		CurrencyOf::<T>::set_balance(&reward_account, ed + origin_weight);

		// set claim preferences to `PermissionlessAll` so any account can claim rewards on member's
		// behalf.
		let _ = Pools::<T>::set_claim_permission(RuntimeOrigin::Signed(depositor.clone()).into(), ClaimPermission::PermissionlessAll);

		// Sanity check
		assert_eq!(
			CurrencyOf::<T>::balance(&depositor),
			origin_weight
		);

		whitelist_account!(depositor);
	}:claim_payout_other(RuntimeOrigin::Signed(claimer), depositor.clone())
	verify {
		assert_eq!(
			CurrencyOf::<T>::balance(&depositor),
			origin_weight + commission * origin_weight
		);
		assert_eq!(
			CurrencyOf::<T>::balance(&reward_account),
			ed + commission * origin_weight
		);
	}


	unbond {
		// The weight the nominator will start at. The value used here is expected to be
		// significantly higher than the first position in a list (e.g. the first bag threshold).
		let origin_weight = Pools::<T>::depositor_min_bond() * 200u32.into();
		let scenario = ListScenario::<T>::new(origin_weight, false)?;
		let amount = origin_weight - scenario.dest_weight;

		let scenario = scenario.add_joiner(amount);
		let member_id = scenario.origin1_member.unwrap().clone();
		let member_id_lookup = T::Lookup::unlookup(member_id.clone());
		let all_points = PoolMembers::<T>::get(&member_id).unwrap().points;
		whitelist_account!(member_id);
	}: _(RuntimeOrigin::Signed(member_id.clone()), member_id_lookup, all_points)
	verify {
		let bonded_after = T::StakeAdapter::active_stake(Pool::from(scenario.origin1));
		// We at least went down to the destination bag
		assert!(bonded_after <= scenario.dest_weight);
		let member = PoolMembers::<T>::get(
			&member_id
		)
		.unwrap();
		assert_eq!(
			member.unbonding_eras.keys().cloned().collect::<Vec<_>>(),
			vec![0 + T::StakeAdapter::bonding_duration()]
		);
		assert_eq!(
			member.unbonding_eras.values().cloned().collect::<Vec<_>>(),
			vec![all_points]
		);
	}

	pool_withdraw_unbonded {
		let s in 0 .. MAX_SPANS;

		let min_create_bond = Pools::<T>::depositor_min_bond();
		let (depositor, pool_account) = create_pool_account::<T>(0, min_create_bond, None);

		// Add a new member
		let min_join_bond = MinJoinBond::<T>::get().max(CurrencyOf::<T>::minimum_balance());
		let joiner = create_funded_user_with_balance::<T>("joiner", 0, min_join_bond * 2u32.into());
		Pools::<T>::join(RuntimeOrigin::Signed(joiner.clone()).into(), min_join_bond, 1)
			.unwrap();

		// Sanity check join worked
		assert_eq!(
			T::StakeAdapter::active_stake(Pool::from(pool_account.clone())),
			min_create_bond + min_join_bond
		);
		assert_eq!(CurrencyOf::<T>::balance(&joiner), min_join_bond);

		// Unbond the new member
		Pools::<T>::fully_unbond(RuntimeOrigin::Signed(joiner.clone()).into(), joiner.clone()).unwrap();

		// Sanity check that unbond worked
		assert_eq!(
			T::StakeAdapter::active_stake(Pool::from(pool_account.clone())),
			min_create_bond
		);
		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);
		// Set the current era
		pallet_staking::CurrentEra::<T>::put(EraIndex::max_value());

		// Add `s` count of slashing spans to storage.
		pallet_staking::benchmarking::add_slashing_spans::<T>(&pool_account, s);
		whitelist_account!(pool_account);
	}: _(RuntimeOrigin::Signed(pool_account.clone()), 1, s)
	verify {
		// The joiners funds didn't change
		assert_eq!(CurrencyOf::<T>::balance(&joiner), min_join_bond);
		// The unlocking chunk was removed
		assert_eq!(pallet_staking::Ledger::<T>::get(pool_account).unwrap().unlocking.len(), 0);
	}

	create {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let depositor: T::AccountId = account("depositor", USER_SEED, 0);
		let depositor_lookup = T::Lookup::unlookup(depositor.clone());

		// Give the depositor some balance to bond
		// it needs to transfer min balance to reward account as well so give additional min balance.
		CurrencyOf::<T>::set_balance(&depositor, min_create_bond + CurrencyOf::<T>::minimum_balance() * 2u32.into());
		// Make sure no Pools exist at a pre-condition for our verify checks
		assert_eq!(RewardPools::<T>::count(), 0);
		assert_eq!(BondedPools::<T>::count(), 0);

		whitelist_account!(depositor);
	}: _(
			RuntimeOrigin::Signed(depositor.clone()),
			min_create_bond,
			depositor_lookup.clone(),
			depositor_lookup.clone(),
			depositor_lookup
		)
	verify {
		assert_eq!(RewardPools::<T>::count(), 1);
		assert_eq!(BondedPools::<T>::count(), 1);
		let (_, new_pool) = BondedPools::<T>::iter().next().unwrap();
		assert_eq!(
			new_pool,
			BondedPoolInner {
				commission: Commission::default(),
				member_counter: 1,
				points: min_create_bond,
				roles: PoolRoles {
					depositor: depositor.clone(),
					root: Some(depositor.clone()),
					nominator: Some(depositor.clone()),
					bouncer: Some(depositor.clone()),
				},
				state: PoolState::Open,
			}
		);
		assert_eq!(
			T::StakeAdapter::active_stake(Pool::from(Pools::<T>::generate_bonded_account(1))),
			min_create_bond
		);
	}

	nominate {
		let n in 1 .. MaxNominationsOf::<T>::get();

		// Create a pool
		let min_create_bond = Pools::<T>::depositor_min_bond() * 2u32.into();
		let (depositor, pool_account) = create_pool_account::<T>(0, min_create_bond, None);

		// Create some accounts to nominate. For the sake of benchmarking they don't need to be
		// actual validators
		 let validators: Vec<_> = (0..n)
			.map(|i| account("stash", USER_SEED, i))
			.collect();

		whitelist_account!(depositor);
	}:_(RuntimeOrigin::Signed(depositor.clone()), 1, validators)
	verify {
		assert_eq!(RewardPools::<T>::count(), 1);
		assert_eq!(BondedPools::<T>::count(), 1);
		let (_, new_pool) = BondedPools::<T>::iter().next().unwrap();
		assert_eq!(
			new_pool,
			BondedPoolInner {
				commission: Commission::default(),
				member_counter: 1,
				points: min_create_bond,
				roles: PoolRoles {
					depositor: depositor.clone(),
					root: Some(depositor.clone()),
					nominator: Some(depositor.clone()),
					bouncer: Some(depositor.clone()),
				},
				state: PoolState::Open,
			}
		);
		assert_eq!(
			T::StakeAdapter::active_stake(Pool::from(Pools::<T>::generate_bonded_account(1))),
			min_create_bond
		);
	}

	set_state {
		// Create a pool
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let (depositor, pool_account) = create_pool_account::<T>(0, min_create_bond, None);
		BondedPools::<T>::mutate(&1, |maybe_pool| {
			// Force the pool into an invalid state
			maybe_pool.as_mut().map(|pool| pool.points = min_create_bond * 10u32.into());
		});

		let caller = account("caller", 0, USER_SEED);
		whitelist_account!(caller);
	}:_(RuntimeOrigin::Signed(caller), 1, PoolState::Destroying)
	verify {
		assert_eq!(BondedPools::<T>::get(1).unwrap().state, PoolState::Destroying);
	}

	set_metadata {
		let n in 1 .. <T as pallet_tangle_lst::Config>::MaxMetadataLen::get();

		// Create a pool
		let (depositor, pool_account) = create_pool_account::<T>(0, Pools::<T>::depositor_min_bond() * 2u32.into(), None);

		// Create metadata of the max possible size
		let metadata: Vec<u8> = (0..n).map(|_| 42).collect();

		whitelist_account!(depositor);
	}:_(RuntimeOrigin::Signed(depositor), 1, metadata.clone())
	verify {
		assert_eq!(Metadata::<T>::get(&1), metadata);
	}

	set_configs {
	}:_(
		RuntimeOrigin::Root,
		ConfigOp::Set(BalanceOf::<T>::max_value()),
		ConfigOp::Set(BalanceOf::<T>::max_value()),
		ConfigOp::Set(u32::MAX),
		ConfigOp::Set(u32::MAX),
		ConfigOp::Set(u32::MAX),
		ConfigOp::Set(Perbill::max_value())
	) verify {
		assert_eq!(MinJoinBond::<T>::get(), BalanceOf::<T>::max_value());
		assert_eq!(MinCreateBond::<T>::get(), BalanceOf::<T>::max_value());
		assert_eq!(MaxPools::<T>::get(), Some(u32::MAX));
		assert_eq!(MaxPoolMembers::<T>::get(), Some(u32::MAX));
		assert_eq!(MaxPoolMembersPerPool::<T>::get(), Some(u32::MAX));
		assert_eq!(GlobalMaxCommission::<T>::get(), Some(Perbill::max_value()));
	}

	update_roles {
		let first_id = pallet_tangle_lst::LastPoolId::<T>::get() + 1;
		let (root, _) = create_pool_account::<T>(0, Pools::<T>::depositor_min_bond() * 2u32.into(), None);
		let random: T::AccountId = account("but is anything really random in computers..?", 0, USER_SEED);
	}:_(
		RuntimeOrigin::Signed(root.clone()),
		first_id,
		ConfigOp::Set(random.clone()),
		ConfigOp::Set(random.clone()),
		ConfigOp::Set(random.clone())
	) verify {
		assert_eq!(
			pallet_tangle_lst::BondedPools::<T>::get(first_id).unwrap().roles,
			pallet_tangle_lst::PoolRoles {
				depositor: root,
				nominator: Some(random.clone()),
				bouncer: Some(random.clone()),
				root: Some(random),
			},
		)
	}

	chill {
		// Create a pool
		let (depositor, pool_account) = create_pool_account::<T>(0, Pools::<T>::depositor_min_bond() * 2u32.into(), None);

		// Nominate with the pool.
		 let validators: Vec<_> = (0..MaxNominationsOf::<T>::get())
			.map(|i| account("stash", USER_SEED, i))
			.collect();

		assert_ok!(T::StakeAdapter::nominate(Pool::from(pool_account.clone()), validators));
		assert!(T::StakeAdapter::nominations(Pool::from(pool_account.clone())).is_some());

		whitelist_account!(depositor);
	}:_(RuntimeOrigin::Signed(depositor.clone()), 1)
	verify {
		assert!(T::StakeAdapter::nominations(Pool::from(pool_account.clone())).is_none());
	}

	set_commission {
		// Create a pool - do not set a commission yet.
		let (depositor, pool_account) = create_pool_account::<T>(0, Pools::<T>::depositor_min_bond() * 2u32.into(), None);
		// set a max commission
		Pools::<T>::set_commission_max(RuntimeOrigin::Signed(depositor.clone()).into(), 1u32.into(), Perbill::from_percent(50)).unwrap();
		// set a change rate
		Pools::<T>::set_commission_change_rate(RuntimeOrigin::Signed(depositor.clone()).into(), 1u32.into(), CommissionChangeRate {
			max_increase: Perbill::from_percent(20),
			min_delay: 0u32.into(),
		}).unwrap();
		// set a claim permission to an account.
		Pools::<T>::set_commission_claim_permission(
			RuntimeOrigin::Signed(depositor.clone()).into(),
			1u32.into(),
			Some(CommissionClaimPermission::Account(depositor.clone()))
		).unwrap();

	}:_(RuntimeOrigin::Signed(depositor.clone()), 1u32.into(), Some((Perbill::from_percent(20), depositor.clone())))
	verify {
		assert_eq!(BondedPools::<T>::get(1).unwrap().commission, Commission {
			current: Some((Perbill::from_percent(20), depositor.clone())),
			max: Some(Perbill::from_percent(50)),
			change_rate: Some(CommissionChangeRate {
					max_increase: Perbill::from_percent(20),
					min_delay: 0u32.into()
			}),
			throttle_from: Some(1u32.into()),
			claim_permission: Some(CommissionClaimPermission::Account(depositor)),
		});
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
