use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

#[derive(
	Default,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	Ord,
	PartialOrd,
	TypeInfo,
	Encode,
	Decode,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
#[serde(tag = "type", content = "id")]
#[non_exhaustive]
pub enum TypedChainId {
	/// None chain type.
	///
	/// This is used when the chain type is not known.
	#[default]
	None,
	/// EVM Based Chain (Mainnet, Polygon, ...etc)
	Evm(u32),
	/// Standalone Substrate Based Chain (Webb, Edgeware, ...etc)
	Substrate(u32),
	/// Polkadot Parachains.
	PolkadotParachain(u32),
	/// Kusama Parachains.
	KusamaParachain(u32),
	/// Rococo Parachains.
	RococoParachain(u32),
	/// Cosmos / CosmWasm Chains.
	Cosmos(u32),
	/// Solana Program.
	Solana(u32),
	/// Ink Based Chains
	Ink(u32),
}

impl TypedChainId {
	/// Length of the [`TypedChainId`] in bytes.
	pub const LENGTH: usize = 6;

	/// Get the chain id as a `u64`. This represents
	/// the typed chain ID that should be used to differentiate
	/// between differently typed chains with the same underlying
	/// chain id.
	#[must_use]
	pub fn chain_id(&self) -> u64 {
		let mut buf: [u8; 8] = [0u8; 8];
		buf[2..8].copy_from_slice(&self.to_bytes());
		u64::from_be_bytes(buf)
	}

	/// Get the chain id as a `u32`. This represents
	/// the un-typed underlying chain ID for the chain.
	#[must_use]
	pub const fn underlying_chain_id(&self) -> u32 {
		match self {
			TypedChainId::Evm(id) |
			TypedChainId::Substrate(id) |
			TypedChainId::PolkadotParachain(id) |
			TypedChainId::KusamaParachain(id) |
			TypedChainId::RococoParachain(id) |
			TypedChainId::Cosmos(id) |
			TypedChainId::Solana(id) |
			TypedChainId::Ink(id) => *id,
			Self::None => 0,
		}
	}

	/// Get the underlying bytes of `ChainType`.
	#[must_use]
	pub fn to_bytes(&self) -> [u8; Self::LENGTH] {
		let mut bytes = [0u8; Self::LENGTH];
		match self {
			TypedChainId::Evm(id) => {
				bytes[0..2].copy_from_slice(&(0x0100u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::Substrate(id) => {
				bytes[0..2].copy_from_slice(&(0x0200u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::PolkadotParachain(id) => {
				bytes[0..2].copy_from_slice(&(0x0301u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::KusamaParachain(id) => {
				bytes[0..2].copy_from_slice(&(0x0302u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::RococoParachain(id) => {
				bytes[0..2].copy_from_slice(&(0x0303u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::Cosmos(id) => {
				bytes[0..2].copy_from_slice(&(0x0400u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::Solana(id) => {
				bytes[0..2].copy_from_slice(&(0x0500u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::Ink(id) => {
				bytes[0..2].copy_from_slice(&(0x0600u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&id.to_be_bytes());
			},
			TypedChainId::None => {
				bytes[0..2].copy_from_slice(&(0x0000u16).to_be_bytes());
				bytes[2..6].copy_from_slice(&0u32.to_be_bytes());
			},
		}
		bytes
	}
	/// Get the underlying bytes of `ChainType`.
	#[must_use]
	pub fn into_bytes(self) -> [u8; Self::LENGTH] {
		self.to_bytes()
	}
}

impl From<TypedChainId> for [u8; TypedChainId::LENGTH] {
	fn from(v: TypedChainId) -> Self {
		v.into_bytes()
	}
}

impl From<[u8; Self::LENGTH]> for TypedChainId {
	fn from(bytes: [u8; Self::LENGTH]) -> Self {
		let ty = [bytes[0], bytes[1]];
		let ty = u16::from_be_bytes(ty);
		let id = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
		match ty {
			0x0100 => TypedChainId::Evm(id),
			0x0200 => TypedChainId::Substrate(id),
			0x0301 => TypedChainId::PolkadotParachain(id),
			0x0302 => TypedChainId::KusamaParachain(id),
			0x0303 => TypedChainId::RococoParachain(id),
			0x0400 => TypedChainId::Cosmos(id),
			0x0500 => TypedChainId::Solana(id),
			0x0600 => TypedChainId::Ink(id),
			_ => Self::None,
		}
	}
}

impl From<[u8; Self::LENGTH + 2]> for TypedChainId {
	fn from(bytes: [u8; Self::LENGTH + 2]) -> Self {
		let ty = [bytes[2], bytes[3]];
		let ty = u16::from_be_bytes(ty);
		let id = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
		match ty {
			0x0100 => TypedChainId::Evm(id),
			0x0200 => TypedChainId::Substrate(id),
			0x0301 => TypedChainId::PolkadotParachain(id),
			0x0302 => TypedChainId::KusamaParachain(id),
			0x0303 => TypedChainId::RococoParachain(id),
			0x0400 => TypedChainId::Cosmos(id),
			0x0500 => TypedChainId::Solana(id),
			0x0600 => TypedChainId::Ink(id),
			_ => Self::None,
		}
	}
}

impl From<u64> for TypedChainId {
	fn from(val: u64) -> Self {
		TypedChainId::from(val.to_be_bytes())
	}
}
