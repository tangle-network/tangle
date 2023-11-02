// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
#![cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{RuntimeEvent, *};
use sp_runtime::traits::BadOrigin;
use tangle_primitives::jobs::{DKGJobType, JobSubmission, JobType};

#[test]
fn jobs_submission_works_for_dkg() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType {
				participants: vec![100, 2, 3, 4, 5],
				threshold: 3,
			}),
		};

		// should fail with invalid validator
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(1), submission),
			Error::<Runtime>::InvalidValidator
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType { participants: vec![1, 2, 3, 4, 5], threshold: 5 }),
		};

		// should fail with invalid threshold
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(1), submission),
			Error::<Runtime>::InvalidJobParams
		);

		// should save and store correctly
		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType { participants: vec![1, 2, 3, 4, 5], threshold: 3 }),
		};

		// should fail with invalid threshold
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(1), submission),);
	});
}
