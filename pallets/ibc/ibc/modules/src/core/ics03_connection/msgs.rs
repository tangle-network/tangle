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

//! Message definitions for the connection handshake datagrams.
//!
//! We define each of the four messages in the connection handshake protocol as a `struct`.
//! Each such message comprises the same fields as the datagrams defined in ICS3 English spec:
//! <https://github.com/cosmos/ibc/tree/master/spec/core/ics-003-connection-semantics>.
//!
//! One departure from ICS3 is that we abstract the three counterparty fields (connection id,
//! prefix, and client id) into a single field of type `Counterparty`; this applies to messages
//! `MsgConnectionOpenInit` and `MsgConnectionOpenTry`. One other difference with regards to
//! abstraction is that all proof-related attributes in a message are encapsulated in `Proofs` type.
//!
//! Another difference to ICS3 specs is that each message comprises an additional field called
//! `signer` which is specific to Cosmos-SDK.

use crate::core::{
	ics02_client::context::ClientTypes,
	ics03_connection::msgs::{
		conn_open_ack::MsgConnectionOpenAck, conn_open_confirm::MsgConnectionOpenConfirm,
		conn_open_init::MsgConnectionOpenInit, conn_open_try::MsgConnectionOpenTry,
	},
};
use alloc::boxed::Box;
use core::fmt::Debug;

pub mod conn_open_ack;
pub mod conn_open_confirm;
pub mod conn_open_init;
pub mod conn_open_try;

/// Enumeration of all possible messages that the ICS3 protocol processes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionMsg<C>
where
	C: ClientTypes + Clone + Debug + PartialEq + Eq,
{
	ConnectionOpenInit(MsgConnectionOpenInit),
	ConnectionOpenTry(Box<MsgConnectionOpenTry<C>>),
	ConnectionOpenAck(Box<MsgConnectionOpenAck<C>>),
	ConnectionOpenConfirm(MsgConnectionOpenConfirm),
}

#[cfg(test)]
pub mod test_util {

	use crate::{
		core::ics24_host::identifier::{ClientId, ConnectionId},
		prelude::*,
	};
	use ibc_proto::ibc::core::{
		commitment::v1::MerklePrefix, connection::v1::Counterparty as RawCounterparty,
	};

	pub fn get_dummy_raw_counterparty() -> RawCounterparty {
		RawCounterparty {
			client_id: ClientId::default().to_string(),
			connection_id: ConnectionId::default().to_string(),
			prefix: Some(MerklePrefix { key_prefix: b"ibc".to_vec() }),
		}
	}
}
