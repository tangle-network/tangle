// Copyright (C) Moondance Labs Ltd.
// This file is part of Tangle.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::pallet_prelude::*;

/// Statistics for operator heartbeats
#[derive(
	Encode, Decode, Clone, Eq, PartialEq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen, Default,
)]
#[scale_info(skip_type_params(BlockNumber))]
#[codec(encode_bound(skip_type_params(BlockNumber)))]
#[codec(decode_bound(skip_type_params(BlockNumber)))]
#[codec(mel_bound(skip_type_params(BlockNumber)))]
pub struct HeartbeatStats {
	/// Total number of heartbeats expected since the service started
	pub expected_heartbeats: u32,
	/// Total number of heartbeats actually received
	pub received_heartbeats: u32,
	/// The last block when a slashing check was performed
	pub last_check_block: u32,
	/// The last block when a heartbeat was received
	pub last_heartbeat_block: u32,
}
