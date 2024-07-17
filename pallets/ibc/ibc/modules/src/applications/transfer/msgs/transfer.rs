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

//! This is the definition of a transfer messages that an application submits to a chain.

use crate::prelude::*;
use core::fmt::Display;

use ibc_proto::{
	cosmos::base::v1beta1::Coin, google::protobuf::Any,
	ibc::applications::transfer::v1::MsgTransfer as RawMsgTransfer,
};
use tendermint_proto::Protobuf;

use crate::{
	applications::transfer::error::Error,
	core::{
		ics02_client::height::Height,
		ics24_host::identifier::{ChannelId, PortId},
	},
	signer::Signer,
	timestamp::Timestamp,
	tx_msg::Msg,
};

pub const TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";

/// Message used to build an ICS20 token transfer packet.
///
/// Note that this message is not a packet yet, as it lacks the proper sequence
/// number, and destination port/channel. This is by design. The sender of the
/// packet, which might be the user of a command line application, should only
/// have to specify the information related to the transfer of the token, and
/// let the library figure out how to build the packet properly.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgTransfer<C = Coin> {
	/// the port on which the packet will be sent
	pub source_port: PortId,
	/// the channel by which the packet will be sent
	pub source_channel: ChannelId,
	/// the tokens to be transferred
	pub token: C,
	/// the sender address
	pub sender: Signer,
	/// the recipient address on the destination chain
	pub receiver: Signer,
	/// Timeout height relative to the current block height.
	/// The timeout is disabled when set to 0.
	pub timeout_height: Height,
	/// Timeout timestamp relative to the current block timestamp.
	/// The timeout is disabled when set to 0.
	pub timeout_timestamp: Timestamp,
	/// Memo field
	pub memo: String,
}

impl<C> Msg for MsgTransfer<C>
where
	C: Clone,
	Coin: From<C>,
{
	type ValidationError = Error;
	type Raw = RawMsgTransfer;

	fn route(&self) -> String {
		crate::keys::ROUTER_KEY.to_string()
	}

	fn type_url(&self) -> String {
		TYPE_URL.to_string()
	}
}

impl<C: TryFrom<Coin>> TryFrom<RawMsgTransfer> for MsgTransfer<C>
where
	Error: From<<C as TryFrom<Coin>>::Error>,
{
	type Error = Error;

	fn try_from(raw_msg: RawMsgTransfer) -> Result<Self, Self::Error> {
		let timeout_timestamp = Timestamp::from_nanoseconds(raw_msg.timeout_timestamp)
			.map_err(|_| Error::invalid_packet_timeout_timestamp(raw_msg.timeout_timestamp))?;

		let timeout_height = match raw_msg.timeout_height.clone() {
			None => Height::zero(),
			Some(raw_height) => raw_height.try_into().map_err(|e| {
				Error::invalid_packet_timeout_height(format!("invalid timeout height {}", e))
			})?,
		};

		Ok(MsgTransfer {
			source_port: raw_msg
				.source_port
				.parse()
				.map_err(|e| Error::invalid_port_id(raw_msg.source_port.clone(), e))?,
			source_channel: raw_msg
				.source_channel
				.parse()
				.map_err(|e| Error::invalid_channel_id(raw_msg.source_channel.clone(), e))?,
			token: C::try_from(raw_msg.token.ok_or_else(Error::invalid_token)?)?,
			sender: raw_msg.sender.parse().map_err(Error::signer)?,
			receiver: raw_msg.receiver.parse().map_err(Error::signer)?,
			timeout_height,
			timeout_timestamp,
			memo: raw_msg.memo,
		})
	}
}

impl<C> From<MsgTransfer<C>> for RawMsgTransfer
where
	Coin: From<C>,
{
	fn from(domain_msg: MsgTransfer<C>) -> Self {
		RawMsgTransfer {
			source_port: domain_msg.source_port.to_string(),
			source_channel: domain_msg.source_channel.to_string(),
			token: Some(domain_msg.token.into()),
			sender: domain_msg.sender.to_string(),
			receiver: domain_msg.receiver.to_string(),
			timeout_height: Some(domain_msg.timeout_height.into()),
			timeout_timestamp: domain_msg.timeout_timestamp.nanoseconds(),
			memo: domain_msg.memo,
		}
	}
}

impl<C: Protobuf<Coin>> Protobuf<RawMsgTransfer> for MsgTransfer<C>
where
	Coin: From<C>,
	<C as TryFrom<Coin>>::Error: Display,
	MsgTransfer<C>: TryFrom<RawMsgTransfer>,
	<MsgTransfer<C> as TryFrom<RawMsgTransfer>>::Error: Display,
{
}

impl<C> TryFrom<Any> for MsgTransfer<C>
where
	C: TryFrom<Any> + Protobuf<Coin>,
	<C as TryFrom<Coin>>::Error: Display,
	Coin: From<C>,
	MsgTransfer<C>: TryFrom<RawMsgTransfer>,
	<MsgTransfer<C> as TryFrom<RawMsgTransfer>>::Error: Display,
{
	type Error = Error;

	fn try_from(raw: Any) -> Result<Self, Self::Error> {
		match raw.type_url.as_str() {
			TYPE_URL => MsgTransfer::decode_vec(&raw.value).map_err(Error::decode_raw_msg),
			_ => Err(Error::unknown_msg_type(raw.type_url)),
		}
	}
}

impl<C> TryFrom<MsgTransfer<C>> for Any
where
	C: Protobuf<Coin>,
	C: From<Coin>,
	Coin: From<C>,
	<C as TryFrom<Coin>>::Error: Display,
	MsgTransfer<C>: TryFrom<RawMsgTransfer>,
	<MsgTransfer<C> as TryFrom<RawMsgTransfer>>::Error: Display,
{
	type Error = Error;

	fn try_from(msg: MsgTransfer<C>) -> Result<Self, Self::Error> {
		Ok(Self {
			type_url: TYPE_URL.to_string(),
			value: msg.encode_vec().map_err(Error::decode_raw_msg)?,
		})
	}
}

#[cfg(test)]
pub mod test_util {
	use core::{ops::Add, time::Duration};

	use super::MsgTransfer;
	use crate::{
		applications::transfer::{BaseCoin, PrefixedCoin},
		bigint::U256,
		core::ics24_host::identifier::{ChannelId, PortId},
		signer::Signer,
		test_utils::get_dummy_bech32_account,
		timestamp::Timestamp,
		Height,
	};

	// Returns a dummy ICS20 `MsgTransfer`, for testing only!
	pub fn get_dummy_msg_transfer(height: u64) -> MsgTransfer<PrefixedCoin> {
		let address: Signer = get_dummy_bech32_account().as_str().parse().unwrap();
		MsgTransfer {
			source_port: PortId::default(),
			source_channel: ChannelId::default(),
			token: BaseCoin { denom: "uatom".parse().unwrap(), amount: U256::from(10).into() }
				.into(),
			sender: address.clone(),
			receiver: address,
			timeout_timestamp: Timestamp::now().add(Duration::from_secs(10)).unwrap(),
			timeout_height: Height { revision_number: 0, revision_height: height },
			memo: "".to_string(),
		}
	}
}
