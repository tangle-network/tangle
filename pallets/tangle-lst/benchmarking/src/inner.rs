//! Benchmarks for the nomination Lst coupled with the staking and bags list pallets.

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
	MaxPools, Metadata, MinCreateBond, MinJoinBond, Pallet as Lst, PoolId, PoolRoles, PoolState,
	RewardPools, SubPoolsStorage,
};
use sp_runtime::{
	traits::{Bounded, StaticLookup, Zero},
	Perbill,
};
use sp_staking::EraIndex;
use sp_staking::StakingInterface;
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

pub struct Pallet<T: Config>(Lst<T>);

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

	Lst::<T>::create(
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
		Lst::<T>::set_commission(
			RuntimeOrigin::Signed(pool_creator.clone()).into(),
			pool_id,
			Some((c, pool_creator.clone())),
		)
		.expect("pool just created, commission can be set by root; qed");
	}

	let pool_account = pallet_tangle_lst::BondedPools::<T>::iter()
		.find(|(_, bonded_pool)| bonded_pool.roles.depositor == pool_creator)
		.map(|(pool_id, _)| Lst::<T>::create_bonded_account(pool_id))
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
		let origin_weight = Lst::<T>::depositor_min_bond() * 2u32.into();

		// setup the worst case list scenario.
		let joiner_free = CurrencyOf::<T>::minimum_balance() * 100u32.into();

		let joiner: T::AccountId
			= create_funded_user_with_balance::<T>("joiner", 0, joiner_free);

		whitelist_account!(joiner);
	}: _(RuntimeOrigin::Signed(joiner.clone()), 100u32.into(), 1)
	verify {
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), joiner_free);
	}

	unbond {
		let member_id : T::AccountId = account("member", USER_SEED, 0);
		whitelist_account!(member_id);
		// create a pool with a single member
		let min_join_bond = MinJoinBond::<T>::get().max(CurrencyOf::<T>::minimum_balance());
		let min_create_bond = Lst::<T>::depositor_min_bond();
		let (depositor, pool_account) = create_pool_account::<T>(0, min_create_bond, None);
		Lst::<T>::join(RuntimeOrigin::Signed(member_id.clone()).into(), min_join_bond, 1)
			.unwrap();
		let member_id_lookup = T::Lookup::unlookup(member_id.clone());
	}: _(RuntimeOrigin::Signed(member_id.clone()), member_id_lookup, 1_u32.into(), 100_u32.into())

	pool_withdraw_unbonded {
		let s in 0 .. MAX_SPANS;

		let min_create_bond = Lst::<T>::depositor_min_bond();
		let (depositor, pool_account) = create_pool_account::<T>(0, min_create_bond, None);

		// Add a new member
		let min_join_bond = MinJoinBond::<T>::get().max(CurrencyOf::<T>::minimum_balance());
		let joiner = create_funded_user_with_balance::<T>("joiner", 0, min_join_bond * 2u32.into());
		Lst::<T>::join(RuntimeOrigin::Signed(joiner.clone()).into(), min_join_bond, 1)
			.unwrap();

		// Sanity check join worked
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond);

		// Unbond the new member
		Lst::<T>::fully_unbond(RuntimeOrigin::Signed(joiner.clone()).into(), joiner.clone(), 1_u32.into()).unwrap();

		// Sanity check that unbond worked
		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);
		// Set the current era
		pallet_staking::CurrentEra::<T>::put(EraIndex::max_value());

		// Add `s` count of slashing spans to storage.
		pallet_staking::benchmarking::add_slashing_spans::<T>(&pool_account, s);
		whitelist_account!(pool_account);
	}: _(RuntimeOrigin::Signed(pool_account.clone()), 1, s)
	verify {
		// The joiners funds didn't change
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond);
		// The unlocking chunk was removed
		assert_eq!(pallet_staking::Ledger::<T>::get(pool_account).unwrap().unlocking.len(), 0);
	}

	create {
		let min_create_bond = Lst::<T>::depositor_min_bond();
		let depositor: T::AccountId = account("depositor", USER_SEED, 0);
		let depositor_lookup = T::Lookup::unlookup(depositor.clone());

		// Give the depositor some balance to bond
		// it needs to transfer min balance to reward account as well so give additional min balance.
		CurrencyOf::<T>::make_free_balance_be(&depositor, min_create_bond + CurrencyOf::<T>::minimum_balance() * 2u32.into());
		// Make sure no Lst exist at a pre-condition for our verify checks
		assert_eq!(RewardPools::<T>::count(), 0);
		assert_eq!(BondedPools::<T>::count(), 0);

		whitelist_account!(depositor);
	}: _(
			RuntimeOrigin::Signed(depositor.clone()),
			min_create_bond,
			depositor_lookup.clone(),
			depositor_lookup.clone(),
			depositor_lookup,
			Default::default()
		)
	verify {
		assert_eq!(RewardPools::<T>::count(), 1);
		assert_eq!(BondedPools::<T>::count(), 1);
		let (_, new_pool) = BondedPools::<T>::iter().next().unwrap();
		assert_eq!(
			new_pool,
			BondedPoolInner {
				commission: Commission::default(),
				roles: PoolRoles {
					depositor: depositor.clone(),
					root: Some(depositor.clone()),
					nominator: Some(depositor.clone()),
					bouncer: Some(depositor.clone()),
				},
				state: PoolState::Open,
				metadata: Default::default(),
			}
		);
	}

	nominate {
		let n in 1 .. MaxNominationsOf::<T>::get();

		// Create a pool
		let min_create_bond = Lst::<T>::depositor_min_bond() * 2u32.into();
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
				roles: PoolRoles {
					depositor: depositor.clone(),
					root: Some(depositor.clone()),
					nominator: Some(depositor.clone()),
					bouncer: Some(depositor.clone()),
				},
				state: PoolState::Open,
				metadata: Default::default(),
			}
		);
	}

	set_metadata {
		let n in 1 .. <T as pallet_tangle_lst::Config>::MaxMetadataLen::get();

		// Create a pool
		let (depositor, pool_account) = create_pool_account::<T>(0, Lst::<T>::depositor_min_bond() * 2u32.into(), None);

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
		assert_eq!(MaxPoolMembersPerLst::<T>::get(), Some(u32::MAX));
		assert_eq!(GlobalMaxCommission::<T>::get(), Some(Perbill::max_value()));
	}

	update_roles {
		let first_id = pallet_tangle_lst::LastPoolId::<T>::get() + 1;
		let (root, _) = create_pool_account::<T>(0, Lst::<T>::depositor_min_bond() * 2u32.into(), None);
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
		let (depositor, pool_account) = create_pool_account::<T>(0, Lst::<T>::depositor_min_bond() * 2u32.into(), None);

		// Nominate with the pool.
		 let validators: Vec<_> = (0..MaxNominationsOf::<T>::get())
			.map(|i| account("stash", USER_SEED, i))
			.collect();

		assert!(T::Staking::nominations(Lst::from(pool_account.clone())).is_some());

		whitelist_account!(depositor);
	}:_(RuntimeOrigin::Signed(depositor.clone()), 1)
	verify {
		assert!(T::Staking::nominations(Lst::from(pool_account.clone())).is_none());
	}

	set_commission {
		// Create a pool - do not set a commission yet.
		let (depositor, pool_account) = create_pool_account::<T>(0, Lst::<T>::depositor_min_bond() * 2u32.into(), None);
		// set a max commission
		Lst::<T>::set_commission_max(RuntimeOrigin::Signed(depositor.clone()).into(), 1u32.into(), Perbill::from_percent(50)).unwrap();
		// set a change rate
		Lst::<T>::set_commission_change_rate(RuntimeOrigin::Signed(depositor.clone()).into(), 1u32.into(), CommissionChangeRate {
			max_increase: Perbill::from_percent(20),
			min_delay: 0u32.into(),
		}).unwrap();
		// set a claim permission to an account.
		Lst::<T>::set_commission_claim_permission(
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
