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

use alloc::string::FromUtf8Error;

use flex_error::{define_error, DisplayOnly, TraceError};
use subtle_encoding::Error as EncodingError;
use tendermint_proto::Error as TendermintProtoError;
use uint::FromStrRadixErr;

use crate::{
	core::{
		ics04_channel::{channel::Order, error as channel_error, Version},
		ics24_host::{
			error::ValidationError,
			identifier::{ChannelId, PortId},
		},
	},
	prelude::*,
	signer::SignerError,
};

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	Error {
		UnknowMessageTypeUrl
			{ url: String }
			| e | { format_args!("unrecognized ICS-20 transfer message type URL {0}", e.url) },

		Ics04Channel
			[ channel_error::Error ]
			|_ | { "Ics04 channel error" },

		DestinationChannelNotFound
			{ port_id: PortId, channel_id: ChannelId }
			| e | { format_args!("destination channel not found in the counterparty of port_id {0} and channel_id {1} ", e.port_id, e.channel_id) },

		InvalidPortId
			{ context: String }
			[ ValidationError ]
			| _ | { "invalid port identifier" },

		InvalidChannelId
			{ context: String }
			[ ValidationError ]
			| _ | { "invalid channel identifier" },

		InvalidPacketTimeoutHeight
			{ context: String }
			| _ | { "invalid packet timeout height value" },

		InvalidPacketTimeoutTimestamp
			{ timestamp: u64 }
			| _ | { "invalid packet timeout timestamp value" },

		Utf8
			[ DisplayOnly<FromUtf8Error> ]
			| _ | { "utf8 decoding error" },

		EmptyBaseDenom
			|_| { "base denomination is empty" },

		InvalidTracePortId
			{ pos: usize }
			[ ValidationError ]
			| e | { format_args!("invalid port id in trace at position: {0}", e.pos) },

		InvalidTraceChannelId
			{ pos: usize }
			[ ValidationError ]
			| e | { format_args!("invalid channel id in trace at position: {0}", e.pos) },

		InvalidTraceLength
			{ len: usize }
			| e | { format_args!("trace length must be even but got: {0}", e.len) },

		InvalidAmount
			[ TraceError<FromStrRadixErr> ]
			| _ | { "invalid amount" },

		InvalidToken
			| _ | { "invalid token" },

		TokenBalanceChange
			| _ | { "token balance change" },

		Signer
			[ SignerError ]
			| _ | { "failed to parse signer" },

		MissingDenomIbcPrefix
			| _ | { "missing 'ibc/' prefix in denomination" },

		MalformedHashDenom
			| _ | { "hashed denom must be of the form 'ibc/{Hash}'" },

		ParseHex
			[ TraceError<EncodingError> ]
			| _ | { "invalid hex string" },

		ChanSeqExceedsLimit
			{ sequence: u64 }
			| e | { format_args!("channel sequence ({0}) exceeds limit of {1}", e.sequence, u32::MAX) },

		ChannelNotUnordered
			{ order: Order }
			| e | { format_args!("expected '{0}' channel, got '{1}'", Order::Unordered, e.order) },

		InvalidVersion
			{ version: Version }
			| e | { format_args!("expected version '{0}', got '{1}'", Version::ics20(), e.version) },

		InvalidCounterpartyVersion
			{ version: Version }
			| e | { format_args!("expected counterparty version '{0}', got '{1}'", Version::ics20(), e.version) },

		CantCloseChannel
			| _ | { "channel cannot be closed" },

		PacketDataDeserialization
			| _ | { "failed to deserialize packet data" },

		AckDeserialization
			| _ | { "failed to deserialize acknowledgement" },

		ReceiveDisabled
			| _ | { "receive is not enabled" },

		SendDisabled
			| _ | { "send is not enabled" },

		ParseAccountFailure
			| _ | { "failed to parse as AccountId" },

		InvalidPort
			{ port_id: PortId, exp_port_id: PortId }
			| e | { format_args!("invalid port: '{0}', expected '{1}'", e.port_id, e.exp_port_id) },

		TraceNotFound
			| _ | { "no trace associated with specified hash" },

		DecodeRawMsg
			[ TraceError<TendermintProtoError> ]
			| _ | { "error decoding raw msg" },

		UnknownMsgType
			{ msg_type: String }
			| e | { format_args!("unknown msg type: {0}", e.msg_type) },

		ImplementationSpecific
			{ reason: String }
			| e | { format_args!("implementation specific error: {}", e.reason) },

	}
}
