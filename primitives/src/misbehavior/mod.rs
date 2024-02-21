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

use crate::{jobs::JobId, roles::RoleType};

use frame_support::pallet_prelude::*;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

pub mod dfns_cggmp21;
pub mod traits;
pub mod zcash_frost;

pub use traits::*;

/// Represents a Signed Round Message by the offender.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct SignedRoundMessage {
	/// Index of a party who sent the message
	pub sender: u16,
	/// Received message
	pub message: Vec<u8>,
	/// Signature of sender + message.
	///
	/// This is the signature of the message by the sender.
	///
	/// # Note
	/// sender_bytes = sender.to_be_bytes();
	/// hash = keccak256(sender_bytes + message);
	/// signature = sign(hash);
	pub signature: Vec<u8>,
}

/// Represents a Misbehavior submission
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct MisbehaviorSubmission {
	/// The role type of the misbehaving node
	pub role_type: RoleType,
	/// The misbehaving party's ECDSA public key.
	pub offender: [u8; 33],
	/// The current Job id.
	pub job_id: JobId,
	/// The justification for the misbehavior
	pub justification: MisbehaviorJustification,
}

/// Represents a Misbehavior Justification kind
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum MisbehaviorJustification {
	DKGTSS(DKGTSSJustification),
	ZkSaaS(ZkSaaSJustification),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum DKGTSSJustification {
	/// dfns CGGMP21 Implementation-specific justification
	DfnsCGGMP21(dfns_cggmp21::DfnsCGGMP21Justification),
	/// zcash FROST Implementation-specific justification
	ZCashFrost(zcash_frost::ZCashFrostJustification),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ZkSaaSJustification {}
