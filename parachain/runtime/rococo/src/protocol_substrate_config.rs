use crate::*;
use frame_support::{
	pallet_prelude::ConstU32,
	traits::{Contains, Nothing},
};
use orml_currencies::{BasicCurrencyAdapter, NativeCurrencyOf};
use webb_primitives::{
	field_ops::ArkworksIntoFieldBn254,
	hashing::{ethereum::Keccak256HasherBn254, ArkworksPoseidonHasherBn254},
	runtime::Element,
	verifying::ArkworksVerifierBn254,
	Amount, ChainId,
};

parameter_types! {
	pub const StringLimit: u32 = 50;
}

impl pallet_hasher::Config<pallet_hasher::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Hasher = ArkworksPoseidonHasherBn254;
	type MaxParameterLength = ConstU32<10000>;
	type WeightInfo = pallet_hasher::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const TreeDeposit: u64 = 1;
	pub const LeafDepositBase: u64 = 1;
	pub const LeafDepositPerByte: u64 = 1;
	pub const Two: u64 = 2;
	pub const MaxTreeDepth: u8 = 30;
	pub const RootHistorySize: u32 = 1096;
	// // 2166383900441693294538235590879059922526650182290791145750497851557825542129
	// pub const DefaultZeroElement: Element = Element([
	// 	108, 175, 153, 072, 237, 133, 150, 036,
	// 	226, 065, 231, 118, 015, 052, 027, 130,
	// 	180, 093, 161, 235, 182, 053, 058, 052,
	// 	243, 171, 172, 211, 096, 076, 229, 047,
	// ]);
	#[derive(Debug, scale_info::TypeInfo)]
	pub const MaxEdges: u32 = 1000;
	#[derive(Debug, scale_info::TypeInfo)]
	pub const MaxDefaultHashes: u32 = 1000;
	pub const NewDefaultZeroElement: Element = Element([0u8; 32]);
}

impl pallet_mt::Config<pallet_mt::Instance1> for Runtime {
	type Currency = Balances;
	type DataDepositBase = LeafDepositBase;
	type DataDepositPerByte = LeafDepositPerByte;
	type DefaultZeroElement = NewDefaultZeroElement;
	type Element = Element;
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Hasher = HasherBn254;
	type LeafIndex = u32;
	type MaxTreeDepth = MaxTreeDepth;
	type RootHistorySize = RootHistorySize;
	type RootIndex = u32;
	type StringLimit = StringLimit;
	type TreeDeposit = TreeDeposit;
	type MaxEdges = MaxEdges;
	type MaxDefaultHashes = MaxDefaultHashes;
	type TreeId = u32;
	type Two = Two;
	type WeightInfo = pallet_mt::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const MaxParameterLength : u32 = 1000;
}

impl pallet_verifier::Config<pallet_verifier::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Verifier = ArkworksVerifierBn254;
	type MaxParameterLength = MaxParameterLength;
	type WeightInfo = pallet_verifier::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const TokenWrapperPalletId: PalletId = PalletId(*b"dw/tkwrp");
	pub const WrappingFeeDivider: Balance = 100;
}

impl pallet_token_wrapper::Config for Runtime {
	type AssetRegistry = AssetRegistry;
	type Currency = Currencies;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TokenWrapperPalletId;
	type TreasuryId = DKGAccountId;
	type ProposalNonce = u32;
	type WeightInfo = pallet_token_wrapper::weights::WebbWeight<Runtime>;
	type WrappingFeeDivider = WrappingFeeDivider;
}

parameter_types! {
	#[derive(Copy, Clone, Debug, PartialEq, Eq, scale_info::TypeInfo)]
	pub const MaxAssetIdInPool: u32 = 100;
}

impl pallet_asset_registry::Config for Runtime {
	type AssetId = webb_primitives::AssetId;
	type AssetNativeLocation = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type NativeAssetId = GetNativeCurrencyId;
	type MaxAssetIdInPool = MaxAssetIdInPool;
	type RegistryOrigin = frame_system::EnsureRoot<AccountId>;
	type StringLimit = RegistryStringLimit;
	type WeightInfo = ();
}

pub type ReserveIdentifier = [u8; 8];
impl orml_tokens::Config for Runtime {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = webb_primitives::AssetId;
	type DustRemovalWhitelist = Nothing;
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposits = AssetRegistry;
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type MaxLocks = ConstU32<2>;
	type MaxReserves = ConstU32<2>;
	type ReserveIdentifier = ReserveIdentifier;
	type CurrencyHooks = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: webb_primitives::AssetId = 0;
}

pub type NativeCurrency = NativeCurrencyOf<Runtime>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, Balance>;
impl orml_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = weights::orml_currencies::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MixerPalletId: PalletId = PalletId(*b"py/mixer");
	pub const RegistryStringLimit: u32 = 10;
}

impl pallet_mixer::Config<pallet_mixer::Instance1> for Runtime {
	type Currency = Currencies;
	type RuntimeEvent = RuntimeEvent;
	type NativeCurrencyId = GetNativeCurrencyId;
	type PalletId = MixerPalletId;
	type Tree = MerkleTreeBn254;
	type Verifier = MixerVerifierBn254;
	type ArbitraryHasher = Keccak256HasherBn254;
	type WeightInfo = pallet_mixer::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const AnchorPalletId: PalletId = PalletId(*b"py/anchr");
	pub const HistoryLength: u32 = 30;
	// Substrate parachain chain ID type
	pub const ChainType: [u8; 2] = [2, 1];
	pub const ChainIdentifier: ChainId = 1081;
}

impl pallet_linkable_tree::Config<pallet_linkable_tree::Instance1> for Runtime {
	type ChainId = ChainId;
	type ChainType = ChainType;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type HistoryLength = HistoryLength;
	type Tree = MerkleTreeBn254;
	type WeightInfo = ();
}

parameter_types! {
	pub const BridgeProposalLifetime: BlockNumber = 50;
	pub const BridgeAccountId: PalletId = PalletId(*b"dw/bridg");
	pub const MaxStringLength: u32 = 1000;
}

pub struct SetResourceProposalFilter;
#[allow(clippy::collapsible_match, clippy::match_single_binding, clippy::match_like_matches_macro)]
impl Contains<RuntimeCall> for SetResourceProposalFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::VAnchorHandlerBn254(method) => match method {
				pallet_vanchor_handler::Call::execute_set_resource_proposal { .. } => true,
				_ => false,
			},
			RuntimeCall::TokenWrapperHandler(method) => match method {
				_ => false,
			},
			_ => false,
		}
	}
}

pub struct ExecuteProposalFilter;
#[allow(clippy::collapsible_match, clippy::match_single_binding, clippy::match_like_matches_macro)]
impl Contains<RuntimeCall> for ExecuteProposalFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::VAnchorHandlerBn254(method) => match method {
				pallet_vanchor_handler::Call::execute_vanchor_create_proposal { .. } => true,
				pallet_vanchor_handler::Call::execute_vanchor_update_proposal { .. } => true,
				_ => false,
			},
			RuntimeCall::TokenWrapperHandler(method) => match method {
				pallet_token_wrapper_handler::Call::execute_add_token_to_pool_share { .. } => true,
				pallet_token_wrapper_handler::Call::execute_remove_token_from_pool_share {
					..
				} => true,
				pallet_token_wrapper_handler::Call::execute_wrapping_fee_proposal { .. } => true,
				_ => false,
			},
			_ => false,
		}
	}
}

type SignatureBridgeInstance = pallet_signature_bridge::Instance1;
impl pallet_signature_bridge::Config<SignatureBridgeInstance> for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BridgeAccountId = BridgeAccountId;
	type ChainId = ChainId;
	type ChainIdentifier = ChainIdentifier;
	type ChainType = ChainType;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	type ProposalLifetime = ProposalLifetime;
	type ProposalNonce = u32;
	type MaintainerNonce = u32;
	type MaxStringLength = MaxStringLength;
	type SetResourceProposalFilter = SetResourceProposalFilter;
	type ExecuteProposalFilter = ExecuteProposalFilter;
	type SignatureVerifier = webb_primitives::signing::SignatureVerifier;
	type WeightInfo = ();
}

parameter_types! {
	pub const VAnchorPalletId: PalletId = PalletId(*b"py/vanch");
	pub const MaxFee: Balance = Balance::MAX - 1;
	pub const MaxExtAmount: Balance = Balance::MAX - 1;
	pub const MaxCurrencyId: webb_primitives::AssetId = webb_primitives::AssetId::MAX - 1;
}

impl pallet_vanchor::Config<pallet_vanchor::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = VAnchorPalletId;
	type ProposalNonce = u32;
	type LinkableTree = LinkableTreeBn254;
	type KeyStorage = KeyStorage;
	type VAnchorVerifier = VAnchorVerifier;
	type EthereumHasher = Keccak256HasherBn254;
	type IntoField = ArkworksIntoFieldBn254;
	type Currency = Currencies;
	type MaxFee = MaxFee;
	type MaxExtAmount = MaxExtAmount;
	type PostDepositHook = ();
	type NativeCurrencyId = GetNativeCurrencyId;
	type MaxCurrencyId = MaxCurrencyId;
	type TokenWrapper = TokenWrapper;
	type WeightInfo = ();
}

parameter_types! {
	pub const ProposalLifetime: BlockNumber = 50;
}

impl pallet_vanchor_handler::Config<pallet_vanchor_handler::Instance1> for Runtime {
	type VAnchor = VAnchorBn254;
	type BridgeOrigin = pallet_signature_bridge::EnsureBridge<Runtime, SignatureBridgeInstance>;
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_token_wrapper_handler::Config for Runtime {
	type BridgeOrigin = pallet_signature_bridge::EnsureBridge<Runtime, SignatureBridgeInstance>;
	type RuntimeEvent = RuntimeEvent;
	type TokenWrapper = TokenWrapper;
}

impl pallet_key_storage::Config<pallet_key_storage::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxPubkeyLength = ConstU32<1000>;
	type MaxPubKeyOwners = ConstU32<100>;
	type WeightInfo = pallet_key_storage::weights::WebbWeight<Runtime>;
}

impl pallet_vanchor_verifier::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Verifier = ArkworksVerifierBn254;
	type MaxParameterLength = MaxParameterLength;
	type WeightInfo = pallet_vanchor_verifier::weights::WebbWeight<Runtime>;
}
