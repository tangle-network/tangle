// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use precompile_utils::prelude::*;
use sp_core::{sr25519, ConstU32};
use sp_io::{crypto::sr25519_verify, hashing::keccak_256};
use sp_std::{marker::PhantomData, prelude::*};

use frost_core::{signature::Signature, verifying_key::VerifyingKey};
use frost_ed25519::Ed25519Sha512;
use frost_ed448::Ed448Shake256;
use frost_p256::P256Sha256;
use frost_p384::P384Sha384;
use frost_ristretto255::Ristretto255Sha512;
use frost_secp256k1::Secp256K1Sha256;
use frost_taproot::Secp256K1Taproot;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Macro to verify a Schnorr signature using the specified signature scheme.
macro_rules! verify_signature {
	($impl_type:ty, $key:expr, $signature:expr, $msg:expr, $key_default:expr, $sig_default:expr) => {{
		let verifying_key: VerifyingKey<$impl_type> =
			VerifyingKey::deserialize($key.try_into().unwrap_or($key_default))
				.map_err(|_| revert("InvalidVerifyingKeyDeserialization"))?;
		let sig: Signature<$impl_type> =
			Signature::deserialize($signature.try_into().unwrap_or($sig_default))
				.map_err(|_| revert("InvalidSignatureDeserialization"))?;
		verifying_key.verify($msg, &sig).map_err(|_| revert("InvalidSignature"))?
	}};
}

/// Utility function to create slice of fixed size
pub fn to_slice_32(val: &[u8]) -> Option<[u8; 32]> {
	if val.len() == 32 {
		let mut key = [0u8; 32];
		key[..32].copy_from_slice(val);

		return Some(key)
	}
	None
}

/// A precompile to verify SchnorrSr25519 signature
pub struct SchnorrSr25519Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrSr25519Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<32>>,
		signature_bytes: BoundedBytes<ConstU32<65>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		// Convert the signature from bytes to sr25519::Signature
		let signature: sr25519::Signature =
			signature_bytes.as_slice().try_into().map_err(|_| revert("Invalid Signature"))?;

		// Convert public key from bytes to sr25519::Public
		let public_key: sr25519::Public = sr25519::Public::from_raw(
			public_bytes.try_into().map_err(|_| revert("Invalid Publci Key"))?,
		);

		// Compute its keccak256 hash
		let hash = keccak_256(&message);

		// Verify the Schnorr signature using sr25519_verify
		let is_confirmed = sr25519_verify(&signature, &hash, &public_key);
		Ok(is_confirmed)
	}
}

/// A precompile to verify SchnorrSecp256k1 signature
pub struct SchnorrSecp256k1Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrSecp256k1Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<33>>,
		signature_bytes: BoundedBytes<ConstU32<65>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			Secp256K1Sha256,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 33],
			[0u8; 65]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrEd25519 signature
pub struct SchnorrEd25519Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrEd25519Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<32>>,
		signature_bytes: BoundedBytes<ConstU32<64>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			Ed25519Sha512,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 32],
			[0u8; 64]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrEd448 signature
pub struct SchnorrEd448Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrEd448Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<57>>,
		signature_bytes: BoundedBytes<ConstU32<114>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			Ed448Shake256,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 57],
			[0u8; 114]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrP256 signature
pub struct SchnorrP256Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrP256Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<33>>,
		signature_bytes: BoundedBytes<ConstU32<65>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			P256Sha256,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 33],
			[0u8; 65]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrP384 signature
pub struct SchnorrP384Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrP384Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<49>>,
		signature_bytes: BoundedBytes<ConstU32<97>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			P384Sha384,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 49],
			[0u8; 97]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrRistretto255 signature
pub struct SchnorrRistretto255Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrRistretto255Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<32>>,
		signature_bytes: BoundedBytes<ConstU32<64>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			Ristretto255Sha512,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 32],
			[0u8; 64]
		);

		Ok(false)
	}
}

/// A precompile to verify SchnorrTaproot signature
pub struct SchnorrTaprootPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> SchnorrTaprootPrecompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<33>>,
		signature_bytes: BoundedBytes<ConstU32<65>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		verify_signature!(
			Secp256K1Taproot,
			public_bytes.as_slice(),
			signature_bytes.as_slice(),
			&message,
			[0u8; 33],
			[0u8; 65]
		);

		Ok(false)
	}
}
