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

use frame_support::{pallet_prelude::*, RuntimeDebug};
/// Represents different types of validator offences.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ValidatorOffence {
	/// The validator has been inactive.
	Inactivity,
	/// The validator has committed duplicate signing.
	Equivocation,
}
