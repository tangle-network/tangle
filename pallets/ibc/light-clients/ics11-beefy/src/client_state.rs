// Copyright (C) 2022 ComposableFi.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::string::ToString;
use core::{convert::TryFrom, fmt::Debug, marker::PhantomData, time::Duration};
use ibc::prelude::*;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_consensus_beefy::{known_payloads::MMR_ROOT_ID, mmr::BeefyNextAuthoritySet};
use sp_core::H256;
use sp_runtime::SaturatedConversion;
use tendermint_proto::Protobuf;

use crate::proto::{BeefyAuthoritySet, ClientState as RawClientState};

use crate::{client_message::BeefyHeader, error::Error};

use crate::client_def::BeefyClient;
use ibc::{
	core::{ics02_client::client_state::ClientType, ics24_host::identifier::ChainId},
	timestamp::Timestamp,
	Height,
};
use light_client_common::RelayChain;

/// Protobuf type url for Beefy ClientState
pub const BEEFY_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ClientState";

#[derive(PartialEq, Clone, Debug, Default, Eq)]
pub struct ClientState<H> {
	/// The chain id
	pub chain_id: ChainId,
	/// Relay chain
	pub relay_chain: RelayChain,
	/// Latest mmr root hash
	pub mmr_root_hash: H256,
	/// block number for the latest mmr_root_hash
	pub latest_beefy_height: u32,
	/// Block height when the client was frozen due to a misbehaviour
	pub frozen_height: Option<Height>,
	/// latest parachain height
	pub latest_para_height: u32,
	/// ParaId of associated parachain
	pub para_id: u32,
	/// authorities for the current round
	pub authority: BeefyNextAuthoritySet<H256>,
	/// authorities for the next round
	pub next_authority_set: BeefyNextAuthoritySet<H256>,
	/// Phantom type
	pub _phantom: PhantomData<H>,
}

impl<H: Clone> Protobuf<RawClientState> for ClientState<H> {}

impl<H: Clone> ClientState<H> {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		relay_chain: RelayChain,
		para_id: u32,
		latest_para_height: u32,
		mmr_root_hash: H256,
		latest_beefy_height: u32,
		authority_set: BeefyNextAuthoritySet<H256>,
		next_authority_set: BeefyNextAuthoritySet<H256>,
	) -> Result<ClientState<H>, Error> {
		if authority_set.id >= next_authority_set.id {
			return Err(Error::Custom(
				"ClientState next authority set id must be greater than current authority set id"
					.to_string(),
			));
		}
		let chain_id = ChainId::new(relay_chain.to_string(), para_id.into());

		Ok(Self {
			chain_id,
			mmr_root_hash,
			latest_beefy_height,
			frozen_height: None,
			authority: authority_set,
			next_authority_set,
			relay_chain,
			latest_para_height,
			para_id,
			_phantom: PhantomData,
		})
	}

	/// Should only be called if this header has been verified successfully
	pub fn from_header(self, header: BeefyHeader) -> Result<Self, Error> {
		let mut clone = self.clone();
		let mut authority_changed = false;
		let (mmr_root_hash, latest_beefy_height, next_authority_set) =
			if let Some(mmr_update) = header.mmr_update_proof {
				if mmr_update.signed_commitment.commitment.validator_set_id
					== self.next_authority_set.id
				{
					authority_changed = true;
				}
				(
					H256::from_slice(
						mmr_update
							.signed_commitment
							.commitment
							.payload
							.get_raw(&MMR_ROOT_ID)
							.ok_or_else(|| Error::Custom("Invalid header".into()))?,
					),
					mmr_update.signed_commitment.commitment.block_number,
					mmr_update.latest_mmr_leaf.beefy_next_authority_set,
				)
			} else {
				(self.mmr_root_hash, self.latest_beefy_height, self.next_authority_set)
			};
		clone.mmr_root_hash = mmr_root_hash;
		clone.latest_beefy_height = latest_beefy_height;
		if authority_changed {
			clone.authority = clone.next_authority_set;
			clone.next_authority_set = next_authority_set;
		}
		Ok(clone)
	}

	/// Verify the time and height delays
	pub fn verify_delay_passed(
		current_time: Timestamp,
		current_height: Height,
		processed_time: Timestamp,
		processed_height: Height,
		delay_period_time: Duration,
		delay_period_blocks: u64,
	) -> Result<(), Error> {
		let earliest_time = (processed_time + delay_period_time)
			.map_err(|_| Error::Custom("Timestamp overflowed!".into()))?;
		if !(current_time == earliest_time || current_time.after(&earliest_time)) {
			return Err(Error::Custom(format!("Not enough time elapsed current time: {current_time}, earliest time: {earliest_time}")));
		}

		let earliest_height = processed_height.add(delay_period_blocks);
		if current_height < earliest_height {
			return Err(Error::Custom(format!("Not enough blocks elapsed, current height: {current_height}, earliest height: {earliest_height}")));
		}

		Ok(())
	}

	pub fn with_frozen_height(self, h: Height) -> Result<Self, Error> {
		if h == Height::zero() {
			return Err(Error::Custom(
				"ClientState frozen height must be greater than zero".to_string(),
			));
		}
		Ok(Self { frozen_height: Some(h), ..self })
	}

	/// Verify that the client is at a sufficient height and unfrozen at the given height
	pub fn verify_height(&self, height: Height) -> Result<(), Error> {
		let latest_para_height = Height::new(self.para_id.into(), self.latest_para_height.into());
		if latest_para_height < height {
			return Err(Error::Custom(format!(
				"Insufficient height, known height: {latest_para_height}, given height: {height}"
			)));
		}

		match self.frozen_height {
			Some(frozen_height) if frozen_height <= height => {
				Err(Error::Custom(format!("Client has been frozen at height {frozen_height}")))
			},
			_ => Ok(()),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeOptions;

impl<H> ClientState<H> {
	pub fn latest_height(&self) -> Height {
		Height::new(self.para_id.into(), self.latest_para_height.into())
	}

	pub fn chain_id(&self) -> ChainId {
		self.chain_id.clone()
	}

	pub fn client_type() -> ClientType {
		"11-beefy".to_string()
	}

	pub fn frozen_height(&self) -> Option<Height> {
		self.frozen_height
	}

	pub fn upgrade(
		mut self,
		upgrade_height: Height,
		_upgrade_options: UpgradeOptions,
		_chain_id: ChainId,
	) -> Self {
		self.frozen_height = None;
		// Upgrade the client state
		self.latest_beefy_height = upgrade_height.revision_height.saturated_into::<u32>();

		self
	}

	/// Check if the state is expired when `elapsed` time has passed since the latest consensus
	/// state timestamp
	pub fn expired(&self, elapsed: Duration) -> bool {
		elapsed > self.relay_chain.trusting_period()
	}
}

impl<H> ibc::core::ics02_client::client_state::ClientState for ClientState<H>
where
	H: light_client_common::HostFunctions + beefy_light_client_primitives::HostFunctions,
{
	type UpgradeOptions = UpgradeOptions;
	type ClientDef = BeefyClient<H>;

	fn chain_id(&self) -> ChainId {
		self.chain_id()
	}

	fn client_def(&self) -> Self::ClientDef {
		BeefyClient::default()
	}

	fn client_type(&self) -> ClientType {
		Self::client_type()
	}

	fn latest_height(&self) -> Height {
		self.latest_height()
	}

	fn frozen_height(&self) -> Option<Height> {
		self.frozen_height()
	}

	fn upgrade(
		self,
		upgrade_height: Height,
		upgrade_options: UpgradeOptions,
		chain_id: ChainId,
	) -> Self {
		self.upgrade(upgrade_height, upgrade_options, chain_id)
	}

	fn expired(&self, elapsed: Duration) -> bool {
		self.expired(elapsed)
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl<H> TryFrom<RawClientState> for ClientState<H> {
	type Error = Error;

	fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
		let authority_set = raw
			.authority
			.and_then(|set| {
				Some(BeefyNextAuthoritySet {
					id: set.id,
					len: set.len,
					keyset_commitment: H256::decode(&mut &*set.authority_root).ok()?,
				})
			})
			.ok_or_else(|| Error::Custom(format!("Current authority set is missing")))?;

		let next_authority_set = raw
			.next_authority_set
			.and_then(|set| {
				Some(BeefyNextAuthoritySet {
					id: set.id,
					len: set.len,
					keyset_commitment: H256::decode(&mut &*set.authority_root).ok()?,
				})
			})
			.ok_or_else(|| Error::Custom(format!("Next authority set is missing")))?;

		let mmr_root_hash = H256::decode(&mut &*raw.mmr_root_hash)?;
		let relay_chain = RelayChain::from_i32(raw.relay_chain)?;
		let chain_id = ChainId::new(relay_chain.to_string(), raw.para_id.into());

		Ok(Self {
			chain_id,
			mmr_root_hash,
			latest_beefy_height: raw.latest_beefy_height,
			frozen_height: raw.frozen_height.map(|height| Height::new(raw.para_id.into(), height)),
			authority: authority_set,
			next_authority_set,
			relay_chain,
			latest_para_height: raw.latest_para_height,
			para_id: raw.para_id,
			_phantom: Default::default(),
		})
	}
}

impl<H> From<ClientState<H>> for RawClientState {
	fn from(client_state: ClientState<H>) -> Self {
		RawClientState {
			mmr_root_hash: client_state.mmr_root_hash.encode(),
			latest_beefy_height: client_state.latest_beefy_height,
			frozen_height: client_state
				.frozen_height
				.map(|frozen_height| frozen_height.revision_height),
			authority: Some(BeefyAuthoritySet {
				id: client_state.authority.id,
				len: client_state.authority.len,
				authority_root: client_state.authority.keyset_commitment.encode(),
			}),
			next_authority_set: Some(BeefyAuthoritySet {
				id: client_state.next_authority_set.id,
				len: client_state.next_authority_set.len,
				authority_root: client_state.next_authority_set.keyset_commitment.encode(),
			}),
			relay_chain: client_state.relay_chain as i32,
			para_id: client_state.para_id,
			latest_para_height: client_state.latest_para_height,
		}
	}
}

#[cfg(test)]
pub mod test_util {
	use super::*;
	use crate::mock::AnyClientState;

	pub fn get_dummy_beefy_state() -> AnyClientState {
		AnyClientState::Beefy(
			ClientState::new(
				RelayChain::Rococo,
				2000,
				0,
				Default::default(),
				0,
				Default::default(),
				Default::default(),
			)
			.unwrap(),
		)
	}
}
