// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use p256::{ecdsa::signature::hazmat::PrehashVerifier, elliptic_curve::group::GroupEncoding};
use precompile_utils::prelude::*;
use sp_core::ConstU32;
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// ECDSA pub key bytes
type ECDSAPubKeyBytes = ConstU32<33>;
// ECDSA signature bytes
type ECDSASignatureBytes = ConstU32<65>;

/// A precompile to verify EcdsaSecp256r1 signature
pub struct EcdsaSecp256r1Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> EcdsaSecp256r1Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ECDSAPubKeyBytes>,
		signature_bytes: BoundedBytes<ECDSASignatureBytes>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		log::trace!(
			target: "Ecdsa-Secp256r1-Precompile",
			"Verify signature {:?} for public {:?} and message {:?}",
			signature_bytes, public_bytes, message,
		);

		let maybe_pub_key_point = p256::AffinePoint::from_bytes(public_bytes.as_slice().into());

		let pub_key_point = if let Some(x) = maybe_pub_key_point.into() {
			x
		} else {
			return Ok(false);
		};

		let maybe_verifying_key = p256::ecdsa::VerifyingKey::from_affine(pub_key_point);
		let verifying_key = if let Ok(x) = maybe_verifying_key {
			x
		} else {
			return Ok(false);
		};

		let maybe_signature = p256::ecdsa::Signature::from_slice(signature_bytes.as_slice());
		let signature = if let Ok(x) = maybe_signature {
			x
		} else {
			return Ok(false);
		};

		let is_confirmed =
			verifying_key.verify_prehash(&message, &signature).map(|_| signature).is_ok();

		log::trace!(
			target: "Ecdsa-Secp256r1-Precompile",
			"Verified signature {:?} is {:?}",
			signature, is_confirmed,
		);

		Ok(is_confirmed)
	}
}
