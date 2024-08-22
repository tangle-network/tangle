#![macro_use]
use crate::{
	self as pools,
	tests::DEFAULT_DURATION,
	types::{AccountType, CurrencyOf},
	BondedPools, Config, NegativeImbalanceOf, NextPoolId, Pallet, PoolId, PoolState,
};
use frame_election_provider_support::VoteWeight;
use frame_support::{
	assert_ok, derive_impl,
	dispatch::DispatchResult,
	parameter_types,
	traits::{Currency, Hooks, OnFinalize, OnUnbalanced},
	PalletId,
};
use sp_runtime::BuildStorage;
use sp_staking::currency_to_vote::SaturatingCurrencyToVote;

use frame_system::RawOrigin;
use polkadot_runtime_common::{BalanceToU256, U256ToBalance};
use sp_core::{bounded::BoundedBTreeMap, ConstU128, ConstU32, ConstU64, ConstU8, Get, U256};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Zero},
	DispatchError, FixedU128, Perbill,
};
use sp_staking::{EraIndex, Stake};
use sp_std::collections::btree_map::BTreeMap;

pub type AccountId = u128;
pub type RewardCounter = FixedU128;
// This sneaky little hack allows us to write code exactly as we would do in the pallet in the tests
// as well, e.g. `StorageItem::<T>::get()`.
pub type T = Runtime;

// Ext builder creates a pool with id 1.
pub fn default_bonded_account() -> AccountId {
	Pools::compute_pool_account_id(0, AccountType::Bonded)
}

// Ext builder creates a pool with id 1.
pub fn default_reward_account() -> AccountId {
	Pools::compute_pool_account_id(0, AccountType::Reward)
}

pub struct MockOnUnbalancedHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for MockOnUnbalancedHandler<T>
where
	T: crate::Config + pallet_balances::Config,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
		// Must resolve into existing but better to be safe.
		CurrencyOf::<T>::resolve_creating(
			&<T as crate::Config>::LstCollectionOwner::get(),
			amount,
		);
	}
}

parameter_types! {
	pub static MinJoinBondConfig: Balance = 2;
	pub static CurrentEra: EraIndex = 0;
	pub static BondingDuration: EraIndex = 3;
	pub storage BondedBalanceMap: BTreeMap<AccountId, Balance> = Default::default();
	pub storage UnbondingBalanceMap: BTreeMap<AccountId, Balance> = Default::default();
	#[derive(Clone, PartialEq)]
	pub static MaxUnbonding: u32 = 8;
	pub static StakingMinBond: Balance = 10;
	pub storage Nominations: Option<Vec<AccountId>> = None;
}

pub struct StakingMock;
impl StakingMock {
	pub(crate) fn set_bonded_balance(who: AccountId, bonded: Balance) {
		let mut x = BondedBalanceMap::get();
		x.insert(who, bonded);
		BondedBalanceMap::set(&x)
	}
}

impl sp_staking::StakingInterface for StakingMock {
	type Balance = Balance;
	type AccountId = AccountId;
	type CurrencyToVote = ();

	fn minimum_nominator_bond() -> Self::Balance {
		StakingMinBond::get()
	}
	fn minimum_validator_bond() -> Self::Balance {
		StakingMinBond::get()
	}

	fn desired_validator_count() -> u32 {
		unimplemented!("method currently not used in testing")
	}

	fn current_era() -> EraIndex {
		CurrentEra::get()
	}

	fn bonding_duration() -> EraIndex {
		BondingDuration::get()
	}

	fn status(
		_: &Self::AccountId,
	) -> Result<sp_staking::StakerStatus<Self::AccountId>, DispatchError> {
		Nominations::get()
			.map(sp_staking::StakerStatus::Nominator)
			.ok_or(DispatchError::Other("NotStash"))
	}

	fn bond_extra(who: &Self::AccountId, extra: Self::Balance) -> DispatchResult {
		let mut x = BondedBalanceMap::get();
		if let Some(v) = x.get_mut(who) {
			*v += extra;
		}
		BondedBalanceMap::set(&x);
		Ok(())
	}

	fn unbond(who: &Self::AccountId, amount: Self::Balance) -> DispatchResult {
		let mut x = BondedBalanceMap::get();
		*x.get_mut(who).unwrap() = x.get_mut(who).unwrap().saturating_sub(amount);
		BondedBalanceMap::set(&x);
		let mut y = UnbondingBalanceMap::get();
		*y.entry(*who).or_insert(Self::Balance::zero()) += amount;
		UnbondingBalanceMap::set(&y);
		Ok(())
	}

	fn chill(_: &Self::AccountId) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn withdraw_unbonded(who: Self::AccountId, _: u32) -> Result<bool, DispatchError> {
		// Simulates removing unlocking chunks and only having the bonded balance locked
		let mut x = UnbondingBalanceMap::get();
		x.remove(&who);
		UnbondingBalanceMap::set(&x);

		Ok(UnbondingBalanceMap::get().is_empty() && BondedBalanceMap::get().is_empty())
	}

	fn bond(stash: &Self::AccountId, value: Self::Balance, _: &Self::AccountId) -> DispatchResult {
		StakingMock::set_bonded_balance(*stash, value);
		Ok(())
	}

	fn nominate(_: &Self::AccountId, nominations: Vec<Self::AccountId>) -> DispatchResult {
		Nominations::set(&Some(nominations));
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn nominations(_pool_stash: &Self::AccountId) -> Option<Vec<Self::AccountId>> {
		Nominations::get()
	}

	fn stash_by_ctrl(_controller: &Self::AccountId) -> Result<Self::AccountId, DispatchError> {
		unimplemented!("method currently not used in testing")
	}

	fn stake(who: &Self::AccountId) -> Result<Stake<Balance>, DispatchError> {
		match (
			UnbondingBalanceMap::get().get(who).copied(),
			BondedBalanceMap::get().get(who).copied(),
		) {
			(None, None) => Err(DispatchError::Other("balance not found")),
			(Some(v), None) => Ok(Stake { total: v, active: 0 }),
			(None, Some(v)) => Ok(Stake { total: v, active: v }),
			(Some(a), Some(b)) => Ok(Stake { total: a + b, active: b }),
		}
	}

	fn election_ongoing() -> bool {
		unimplemented!("method currently not used in testing")
	}

	fn force_unstake(_who: Self::AccountId) -> sp_runtime::DispatchResult {
		unimplemented!("method currently not used in testing")
	}

	fn is_exposed_in_era(_who: &Self::AccountId, _era: &EraIndex) -> bool {
		unimplemented!("method currently not used in testing")
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add_era_stakers(
		_current_era: &EraIndex,
		_stash: &Self::AccountId,
		_exposures: Vec<(Self::AccountId, Self::Balance)>,
	) {
		unimplemented!("method currently not used in testing")
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_current_era(_era: EraIndex) {
		unimplemented!("method currently not used in testing")
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn max_exposure_page_size() -> sp_staking::Page {
		unimplemented!("method currently not used in testing")
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type SS58Prefix = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u32;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type DbWeight = ();
	type BlockLength = ();
	type BlockWeights = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = RuntimeTask;
}

parameter_types! {
	pub static ExistentialDeposit: Balance = 5;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = frame_support::traits::ConstU32<1024>;
	type MaxReserves = ConstU32<1024>;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

parameter_types! {
	pub static BagThresholds: &'static [VoteWeight] = &[10, 20, 30, 40, 50, 60, 1_000, 2_000, 10_000];
}

impl pallet_bags_list::Config<pallet_bags_list::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BagThresholds = BagThresholds;
	type ScoreProvider = Staking;
	type Score = VoteWeight;
}

// this staking pallet is not actually used in the test, its implemented to satisfy the
// requirement of pallet_staking::Config for the pallet
impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<16>;
	type CurrencyBalance = Balance;
	type UnixTime = pallet_timestamp::Pallet<Self>;
	type CurrencyToVote = SaturatingCurrencyToVote;
	type RewardRemainder = MockOnUnbalancedHandler<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = ();
	type SlashDeferDuration = ();
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BondingDuration = BondingDuration;
	type SessionInterface = ();
	type EraPayout = ();
	type NextNewSession = ();
	type MaxExposurePageSize = ConstU32<512>;
	type MaxControllersInDeprecationBatch = ConstU32<5314>;
	type OffendingValidatorsThreshold = ();
	type ElectionProvider =
		frame_election_provider_support::NoElection<(AccountId, BlockNumber, Staking, ())>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = VoterList;
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = ConstU32<84>;
	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
	type EventListeners = Pools;
	type WeightInfo = ();
}

parameter_types! {
	pub static PostUnbondingPoolsWindow: u32 = 2;
	pub static MaxMetadataLen: u32 = 2;
	pub static CheckLevel: u8 = 255;
	pub const PoolsPalletId: PalletId = PalletId(*b"py/nopls");
	pub const CollatorRewardPool: PalletId = PalletId(*b"py/colrp");
	pub LstCollectionOwner: AccountId = PoolsPalletId::get().into_account_truncating();
	pub const BonusPercentage: Perbill = parameters::nomination_pools::BONUS_PERCENTAGE;
	pub const BaseBonusRewardPercentage: Perbill = parameters::nomination_pools::BASE_BONUS_REWARD_PERCENTAGE;
	pub static UnclaimedBalanceReceiver: AccountId = 7534908;
	pub const CapacityMutationPeriod: EraIndex = 14;
}

pub struct BalanceToU256;
impl Convert<Balance, U256> for BalanceToU256 {
	fn convert(n: Balance) -> U256 {
		n.into()
	}
}

pub struct U256ToBalance;
impl Convert<U256, Balance> for U256ToBalance {
	fn convert(n: U256) -> Balance {
		n.try_into().unwrap()
	}
}

impl pools::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RewardCounter = RewardCounter;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type Staking = StakingMock;
	type PostUnbondingPoolsWindow = PostUnbondingPoolsWindow;
	type PalletId = PoolsPalletId;
	type CollatorRewardPool = CollatorRewardPool;
	type MaxUnbonding = MaxUnbonding;
	type MaxPointsToBalance = frame_support::traits::ConstU8<10>;
	type FungibleHandler = FungibleHandler;
	type MinDuration = ConstU32<{ parameters::nomination_pools::MIN_POOL_DURATION }>;
	type MaxDuration = ConstU32<{ parameters::nomination_pools::MAX_POOL_DURATION }>;
	type PoolCollectionId = ConstU128<{ parameters::nomination_pools::DEGEN_COLLECTION_ID }>;
	type LstCollectionId =
		ConstU128<{ parameters::nomination_pools::STAKED_LST_COLLECTION_ID }>;
	type LstCollectionOwner = LstCollectionOwner;
	type BonusPercentage = BonusPercentage;
	type BaseBonusRewardPercentage = BaseBonusRewardPercentage;
	type UnclaimedBalanceReceiver = UnclaimedBalanceReceiver;
	type CapacityMutationPeriod = CapacityMutationPeriod;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BlockNumberToBalance = BalanceToU256;
	type GlobalMaxCapacity = GlobalMaxCapacity;
	type DefaultMaxCapacity = DefaultMaxCapacity;
	type AttributeKeyMaxLength = AttributeKeyMaxLength;
	type AttributeValueMaxLength = AttributeValueMaxLength;
	type MaxCapacityAttributeKey = MaxCapacityAttributeKey;
	type MaxPoolNameLength = MaxPoolNameLength;
}

type Block = frame_system::mocking::MockBlockU32<Runtime>;
frame_support::construct_runtime!(
	pub struct Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Pools: pools,
		Timestamp: pallet_timestamp,
		Staking: pallet_staking,
		VoterList: pallet_bags_list::<Instance1>,
	}
);

pub struct ExtBuilder {
	members: Vec<(AccountId, Balance)>,
	max_members: Option<u32>,
	max_members_per_pool: Option<u32>,
	min_validator_commission: Option<Perbill>,
	capacity: Balance,
	duration: EraIndex,
	global_max_commission: Option<Perbill>,
	create_pool: bool,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			members: Default::default(),
			max_members: Some(4),
			max_members_per_pool: Some(3),
			min_validator_commission: Some(Perbill::from_percent(1)),
			capacity: 1_000,
			duration: DEFAULT_DURATION,
			global_max_commission: Some(Perbill::from_percent(10)),
			create_pool: true,
		}
	}
}

impl ExtBuilder {
	// Add members to pool 0.
	pub fn add_members(mut self, members: Vec<(AccountId, Balance)>) -> Self {
		self.members = members;
		self
	}

	pub fn ed(self, ed: Balance) -> Self {
		ExistentialDeposit::set(ed);
		self
	}

	pub fn min_bond(self, min: Balance) -> Self {
		StakingMinBond::set(min);
		self
	}

	pub fn min_join_bond(self, min: Balance) -> Self {
		MinJoinBondConfig::set(min);
		self
	}

	pub fn with_check(self, level: u8) -> Self {
		CheckLevel::set(level);
		self
	}

	pub fn max_members(mut self, max: Option<u32>) -> Self {
		self.max_members = max;
		self
	}

	pub fn max_members_per_pool(mut self, max: Option<u32>) -> Self {
		self.max_members_per_pool = max;
		self
	}

	pub fn set_duration(mut self, duration: EraIndex) -> Self {
		self.duration = duration;
		self
	}

	pub fn set_capacity(mut self, capacity: Balance) -> Self {
		self.capacity = capacity;
		self
	}

	pub fn without_pool(mut self) -> Self {
		self.create_pool = false;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let mut storage =
			frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		let _ = crate::GenesisConfig::<Runtime> {
			min_join_bond: MinJoinBondConfig::get(),
			min_create_bond: 2,
			max_pools: Some(2),
			max_members_per_pool: self.max_members_per_pool,
			max_members: self.max_members,
			min_validator_commission: self.min_validator_commission,
			global_max_commission: self.global_max_commission,
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);

		ext.execute_with(|| {
			// for events to be deposited.
			frame_system::Pallet::<Runtime>::set_block_number(1);

			// create collection and token
			let token_id = crate::tests::DEFAULT_TOKEN_ID;

			// make a pool
			let amount_to_bond = Pools::depositor_min_bond();
			Balances::make_free_balance_be(&10, 10_000_000 * UNIT);
			Balances::make_free_balance_be(
				&<Runtime as Config>::LstCollectionOwner::get(),
				10_000_000 * UNIT,
			);

			let pool_id = NextPoolId::<Runtime>::get();
			if self.create_pool {
				setup_multi_tokens(vec![token_id]);
				assert_ok!(Pools::create(
					RawOrigin::Signed(crate::tests::DEFAULT_MANAGER).into(),
					token_id,
					amount_to_bond,
					self.capacity,
					self.duration,
					Default::default(),
				));
			} else {
				setup_multi_tokens(vec![]);
			}

			for (account_id, bonded) in self.members {
				Balances::make_free_balance_be(&account_id, bonded * 2);

				assert_ok!(Pools::bond(
					RawOrigin::Signed(account_id).into(),
					pool_id,
					bonded.into()
				));
			}
		});

		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce()) {
		self.build().execute_with(|| {
			test();
			Pools::do_try_state(CheckLevel::get()).unwrap();
		})
	}
}

/// Creates the pool's NFT collection and lst collection, mints pool NFTs for each `token_id` and
/// sets the pool's config.
fn setup_multi_tokens(token_ids: Vec<TokenId>) {
	// create the nft collection
	let pool_collection_id = <Runtime as Config>::PoolCollectionId::get();
	let lst_collection_id = <Runtime as Config>::LstCollectionId::get();

	FungibleHandler::force_create_collection(
		RawOrigin::Root.into(),
		10,
		pool_collection_id,
		Box::new(DefaultCollectionDescriptor {
			policy: DefaultCollectionPolicyDescriptor {
				mint: DefaultMintPolicyDescriptor {
					max_token_count: None,
					max_token_supply: Some(1),
					force_collapsing_supply: false
				},
				..Default::default()
			},
			..Default::default()
		}),
	)
	.unwrap();

	// mint collection for `lst` tokens
	FungibleHandler::force_create_collection(
		RawOrigin::Root.into(),
		<Runtime as Config>::LstCollectionOwner::get(),
		lst_collection_id,
		Default::default(),
	)
	.unwrap();

	for token_id in token_ids {
		mint_pool_token(token_id, 10);
	}
}

pub fn unsafe_set_state(pool_id: PoolId, state: PoolState) {
	BondedPools::<Runtime>::try_mutate(pool_id, |maybe_bonded_pool| {
		maybe_bonded_pool.as_mut().ok_or(()).map(|bonded_pool| {
			bonded_pool.state = state;
		})
	})
	.unwrap()
}

parameter_types! {
	storage PoolsEvents: u32 = 0;
	storage BalancesEvents: u32 = 0;
	/// Bound for `MockRewards` storage
	pub const MockRewardsBound: u32 = 50;
	/// Storage used for mocking rewards in [`Pallet::payout_rewards`]
	pub storage MockRewards: BoundedBTreeMap<(AccountId, EraIndex), BoundedBTreeMap<PoolId, Balance, MockRewardsBound>, MockRewardsBound> = Default::default();
}

/// All events of this pallet.
pub fn pool_events_since_last_call() -> Vec<super::Event<Runtime>> {
	let events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Pools(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();
	let already_seen = PoolsEvents::get();
	PoolsEvents::set(&(events.len() as u32));
	events.into_iter().skip(already_seen as usize).collect()
}

/// filters `$events` by `$event_pattern`
macro_rules! filter_events {
	($events: expr, $event_pattern:pat_param) => {
		$events
			.iter()
			.filter(|e| matches!(e, $event_pattern))
			.cloned()
			.collect::<Vec<super::Event<Runtime>>>()
	};
}

/// All events of the `Balances` pallet.
pub fn balances_events_since_last_call() -> Vec<pallet_balances::Event<Runtime>> {
	let events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Balances(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();
	let already_seen = BalancesEvents::get();
	BalancesEvents::set(&(events.len() as u32));
	events.into_iter().skip(already_seen as usize).collect()
}

/// Same as `fully_unbond`, in permissioned setting.
pub fn fully_unbond_permissioned(pool_id: PoolId, member: AccountId) -> DispatchResult {
	let points = Pools::member_points(pool_id, member);
	let result = Pools::unbond(RuntimeOrigin::signed(member), pool_id, member, points);
	if result.is_ok() {
		assert_eq!(Pools::member_points(pool_id, member), 0);
	}
	result
}

/// Helper to run a specified amount of blocks.
pub fn run_blocks(n: u32) {
	let current_block = System::block_number();
	run_to_block(n + current_block);
}

/// Helper to run to a specific block.
pub fn run_to_block(n: u32) {
	let current_block = System::block_number();
	assert!(n > current_block);
	while System::block_number() < n {
		<Pallet<Runtime> as OnFinalize<BlockNumber>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Pools::on_initialize(System::block_number());
	}
}

/// Sets a mock reward for [`Pallet::payout_rewards`]
pub fn set_reward(validator: AccountId, era: EraIndex, pool_id: PoolId, amount: Balance) {
	let mut rewards = MockRewards::get().into_inner();
	let pool_rewards = rewards.entry((validator, era)).or_default();
	pool_rewards.try_insert(pool_id, amount).unwrap();
	MockRewards::set(&rewards.try_into().unwrap());
}

/// Gets a mock reward for [`Pallet::payout_rewards`]
pub fn get_reward(validator: AccountId, era: EraIndex, pool_id: PoolId) -> Balance {
	MockRewards::get()
		.get(&(validator, era))
		.and_then(|rewards| rewards.get(&pool_id))
		.copied()
		.unwrap_or_default()
}

/// Gets the total mock reward for all pools in `era`
pub fn get_total_reward(validator: AccountId, era: EraIndex) -> Balance {
	MockRewards::get()
		.get(&(validator, era))
		.map(|rewards| rewards.values().sum())
		.unwrap_or_default()
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn u256_to_balance_convert_works() {
		assert_eq!(U256ToBalance::convert(0u32.into()), Zero::zero());
		assert_eq!(U256ToBalance::convert(Balance::MAX.into()), Balance::MAX)
	}

	#[test]
	#[should_panic]
	fn u256_to_balance_convert_panics_correctly() {
		U256ToBalance::convert(U256::from(Balance::MAX).saturating_add(1u32.into()));
	}

	#[test]
	fn balance_to_u256_convert_works() {
		assert_eq!(BalanceToU256::convert(0u32.into()), U256::zero());
		assert_eq!(BalanceToU256::convert(Balance::MAX), Balance::MAX.into())
	}
}
