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
use precompile_utils::testing::*;
use sp_core::{sr25519, Pair, H160};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn signature_verification_works_sr25519_schnorr() {
	ExtBuilder.build().execute_with(|| {
		let pair = sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();
		let message = b"hello world";
		let signature = pair.sign(message);

		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PcallSchnorrSr25519::verify {
					public_bytes: public.0.to_vec().into(),
					signature_bytes: signature.0.to_vec().into(),
					message: message.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(false);
	});
}
