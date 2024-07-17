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

use crate::{
	applications::transfer::{
		acknowledgement::Acknowledgement, Amount, PrefixedDenom, MODULE_ID_STR,
	},
	events::{IbcEvent, ModuleEvent},
	prelude::*,
	signer::Signer,
};

const EVENT_TYPE_PACKET: &str = "fungible_token_packet";
const EVENT_TYPE_TIMEOUT: &str = "timeout";
const EVENT_TYPE_DENOM_TRACE: &str = "denomination_trace";
const EVENT_TYPE_TRANSFER: &str = "ibc_transfer";

pub enum Event {
	Recv(RecvEvent),
	Ack(AckEvent),
	AckStatus(AckStatusEvent),
	Timeout(TimeoutEvent),
	DenomTrace(DenomTraceEvent),
	Transfer(TransferEvent),
}

pub struct RecvEvent {
	pub receiver: Signer,
	pub denom: PrefixedDenom,
	pub amount: Amount,
	pub success: bool,
}

impl From<RecvEvent> for IbcEvent {
	fn from(ev: RecvEvent) -> Self {
		let RecvEvent { receiver, denom, amount, success } = ev;
		IbcEvent::AppModule(ModuleEvent {
			kind: EVENT_TYPE_PACKET.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![
				("receiver", receiver).into(),
				("denom", denom).into(),
				("amount", amount).into(),
				("success", success).into(),
			],
		})
	}
}

pub struct AckEvent {
	pub receiver: Signer,
	pub denom: PrefixedDenom,
	pub amount: Amount,
	pub acknowledgement: Acknowledgement,
}

impl From<AckEvent> for IbcEvent {
	fn from(ev: AckEvent) -> Self {
		let AckEvent { receiver, denom, amount, acknowledgement } = ev;
		IbcEvent::AppModule(ModuleEvent {
			kind: EVENT_TYPE_PACKET.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![
				("receiver", receiver).into(),
				("denom", denom).into(),
				("amount", amount).into(),
				("acknowledgement", acknowledgement).into(),
			],
		})
	}
}

pub struct AckStatusEvent {
	pub acknowledgement: Acknowledgement,
}

impl From<AckStatusEvent> for IbcEvent {
	fn from(ev: AckStatusEvent) -> Self {
		let AckStatusEvent { acknowledgement } = ev;
		let mut event = ModuleEvent {
			kind: EVENT_TYPE_PACKET.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![],
		};
		let attr_label = if acknowledgement.is_successful() { "success" } else { "error" };
		event.attributes.push((attr_label, acknowledgement.to_string()).into());
		IbcEvent::AppModule(event)
	}
}

pub struct TimeoutEvent {
	pub refund_receiver: Signer,
	pub refund_denom: PrefixedDenom,
	pub refund_amount: Amount,
}

impl From<TimeoutEvent> for IbcEvent {
	fn from(ev: TimeoutEvent) -> Self {
		let TimeoutEvent { refund_receiver, refund_denom, refund_amount } = ev;
		IbcEvent::AppModule(ModuleEvent {
			kind: EVENT_TYPE_TIMEOUT.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![
				("refund_receiver", refund_receiver).into(),
				("refund_denom", refund_denom).into(),
				("refund_amount", refund_amount).into(),
			],
		})
	}
}

pub struct DenomTraceEvent {
	pub trace_hash: Option<String>,
	pub denom: PrefixedDenom,
}

impl From<DenomTraceEvent> for IbcEvent {
	fn from(ev: DenomTraceEvent) -> Self {
		let DenomTraceEvent { trace_hash, denom } = ev;
		let mut ev = ModuleEvent {
			kind: EVENT_TYPE_DENOM_TRACE.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![("denom", denom).into()],
		};
		if let Some(hash) = trace_hash {
			ev.attributes.push(("trace_hash", hash).into());
		}
		IbcEvent::AppModule(ev)
	}
}

pub struct TransferEvent {
	pub sender: Signer,
	pub receiver: Signer,
}

impl From<TransferEvent> for IbcEvent {
	fn from(ev: TransferEvent) -> Self {
		let TransferEvent { sender, receiver } = ev;
		IbcEvent::AppModule(ModuleEvent {
			kind: EVENT_TYPE_TRANSFER.to_string(),
			module_name: MODULE_ID_STR.parse().expect("invalid ModuleId"),
			attributes: vec![("sender", sender).into(), ("receiver", receiver).into()],
		})
	}
}
