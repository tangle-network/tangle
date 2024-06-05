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

use crate::mock::*;
use hex_literal::hex;
use precompile_utils::testing::*;
use sp_core::{ecdsa, keccak_256, Pair, H160};
use rand_core::OsRng;
use p256::ecdsa::{signature::hazmat::PrehashSigner, SigningKey, VerifyingKey};


fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn wrong_signature_length_returns_false() {
	ExtBuilder::default().build().execute_with(|| {
		let pair = ecdsa::Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();
		let signature = hex!["0042"];
		let message = hex!["00"];

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: public.0.to_vec().into(),
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
	ExtBuilder::default().build().execute_with(|| {
        let mut rng = OsRng;
		let secret_key = SigningKey::random(&mut rng);
		let public_key = VerifyingKey::from(&secret_key);
		let message = b"hello world";
		let prehash = keccak_256(message);
		let (signature, _) = secret_key.sign_prehash(&prehash).unwrap();

        let bad_message = hex!["00"];

        precompiles()
            .prepare_test(
                TestAccount::Alex,
                H160::from_low_u64_be(1),
                PCall::verify {
                    public_bytes: public_key
                        .to_encoded_point(true)
                        .to_bytes()
                        .to_vec()
                    .   into(),
                    signature_bytes: signature.to_vec().into(),
                    message: bad_message.into(),
                },
            )
            .expect_no_logs()
            .execute_returns(false);
    });
}

#[test]
fn signature_verification_works_secp256r1_ecdsa() {
	ExtBuilder::default().build().execute_with(|| {

        let mut rng = OsRng;
		let secret_key = SigningKey::random(&mut rng);
		let public_key = VerifyingKey::from(&secret_key);
		let message = b"hello world";
		let prehash = keccak_256(message);
		let (signature, _) = secret_key.sign_prehash(&prehash).unwrap();


        precompiles()
            .prepare_test(
                TestAccount::Alex,
                H160::from_low_u64_be(1),
                PCall::verify {
                    public_bytes: public_key
                        .to_encoded_point(true)
                        .to_bytes()
                        .to_vec()
                        .into(),
                    signature_bytes: signature.to_vec().into(),
                    message: prehash.into(),
                },
            )
            .expect_no_logs()
            .execute_returns(true);
    });
}
