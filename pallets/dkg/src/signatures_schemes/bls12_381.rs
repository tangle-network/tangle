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
use sp_io::hashing::keccak_256;

use crate::{Config, Error};
/// Verifies the DKG signature result with the given public key, data and signature.
///
/// The signature is verified using the BLS12-381 curve.
pub fn verify_dkg_signature_bls12_381<T: Config>(
	msg: &[u8],
	signature: &[u8],
	expected_key: &[u8],
) -> DispatchResult {
	let public_key = snowbridge_milagro_bls::PublicKey::from_uncompressed_bytes(expected_key)
		.map_err(|_err| Error::<T>::InvalidBlsPublicKey)?;
	let signature = snowbridge_milagro_bls::Signature::from_bytes(signature)
		.map_err(|_err| Error::<T>::InvalidSignatureData)?;
	let signed_data = keccak_256(msg);

	ensure!(signature.verify(&signed_data, &public_key), Error::<T>::SigningKeyMismatch);

	Ok(())
}
