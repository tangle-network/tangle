// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::prelude::*;

use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
use tendermint_proto::Protobuf;

use crate::{
	core::ics04_channel::{
		error::Error,
		packet::{Packet, Sequence},
	},
	proofs::Proofs,
	signer::Signer,
	tx_msg::Msg,
};

pub const TYPE_URL: &str = "/ibc.core.channel.v1.MsgTimeoutOnClose";

///
/// Message definition for packet timeout domain type.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgTimeoutOnClose {
	pub packet: Packet,
	pub next_sequence_recv: Sequence,
	pub proofs: Proofs,
	pub signer: Signer,
}

impl MsgTimeoutOnClose {
	pub fn new(
		packet: Packet,
		next_sequence_recv: Sequence,
		proofs: Proofs,
		signer: Signer,
	) -> MsgTimeoutOnClose {
		Self { packet, next_sequence_recv, proofs, signer }
	}
}

impl Msg for MsgTimeoutOnClose {
	type ValidationError = Error;
	type Raw = RawMsgTimeoutOnClose;

	fn route(&self) -> String {
		crate::keys::ROUTER_KEY.to_string()
	}

	fn type_url(&self) -> String {
		TYPE_URL.to_string()
	}
}

impl Protobuf<RawMsgTimeoutOnClose> for MsgTimeoutOnClose {}

impl TryFrom<RawMsgTimeoutOnClose> for MsgTimeoutOnClose {
	type Error = Error;

	fn try_from(raw_msg: RawMsgTimeoutOnClose) -> Result<Self, Self::Error> {
		let proofs = Proofs::new(
			raw_msg.proof_unreceived.try_into().map_err(Error::invalid_proof)?,
			None,
			None,
			Some(raw_msg.proof_close.try_into().map_err(Error::invalid_proof)?),
			raw_msg.proof_height.ok_or_else(Error::missing_height)?.into(),
		)
		.map_err(Error::invalid_proof)?;

		// TODO: Domain type verification for the next sequence: this should probably be > 0.

		Ok(MsgTimeoutOnClose {
			packet: raw_msg.packet.ok_or_else(Error::missing_packet)?.try_into()?,
			next_sequence_recv: Sequence::from(raw_msg.next_sequence_recv),
			signer: raw_msg.signer.parse().map_err(Error::signer)?,
			proofs,
		})
	}
}

impl From<MsgTimeoutOnClose> for RawMsgTimeoutOnClose {
	fn from(domain_msg: MsgTimeoutOnClose) -> Self {
		RawMsgTimeoutOnClose {
			packet: Some(domain_msg.packet.into()),
			proof_unreceived: domain_msg.proofs.object_proof().clone().into(),
			proof_close: domain_msg
				.proofs
				.other_proof()
				.clone()
				.map_or_else(Vec::new, |v| v.into()),
			proof_height: Some(domain_msg.proofs.height().into()),
			next_sequence_recv: domain_msg.next_sequence_recv.into(),
			signer: domain_msg.signer.to_string(),
		}
	}
}

#[cfg(test)]
pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose, client::v1::Height as RawHeight,
	};

	use crate::{
		core::ics04_channel::packet::test_utils::get_dummy_raw_packet,
		test_utils::{get_dummy_bech32_account, get_dummy_proof},
	};

	/// Returns a dummy `RawMsgTimeoutOnClose`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_timeout_on_close(
		height: u64,
		timeout_timestamp: u64,
	) -> RawMsgTimeoutOnClose {
		RawMsgTimeoutOnClose {
			packet: Some(get_dummy_raw_packet(height, timeout_timestamp)),
			proof_unreceived: get_dummy_proof(),
			proof_close: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: height }),
			next_sequence_recv: 1,
			signer: get_dummy_bech32_account(),
		}
	}
}
