// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::{mock::*, *};
use precompile_utils::testing::*;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn submit_dkg_phase_one_job() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 40)])
		.build()
		.execute_with(|| {
			let _ = precompiles().prepare_test(
				Address(CryptoAlith.into()),
				Precompile1,
				PCall::submit_dkg_phase_one_job {
					role_type: RoleType::Tss(ThresholdSignatureRoleType::TssGG20).to_u16(),
					expiry: 100,
					participants: vec![],
					threshold: 2,
					permitted_caller: Address(CryptoAlith.into()),
				},
			);
		})
}

#[test]
fn submit_dkg_phase_two_job() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 40)])
		.build()
		.execute_with(|| {
			let _ = precompiles().prepare_test(
				Address(CryptoAlith.into()),
				Precompile1,
				PCall::submit_dkg_phase_two_job {
					expiry: 100,
					phase_one_id: 1,
					submission: vec![].into(),
				},
			);
		})
}
