// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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
use crate::{mock::*, types::FeeInfo, FeeInfo as FeeInfoStorage};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn set_fees_works() {
	new_test_ext().execute_with(|| {
		let new_fee = FeeInfo { base_fee: 10, circuit_fee: 5, prove_fee: 5 };

		// should fail for non update origin
		assert_noop!(ZKSaaS::set_fee(RuntimeOrigin::signed(10), new_fee.clone()), BadOrigin);

		// Dispatch a signed extrinsic.
		assert_ok!(ZKSaaS::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		assert_eq!(FeeInfoStorage::<Runtime>::get(), new_fee);
	});
}
