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

use frame_support::pallet_prelude::*;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

use super::SignedRoundMessage;

pub const KEYGEN_EID: &[u8] = b"zcash.frost.keygen";
pub const SIGN_EID: &[u8] = b"zcash.frost.sign";

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ZCashFrostJustification {
	Keygen { participants: Vec<[u8; 33]>, t: u16, reason: KeygenAborted },
	Signing { participants: Vec<[u8; 33]>, t: u16, reason: SigningAborted },
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum KeygenAborted {
	/// party decommitment doesn't match commitment.
	InvalidProofOfKnowledge { round: SignedRoundMessage },
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum SigningAborted {
	/// Invalid signature share for aggregation
	InvalidSignatureShare { round1: Vec<SignedRoundMessage>, round2: Vec<SignedRoundMessage> },
}
