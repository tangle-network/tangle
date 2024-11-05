//! Types used to connect to the Tangle chain.

pub mod codegen_runtime;

use bp_polkadot_core::SuffixedCommonSignedExtensionExt;
use bp_westend::WESTEND_SYNCED_HEADERS_GRANDPA_INFO_METHOD;
use codec::Encode;
use relay_substrate_client::{
	Chain, ChainWithBalances, ChainWithGrandpa, ChainWithRuntimeVersion, ChainWithTransactions,
	Error as SubstrateError, RelayChain, SignParam, SimpleRuntimeVersion, UnderlyingChainProvider,
	UnsignedTransaction,
};
use sp_core::{storage::StorageKey, Pair};
use sp_runtime::{generic::SignedPayload, traits::IdentifyAccount, MultiAddress};
use sp_session::MembershipProof;
use std::time::Duration;

pub use codegen_runtime::api::runtime_types;

pub type RuntimeCall = runtime_types::tangle_runtime::RuntimeCall;

pub type GrandpaCall = runtime_types::pallet_grandpa::pallet::Call;

/// Westend header id.
pub type HeaderId = relay_utils::HeaderId<bp_westend::Hash, bp_westend::BlockNumber>;

/// Westend header type used in headers sync.
pub type SyncHeader = relay_substrate_client::SyncHeader<bp_westend::Header>;

/// The address format for describing accounts.
pub type Address = MultiAddress<bp_westend::AccountId, ()>;

/// Westend chain definition
#[derive(Debug, Clone, Copy)]
pub struct Tangle;

impl UnderlyingChainProvider for Tangle {
	type Chain = bp_westend::Westend;
}

impl Chain for Tangle {
	const NAME: &'static str = "Tangle";
	const BEST_FINALIZED_HEADER_ID_METHOD: &'static str =
		bp_westend::BEST_FINALIZED_WESTEND_HEADER_METHOD;
	const FREE_HEADERS_INTERVAL_METHOD: &'static str =
		bp_westend::FREE_HEADERS_INTERVAL_FOR_WESTEND_METHOD;
	const AVERAGE_BLOCK_INTERVAL: Duration = Duration::from_secs(6);

	type SignedBlock = bp_westend::SignedBlock;
	type Call = RuntimeCall;
}

impl ChainWithGrandpa for Tangle {
	const SYNCED_HEADERS_GRANDPA_INFO_METHOD: &'static str =
		WESTEND_SYNCED_HEADERS_GRANDPA_INFO_METHOD;

	type KeyOwnerProof = MembershipProof;
}

impl RelayChain for Tangle {
	const PARAS_PALLET_NAME: &'static str = bp_westend::PARAS_PALLET_NAME;
	const WITH_CHAIN_BRIDGE_PARACHAINS_PALLET_NAME: &'static str =
		bp_westend::WITH_WESTEND_BRIDGE_PARACHAINS_PALLET_NAME;
}

impl ChainWithBalances for Tangle {
	fn account_info_storage_key(account_id: &Self::AccountId) -> StorageKey {
		bp_westend::AccountInfoStorageMapKeyProvider::final_key(account_id)
	}
}

impl ChainWithTransactions for Tangle {
	type AccountKeyPair = sp_core::sr25519::Pair;
	type SignedTransaction =
		bp_polkadot_core::UncheckedExtrinsic<Self::Call, bp_westend::SignedExtension>;

	fn sign_transaction(
		param: SignParam<Self>,
		unsigned: UnsignedTransaction<Self>,
	) -> Result<Self::SignedTransaction, SubstrateError> {
		let raw_payload = SignedPayload::new(
			unsigned.call,
			bp_westend::SignedExtension::from_params(
				param.spec_version,
				param.transaction_version,
				unsigned.era,
				param.genesis_hash,
				unsigned.nonce,
				unsigned.tip,
				((), ()),
			),
		)?;

		let signature = raw_payload.using_encoded(|payload| param.signer.sign(payload));
		let signer: sp_runtime::MultiSigner = param.signer.public().into();
		let (call, extra, _) = raw_payload.deconstruct();

		Ok(Self::SignedTransaction::new_signed(
			call,
			signer.into_account().into(),
			signature.into(),
			extra,
		))
	}
}

impl ChainWithRuntimeVersion for Tangle {
	const RUNTIME_VERSION: Option<SimpleRuntimeVersion> =
		Some(SimpleRuntimeVersion { spec_version: 1_016_001, transaction_version: 26 });
}
