//! Benchmarks for the nomination pools coupled with the staking an bags list pallets.
#![allow(dead_code)] // TODO : Remove after EFI-2849
use frame_benchmarking::v2::*;
use frame_election_provider_support::{BoundedSupportsOf, ElectionProvider, SortedListProvider};
use frame_support::{
	assert_ok, ensure,
	traits::{Currency, Get, Hooks, OnFinalize},
	BoundedVec,
};
use frame_system::{
	pallet_prelude::BlockNumberFor, Pallet as FrameSystem, RawOrigin as RuntimeOrigin,
};
use pallet_nomination_pools::{
	AccountType, BalanceOf, BondedPools, Call, CollectionIdOf, Commission,
	CommissionChangeRate, ConfigOp, EraPayout, EraPayoutInfo,
	GlobalMaxCommission, MinCreateBond, MinJoinBond, NextPoolId, Pallet as Pools, PoolBonusInfoOf,
	PoolBonusInfos, PoolId, PoolMutationOf, PoolState, SubPoolsStorage, TokenBalanceOf, TokenIdOf,
	UnbondingMembers,
};
use pallet_staking::{Exposure, IndividualExposure, MaxNominationsOf, Pallet as Staking};
use sp_runtime::{
	traits::{Bounded, One, StaticLookup, Zero},
	Perbill, SaturatedConversion, Saturating,
};
use sp_staking::{currency_to_vote::CurrencyToVote, EraIndex, SessionIndex, StakingInterface};
use sp_std::{vec, vec::Vec};

type CurrencyOf<T> = <T as pallet_staking::Config>::Currency;

const USER_SEED: u32 = 0;
const TOKEN_SEED: u32 = 315;
const MAX_SPANS: u32 = 100;
const UNIT: u128 = 1_000_000_000_000_000_000;
/// Max number of pools to benchmark
const MAX_POOLS: u32 = 100;
const MAX_USERS_PER_POOL: u32 = 100;

/// Helper to run to a specific block.
pub fn run_to_block<T>(n: BlockNumberFor<T>)
where
	T: frame_system::Config + pallet_nomination_pools::Config,
{
	let current_block = FrameSystem::<T>::block_number();
	assert!(n > current_block);
	while FrameSystem::<T>::block_number() < n {
		<pallet_nomination_pools::Pallet<T> as OnFinalize<BlockNumberFor<T>>>::on_finalize(
			FrameSystem::<T>::block_number(),
		);
		FrameSystem::<T>::set_block_number(FrameSystem::<T>::block_number() + One::one());
		pallet_nomination_pools::Pallet::<T>::on_initialize(FrameSystem::<T>::block_number());
	}
}

fn pool_creator<
	T: pallet_nomination_pools::Config
		+ FungibleHandlerConfig<
			CollectionId = CollectionIdOf<T>,
			TokenId = TokenIdOf<T>,
			TokenBalance = TokenBalanceOf<T>,
			CollectionPolicy = DefaultCollectionPolicyOf<T>,
			TokenMetadata = DefaultTokenMetadataOf<T>,
		>,
>() -> T::AccountId {
	let collection_id = T::PoolCollectionId::get();
	let lst_collection_id = T::LstCollectionId::get();

	let collection = pallet_multi_tokens::Collections::<T>::get(collection_id);
	let lst_collection = pallet_multi_tokens::Collections::<T>::get(lst_collection_id);

	// create lst collection if it doesn't exist
	if lst_collection.is_none() {
		CurrencyOf::<T>::make_free_balance_be(
			&T::LstCollectionOwner::get(),
			BalanceOf::<T>::max_value() / 100_u32.saturated_into(),
		);

		FungibleHandler::<T>::force_create_collection(
			RuntimeOrigin::Root.into(),
			T::LstCollectionOwner::get(),
			lst_collection_id,
			Default::default(),
		)
		.unwrap();
	}

	if let Some(collection) = collection {
		collection.owner
	} else {
		let pool_creator: T::AccountId = create_funded_user_with_balance::<T>(
			"pool_creator",
			0,
			BalanceOf::<T>::max_value() / 4u32.saturated_into(),
		);
		FungibleHandler::<T>::force_create_collection(
			RuntimeOrigin::Root.into(),
			pool_creator.clone(),
			collection_id,
			Default::default(),
		)
		.unwrap();

		pool_creator
	}
}

pub(crate) type VoterBagsListInstance = pallet_bags_list::Instance1;
pub trait Config:
	pallet_nomination_pools::Config
	+ pallet_staking::Config
	+ pallet_bags_list::Config<VoterBagsListInstance>
{
}

pub struct Pallet<T: Config>(Pools<T>);

fn create_funded_user_with_balance<T: pallet_nomination_pools::Config>(
	string: &'static str,
	n: u32,
	balance: BalanceOf<T>,
) -> T::AccountId {
	let user = account(string, n, USER_SEED);
	CurrencyOf::<T>::make_free_balance_be(&user, balance);
	user
}

// Create a bonded pool account, bonding `balance` and giving the account `balance * 2` free
// balance.
fn create_pool_account<
	T: pallet_nomination_pools::Config
		+ FungibleHandlerConfig<
			CollectionId = CollectionIdOf<T>,
			TokenId = TokenIdOf<T>,
			TokenBalance = TokenBalanceOf<T>,
			CollectionPolicy = DefaultCollectionPolicyOf<T>,
			TokenMetadata = DefaultTokenMetadataOf<T>,
		>,
>(
	n: u32,
	balance: BalanceOf<T>,
	capacity: Option<BalanceOf<T>>,
) -> (T::AccountId, T::AccountId) {
	let ed = CurrencyOf::<T>::minimum_balance();
	let pool_creator: T::AccountId = create_funded_user_with_balance::<T>(
		"pool_creator",
		n,
		ed * 2_u32.into() + balance * 2u32.into(),
	);
	create_pool::<T>(pool_creator, (TOKEN_SEED + n).saturated_into(), balance, capacity)
}

fn create_pool<
	T: pallet_nomination_pools::Config
		+ FungibleHandlerConfig<
			CollectionId = CollectionIdOf<T>,
			TokenId = TokenIdOf<T>,
			TokenBalance = TokenBalanceOf<T>,
			CollectionPolicy = DefaultCollectionPolicyOf<T>,
			TokenMetadata = DefaultTokenMetadataOf<T>,
		>,
>(
	pool_creator: T::AccountId,
	token_id: T::TokenId,
	balance: BalanceOf<T>,
	capacity: Option<BalanceOf<T>>,
) -> (T::AccountId, T::AccountId) {
	let min_duration = T::MinDuration::get();

	mint_pool_token::<T>(token_id, pool_creator.clone());

	let pool_id = NextPoolId::<T>::get();
	Pools::<T>::create(
		RuntimeOrigin::Signed(pool_creator.clone()).into(),
		token_id,
		balance,
		capacity.unwrap_or((100_000 * UNIT).saturated_into()),
		min_duration,
		Default::default(),
	)
	.unwrap();

	// set commission
	Pools::<T>::mutate(
		RuntimeOrigin::Signed(pool_creator.clone()).into(),
		pool_id,
		PoolMutationOf::<T> {
			new_commission: SomeMutation(Some(Perbill::from_percent(5))),
			..Default::default()
		},
	)
	.expect("pool just created, commission can be set by admin; qed");

	let pool_account = BondedPools::<T>::iter()
		.find(|(_, bonded_pool)| bonded_pool.token_id == token_id)
		.map(|(pool_id, _pool)| Pools::<T>::compute_pool_account_id(pool_id, AccountType::Bonded))
		.expect("pool_creator created a pool above");

	(pool_creator, pool_account)
}

fn create_pool_at_block<T>(
	creator: T::AccountId,
	block_number: Option<BlockNumberFor<T>>,
	token_id: u128,
) -> PoolId
where
	T: FungibleHandlerConfig<
			CollectionId = CollectionIdOf<T>,
			TokenId = TokenIdOf<T>,
			TokenBalance = TokenBalanceOf<T>,
			CollectionPolicy = DefaultCollectionPolicyOf<T>,
			TokenMetadata = DefaultTokenMetadataOf<T>,
		> + frame_system::Config
		+ pallet_nomination_pools::Config
		+ pallet_balances::Config,
	<T as pallet_staking::Config>::CurrencyBalance: From<<T as pallet_balances::Config>::Balance>,
{
	if let Some(block_number) = block_number {
		frame_system::Pallet::<T>::set_block_number(block_number);
	}

	let pool_id = pallet_nomination_pools::NextPoolId::<T>::get();
	let token_id = TokenIdOf::<T>::from(token_id.try_into().unwrap());
	let minimum_bond = Pools::<T>::depositor_min_bond() + One::one();

	let min_duration = <T::MinDuration as Get<EraIndex>>::get();

	mint_pool_token::<T>(token_id, creator.clone());

	assert_ok!(Pools::<T>::create(
		RuntimeOrigin::Signed(creator).into(),
		token_id,
		minimum_bond * 100_u32.into(),
		minimum_bond * 1_000u32.into(),
		min_duration,
		Default::default(),
	));

	let pool = BondedPools::<T>::get(pool_id).unwrap();
	assert_eq!(
		pool.creation_block,
		block_number.unwrap_or(frame_system::Pallet::<T>::block_number())
	);

	pool_id
}

fn vote_to_balance<T: pallet_nomination_pools::Config>(
	vote: u64,
) -> Result<BalanceOf<T>, &'static str> {
	Ok(vote.into())
}

#[allow(unused)]
struct ListScenario<T: pallet_nomination_pools::Config> {
	/// Stash/Controller that is expected to be moved.
	origin1: T::AccountId,
	creator1: T::AccountId,
	dest_weight: BalanceOf<T>,
	origin1_member: Option<T::AccountId>,
}

impl<
		T: Config
			+ FungibleHandlerConfig<
				CollectionId = CollectionIdOf<T>,
				TokenId = TokenIdOf<T>,
				TokenBalance = TokenBalanceOf<T>,
				CollectionPolicy = DefaultCollectionPolicyOf<T>,
				TokenMetadata = DefaultTokenMetadataOf<T>,
			>,
	> ListScenario<T>
{
	/// An expensive scenario for bags-list implementation:
	///
	/// - the node to be updated (r) is the head of a bag that has at least one other node. The bag
	///   itself will need to be read and written to update its head. The node pointed to by r.next
	///   will need to be read and written as it will need to have its prev pointer updated. Note
	///   that there are two other worst case scenarios for bag removal: 1) the node is a tail and
	///   2) the node is a middle node with prev and next; all scenarios end up with the same number
	///   of storage reads and writes.
	///
	/// - the destination bag has at least one node, which will need its next pointer updated.
	pub(crate) fn new(
		origin_weight: BalanceOf<T>,
		is_increase: bool,
	) -> Result<Self, &'static str> {
		use sp_runtime::traits::AccountIdConversion;

		ensure!(!origin_weight.is_zero(), "origin weight must be greater than 0");

		// create the pool token
		let pool_collection_id =
		<<T as pallet_nomination_pools::Config>::PoolCollectionId as Get<
			CollectionIdOf<T>,
		>>::get();

		// if pool token already exists, skip
		// if the benchmark is run as test, the pool token should already exist
		// but when run to generate weights, the pool token will not exist and will be created here
		if pallet_multi_tokens::Collections::<T>::get(pool_collection_id).is_none() {
			let ed = CurrencyOf::<T>::minimum_balance();

			let pool_creator: T::AccountId =
				create_funded_user_with_balance::<T>("pool_creator", 100, ed * 200_0000_u32.into());

			assert_ok!(FungibleHandler::<T>::force_create_collection(
				RuntimeOrigin::Root.into(),
				pool_creator,
				pool_collection_id,
				Default::default(),
			));
		}

		// create the lst token
		let lst_collection_id =
			<<T as pallet_nomination_pools::Config>::LstCollectionId as Get<
				CollectionIdOf<T>,
			>>::get();

		// if pool token already exists, skip
		// if the benchmark is run as test, the pool token should already exist
		// but when run to generate weights, the pool token will not exist and will be created here
		if pallet_multi_tokens::Collections::<T>::get(lst_collection_id).is_none() {
			FungibleHandler::<T>::force_create_collection(
				RuntimeOrigin::Root.into(),
				<<T as pallet_nomination_pools::Config>::PalletId as Get<
					frame_support::PalletId,
				>>::get()
				.into_account_truncating(),
				lst_collection_id,
				Default::default(),
			)
			.unwrap();
		}

		// Burn the entire issuance.
		let i = CurrencyOf::<T>::burn(CurrencyOf::<T>::total_issuance());
		sp_std::mem::forget(i);

		// Create accounts with the origin weight
		let (pool_creator1, pool_origin1) = create_pool_account::<T>(
			USER_SEED + 1,
			origin_weight,
			Some((100 * UNIT).saturated_into()),
		);
		T::Staking::nominate(
			&pool_origin1,
			// NOTE: these don't really need to be validators.
			vec![account("random_validator", 0, USER_SEED)],
		)
		.unwrap();

		let (_, pool_origin2) = create_pool_account::<T>(
			USER_SEED + 2,
			origin_weight,
			Some((100 * UNIT).saturated_into()),
		);

		T::Staking::nominate(&pool_origin2, vec![account("random_validator", 0, USER_SEED)])
			.unwrap();

		// Find a destination weight that will trigger the worst case scenario
		let dest_weight_as_vote =
			<T as pallet_staking::Config>::VoterList::score_update_worst_case(
				&pool_origin1,
				is_increase,
			)
			.max(Pools::<T>::depositor_min_bond().saturated_into());

		let dest_weight: BalanceOf<T> = dest_weight_as_vote.into();

		// Create an account with the worst case destination weight
		let (_, pool_dest1) = create_pool_account::<T>(
			USER_SEED + 3,
			dest_weight,
			Some((100 * UNIT).saturated_into()),
		);

		T::Staking::nominate(&pool_dest1, vec![account("random_validator", 0, USER_SEED)]).unwrap();

		let weight_of = pallet_staking::Pallet::<T>::weight_of_fn();
		assert_eq!(vote_to_balance::<T>(weight_of(&pool_origin1)).unwrap(), origin_weight);
		assert_eq!(vote_to_balance::<T>(weight_of(&pool_origin2)).unwrap(), origin_weight);
		assert_eq!(vote_to_balance::<T>(weight_of(&pool_dest1)).unwrap(), dest_weight);

		Ok(ListScenario {
			origin1: pool_origin1,
			creator1: pool_creator1,
			dest_weight,
			origin1_member: None,
		})
	}

	fn add_joiner(mut self, amount: BalanceOf<T>) -> Self {
		let pool_id = 0;
		let amount = MinJoinBond::<T>::get()
			.max(CurrencyOf::<T>::minimum_balance())
			// Max `amount` with minimum thresholds for account balance and joining a pool
			// to ensure 1) the user can be created and 2) can join the pool
			.max(amount);

		let joiner: T::AccountId = account("joiner", 0, USER_SEED);
		self.origin1_member = Some(joiner.clone());
		CurrencyOf::<T>::make_free_balance_be(&joiner, amount * 2u32.into());

		let original_bonded = T::Staking::active_stake(&self.origin1).unwrap();

		// Unbond `amount` from the underlying pool account so when the member joins
		// we will maintain `current_bonded`.
		T::Staking::unbond(&self.origin1, amount).expect("the pool was created in `Self::new`.");

		// Account pool points for the unbonded balance.
		T::FungibleHandler::burn(
			self.origin1.clone(),
			T::LstCollectionId::get(),
			false,
		)
		.unwrap();

		Pools::<T>::bond(RuntimeOrigin::Signed(joiner.clone()).into(), pool_id, amount.into())
			.unwrap();

		// check that the vote weight is still the same as the original bonded
		let weight_of = pallet_staking::Pallet::<T>::weight_of_fn();
		assert_eq!(vote_to_balance::<T>(weight_of(&self.origin1)).unwrap(), original_bonded);

		// check the member was added correctly
		assert_eq!(Pools::<T>::member_points(pool_id, joiner), amount);

		self
	}
}

/// These functions are used with the `payout_rewards` benchmark. They were taken from
/// `pallet_staking` benchmarks, and they have been modified to support pools.
mod rewards {
	use super::*;

	use pallet_staking::MaxWinnersOf;

	/// Consume a set of [`BoundedSupports`] from [`sp_npos_elections`] and collect them into a
	/// [`Exposure`].
	#[allow(clippy::type_complexity)]
	fn collect_exposures<T: pallet_staking::Config>(
		supports: BoundedSupportsOf<T::ElectionProvider>,
	) -> BoundedVec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>), MaxWinnersOf<T>> {
		let total_issuance = T::Currency::total_issuance();
		let to_currency = |e: frame_election_provider_support::ExtendedBalance| {
			T::CurrencyToVote::to_currency(e, total_issuance)
		};

		supports
			.into_iter()
			.map(|(validator, support)| {
				// Build `struct exposure` from `support`.
				let mut others = Vec::with_capacity(support.voters.len());
				let mut own: BalanceOf<T> = Zero::zero();
				let mut total: BalanceOf<T> = Zero::zero();
				support
					.voters
					.into_iter()
					.map(|(nominator, weight)| (nominator, to_currency(weight)))
					.for_each(|(nominator, stake)| {
						if nominator == validator {
							own = own.saturating_add(stake);
						} else {
							others.push(IndividualExposure { who: nominator, value: stake });
						}
						total = total.saturating_add(stake);
					});

				let exposure = Exposure { own, others, total };
				(validator, exposure)
			})
			.collect::<Vec<_>>()
			.try_into()
			.expect("we only map through support vector which cannot change the size; qed")
	}

	/// Potentially plan a new era.
	///
	/// Get election result from `T::ElectionProvider`.
	/// In case election result has more than [`MinimumValidatorCount`] validator trigger a new era.
	///
	/// In case a new era is planned, the new validator set is returned.
	fn try_trigger_new_era<T: pallet_staking::Config>(
		start_session_index: SessionIndex,
		is_genesis: bool,
	) -> Option<BoundedVec<T::AccountId, pallet_staking::MaxWinnersOf<T>>> {
		let election_result: BoundedVec<_, pallet_staking::MaxWinnersOf<T>> = if is_genesis {
			let result = <T::GenesisElectionProvider>::elect().map_err(|e| {
				log::warn!("genesis election provider failed due to {:?}", e);
			});
			result
				.ok()
				.unwrap()
				.into_inner()
				.try_into()
				// both bounds checked in integrity test to be equal
				.unwrap_or_default()
		} else {
			let result = <T::ElectionProvider>::elect().map_err(|e| {
				log::warn!("election provider failed due to {:?}", e);
			});
			result.ok().unwrap()
		};
		log::debug!("election result: {:?}", election_result);

		let exposures = collect_exposures::<T>(election_result);
		if (exposures.len() as u32) < Staking::<T>::minimum_validator_count().max(1) {
			// Session will panic if we ever return an empty validator set, thus max(1) ^^.
			match pallet_staking::CurrentEra::<T>::get() {
				Some(current_era) if current_era > 0 => log::warn!(
					"chain does not have enough staking candidates to operate for era {:?} ({} \
					elected, minimum is {})",
					pallet_staking::CurrentEra::<T>::get().unwrap_or(0),
					exposures.len(),
					Staking::<T>::minimum_validator_count(),
				),
				None => {
					// The initial era is allowed to have no exposures.
					// In this case the SessionManager is expected to choose a sensible validator
					// set.
					pallet_staking::CurrentEra::<T>::put(0);
					pallet_staking::ErasStartSessionIndex::<T>::insert(0, start_session_index);
				},
				_ => (),
			}
			return None;
		}

		Some(Staking::<T>::trigger_new_era(start_session_index, exposures))
	}

	// This function clears all existing validators and nominators from the set, and generates one
	// new validator being nominated by n nominators, and returns the validator stash account and
	// the nominators' stash and controller. It also starts an era and creates pending payouts.
	#[allow(clippy::type_complexity)]
	pub(super) fn create_validator_with_nominators<
		T: Config
			+ FungibleHandlerConfig<
				CollectionId = CollectionIdOf<T>,
				TokenId = TokenIdOf<T>,
				TokenBalance = TokenBalanceOf<T>,
				CollectionPolicy = DefaultCollectionPolicyOf<T>,
				TokenMetadata = DefaultTokenMetadataOf<T>,
			>,
	>(
		nominator_count: u32,
		upper_bound: u32,
		_dead_controller: bool,
		unique_controller: bool,
		destination: pallet_staking::RewardDestination<T::AccountId>,
	) -> Result<(T::AccountId, Vec<(T::AccountId, T::AccountId)>), &'static str> {
		// Clean up any existing state.
		pallet_staking::testing_utils::clear_validators_and_nominators::<T>();
		let mut points_total = 0;
		let mut points_individual = Vec::new();

		let (v_stash, v_controller) = if unique_controller {
			pallet_staking::testing_utils::create_unique_stash_controller::<T>(
				0,
				10_000,
				destination.clone(),
				false,
			)
			.unwrap()
		} else {
			pallet_staking::testing_utils::create_stash_controller::<T>(
				0,
				10_000,
				destination.clone(),
			)
			.unwrap()
		};

		let validator_prefs = pallet_staking::ValidatorPrefs {
			commission: Perbill::from_percent(50),
			..Default::default()
		};
		Staking::<T>::validate(RuntimeOrigin::Signed(v_controller).into(), validator_prefs)
			.unwrap();
		let stash_lookup = T::Lookup::unlookup(v_stash.clone());

		points_total += 10;
		points_individual.push((v_stash.clone(), 10));

		let original_nominator_count = pallet_staking::Nominators::<T>::count();
		let mut nominators = Vec::new();

		// Give the validator n nominators, but keep total users in the system the same.
		let origin_weight = (100 * UNIT).saturated_into();
		for i in 0..upper_bound {
			let (_pool_creator, pool_account) =
				create_pool_account::<T>(USER_SEED + i + 1, origin_weight, Some(origin_weight));
			if i < nominator_count {
				Staking::<T>::nominate(
					RuntimeOrigin::Signed(pool_account.clone()).into(),
					vec![stash_lookup.clone()],
				)
				.unwrap();
				nominators.push((pool_account.clone(), pool_account));
			}
		}
		assert_eq!(
			pallet_staking::Nominators::<T>::count(),
			original_nominator_count + nominators.len() as u32
		);

		pallet_staking::ValidatorCount::<T>::put(1);

		// Start a new Era
		let new_validators = try_trigger_new_era::<T>(SessionIndex::one(), true).unwrap();

		assert_eq!(new_validators.len(), 1);
		assert_eq!(new_validators[0], v_stash, "Our validator was not selected!");
		assert_ne!(pallet_staking::Validators::<T>::count(), 0);

		// Give Era Points
		let reward = pallet_staking::EraRewardPoints::<T::AccountId> {
			total: points_total,
			individual: points_individual.into_iter().collect(),
		};

		let current_era = pallet_staking::CurrentEra::<T>::get().unwrap();
		pallet_staking::ErasRewardPoints::<T>::insert(current_era, reward);

		// Create reward pool
		let total_payout = CurrencyOf::<T>::minimum_balance()
			.saturating_mul(upper_bound.into())
			.saturating_mul(1000u32.into());
		<pallet_staking::ErasValidatorReward<T>>::insert(current_era, total_payout);

		Ok((v_stash, nominators))
	}
}

#[benchmarks(
	where T: FungibleHandlerConfig<
		CollectionId = CollectionIdOf<T>,
		TokenId = TokenIdOf<T>,
		TokenBalance = TokenBalanceOf<T>,
		CollectionPolicy = DefaultCollectionPolicyOf<T>,
		TokenMetadata = DefaultTokenMetadataOf<T>,
		Currency = CurrencyOf::<T>,
	> + pallet_balances::Config,
	<T as pallet_balances::Config>::Balance: From<<T as pallet_staking::Config>::CurrencyBalance>,
	<T as pallet_staking::Config>::CurrencyBalance: From<<T as pallet_balances::Config>::Balance>,
	<T as pallet_staking::Config>::CurrencyBalance: From<u128>,
)]
#[allow(clippy::missing_docs_in_private_items)]
mod benchmarks {
	use super::*;

	// first time bonding
	#[benchmark]
	fn bond() {
		let origin_weight = Pools::<T>::depositor_min_bond() * 2u32.into();

		// setup the worst case list scenario.
		let scenario = ListScenario::<T>::new(origin_weight, true).unwrap();
		assert_eq!(T::Staking::active_stake(&scenario.origin1).unwrap(), origin_weight);

		let max_additional = scenario.dest_weight - origin_weight;
		let joiner_free = CurrencyOf::<T>::minimum_balance() + max_additional;

		let joiner: T::AccountId = create_funded_user_with_balance::<T>("joiner", 0, joiner_free);

		whitelist_account!(joiner);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(joiner.clone()), 0, max_additional.into());

		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), joiner_free - max_additional);
		assert_eq!(T::Staking::active_stake(&scenario.origin1).unwrap(), scenario.dest_weight);
	}

	#[benchmark]
	fn unbond() {
		// The weight the nominator will start at. The value used here is expected to be
		// significantly higher than the first position in a list (e.g. the first bag threshold).
		let origin_weight = Pools::<T>::depositor_min_bond() * 20_u32.into();
		let scenario = ListScenario::<T>::new(origin_weight, false).unwrap();
		let pool_id = 0;
		let amount = origin_weight - scenario.dest_weight;

		let scenario = scenario.add_joiner(amount);
		let member_id = scenario.origin1_member.unwrap();
		let member_id_lookup = T::Lookup::unlookup(member_id.clone());
		let all_points = Pools::<T>::member_points(pool_id, member_id.clone());
		whitelist_account!(member_id);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(member_id.clone()), pool_id, member_id_lookup, all_points);

		let bonded_after = T::Staking::active_stake(&scenario.origin1).unwrap();
		// We at least went down to the destination bag
		assert!(bonded_after <= scenario.dest_weight);
		let member = UnbondingMembers::<T>::get(pool_id, &member_id).unwrap();
		assert_eq!(
			member.unbonding_eras.keys().cloned().collect::<Vec<_>>(),
			vec![T::Staking::bonding_duration()]
		);
		assert_eq!(member.unbonding_eras.values().cloned().collect::<Vec<_>>(), vec![all_points]);
	}

	#[benchmark]
	fn unbond_deposit() {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let pool_id = NextPoolId::<T>::get();
		let (depositor, pool_account) =
			create_pool_account::<T>(0, min_create_bond, Some((1_000 * UNIT).saturated_into()));

		// We set the pool to the destroying state so the depositor can leave
		BondedPools::<T>::try_mutate(pool_id, |maybe_bonded_pool| {
			maybe_bonded_pool.as_mut().ok_or(()).map(|bonded_pool| {
				bonded_pool.state = PoolState::Destroying;
			})
		})
		.unwrap();

		// Unbond the creator
		pallet_staking::CurrentEra::<T>::put(0);
		// Simulate some rewards so we can check if the rewards storage is cleaned up.
		let reward_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Reward);
		assert!(frame_system::Account::<T>::contains_key(&reward_account));

		Pools::<T>::fully_unbond(
			RuntimeOrigin::Signed(depositor.clone()).into(),
			pool_id,
			depositor.clone(),
		)
		.unwrap();
		let _ = UnbondingMembers::<T>::clear_prefix(pool_id, 5, None);

		// Sanity check that unbond worked and only deposit is left
		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), min_create_bond);
		assert_eq!(CurrencyOf::<T>::free_balance(&pool_account), min_create_bond);
		#[extrinsic_call]
		unbond_deposit(RuntimeOrigin::Signed(depositor.clone()), pool_id);

		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);
		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), Zero::zero());

		// Some last checks that storage items we expect to get cleaned up are present
		assert!(pallet_staking::Ledger::<T>::contains_key(&pool_account));
		assert!(BondedPools::<T>::contains_key(pool_id));
		assert!(SubPoolsStorage::<T>::contains_key(pool_id));
		assert!(frame_system::Account::<T>::contains_key(&reward_account));
	}

	#[benchmark]
	fn pool_withdraw_unbonded(s: Linear<0, MAX_SPANS>) {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let (_depositor, pool_account) =
			create_pool_account::<T>(1000, min_create_bond, Some((100 * UNIT).saturated_into()));

		// Add a new member
		let min_join_bond = MinJoinBond::<T>::get().max(CurrencyOf::<T>::minimum_balance());
		let joiner = create_funded_user_with_balance::<T>("joiner", 0, min_join_bond * 2u32.into());
		let pool_id = 0;
		Pools::<T>::bond(
			RuntimeOrigin::Signed(joiner.clone()).into(),
			pool_id,
			min_join_bond.into(),
		)
		.unwrap();

		// Sanity check join worked
		assert_eq!(
			T::Staking::active_stake(&pool_account).unwrap(),
			min_create_bond + min_join_bond
		);
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond);

		// Unbond the new member
		Pools::<T>::fully_unbond(
			RuntimeOrigin::Signed(joiner.clone()).into(),
			pool_id,
			joiner.clone(),
		)
		.unwrap();

		// Sanity check that unbond worked
		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), min_create_bond);
		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);
		// Set the current era
		pallet_staking::CurrentEra::<T>::put(EraIndex::max_value());

		// Add `s` count of slashing spans to storage.
		pallet_staking::benchmarking::add_slashing_spans::<T>(&pool_account, s);
		whitelist_account!(pool_account);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(pool_account.clone()), pool_id, s);

		// The joiners funds didn't change
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond);
		// The unlocking chunk was removed
		assert_eq!(pallet_staking::Ledger::<T>::get(pool_account).unwrap().unlocking.len(), 0);
	}

	#[benchmark]
	fn withdraw_unbonded_update(s: Linear<0, MAX_SPANS>) {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let (_depositor, pool_account) =
			create_pool_account::<T>(0, min_create_bond, Some((100 * UNIT).saturated_into()));

		// Add a new member
		let min_join_bond = MinJoinBond::<T>::get().max(CurrencyOf::<T>::minimum_balance());
		let joiner = create_funded_user_with_balance::<T>("joiner", 0, min_join_bond * 2u32.into());
		let joiner_lookup = T::Lookup::unlookup(joiner.clone());
		let pool_id = 0;
		Pools::<T>::bond(
			RuntimeOrigin::Signed(joiner.clone()).into(),
			pool_id,
			min_join_bond.into(),
		)
		.unwrap();

		// Sanity check join worked
		assert_eq!(
			T::Staking::active_stake(&pool_account).unwrap(),
			min_create_bond + min_join_bond
		);
		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond);

		// Unbond the new member
		pallet_staking::CurrentEra::<T>::put(0);
		Pools::<T>::fully_unbond(
			RuntimeOrigin::Signed(joiner.clone()).into(),
			pool_id,
			joiner.clone(),
		)
		.unwrap();

		// Sanity check that unbond worked
		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), min_create_bond);
		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);

		// Set the current era to ensure we can withdraw unbonded funds
		pallet_staking::CurrentEra::<T>::put(EraIndex::max_value());

		pallet_staking::benchmarking::add_slashing_spans::<T>(&pool_account, s);
		whitelist_account!(joiner);
		#[extrinsic_call]
		withdraw_unbonded(RuntimeOrigin::Signed(joiner.clone()), pool_id, joiner_lookup, s);

		assert_eq!(CurrencyOf::<T>::free_balance(&joiner), min_join_bond * 2u32.into());
		// The unlocking chunk was removed
		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 0);
	}

	#[benchmark]
	fn withdraw_deposit() {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let pool_id = NextPoolId::<T>::get();
		let (depositor, pool_account) =
			create_pool_account::<T>(0, min_create_bond, Some((1_000 * UNIT).saturated_into()));

		// We set the pool to the destroying state so the depositor can leave
		BondedPools::<T>::try_mutate(pool_id, |maybe_bonded_pool| {
			maybe_bonded_pool.as_mut().ok_or(()).map(|bonded_pool| {
				bonded_pool.state = PoolState::Destroying;
			})
		})
		.unwrap();

		// Unbond the creator
		pallet_staking::CurrentEra::<T>::put(0);
		// Simulate some rewards so we can check if the rewards storage is cleaned up. We check this
		// here to ensure the complete flow for destroying a pool works - the reward pool account
		// should never exist by time the depositor withdraws so we test that it gets cleaned
		// up when unbonding.
		let reward_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Reward);
		assert!(frame_system::Account::<T>::contains_key(&reward_account));
		Pools::<T>::fully_unbond(
			RuntimeOrigin::Signed(depositor.clone()).into(),
			pool_id,
			depositor.clone(),
		)
		.unwrap();

		// Sanity check that unbond worked and only deposit is left
		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), min_create_bond);
		assert_eq!(CurrencyOf::<T>::free_balance(&pool_account), min_create_bond);

		let _ = UnbondingMembers::<T>::clear_prefix(pool_id, 5, None);

		// now unbond the deposit
		Pools::<T>::unbond_deposit(RuntimeOrigin::Signed(depositor.clone()).into(), pool_id)
			.unwrap();

		assert_eq!(T::Staking::active_stake(&pool_account).unwrap(), Zero::zero());

		assert_eq!(pallet_staking::Ledger::<T>::get(&pool_account).unwrap().unlocking.len(), 1);

		// Set the current era to ensure we can withdraw unbonded funds
		pallet_staking::CurrentEra::<T>::put(EraIndex::max_value());

		// Some last checks that storage items we expect to get cleaned up are present
		assert!(pallet_staking::Ledger::<T>::contains_key(&pool_account));
		assert!(BondedPools::<T>::contains_key(pool_id));
		assert!(SubPoolsStorage::<T>::contains_key(pool_id));
		assert!(frame_system::Account::<T>::contains_key(&reward_account));

		whitelist_account!(depositor);
		#[extrinsic_call]
		withdraw_deposit(RuntimeOrigin::Signed(depositor.clone()), pool_id);

		// Pool removal worked
		assert!(!pallet_staking::Ledger::<T>::contains_key(&pool_account));
		assert!(!BondedPools::<T>::contains_key(pool_id));
		assert!(!SubPoolsStorage::<T>::contains_key(pool_id));
		assert!(!UnbondingMembers::<T>::contains_key(pool_id, &depositor));
		assert!(!frame_system::Account::<T>::contains_key(&pool_account));
		assert!(!frame_system::Account::<T>::contains_key(&reward_account));
	}

	#[benchmark]
	fn withdraw_free_balance() {
		// Create a pool
		let pool_id = 0;

		// add free balance
		let destination: T::AccountId = account("destination", USER_SEED, 0);
		let amount = 10 * UNIT;
		let pool_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Bonded);
		CurrencyOf::<T>::make_free_balance_be(&pool_account, (amount + UNIT).saturated_into());
		#[extrinsic_call]
		_(
			RuntimeOrigin::Root,
			pool_id,
			T::Lookup::unlookup(destination.clone()),
			amount.saturated_into(),
		);

		assert_eq!(CurrencyOf::<T>::free_balance(&pool_account), UNIT.into());
		assert_eq!(CurrencyOf::<T>::total_balance(&destination), amount.into());
	}

	#[benchmark]
	fn create() {
		let min_create_bond = Pools::<T>::depositor_min_bond();
		let depositor: T::AccountId = account("depositor", USER_SEED, 0);
		let capacity = min_create_bond * 4u32.into();
		let token_id: T::TokenId = (TOKEN_SEED + 1).saturated_into();

		mint_pool_token::<T>(token_id, depositor.clone());

		// Give the depositor some balance to bond
		CurrencyOf::<T>::make_free_balance_be(&depositor, min_create_bond * 3u32.into());

		// Make sure no Pools exist at a pre-condition for our verify checks
		assert_eq!(BondedPools::<T>::count(), 0);

		whitelist_account!(depositor);
		#[extrinsic_call]
		_(
			RuntimeOrigin::Signed(depositor.clone()),
			token_id,
			min_create_bond,
			capacity,
			T::MinDuration::get(),
			Default::default(),
		);

		assert_eq!(BondedPools::<T>::count(), 1);
		let (pool_id, new_pool) = BondedPools::<T>::iter().next().unwrap();
		assert_eq!(new_pool.state, PoolState::Open);
		assert_eq!(new_pool.capacity, capacity);
		assert_eq!(new_pool.token_id, token_id);
		assert_eq!(
			pallet_nomination_pools::BondedPool::<T>::get(pool_id).unwrap().points(),
			min_create_bond
		);
		assert_eq!(
			T::Staking::active_stake(&Pools::<T>::compute_pool_account_id(0, AccountType::Bonded)),
			Ok(min_create_bond)
		);
	}

	#[benchmark]
	fn nominate(n: Linear<1, { MaxNominationsOf::<T>::get() }>) {
		// Create a pool
		let pool_id = 0;
		let min_create_bond = Pools::<T>::depositor_min_bond() * 2u32.into();
		let (depositor, _pool_account) = create_pool_account::<T>(
			USER_SEED + 3,
			min_create_bond,
			Some((100 * UNIT).saturated_into()),
		);

		// Create some accounts to nominate. For the sake of benchmarking they don't need to be
		// actual validators
		let validators: Vec<_> = (0..n).map(|i| account("stash", USER_SEED, i)).collect();

		whitelist_account!(depositor);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(depositor.clone()), pool_id, validators);

		assert_eq!(BondedPools::<T>::count(), 1);
		let (pool_id, new_pool) = BondedPools::<T>::iter().next().unwrap();
		let block = frame_system::Pallet::<T>::block_number();
		assert_eq!(new_pool.capacity, (100 * UNIT).saturated_into());
		assert_eq!(new_pool.token_id, (USER_SEED + 3 + TOKEN_SEED).saturated_into());
		assert_eq!(new_pool.commission.current, Some(Perbill::from_percent(5)));
		assert_eq!(
			new_pool.commission.throttle_from,
			Some(new_pool.commission.throttle_from.unwrap_or(block))
		);
		assert_eq!(
			pallet_nomination_pools::BondedPool::<T>::get(pool_id).unwrap().points(),
			min_create_bond
		);
		assert_eq!(
			T::Staking::active_stake(&Pools::<T>::compute_pool_account_id(
				pool_id,
				AccountType::Bonded
			)),
			Ok(min_create_bond)
		);
	}

	// partially copied from pallet-staking::benchmarking payout_stakers_alive_staked
	#[benchmark]
	fn payout_rewards(
		n: Linear<0, { T::MaxExposurePageSize::get() }>,
	) -> Result<(), BenchmarkError> {
		let (validator, nominators) = rewards::create_validator_with_nominators::<T>(
			n,
			T::MaxExposurePageSize::get(),
			false,
			true,
			pallet_staking::RewardDestination::Staked,
		)
		.unwrap();
		let current_era = pallet_staking::CurrentEra::<T>::get().unwrap();
		// set the commission for this particular era as well.
		<pallet_staking::ErasValidatorPrefs<T>>::insert(
			current_era,
			validator.clone(),
			Staking::<T>::validators(&validator),
		);

		let balance_before = CurrencyOf::<T>::free_balance(&validator);
		let mut nominator_balances_before = Vec::new();
		for (stash, _) in &nominators {
			let stake = Staking::<T>::active_stake(stash).unwrap();
			nominator_balances_before.push(stake);
		}

		let pool_ids = (0..nominators.len()).map(|i| i as PoolId).collect::<Vec<_>>();

		let minimum_balance = CurrencyOf::<T>::minimum_balance();
		for pool_id in pool_ids.iter().copied() {
			// make the bonus high to ensure it will be paid out
			let bonus_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Bonus);
			CurrencyOf::<T>::make_free_balance_be(
				&bonus_account,
				BalanceOf::<T>::max_value() / 100_u32.saturated_into(),
			);

			// reward account should have minimum balance
			let reward_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Reward);
			assert_eq!(CurrencyOf::<T>::free_balance(&reward_account), minimum_balance);
		}

		let caller = create_funded_user_with_balance::<T>(
			"caller",
			0,
			CurrencyOf::<T>::free_balance(&validator),
		);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(caller), validator.clone(), current_era);

		let balance_after = CurrencyOf::<T>::free_balance(&validator);
		assert_ne!(balance_before, balance_after);
		ensure!(
			balance_before < balance_after,
			"Balance of validator stash should have increased after payout.",
		);

		// make sure rewards were paid out
		for pool_id in pool_ids {
			let reward_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Reward);
			let balance = CurrencyOf::<T>::free_balance(&reward_account);
			assert!(balance > minimum_balance, "rewards were not distributed");
		}

		// make sure staking rewards have been reinvested. This means bonus was distributed.
		for ((stash, _), balance_before) in nominators.iter().zip(nominator_balances_before.iter())
		{
			let balance_after = Staking::<T>::active_stake(stash).unwrap();
			ensure!(
				balance_before < &balance_after,
				"Balance of nominator stash should have increased after payout.",
			);
		}

		Ok(())
	}

	#[benchmark]
	fn process_payouts(n: Linear<0, MAX_POOLS>) {
		// create n pools
		let min_create_bond = Pools::<T>::depositor_min_bond() * 2u32.into();
		for pool_id in 0..n {
			let (_depositor, _pool_account) = create_pool_account::<T>(
				pool_id,
				min_create_bond,
				Some((1_000 * UNIT).saturated_into()),
			);
			let reward_account = Pools::<T>::compute_pool_account_id(pool_id, AccountType::Reward);
			CurrencyOf::<T>::make_free_balance_be(&reward_account, (2 * UNIT).saturated_into());
		}

		// set up era
		let era = if let Some(era) = pallet_staking::CurrentEra::<T>::get() {
			era
		} else {
			pallet_staking::CurrentEra::<T>::put(0);
			0
		};

		// insert payout count to match the number of validators
		let validator_count = pallet_staking::Validators::<T>::count();
		EraPayoutInfo::<T>::set(EraPayout {
			era,
			payout_count: validator_count,
			..Default::default()
		});

		let caller =
			create_funded_user_with_balance::<T>("caller", 0, (1_000 * UNIT).saturated_into());
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(caller), n);

		// each pool will have the era added to bonuses_paid if it succeeded
		for pool_id in 0..n {
			let pool = BondedPools::<T>::get(pool_id as PoolId).unwrap();
			assert!(pool.bonuses_paid.contains(&era));
		}
	}

	#[benchmark]
	fn set_configs() {
		#[extrinsic_call]
		_(
			RuntimeOrigin::Root,
			ConfigOp::Set(BalanceOf::<T>::max_value()),
			ConfigOp::Set(BalanceOf::<T>::max_value()),
			ConfigOp::Set(Perbill::from_percent(5)),
			ConfigOp::Set(Perbill::from_percent(20)),
		);
		assert_eq!(MinJoinBond::<T>::get(), BalanceOf::<T>::max_value());
		assert_eq!(MinCreateBond::<T>::get(), BalanceOf::<T>::max_value());
		assert_eq!(GlobalMaxCommission::<T>::get(), Some(Perbill::from_percent(5)));
		assert_eq!(EraPayoutInfo::<T>::get().required_payments_percent, Perbill::from_percent(20));
	}

	#[benchmark]
	fn chill() {
		// Create a pool
		let (depositor, pool_account) = create_pool_account::<T>(
			0,
			Pools::<T>::depositor_min_bond() * 2u32.into(),
			Some((100 * UNIT).saturated_into()),
		);
		let pool_id = 0;
		// Nominate with the pool.
		let validators: Vec<_> = (0..MaxNominationsOf::<T>::get())
			.map(|i| account("stash", USER_SEED, i))
			.collect();

		assert_ok!(T::Staking::nominate(&pool_account, validators));
		assert!(T::Staking::nominations(&Pools::<T>::compute_pool_account_id(
			pool_id,
			AccountType::Bonded
		))
		.is_some());

		whitelist_account!(depositor);
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(depositor.clone()), pool_id);

		assert!(T::Staking::nominations(&Pools::<T>::compute_pool_account_id(
			pool_id,
			AccountType::Bonded
		))
		.is_none());
	}

	#[benchmark]
	fn destroy() {
		let first_id = pallet_nomination_pools::NextPoolId::<T>::get();
		let (admin, _) = create_pool_account::<T>(
			0,
			Pools::<T>::depositor_min_bond() * 2u32.into(),
			Some(Pools::<T>::depositor_min_bond() * 4u32.into()),
		);

		mint_pool_token::<T>((TOKEN_SEED + 3).saturated_into(), admin.clone());
		#[extrinsic_call]
		_(RuntimeOrigin::Signed(admin.clone()), first_id);

		assert_eq!(
			pallet_nomination_pools::BondedPools::<T>::get(first_id).unwrap().state,
			pallet_nomination_pools::PoolState::Destroying,
		);
	}

	#[benchmark]
	fn mutate() {
		let pool_id = pallet_nomination_pools::NextPoolId::<T>::get();
		let (admin, _) = create_pool_account::<T>(
			0,
			Pools::<T>::depositor_min_bond() * 2u32.into(),
			Some(Pools::<T>::depositor_min_bond() * 4u32.into()),
		);

		let mutation = PoolMutationOf::<T> {
			duration: Some(50),
			new_commission: ShouldMutate::SomeMutation(Some(Perbill::from_percent(5))),
			max_commission: Some(Perbill::from_percent(10)),
			change_rate: Some(CommissionChangeRate {
				max_delta: Perbill::from_percent(4),
				min_delay: 1000u32.into(),
			}),
			capacity: Some(Pools::<T>::depositor_min_bond() * 10u32.into()),
			name: Some(b"new_name".to_vec().try_into().unwrap()),
		};

		#[extrinsic_call]
		_(RuntimeOrigin::Signed(admin.clone()), pool_id, mutation.clone());

		let pool = BondedPools::<T>::get(pool_id).unwrap();

		assert_eq!(pool.bonus_cycle.pending_duration, mutation.duration,);

		assert_eq!(
			pool.commission,
			Commission {
				current: Some(Perbill::from_percent(5)),
				max: Some(Perbill::from_percent(10)),
				change_rate: Some(CommissionChangeRate {
					max_delta: Perbill::from_percent(4),
					min_delay: 1000u32.into()
				}),
				throttle_from: Some(1u32.into()),
			}
		);

		assert_eq!(pool.capacity, mutation.capacity.unwrap(),);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
}
