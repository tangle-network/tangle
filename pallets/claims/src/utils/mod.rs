use pallet_evm::{AddressMapping, HashedAddressMapping};
use parity_scale_codec::{Decode, Encode};
use scale_info::{
	TypeInfo,
	prelude::{format, string::String},
};
use serde::{self, Deserialize, Serialize};
use sp_core::{H160, sr25519::Signature};
use sp_runtime::{AccountId32, RuntimeDebug, traits::BlakeTwo256};
use sp_std::{hash::Hash, prelude::*};

pub mod ethereum_address;

pub use ethereum_address::{EcdsaSignature, EthereumAddress};

#[derive(
	Encode,
	Decode,
	Clone,
	Eq,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
	Serialize,
	Deserialize,
	Ord,
	PartialOrd,
)]
pub enum MultiAddress {
	/// Claimer is Ethereum address
	EVM(EthereumAddress),
	/// Claimer is Substrate address
	Native(AccountId32),
}

impl Hash for MultiAddress {
	fn hash<H: sp_std::hash::Hasher>(&self, state: &mut H) {
		match self {
			MultiAddress::EVM(ethereum_address) => ethereum_address.0.hash(state),
			MultiAddress::Native(substrate_address) => substrate_address.encode().hash(state),
		}
	}
}

impl MultiAddress {
	pub fn to_account_id_32(&self) -> AccountId32 {
		match self {
			MultiAddress::EVM(ethereum_address) => {
				HashedAddressMapping::<BlakeTwo256>::into_account_id(H160::from(ethereum_address.0))
			},
			MultiAddress::Native(substrate_address) => substrate_address.clone(),
		}
	}

	pub fn to_ethereum_address(&self) -> Option<EthereumAddress> {
		match self {
			MultiAddress::EVM(ethereum_address) => Some(ethereum_address.clone()),
			MultiAddress::Native(_) => None,
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MultiAddressSignature {
	EVM(EcdsaSignature),
	Native(Sr25519Signature),
}

#[derive(Clone, Eq, Encode, PartialEq, Decode, TypeInfo, RuntimeDebug)]
pub struct Sr25519Signature(pub Signature);
