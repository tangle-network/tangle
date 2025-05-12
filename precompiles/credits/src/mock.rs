use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, Everything},
    weights::Weight,
};
use sp_core::{H160, H256, U256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32, Permill,
};

use pallet_evm::{
    Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
    IdentityAddressMapping, PrecompileSet,
};
use precompile_utils::{
    precompile_set::*,
    testing::{Address, MockAccount},
};

use super::*;
use std::{collections::BTreeMap, str::FromStr};

pub type AccountId = AccountId32;
pub type AssetId = u32;
pub type Balance = u128;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Credits: pallet_credits,
        Evm: pallet_evm,
        MultiAssetDelegation: pallet_multi_asset_delegation,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Permill = Permill::one();
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = Everything;
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Runtime {
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type MaxLocks = ();
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxHolds = ();
    type MaxFreezes = ();
}

// Configure the MultiAssetDelegation pallet for testing
parameter_types! {
    pub const MaxLockIdLength: u32 = 32;
    pub const MaxDelegatorsPerOperator: u32 = 100;
    pub const MaxDepositPerAddress: u32 = 100;
    pub const MaxEVMLocks: u32 = 100;
    pub const TntCurrency: AssetId = 1;
}

impl pallet_multi_asset_delegation::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MultiAsset = Balances;
    type AssetId = AssetId;
    type TntCurrency = TntCurrency;
    type MaxLockIdLength = MaxLockIdLength;
    type MaxDelegatorsPerOperator = MaxDelegatorsPerOperator;
    type MaxDepositPerAddress = MaxDepositPerAddress;
    type MaxEVMLocks = MaxEVMLocks;
    type WeightInfo = ();
    type GetNativeCurrencyId = TntCurrency;
}

// Configure the Credits pallet for testing
parameter_types! {
    pub const TntAssetId: AssetId = 1;
    pub const BurnConversionRate: Balance = 10;
    pub const ClaimWindowBlocks: BlockNumber = 100;
    pub CreditBurnRecipient: Option<AccountId> = None;
    pub const MaxOffchainAccountIdLength: u32 = 64;
    pub const MaxStakeTiers: u32 = 5;
}

impl pallet_credits::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type AssetId = AssetId;
    type TntAssetId = TntAssetId;
    type MultiAssetDelegationInfo = MultiAssetDelegation;
    type BurnConversionRate = BurnConversionRate;
    type ClaimWindowBlocks = ClaimWindowBlocks;
    type CreditBurnRecipient = CreditBurnRecipient;
    type MaxOffchainAccountIdLength = MaxOffchainAccountIdLength;
    type MaxStakeTiers = MaxStakeTiers;
    type WeightInfo = ();
}

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(u64::MAX);
    pub PrecompilesValue: Precompiles<Runtime> = Precompiles::<_>::new();
    pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

pub struct MockPrecompile;
impl MockPrecompile {
    pub fn new_mock_precompile(
        context: &Context,
    ) -> Box<dyn PrecompileHandle> {
        Box::new(context.clone())
    }
}

pub struct Context {
    pub address: H160,
    pub caller: H160,
    pub apparent_value: U256,
}

impl PrecompileHandle for Context {
    fn call_type(&self) -> fp_evm::CallType {
        fp_evm::CallType::Call
    }

    fn gas_limit(&self) -> Option<u64> {
        Some(u64::MAX)
    }

    fn gas_price(&self) -> U256 {
        U256::zero()
    }

    fn gas_payment(&self) -> Result<(), ExitError> {
        Ok(())
    }

    fn record_cost(&mut self, _: u64) -> Result<(), ExitError> {
        Ok(())
    }

    fn record_external_cost(
        &mut self,
        _: Option<u64>,
        _: Option<u64>,
        _: Option<u64>,
    ) -> Result<(), ExitError> {
        Ok(())
    }

    fn refund_external_cost(&mut self, _: Option<u64>) {}

    fn remaining_gas(&self) -> u64 {
        u64::MAX
    }

    fn log(
        &mut self,
        _: H160,
        _: Vec<H256>,
        _: Vec<u8>,
    ) -> Result<(), ExitError> {
        Ok(())
    }

    fn code_address(&self) -> H160 {
        self.address
    }

    fn input(&self) -> &[u8] {
        &[]
    }

    fn sender(&self) -> H160 {
        self.caller
    }

    fn transfer(&mut self, _: &H160, _: U256) -> Result<(), ExitError> {
        Ok(())
    }

    fn reset_context(&mut self) {}

    fn is_static(&self) -> bool {
        false
    }

    fn context(&self) -> &fp_evm::Context {
        // This is only needed for the read-only view functions
        const DUMMY_CONTEXT: fp_evm::Context = fp_evm::Context {
            address: H160::zero(),
            caller: H160::zero(),
            apparent_value: U256::zero(),
        };
        &DUMMY_CONTEXT
    }
}

#[precompile_utils::precompile_name_from_address]
pub type PrecompileName = ();

pub const PRECOMPILE_ADDRESS: Address = Address(
    H160::from_str("0x0000000000000000000000000000000000000800").unwrap()
);

pub type Precompiles<R> = PrecompileSetBuilder<
    R,
    (
        PrecompileAt<
            PRECOMPILE_ADDRESS,
            CreditsPrecompile<R>,
        >,
    ),
>;

impl pallet_evm::Config for Runtime {
    type FeeCalculator = ();
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAddressNever<AccountId>;
    type AddressMapping = IdentityAddressMapping;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = Precompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = ();
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = ();
    type GasLimitPovSizeRatio = ();
    type Timestamp = ();
    type WeightInfo = ();
}

pub(crate) struct ExtBuilder {
    // endowed accounts with balances
    balances: Vec<(AccountId, Balance)>,
    // [account, amount, threshold, rate_per_block]
    stake_tiers: Vec<(Balance, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> ExtBuilder {
        ExtBuilder {
            balances: vec![],
            stake_tiers: vec![
                (100_000, 10),
                (500_000, 50),
                (1_000_000, 100),
            ],
        }
    }
}

impl ExtBuilder {
    pub(crate) fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub(crate) fn stake_tiers(mut self, stake_tiers: Vec<(Balance, Balance)>) -> Self {
        self.stake_tiers = stake_tiers;
        self
    }

    pub(crate) fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self.balances,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut stake_tiers = Vec::new();
        for (threshold, rate_per_block) in self.stake_tiers {
            stake_tiers.push(StakeTier {
                threshold,
                rate_per_block,
            });
        }

        pallet_credits::GenesisConfig::<Runtime> {
            stake_tiers,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub(crate) fn accounts() -> BTreeMap<&'static str, MockAccount> {
    BTreeMap::from([
        ("alice", MockAccount(Address::from(H160::from_low_u64_be(1)), 1, H256::repeat_byte(1))),
        ("bob", MockAccount(Address::from(H160::from_low_u64_be(2)), 2, H256::repeat_byte(2))),
    ])
}