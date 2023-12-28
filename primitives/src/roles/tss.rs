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

use crate::jobs::DigitalSignatureType;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

/// Threshold signature role types and their specific elliptic curve.
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Default, Eq, TypeInfo, PartialOrd, Ord,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ThresholdSignatureRoleType {
	#[default]
	TssGG20,
	TssCGGMP,
	TssFrostSr25519,
	TssFrostP256,
	TssFrostSecp256k1,
	TssFrostRistretto255,
	TssFrostBabyJubJub,
	TssFrostEd25519,
	TssEdDSABabyJubJub,
	TssBls381,
}

/// Associated metadata needed for a DKG/TSS role
#[derive(
	Encode, Decode, Clone, RuntimeDebug, PartialEq, Default, Eq, TypeInfo, PartialOrd, Ord,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TssRoleMetadata {
	/// The threshold role type for the DKG.
	pub role_type: ThresholdSignatureRoleType,

	/// The authority key associated with the role.
	pub authority_key: Vec<u8>,
}
