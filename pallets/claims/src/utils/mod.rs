use pallet_evm::{AddressMapping, HashedAddressMapping};
use parity_scale_codec::{Decode, Encode};
use scale_info::{
	prelude::{format, string::String},
	TypeInfo,
};
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_core::{H160};
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

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MultiAddressSignature {
	EVM(EcdsaSignature),
	Native(Sr25519Signature),
}

#[derive(Clone, Copy, Eq, Encode, Decode, TypeInfo)]
pub struct Sr25519Signature(pub [u8; 65]);

impl PartialEq for Sr25519Signature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

impl sp_std::fmt::Debug for Sr25519Signature {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "Sr25519Signature({:?})", &self.0[..])
	}
}
