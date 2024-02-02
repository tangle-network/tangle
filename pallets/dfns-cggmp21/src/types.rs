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
use frame_support::traits::Currency;

pub type DefaultDigest = sha2::Sha256;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.keygen.threshold.tag")]
pub enum KeygenTag<'a> {
	/// Tag that includes the prover index
	Indexed {
		party_index: u16,
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
	/// Tag w/o party index
	Unindexed {
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
}

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.aux_gen.tag")]
pub enum AuxGenTag<'a> {
	/// Tag that includes the prover index
	Indexed {
		party_index: u16,
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
	/// Tag w/o party index
	Unindexed {
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
}
