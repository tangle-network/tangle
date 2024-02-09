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

use frame_support::{ensure, pallet_prelude::DispatchResult};
use parity_scale_codec::Encode;
use sp_core::sr25519;
use sp_io::{crypto::sr25519_verify, hashing::keccak_256};
use sp_std::vec::Vec;
use tangle_primitives::jobs::DKGTSSKeySubmissionResult;

use crate::{Config, Error};

use super::to_slice_32;

/// Verifies the DKG signature result for Schnorr signatures over sr25519.
///
/// This function uses the Schnorr signature algorithm to verify the provided signature
/// based on the message data, signature, and signing key in the DKG signature result.
///
/// # Arguments
///
/// * `msg` - The message data that was signed.
/// * `signature` - The Schnorr signature to be verified.
/// * `key` - The public key associated with the signature.
pub fn verify_schnorr_sr25519_signature<T: Config>(
	msg: &[u8],
	signature: &[u8],
	key: &[u8],
) -> DispatchResult {
	// Convert the signature from bytes to sr25519::Signature
	let signature: sr25519::Signature =
		signature.try_into().map_err(|_| Error::<T>::CannotRetreiveSigner)?;

	// Encode the message data and compute its keccak256 hash
	let msg = msg.encode();
	let hash = keccak_256(&msg);

	// Verify the Schnorr signature using sr25519_verify
	if !sr25519_verify(
		&signature,
		&hash,
		&sr25519::Public(
			to_slice_32(key)
				.unwrap_or_else(|| panic!("Failed to convert input to sr25519 public key")),
		),
	) {
		return Err(Error::<T>::InvalidSignature.into())
	}

	Ok(())
}
