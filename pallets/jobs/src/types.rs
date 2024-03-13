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
use super::*;
use tangle_primitives::jobs::*;

pub type JobSubmissionOf<T> = JobSubmission<
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	<T as Config>::MaxParticipants,
	<T as Config>::MaxSubmissionLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type JobInfoOf<T> = JobInfo<
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	BalanceOf<T>,
	<T as Config>::MaxParticipants,
	<T as Config>::MaxSubmissionLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type PhaseResultOf<T> = PhaseResult<
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	<T as Config>::MaxParticipants,
	<T as Config>::MaxKeyLen,
	<T as Config>::MaxDataLen,
	<T as Config>::MaxSignatureLen,
	<T as Config>::MaxSubmissionLen,
	<T as Config>::MaxProofLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type JobResultOf<T> = JobResult<
	<T as Config>::MaxParticipants,
	<T as Config>::MaxKeyLen,
	<T as Config>::MaxSignatureLen,
	<T as Config>::MaxDataLen,
	<T as Config>::MaxProofLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type DKGTSSKeySubmissionResultOf<T> = DKGTSSKeySubmissionResult<
	<T as Config>::MaxKeyLen,
	<T as Config>::MaxParticipants,
	<T as Config>::MaxSignatureLen,
>;

pub type DKGTSSSignatureResultOf<T> = DKGTSSSignatureResult<
	<T as Config>::MaxDataLen,
	<T as Config>::MaxKeyLen,
	<T as Config>::MaxSignatureLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type DKGTSSKeyRotationResultOf<T> = DKGTSSKeyRotationResult<
	<T as Config>::MaxKeyLen,
	<T as Config>::MaxSignatureLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type ZkSaaSCircuitResultOf<T> = ZkSaaSCircuitResult<<T as Config>::MaxParticipants>;

pub type ZkSaaSProofResultOf<T> = ZkSaaSProofResult<<T as Config>::MaxProofLen>;

pub type RpcResponseJobsDataOf<T> = RpcResponseJobsData<
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	<T as Config>::MaxParticipants,
	<T as Config>::MaxSubmissionLen,
	<T as Config>::MaxAdditionalParamsLen,
>;

pub type ParticipantKeysOf<T> = BoundedVec<ParticipantKeyOf<T>, <T as Config>::MaxParticipants>;

pub type ParticipantKeyOf<T> = BoundedVec<u8, <T as Config>::MaxKeyLen>;
