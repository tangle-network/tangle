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
use sp_core::{bytes::to_hex, ConstU32};
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// A precompile to verify Bls-381 signatures
pub struct Bls381Precompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> Bls381Precompile<Runtime> {
	#[precompile::public("verify(bytes,bytes,bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		public_bytes: BoundedBytes<ConstU32<96>>,
		signature_bytes: BoundedBytes<ConstU32<384>>,
		message: UnboundedBytes,
	) -> EvmResult<bool> {
		// Parse arguments
		let public_bytes: Vec<u8> = public_bytes.into();
		let signature_bytes: Vec<u8> = signature_bytes.into();
		let message: Vec<u8> = message.into();

		log::trace!(
			target: "Bls-381-Precompile",
			"Verify signature {:?} for public {:?} and message {:?}",
			signature_bytes, public_bytes, message,
		);

		let public_key = if let Ok(p_key) =
			snowbridge_milagro_bls::PublicKey::from_uncompressed_bytes(&public_bytes)
		{
			p_key
		} else {
			return Ok(false);
		};

		let signature =
			if let Ok(sig) = snowbridge_milagro_bls::Signature::from_bytes(&signature_bytes) {
				sig
			} else {
				return Ok(false);
			};

		let is_confirmed = signature.verify(&message, &public_key);

		log::trace!(
			target: "Bls-381-Precompile",
			"Verified signature {} is {:?}",
			to_hex(&signature.as_bytes()[..], false), is_confirmed,
		);

		Ok(is_confirmed)
	}
}
