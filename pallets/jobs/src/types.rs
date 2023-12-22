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

pub type JobSubmissionOf<T> =
	JobSubmission<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

pub type JobInfoOf<T> =
	JobInfo<<T as frame_system::Config>::AccountId, BlockNumberFor<T>, BalanceOf<T>>;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type PhaseOneResultOf<T> =
	PhaseOneResult<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;
