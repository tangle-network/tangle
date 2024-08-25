
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen, VoteWeight,
};
use frame_support::{
	assert_ok, derive_impl,
	dispatch::RawOrigin,
	pallet_prelude::*,
	parameter_types,
	traits::{ConstU64, Get, OneSessionHandler},
	PalletId,
};
use pallet_nomination_pools::CollectionIdOf;
use sp_core::{ConstU128, ConstU8};
use sp_runtime::{
	testing::UintAuthorityId,
	traits::{AccountIdConversion, Convert, IdentityLookup, One},
	BuildStorage, FixedU128, Perbill,
};
use sp_staking::{EraIndex, SessionIndex};

pub type AccountId = u128;
pub type AccountIndex = u32;
pub type Balance = u128;

parameter_types! {
	/// Block & extrinsics weights: base values and limits.
	pub RuntimeBlockWeights: frame_system::limits::BlockWeights = Default::default();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = Nonce;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = RuntimeTask;
}

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub other: OtherSessionHandler,
	}
}

pub struct OtherSessionHandler;
impl OneSessionHandler<AccountId> for OtherSessionHandler {
	type Key = UintAuthorityId;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_disabled(_validator_index: u32) {}
}

impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
	type Public = UintAuthorityId;
}

parameter_types! {
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Runtime, Staking>;
	type Keys = SessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionHandler = (OtherSessionHandler,);
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Runtime>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

parameter_types! {
	pub static ExistentialDeposit: Balance = 5;
}
impl pallet_balances::Config for Runtime {
	type MaxLocks = ();
	type MaxReserves = ConstU32<1000>;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

pallet_staking_reward_curve::build! {
	const I_NPOS: sp_runtime::curve::PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub static BondingDuration: u32 = 3;
	pub static EpochDuration: u64 = 10;
	pub static SessionsPerEra: SessionIndex = 3;
}

pub struct EraPayout;
impl pallet_staking::EraPayout<Balance> for EraPayout {
	fn era_payout(
		_total_staked: Balance,
		total_issuance: Balance,
		_era_duration_millis: u64,
	) -> (Balance, Balance) {
		(0, 0)
	}
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = pallet_timestamp::Pallet<Self>;
	type CurrencyToVote = ();
	type NominationsQuota = pallet_staking::FixedNominationsQuota<16>;
	type RuntimeEvent = RuntimeEvent;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = ConstU32<3>;
	type SlashDeferDuration = ();
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BondingDuration = BondingDuration;
	type SessionInterface = ();
	type EraPayout = EraPayout;
	type NextNewSession = (); // TODO: need to integrate sessions for EraStakersClipped to get updated
	type MaxExposurePageSize = ConstU32<64>;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type OffendingValidatorsThreshold = ();
	type ElectionProvider =
		frame_election_provider_support::onchain::OnChainExecution<OnChainSeqPhragmen>;
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
	pub static BagThresholds: &'static [VoteWeight] = &[10, 20, 30, 40, 50, 60, 1_000, 2_000, 10_000];
}

impl pallet_bags_list::Config<pallet_bags_list::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BagThresholds = BagThresholds;
	type ScoreProvider = Staking;
	type Score = VoteWeight;
}

parameter_types! {
	pub static ElectionsBounds: ElectionBounds = ElectionBoundsBuilder::default().build();
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<AccountId, Perbill>;
	type DataProvider = Staking;
	type WeightInfo = ();
	type MaxWinners = ConstU32<100>;
	type Bounds = ElectionsBounds;
}

pub struct BalanceToU256;
impl Convert<Balance, sp_core::U256> for BalanceToU256 {
	fn convert(n: Balance) -> sp_core::U256 {
		n.into()
	}
}

pub struct U256ToBalance;
impl Convert<sp_core::U256, Balance> for U256ToBalance {
	fn convert(n: sp_core::U256) -> Balance {
		n.try_into().unwrap()
	}
}

parameter_types! {
	pub static PostUnbondingPoolsWindow: u32 = 10;
	pub const PoolsPalletId: PalletId = PalletId(*b"py/nopls");
	pub const CollatorRewardPool: PalletId = PalletId(*b"py/colrp");
	pub const MaxPointsToBalance: u8 = 10;
	pub LstCollectionOwner: AccountId = PoolsPalletId::get().into_account_truncating();
	pub static UnclaimedBalanceReceiver: AccountId = 912834;
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

impl pallet_nomination_pools::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RewardCounter = FixedU128;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type Staking = Staking;
	type PostUnbondingPoolsWindow = PostUnbondingPoolsWindow;
	type MaxUnbonding = ConstU32<8>;
	type PalletId = PoolsPalletId;
	type CollatorRewardPool = CollatorRewardPool;
	type MaxPointsToBalance = MaxPointsToBalance;
	type Fungibles = Fungibles;
	type MinDuration = ConstU32<{ parameters::nomination_pools::MIN_POOL_DURATION }>;
	type MaxDuration = ConstU32<{ parameters::nomination_pools::MAX_POOL_DURATION }>;
	type PoolCollectionId = ConstU128<{ parameters::nomination_pools::DEGEN_COLLECTION_ID }>;
	type LstCollectionId =
		ConstU128<{ parameters::nomination_pools::LST_COLLECTION_ID }>;
	type LstCollectionOwner = LstCollectionOwner;
	type BonusPercentage = BonusPercentage;
	type BaseBonusRewardPercentage = BaseBonusRewardPercentage;
	type UnclaimedBalanceReceiver = UnclaimedBalanceReceiver;
	type CapacityMutationPeriod = CapacityMutationPeriod;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type GlobalMaxCapacity = GlobalMaxCapacity;
	type DefaultMaxCapacity = DefaultMaxCapacity;
	type AttributeKeyMaxLength = AttributeKeyMaxLength;
	type AttributeValueMaxLength = AttributeValueMaxLength;
	type MaxCapacityAttributeKey = MaxCapacityAttributeKey;
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::Config for Runtime {}

type Block = frame_system::mocking::MockBlockU32<Runtime>;

frame_support::construct_runtime!(
	pub enum Runtime
	{
		System: frame_system::{Pallet, Call, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
		Session: pallet_session,
		Historical: pallet_session::historical,
		VoterList: pallet_bags_list::<Instance1>::{Pallet, Call, Storage, Event<T>},
		Fungibles: pallet_multi_tokens,
		Pools: pallet_nomination_pools::{Pallet, Call, Storage, Event<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let mut storage = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	pallet_nomination_pools::GenesisConfig::<Runtime> {
		min_join_bond: 2,
		min_create_bond: 2,
		max_pools: Some(3),
		max_members_per_pool: Some(5),
		max_members: Some(3 * 5),
		min_validator_commission: None,
		global_max_commission: Some(Perbill::from_percent(10)),
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let lst_collection_owner =
		<<Runtime as pallet_nomination_pools::Config>::PalletId as Get<PalletId>>::get()
			.into_account_truncating();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(10, 10_000 * UNIT),
			(20, 100),
			(21, 100),
			(22, 100),
			(100, 100),
			(lst_collection_owner, 100 * UNIT),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::from(storage);

	ext.execute_with(|| {
		// for events to be deposited.
		frame_system::Pallet::<Runtime>::set_block_number(1);

		// set some limit for nominations.
		assert_ok!(Staking::set_staking_configs(
			RuntimeOrigin::root(),
			pallet_staking::ConfigOp::Set(10), // minimum nominator bond
			pallet_staking::ConfigOp::Noop,
			pallet_staking::ConfigOp::Noop,
			pallet_staking::ConfigOp::Noop,
			pallet_staking::ConfigOp::Noop,
			pallet_staking::ConfigOp::Noop,
			pallet_staking::ConfigOp::Noop,
		));

		let token_id = crate::tests::DEFAULT_TOKEN_ID;

		setup_multi_tokens(vec![token_id])
	});

	ext
}

/// Creates the pool's nft collection, mints NFT for each `token_id` and sets the pool's config
fn setup_multi_tokens(token_ids: Vec<AssetId>) {
	// create the nft collection
	let pool_collection_id =
		<<Runtime as pallet_nomination_pools::Config>::PoolCollectionId as Get<
			CollectionIdOf<Runtime>,
		>>::get();

	// collection for lst
	let lst_collection_id =
		<<Runtime as pallet_nomination_pools::Config>::LstCollectionId as Get<
			CollectionIdOf<Runtime>,
		>>::get();

	Fungibles::force_create_collection(
		RawOrigin::Root.into(),
		10,
		pool_collection_id,
		Default::default(),
	)
	.unwrap();

	// mint collection for `lst` tokens
	Fungibles::force_create_collection(
		RawOrigin::Root.into(),
		<<Runtime as pallet_nomination_pools::Config>::PalletId as Get<PalletId>>::get()
			.into_account_truncating(),
		lst_collection_id,
		Default::default(),
	)
	.unwrap();

	for token_id in token_ids {
		mint(token_id, 10);
	}
}

parameter_types! {
	static ObservedEventsPools: usize = 0;
	static ObservedEventsStaking: usize = 0;
	static ObservedEventsBalances: usize = 0;
}

pub fn pool_events_since_last_call() -> Vec<pallet_nomination_pools::Event<Runtime>> {
	let events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Pools(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();
	let already_seen = ObservedEventsPools::get();
	ObservedEventsPools::set(events.len());
	events.into_iter().skip(already_seen).collect()
}

pub fn staking_events_since_last_call() -> Vec<pallet_staking::Event<Runtime>> {
	let events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Staking(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();
	let already_seen = ObservedEventsStaking::get();
	ObservedEventsStaking::set(events.len());
	events.into_iter().skip(already_seen).collect()
}
