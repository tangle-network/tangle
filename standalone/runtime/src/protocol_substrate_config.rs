use crate::*;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::ConstU32, traits::Nothing};
use orml_currencies::{BasicCurrencyAdapter, NativeCurrencyOf};
use webb_primitives::{
	field_ops::ArkworksIntoFieldBn254,
	hashing::{ethereum::Keccak256HasherBn254, ArkworksPoseidonHasherBn254},
	verifying::ArkworksVerifierBn254,
	Amount, ChainId, ElementTrait,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

parameter_types! {
	pub const StringLimit: u32 = 50;
}

impl pallet_hasher::Config<pallet_hasher::Instance1> for Runtime {
	type Event = Event;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Hasher = ArkworksPoseidonHasherBn254;
	type WeightInfo = pallet_hasher::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const TreeDeposit: u64 = 1;
	pub const LeafDepositBase: u64 = 1;
	pub const LeafDepositPerByte: u64 = 1;
	pub const Two: u64 = 2;
	pub const MaxTreeDepth: u8 = 30;
	pub const RootHistorySize: u32 = 1096;
	// 21663839004416932945382355908790599225266501822907911457504978515578255421292
	// pub const DefaultZeroElement: Element = Element([
	// 	108, 175, 153, 072, 237, 133, 150, 036,
	// 	226, 065, 231, 118, 015, 052, 027, 130,
	// 	180, 093, 161, 235, 182, 053, 058, 052,
	// 	243, 171, 172, 211, 096, 076, 229, 047,
	// ]);
	pub const NewDefaultZeroElement: Element = Element([0u8; 32]);
}

#[derive(Debug, Encode, Decode, Default, Copy, Clone, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Element([u8; 32]);

impl ElementTrait for Element {
	fn to_bytes(&self) -> &[u8] {
		&self.0
	}

	fn from_bytes(input: &[u8]) -> Self {
		let mut buf = [0u8; 32];
		buf.copy_from_slice(input);
		Self(buf)
	}
}

impl pallet_mt::Config<pallet_mt::Instance1> for Runtime {
	type Currency = Balances;
	type DataDepositBase = LeafDepositBase;
	type DataDepositPerByte = LeafDepositPerByte;
	type DefaultZeroElement = NewDefaultZeroElement;
	type Element = Element;
	type Event = Event;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Hasher = HasherBn254;
	type LeafIndex = u32;
	type MaxTreeDepth = MaxTreeDepth;
	type RootHistorySize = RootHistorySize;
	type RootIndex = u32;
	type StringLimit = StringLimit;
	type TreeDeposit = TreeDeposit;
	type TreeId = u32;
	type Two = Two;
	type WeightInfo = pallet_mt::weights::WebbWeight<Runtime>;
}

impl pallet_verifier::Config<pallet_verifier::Instance1> for Runtime {
	type Event = Event;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Verifier = ArkworksVerifierBn254;
	type WeightInfo = pallet_verifier::weights::WebbWeight<Runtime>;
}

impl pallet_verifier::Config<pallet_verifier::Instance2> for Runtime {
	type Event = Event;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Verifier = ArkworksVerifierBn254;
	type WeightInfo = pallet_verifier::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const TokenWrapperPalletId: PalletId = PalletId(*b"dw/tkwrp");
	pub const WrappingFeeDivider: Balance = 100;
}

impl pallet_token_wrapper::Config for Runtime {
	type AssetRegistry = AssetRegistry;
	type Currency = Currencies;
	type Event = Event;
	type PalletId = TokenWrapperPalletId;
	type TreasuryId = DKGAccountId;
	type WeightInfo = pallet_token_wrapper::weights::WebbWeight<Runtime>;
	type WrappingFeeDivider = WrappingFeeDivider;
}

impl pallet_asset_registry::Config for Runtime {
	type AssetId = webb_primitives::AssetId;
	type AssetNativeLocation = u32;
	type Balance = Balance;
	type Event = Event;
	type NativeAssetId = GetNativeCurrencyId;
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
	type Event = Event;
	type ExistentialDeposits = AssetRegistry;
	type OnDust = ();
	type WeightInfo = ();
	type MaxLocks = ConstU32<2>;
	type MaxReserves = ConstU32<2>;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
	type ReserveIdentifier = ReserveIdentifier;
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
	type WeightInfo = ();
}

parameter_types! {
	pub const MixerPalletId: PalletId = PalletId(*b"py/mixer");
	pub const RegistryStringLimit: u32 = 10;
}

impl pallet_mixer::Config<pallet_mixer::Instance1> for Runtime {
	type Currency = Currencies;
	type Event = Event;
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
	pub const ChainType: [u8; 2] = [2, 0];
	pub const ChainIdentifier: ChainId = 1080;
}

impl pallet_linkable_tree::Config<pallet_linkable_tree::Instance1> for Runtime {
	type ChainId = ChainId;
	type ChainType = ChainType;
	type ChainIdentifier = ChainIdentifier;
	type Event = Event;
	type HistoryLength = HistoryLength;
	type Tree = MerkleTreeBn254;
	type WeightInfo = ();
}

type SignatureBridgeInstance = pallet_signature_bridge::Instance1;
impl pallet_signature_bridge::Config<SignatureBridgeInstance> for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BridgeAccountId = BridgeAccountId;
	type ChainId = ChainId;
	type ChainIdentifier = ChainIdentifier;
	type ChainType = ChainType;
	type Event = Event;
	type Proposal = Call;
	type ProposalLifetime = ProposalLifetime;
	type ProposalNonce = u32;
	type MaintainerNonce = u32;
	type SignatureVerifier = webb_primitives::signing::SignatureVerifier;
	type WeightInfo = ();
}

impl pallet_verifier::Config<pallet_verifier::Instance3> for Runtime {
	type Event = Event;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Verifier = ArkworksVerifierBn254;
	type WeightInfo = pallet_verifier::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const VAnchorPalletId: PalletId = PalletId(*b"py/vanch");
	pub const MaxFee: Balance = Balance::MAX - 1;
	pub const MaxExtAmount: Balance = Balance::MAX - 1;
}

impl pallet_vanchor::Config<pallet_vanchor::Instance1> for Runtime {
	type Event = Event;
	type PalletId = VAnchorPalletId;
	type LinkableTree = LinkableTreeBn254;
	type Verifier2x2 = VAnchorVerifier2x2Bn254;
	type EthereumHasher = Keccak256HasherBn254;
	type IntoField = ArkworksIntoFieldBn254;
	type Currency = Currencies;
	type MaxFee = MaxFee;
	type MaxExtAmount = MaxExtAmount;
	type PostDepositHook = ();
	type NativeCurrencyId = GetNativeCurrencyId;
}

parameter_types! {
	pub const ProposalLifetime: BlockNumber = 50;
	pub const BridgeAccountId: PalletId = PalletId(*b"dw/bridg");
}

type BridgeInstance = pallet_bridge::Instance1;
impl pallet_bridge::Config<BridgeInstance> for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BridgeAccountId = BridgeAccountId;
	type ChainId = ChainId;
	type ChainIdentifier = ChainIdentifier;
	type ChainType = ChainType;
	type Event = Event;
	type Proposal = Call;
	type ProposalLifetime = ProposalLifetime;
}

impl pallet_vanchor_handler::Config<pallet_vanchor_handler::Instance1> for Runtime {
	type VAnchor = VAnchorBn254;
	type BridgeOrigin = pallet_bridge::EnsureBridge<Runtime, BridgeInstance>;
	type Event = Event;
}
