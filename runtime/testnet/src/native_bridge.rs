use bp_header_chain::ChainWithGrandpa;
use frame_support::parameter_types;
use bp_runtime::Chain;
use sp_core::ConstU32;
use sp_runtime::StateVersion;

/// Block hash of the bridged relay chain.
pub type RelayBlockHash = bp_polkadot_core::Hash;
/// Block number of the bridged relay chain.
pub type RelayBlockNumber = bp_polkadot_core::BlockNumber;
/// Hasher of the bridged relay chain.
pub type RelayBlockHasher = bp_polkadot_core::Hasher;

parameter_types! {
	pub const HeadersToKeep: u32 = 5;
	pub const FreeHeadersInterval: u32 = 15;
}

#[derive(Debug)]
pub struct TangleBridgedChain;

impl Chain for TangleBridgedChain {
	const ID: ChainId = *b"tngl";

	type BlockNumber = RelayBlockNumber;
	type Hash = RelayBlockHash;
	type Hasher = RelayBlockHasher;
	type Header = RelayBlockHeader;

	type AccountId = AccountId;
	type Balance = u32;
	type Nonce = u32;
	type Signature = sp_runtime::testing::TestSignature;

	fn max_extrinsic_size() -> u32 {
		unreachable!()
	}

	fn max_extrinsic_weight() -> Weight {
		unreachable!()
	}
}

impl ChainWithGrandpa for TangleBridgedChain {
	const WITH_CHAIN_GRANDPA_PALLET_NAME: &'static str = "";
	const MAX_AUTHORITIES_COUNT: u32 = 16;
	const REASONABLE_HEADERS_IN_JUSTIFICATON_ANCESTRY: u32 = 8;
	const MAX_MANDATORY_HEADER_SIZE: u32 = 256;
	const AVERAGE_HEADER_SIZE: u32 = 64;
}

impl pallet_bridge_grandpa::Config<pallet_bridge_grandpa::Instance1> for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = TangleBridgedChain;
	type MaxFreeMandatoryHeadersPerBlock = ConstU32<2>;
	type HeadersToKeep = HeadersToKeep;
	type WeightInfo = ();
}

pub struct TangleRestakingParachain;

impl Chain for TangleRestakingParachain {
	const ID: ChainId = *b"rstk";

	type BlockNumber = u64;
	type Hash = H256;
	type Hasher = RegularParachainHasher;
	type Header = RegularParachainHeader;
	type AccountId = u64;
	type Balance = u64;
	type Nonce = u64;
	type Signature = MultiSignature;

	fn max_extrinsic_size() -> u32 {
		0
	}
	fn max_extrinsic_weight() -> Weight {
		Weight::zero()
	}
}

impl Parachain for TangleRestakingParachain {
	const PARACHAIN_ID: u32 = 1;
	const MAX_HEADER_SIZE: u32 = 1_024;
}

parameter_types! {
	pub const HeadsToKeep: u32 = 4;
	pub const ParasPalletName: &'static str = PARAS_PALLET_NAME;
	pub GetTenFirstParachains: Vec<ParaId> = (0..10).map(ParaId).collect();
}

impl pallet_bridge_parachains::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BridgesGrandpaPalletInstance = pallet_bridge_grandpa::Instance1;
	type ParasPalletName = ParasPalletName;
	type ParaStoredHeaderDataBuilder = (TangleRestakingParachain);
	type HeadsToKeep = HeadsToKeep;
	type MaxParaHeadDataSize = ConstU32<MAXIMAL_PARACHAIN_HEAD_DATA_SIZE>;
}