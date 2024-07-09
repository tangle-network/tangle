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
use generic_ec::{coords::HasAffineX, curves::Stark, Point, Scalar};
use precompile_utils::prelude::*;
use sp_core::ConstU32;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// ECDSA pub key bytes
type ECDSAPubKeyBytes = ConstU32<33>;
// ECDSA signature bytes
type ECDSASignatureBytes = ConstU32<65>;

/// A precompile to verify EcdsaStark signature
pub struct EcdsaStarkPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> EcdsaStarkPrecompile<Runtime> {
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
			target: "Ecdsa-Stark-Precompile",
			"Verify signature {:?} for public {:?} and message {:?}",
			signature_bytes, public_bytes, message,
		);

		// Parse Signature
		let r_bytes = &signature_bytes[0..signature_bytes.len() / 2];
		let s_bytes = &signature_bytes[signature_bytes.len() / 2..];
		let r = if let Ok(x) = Scalar::from_be_bytes(r_bytes) {
			x
		} else {
			return Ok(false);
		};

		let s = if let Ok(x) = Scalar::from_be_bytes(s_bytes) {
			x
		} else {
			return Ok(false);
		};

		let public_key_point = if let Ok(x) = Point::from_bytes(public_bytes) {
			x
		} else {
			return Ok(false);
		};

		let public_key_x: Scalar<Stark> = if let Some(x) = public_key_point.x() {
			x.to_scalar()
		} else {
			return Ok(false);
		};

		let public_key = if let Ok(x) = convert_stark_scalar(&public_key_x) {
			x
		} else {
			return Ok(false);
		};
		let msg = if let Ok(x) =
			convert_stark_scalar(&Scalar::<Stark>::from_be_bytes_mod_order(message))
		{
			x
		} else {
			return Ok(false);
		};

		let r = if let Ok(x) = convert_stark_scalar(&r) {
			x
		} else {
			return Ok(false);
		};

		let s = if let Ok(x) = convert_stark_scalar(&s) {
			x
		} else {
			return Ok(false);
		};

		let is_confirmed = starknet_crypto::verify(&public_key, &msg, &r, &s).is_ok();

		log::trace!(
			target: "Ecdsa-Stark-Precompile",
			"Verified signature {:?} is {:?}",
			signature_bytes, is_confirmed,
		);

		Ok(is_confirmed)
	}
}

pub fn convert_stark_scalar(
	x: &Scalar<Stark>,
) -> Result<starknet_crypto::FieldElement, &'static str> {
	let bytes = x.to_be_bytes();
	debug_assert_eq!(bytes.len(), 32);
	let mut buffer = [0u8; 32];
	buffer.copy_from_slice(bytes.as_bytes());
	starknet_crypto::FieldElement::from_bytes_be(&buffer).map_err(|_| "FieldElementError")
}
