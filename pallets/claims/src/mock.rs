use super::*;
use pallet_evm::HashedAddressMapping;
use secp_utils::*;
use sp_core::{sr25519, ConstU32, Pair, H256};
use sp_std::convert::TryFrom;
// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
use crate::{pallet as pallet_airdrop_claims, sr25519_utils::sub, tests::get_bounded_vec};
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{OnFinalize, OnInitialize, WithdrawReasons},
};
use pallet_balances;
use sp_runtime::{
	traits::{BlakeTwo256, Identity, IdentityLookup},
	AccountId32, BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		VestingPallet: pallet_vesting,
		ClaimsPallet: pallet_airdrop_claims,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Block = Block;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub const MinVestedTransfer: u64 = 1;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = Identity;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	pub Prefix: &'static [u8] = b"Pay RUSTs to the TEST account:";
}
ord_parameter_types! {
	pub const Six: AccountId32 = get_multi_address_account_id(6).to_account_id_32();
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type VestingSchedule = VestingPallet;
	type ForceOrigin = frame_system::EnsureRoot<AccountId32>;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Prefix = Prefix;
	type MaxVestingSchedules = ConstU32<8>;
	type MoveClaimOrigin = frame_system::EnsureSignedBy<Six, AccountId32>;
	type WeightInfo = TestWeightInfo;
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		ClaimsPallet::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}

pub fn alice() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}
pub fn bob() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}
pub fn dave() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Dave")).unwrap()
}
pub fn eve() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Eve")).unwrap()
}
pub fn frank() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Frank")).unwrap()
}

pub fn get_multi_address_account_id(id: u8) -> MultiAddress {
	MultiAddress::Native(AccountId32::new([id; 32]))
}

pub fn alice_sr25519() -> sr25519::Pair {
	sr25519::Pair::from_string(&format!("//Alice"), None).expect("static values are valid; qed")
}

pub fn bob_sr25519() -> sr25519::Pair {
	sr25519::Pair::from_string(&format!("//Bob"), None).expect("static values are valid; qed")
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	// We use default for brevity, but you can configure as desired if needed.
	pallet_balances::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	pallet_airdrop_claims::GenesisConfig::<Test> {
		claims: vec![
			(eth(&alice()), 100, None),
			(eth(&dave()), 200, Some(StatementKind::Regular)),
			(eth(&eve()), 300, Some(StatementKind::Safe)),
			(eth(&frank()), 400, None),
			(sub(&alice_sr25519()), 500, None),
			(sub(&bob_sr25519()), 600, None),
		],
		vesting: vec![(eth(&alice()), get_bounded_vec((50, 10, 1)))],
		expiry: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
