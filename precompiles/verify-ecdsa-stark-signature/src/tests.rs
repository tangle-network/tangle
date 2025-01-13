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
use generic_ec::{
	coords::{Coordinate, HasAffineXAndParity, Parity},
	curves::Stark,
	Point,
};
use hex_literal::hex;
use precompile_utils::testing::*;
use sp_core::H160;
use starknet_crypto::Felt;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn wrong_signature_length_returns_false() {
	ExtBuilder.build().execute_with(|| {
		let public = [1u8; 33];
		let signature = hex!["0042"];
		let message = hex!["00"];

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: public.into(),
					signature_bytes: signature.into(),
					message: message.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn signature_verification_works_secp256r1_ecdsa() {
	ExtBuilder.build().execute_with(|| {
		let private_key = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000001",
		);
		let message = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000002",
		);
		let k = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000003",
		);
		let signature = starknet_crypto::sign(&private_key, &message, &k).unwrap();
		let public_key = starknet_crypto::get_public_key(&private_key);
		let public_key_point: Point<Stark> = Point::from_x_and_parity(
			&Coordinate::from_be_bytes(&public_key.to_bytes_be()).unwrap(),
			Parity::Odd,
		)
		.unwrap();
		let mut encoded_signature: [u8; 64] = [0u8; 64];
		encoded_signature[..32].copy_from_slice(&signature.r.to_bytes_be());
		encoded_signature[32..].copy_from_slice(&signature.s.to_bytes_be());

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::verify {
					public_bytes: public_key_point.to_bytes(true).to_vec().into(),
					signature_bytes: encoded_signature.to_vec().into(),
					message: message.to_bytes_be().to_vec().into(),
				},
			)
			.expect_no_logs()
			.execute_returns(true);
	});
}

pub fn field_element_from_be_hex(hex: &str) -> Felt {
	let decoded = hex::decode(hex.trim_start_matches("0x")).unwrap();

	if decoded.len() > 32 {
		panic!("hex string too long");
	}

	let mut buffer = [0u8; 32];
	buffer[(32 - decoded.len())..].copy_from_slice(&decoded[..]);

	Felt::from_bytes_be(&buffer)
}
