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

use crate::mock::*;
use hex_literal::hex;
use precompile_utils::testing::*;
use snowbridge_milagro_bls::{PublicKey, SecretKey, Signature};
use sp_core::{keccak_256, H160};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn wrong_signature_length_returns_false() {
	ExtBuilder.build().execute_with(|| {
		const BLS_SECRET_KEY: [u8; 32] = [
			78, 252, 122, 126, 32, 0, 75, 89, 252, 31, 42, 130, 254, 88, 6, 90, 138, 202, 135, 194,
			233, 117, 181, 75, 96, 238, 79, 100, 237, 59, 140, 111,
		];
		let secret_key = SecretKey::from_bytes(&BLS_SECRET_KEY).unwrap();
		let pub_key = PublicKey::from_secret_key(&secret_key);
		let signature = hex!["0042"];
		let message = hex!["00"];

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: pub_key.as_uncompressed_bytes().into(),
					signature_bytes: signature.into(),
					message: message.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn bad_signature_returns_false() {
	ExtBuilder.build().execute_with(|| {
		const BLS_SECRET_KEY: [u8; 32] = [
			78, 252, 122, 126, 32, 0, 75, 89, 252, 31, 42, 130, 254, 88, 6, 90, 138, 202, 135, 194,
			233, 117, 181, 75, 96, 238, 79, 100, 237, 59, 140, 111,
		];

		const BLS_DATA_TO_SIGN: &[u8; 13] = b"Hello, world!";

		let secret_key = SecretKey::from_bytes(&BLS_SECRET_KEY).unwrap();
		let pub_key = PublicKey::from_secret_key(&secret_key);
		let msg_hash = keccak_256(BLS_DATA_TO_SIGN);
		let signature = Signature::new(&msg_hash, &secret_key);

		let bad_message = hex!["00"];

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: pub_key.as_uncompressed_bytes().into(),
					signature_bytes: signature.as_bytes()[..10].to_vec().into(),
					message: bad_message.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn signature_verification_works_with_bls318() {
	ExtBuilder.build().execute_with(|| {
		const BLS_SECRET_KEY: [u8; 32] = [
			78, 252, 122, 126, 32, 0, 75, 89, 252, 31, 42, 130, 254, 88, 6, 90, 138, 202, 135, 194,
			233, 117, 181, 75, 96, 238, 79, 100, 237, 59, 140, 111,
		];

		const BLS_DATA_TO_SIGN: &[u8; 13] = b"Hello, world!";

		let secret_key = SecretKey::from_bytes(&BLS_SECRET_KEY).unwrap();
		let pub_key = PublicKey::from_secret_key(&secret_key);
		let msg_hash = keccak_256(BLS_DATA_TO_SIGN);
		let signature = Signature::new(&msg_hash, &secret_key);

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: pub_key.as_uncompressed_bytes().into(),
					signature_bytes: signature.as_bytes().to_vec().into(),
					message: msg_hash.to_vec().into(),
				},
			)
			.expect_no_logs()
			.execute_returns(true);
	});
}
