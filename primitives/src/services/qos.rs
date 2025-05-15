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

use super::{AssetSecurityCommitment, BoundedString, Constraints};
use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::keccak_256;

/// Statistics for operator heartbeats
#[derive(
	Encode, Decode, Clone, Eq, PartialEq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen, Default,
)]
#[scale_info(skip_type_params(BlockNumber))]
#[codec(encode_bound(skip_type_params(BlockNumber)))]
#[codec(decode_bound(skip_type_params(BlockNumber)))]
#[codec(mel_bound(skip_type_params(BlockNumber)))]
pub struct HeartbeatStats<BlockNumber> {
	/// Total number of heartbeats expected since the service started
	pub expected_heartbeats: u32,
	/// Total number of heartbeats actually received
	pub received_heartbeats: u32,
	/// The last block when a slashing check was performed
	pub last_check_block: BlockNumber,
	/// The last block when a heartbeat was received
	pub last_heartbeat_block: BlockNumber,
}

/// Helper function to create and store a heartbeat slash
fn create_heartbeat_slash(
	blueprint_id: BlueprintId,
	service_id: InstanceId,
	operator: T::AccountId,
	slash_percent: Percent,
) {
	// Create an unapplied slash
	let unapplied_slash = UnappliedSlash {
		era: T::OperatorDelegationManager::get_current_round(),
		blueprint_id,
		service_id,
		operator: operator.clone(),
		slash_percent,
	};

	// Store the slash for later processing
	let index = Self::next_unapplied_slash_index();
	UnappliedSlashes::<T>::insert(unapplied_slash.era, index, unapplied_slash.clone());
	NextUnappliedSlashIndex::<T>::set(index.saturating_add(1));

	// Emit an event for the unapplied slash
	Self::deposit_event(Event::<T>::UnappliedSlash {
		index,
		operator,
		blueprint_id,
		service_id,
		slash_percent,
		era: unapplied_slash.era,
	});
}
