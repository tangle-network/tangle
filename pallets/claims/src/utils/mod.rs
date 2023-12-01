use pallet_evm::{AddressMapping, HashedAddressMapping};
use parity_scale_codec::{Decode, Encode};
use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_core::{sr25519::Signature, H160};
use sp_runtime::{traits::BlakeTwo256, AccountId32, RuntimeDebug};
use sp_std::prelude::*;

pub mod ethereum_address;

use ethereum_address::{EcdsaSignature, EthereumAddress};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Serialize, Deserialize)]
pub enum MultiAddress {
	/// Claimer is Ethereum address
	EVM(EthereumAddress),
	/// Claimer is Substrate address
	Native(AccountId32),
}

impl MultiAddress {
	pub fn to_account_id_32(&self) -> AccountId32 {
		match self {
			MultiAddress::EVM(ethereum_address) =>
				HashedAddressMapping::<BlakeTwo256>::into_account_id(H160::from(ethereum_address.0)),
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
