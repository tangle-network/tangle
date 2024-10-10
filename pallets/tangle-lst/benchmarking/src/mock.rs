// This file is part of Substrate.

use super::*;
use frame_election_provider_support::VoteWeight;
use frame_support::traits::AsEnsureOriginWithArg;
use frame_support::traits::Hooks;
use frame_support::traits::OnFinalize;
use frame_support::{assert_ok, derive_impl, parameter_types, PalletId};
use frame_system::RawOrigin;
use pallet_tangle_lst::BondedPools;
use pallet_tangle_lst::Config;
use pallet_tangle_lst::Event;
use pallet_tangle_lst::LastPoolId;
use pallet_tangle_lst::PoolId;
use pallet_tangle_lst::PoolState;
use sp_core::U256;
use sp_runtime::traits::ConstU128;
use sp_runtime::traits::ConstU32;
use sp_runtime::traits::ConstU64;
use sp_runtime::traits::Convert;
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;
use sp_runtime::DispatchResult;
use sp_runtime::Perbill;
use sp_runtime::{BuildStorage, FixedU128};
use sp_runtime_interface::sp_tracing;
use sp_staking::EraIndex;
use sp_staking::{OnStakingUpdate, Stake};
use sp_std::collections::btree_map::BTreeMap;

pub type BlockNumber = u64;
pub type AccountId = u128;
pub type Balance = u128;
pub type RewardCounter = FixedU128;
pub type AssetId = u32;
// This sneaky little hack allows us to write code exactly as we would do in the pallet in the tests
// as well, e.g. `StorageItem::<T>::get()`.
pub type T = Runtime;
pub type Currency = <T as pallet_tangle_lst::Config>::Currency;

// Ext builder creates a pool with id 1.
pub fn default_bonded_account() -> AccountId {
	Lst::create_bonded_account(1)
}

// Ext builder creates a pool with id 1.
pub fn default_reward_account() -> AccountId {
	Lst::create_reward_account(1)
}

parameter_types! {
	pub static MinJoinBondConfig: Balance = 2;
	pub static CurrentEra: EraIndex = 0;
	pub static BondingDuration: EraIndex = 3;
	pub storage BondedBalanceMap: BTreeMap<AccountId, Balance> = Default::default();
	// map from a user to a vec of eras and amounts being unlocked in each era.
	pub storage UnbondingBalanceMap: BTreeMap<AccountId, Vec<(EraIndex, Balance)>> = Default::default();
	#[derive(Clone, PartialEq)]
	pub static MaxUnbonding: u32 = 8;
	pub static StakingMinBond: Balance = 10;
	pub storage Nominations: Option<Vec<AccountId>> = None;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type SS58Prefix = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Block = Block;
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
	type MaxLocks = frame_support::traits::ConstU32<1024>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<1>;
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
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
	pub const RewardCurve: &'static sp_runtime::curve::PiecewiseLinear<'static> = &I_NPOS;
}
#[derive_impl(pallet_staking::config_preludes::TestDefaultConfig)]
impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = pallet_timestamp::Pallet<Self>;
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type ElectionProvider =
		frame_election_provider_support::NoElection<(AccountId, BlockNumber, Staking, ())>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = VoterList;
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type EventListeners = ();
}

parameter_types! {
	pub static BagThresholds: &'static [VoteWeight] = &[10, 20, 30, 40, 50, 60, 1_000, 2_000, 10_000];
}

impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BagThresholds = BagThresholds;
	type ScoreProvider = Staking;
	type Score = VoteWeight;
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

parameter_types! {
	pub static PostUnbondingPoolsWindow: u32 = 2;
	pub static MaxMetadataLen: u32 = 2;
	pub static CheckLevel: u8 = 255;
	pub const PoolsPalletId: PalletId = PalletId(*b"py/nopls");
}

impl pallet_tangle_lst::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RewardCounter = RewardCounter;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type Staking = Staking;
	type PostUnbondingPoolsWindow = PostUnbondingPoolsWindow;
	type PalletId = PoolsPalletId;
	type MaxMetadataLen = MaxMetadataLen;
	type MaxUnbonding = MaxUnbonding;
	type MaxNameLength = ConstU32<50>;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type PoolId = PoolId;
	type ForceOrigin = frame_system::EnsureRoot<u128>;
	type MaxPointsToBalance = frame_support::traits::ConstU8<10>;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = AssetId;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u128>>;
	type ForceOrigin = frame_system::EnsureRoot<u128>;
	type AssetDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;
frame_support::construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Staking: pallet_staking,
		VoterList: pallet_bags_list::<Instance1>,
		Assets: pallet_assets,
		Lst: pallet_tangle_lst,
	}
);

pub struct ExtBuilder {
	members: Vec<(AccountId, Balance)>,
	max_members: Option<u32>,
	max_members_per_pool: Option<u32>,
	global_max_commission: Option<Perbill>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			members: Default::default(),
			max_members: Some(4),
			max_members_per_pool: Some(3),
			global_max_commission: Some(Perbill::from_percent(90)),
		}
	}
}

#[cfg_attr(feature = "fuzzing", allow(dead_code))]
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

	pub fn global_max_commission(mut self, commission: Option<Perbill>) -> Self {
		self.global_max_commission = commission;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let mut storage =
			frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		let _ = pallet_tangle_lst::GenesisConfig::<Runtime> {
			min_join_bond: MinJoinBondConfig::get(),
			min_create_bond: 2,
			max_pools: Some(2),
			max_members_per_pool: self.max_members_per_pool,
			max_members: self.max_members,
			global_max_commission: self.global_max_commission,
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);

		ext.execute_with(|| {
			use frame_support::traits::Currency;
			// for events to be deposited.
			frame_system::Pallet::<Runtime>::set_block_number(1);

			// make a pool
			let amount_to_bond = Lst::depositor_min_bond();
			<Runtime as pallet_tangle_lst::Config>::Currency::make_free_balance_be(
				&10u32.into(),
				amount_to_bond * 5,
			);
			assert_ok!(Lst::create(
				RawOrigin::Signed(10).into(),
				amount_to_bond,
				900,
				901,
				902,
				Default::default()
			));
			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1]));
			let last_pool = LastPoolId::<Runtime>::get();
			for (account_id, bonded) in self.members {
				<Runtime as pallet_tangle_lst::Config>::Currency::make_free_balance_be(
					&account_id,
					bonded * 2,
				);
				assert_ok!(Lst::join(RawOrigin::Signed(account_id).into(), bonded, last_pool));
			}
		});

		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce()) {
		self.build().execute_with(|| {
			test();
			//Pools::do_try_state(CheckLevel::get()).unwrap();
		})
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
}

/// Helper to run a specified amount of blocks.
pub fn run_blocks(n: u64) {
	let current_block = System::block_number();
	run_to_block(n + current_block);
}

/// Helper to run to a specific block.
pub fn run_to_block(n: u64) {
	let current_block = System::block_number();
	assert!(n > current_block);
	while System::block_number() < n {
		<pallet_tangle_lst::Pallet<Runtime> as OnFinalize<u64>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Lst::on_initialize(System::block_number());
	}
}

/// All events of this pallet.
pub fn pool_events_since_last_call() -> Vec<pallet_tangle_lst::Event<Runtime>> {
	let events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Lst(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();
	let already_seen = PoolsEvents::get();
	PoolsEvents::set(&(events.len() as u32));
	events.into_iter().skip(already_seen as usize).collect()
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
	let points = Assets::balance(pool_id, member);
	Lst::unbond(RuntimeOrigin::signed(member), member, pool_id, points)
}

#[derive(PartialEq, Debug)]
pub enum RewardImbalance {
	// There is no reward deficit.
	Surplus(Balance),
	// There is a reward deficit.
	Deficit(Balance),
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	let _ = pallet_tangle_lst::GenesisConfig::<Runtime> {
		min_join_bond: 2,
		min_create_bond: 2,
		max_pools: Some(3),
		max_members_per_pool: Some(3),
		max_members: Some(3 * 3),
		global_max_commission: Some(Perbill::from_percent(50)),
	}
	.assimilate_storage(&mut storage);
	sp_io::TestExternalities::from(storage)
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
