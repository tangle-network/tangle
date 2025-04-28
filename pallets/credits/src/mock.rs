// Mock runtime for pallet-credits testing
#![cfg(test)]

use crate::{self as pallet_credits, types::*, Config, Pallet as CreditsPallet};
use frame_support::{
	bounded_vec, construct_runtime, derive_impl, parameter_types,
	traits::{
		fungible::Credit, // Needed for Balance pallet AccountData
		tokens::fungibles::{
			self, DepositConsequence, Inspect, Mutate, Transfer, WithdrawConsequence,
		},
		tokens::{Fortunes, Precision}, // Import Precision/Fortunes here
		AsEnsureOriginWithArg,
		ConstU128,
		ConstU16,
		ConstU32,
		ConstU64,
		EnsureOrigin,
		Everything,
		OneSessionHandler,
	},
	weights::Weight,
	BoundedVec, PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::{sr25519, H256};
use sp_runtime::{
	generic,
	testing::UintAuthorityId,
	traits::{
		AccountIdLookup, BlakeTwo256, BlockNumberProvider, ConvertInto, IdentifyAccount,
		IdentityLookup, MaybeDisplay, OpaqueKeys, Saturating, Verify, Zero,
	},
	AccountId32, BoundToRuntimeAppPublic, BuildStorage, DispatchError, DispatchResult, Perbill,
};
use sp_std::{
	cell::RefCell, collections::btree_map::BTreeMap, fmt::Debug, marker::PhantomData, vec::Vec,
};
use tangle_primitives::{
	traits::{MultiAssetDelegationInfo, RewardsManager, ServiceManager},
	types::rewards::LockMultiplier,
};

// Type definitions
type Balance = u128;
type AssetId = u128;
type BlockNumber = u64;
type Signature = sp_runtime::testing::MultiSignature;
type AccountPublic = <Signature as Verify>::Signer;
type AccountId = AccountId32; // Use AccountId32 for compatibility
type Header = generic::Header<BlockNumber, BlakeTwo256>;
type Block = generic::Block<Header, UncheckedExtrinsic>;
type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;

// Constants
pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([3u8; 32]);
pub const ADMIN: AccountId = AccountId32::new([99u8; 32]);
pub const TNT_ASSET_ID: AssetId = 0; // Assume 0 is native / TNT for tests
pub const WEEK: BlockNumber = 100_800; // Approx blocks per week (6s block time)

// Construct Runtime must come BEFORE type aliases that depend on it (RuntimeCall, etc)
construct_runtime! {
	pub enum Test { // Changed name to Test to avoid conflict
		System: system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Historical: pallet_session_historical::{Pallet},
		MultiAssetDelegation: pallet_multi_asset_delegation::{Pallet, Call, Storage, Event<T>},
		Credits: pallet_credits::{Pallet, Call, Storage, Event<T>},
	}
}

// Define SignedExtra needed by construct_runtime
type SignedExtra = (
	system::CheckNonZeroSender<Test>,
	system::CheckSpecVersion<Test>,
	system::CheckTxVersion<Test>,
	system::CheckGenesis<Test>,
	system::CheckEra<Test>,
	system::CheckNonce<Test>,
	system::CheckWeight<Test>,
);
type OriginCaller = <RuntimeOrigin as frame_support::traits::OriginTrait>::Caller;

// Build genesis storage
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap();

	// Configure Balances
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 1_000_000),
			(BOB, 1_000_000),
			(CHARLIE, 1_000_000),
			(ADMIN, 1_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// Configure Assets - Create TNT Asset (ID 0)
	pallet_assets::GenesisConfig::<Test> {
		assets: vec![
			(TNT_ASSET_ID, ADMIN, true, 1), // Admin is owner, min_balance = 1
		],
		metadata: vec![
			(TNT_ASSET_ID, b"TNT Token".to_vec(), b"TNT".to_vec(), 12), // Name, Symbol, Decimals
		],
		accounts: vec![
			(TNT_ASSET_ID, ALICE, 10_000), // Give Alice initial TNT
			(TNT_ASSET_ID, BOB, 5_000),    // Give Bob initial TNT
		],
		next_asset_id: Some(TNT_ASSET_ID + 1),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// Configure MultiAssetDelegation genesis
	// Initialize Alice and Bob as operators first (needs native balance)
	pallet_multi_asset_delegation::GenesisConfig::<Test> {
		operators: vec![(ALICE, 1000), (BOB, 1000)], // Bond native currency
		delegations: vec![],                         // Start with no delegations
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		// Delegate TNT after genesis (requires operator setup first)
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(ALICE),
			ALICE, // Delegate to self
			tangle_primitives::services::Asset::Custom(TNT_ASSET_ID),
			1000 // Stake 1000 TNT (Tier 3)
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(BOB),
			BOB, // Delegate to self
			tangle_primitives::services::Asset::Custom(TNT_ASSET_ID),
			150 // Stake 150 TNT (Tier 1)
		));

		// Clear pallet-credits state explicitly
		pallet_credits::CreditBalances::<Test>::remove_all(None);
		pallet_credits::LinkedAccounts::<Test>::remove_all(None);
		pallet_credits::LastRewardUpdateBlock::<Test>::remove_all(None);
		pallet_credits::LastInteractionBlock::<Test>::remove_all(None);
	});
	ext
}

// Helper to run to a specific block
pub fn run_to_block(n: BlockNumber) {
	while System::block_number() < n {
		// Simulate block progression lifecycle
		let block_num = System::block_number() + 1;
		if System::block_number() > 0 {
			// Don't finalize block 0
			// Order is important: finalize first, then set block number, then initialize
			System::on_finalize(System::block_number());
			Session::on_finalize(System::block_number());
			MultiAssetDelegation::on_finalize(System::block_number());
			// Add other pallets' on_finalize if needed
		}
		System::set_block_number(block_num);
		System::on_initialize(block_num);
		Timestamp::on_initialize(block_num);
		Session::on_initialize(block_num);
		Staking::on_initialize(block_num);
		MultiAssetDelegation::on_initialize(block_num);
		// CreditsPallet::on_initialize(block_num); // If it had hooks
	}
}
