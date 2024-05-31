// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]

extern crate alloc;

use alloc::vec::Vec;
use core::cmp::min;

use fp_evm::{ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure};

/// Precompile to verify EcdsaSecp256k1 signature .
pub struct VerifyEcdsaSecp256k1;

impl LinearCostPrecompile for VerifyEcdsaSecp256k1 {
	const BASE: u64 = 3000;
	const WORD: u64 = 0;

	fn execute(i: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		let mut input = [0u8; 161];
		input[..min(i.len(), 161)].copy_from_slice(&i[..min(i.len(), 161)]);

		let mut msg = [0u8; 32];
		let mut sig = [0u8; 65];
		let mut expected_key = [0u8;33];



		msg[0..32].copy_from_slice(&input[0..32]);
		sig[0..32].copy_from_slice(&input[64..96]); // r
		sig[32..64].copy_from_slice(&input[96..128]); // s
		sig[64] = input[63]; // v
		expected_key[0..33].copy_from_slice(&input[128..161]);

		// v can only be 27 or 28 on the full 32 bytes value.
		// https://github.com/ethereum/go-ethereum/blob/a907d7e81aaeea15d80b2d3209ad8e08e3bf49e0/core/vm/contracts.go#L177
		if input[32..63] != [0u8; 31] || ![27, 28].contains(&input[63]) {
			return Ok((ExitSucceed::Returned, [0u8; 0].to_vec()));
		}

		let pub_key = expected_key.to_vec();

		let pub_key_point = k256::AffinePoint::from_bytes(pub_key.as_slice().into());
		if pub_key_point.is_none().into() {
			Err(Error::<T>::InvalidPublicKey)?;
		}
		let verifying_key = k256::ecdsa::VerifyingKey::from_affine(pub_key_point.unwrap())
			.map_err(|_| Error::<T>::InvalidPublicKey)?;
		let signature = k256::ecdsa::Signature::from_slice(signature)
			.map_err(|_| Error::<T>::InvalidSignatureDeserialization)?;

		let message = keccak_256(msg);

		Ok((ExitSucceed::Returned, result))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use pallet_evm_test_vector_support::test_precompile_test_vectors;

	// TODO: this fails on the test "InvalidHighV-bits-1" where it is expected to return ""
	#[test]
	fn process_consensus_tests_for_ecrecover() -> Result<(), String> {
		test_precompile_test_vectors::<TestECRecover>("../testdata/ecRecover.json")?;
		Ok(())
	}
}
