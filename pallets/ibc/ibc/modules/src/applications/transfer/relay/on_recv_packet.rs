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
		context::Ics20Context, error::Error as Ics20Error, events::DenomTraceEvent,
		is_receiver_chain_source, packet::PacketData, TracePrefix,
	},
	core::{ics04_channel::packet::Packet, ics26_routing::context::ModuleOutputBuilder},
	prelude::*,
};

pub fn process_recv_packet<Ctx: 'static + Ics20Context>(
	ctx: &mut Ctx,
	output: &mut ModuleOutputBuilder,
	packet: &Packet,
	data: PacketData,
) -> Result<(), Ics20Error> {
	if !ctx.is_receive_enabled() {
		return Err(Ics20Error::receive_disabled())
	}

	let receiver_account = data
		.receiver
		.clone()
		.try_into()
		.map_err(|_| Ics20Error::parse_account_failure())?;

	if is_receiver_chain_source(
		packet.source_port.clone(),
		packet.source_channel,
		&data.token.denom,
	) {
		// sender chain is not the source, unescrow tokens
		let prefix = TracePrefix::new(packet.source_port.clone(), packet.source_channel);
		let coin = {
			let mut c = data.token;
			c.denom.remove_trace_prefix(&prefix);
			c
		};

		let escrow_address =
			ctx.get_channel_escrow_address(&packet.destination_port, packet.destination_channel)?;

		ctx.send_coins(&escrow_address, &receiver_account, &coin)
	} else {
		// sender chain is the source, mint vouchers
		let prefix = TracePrefix::new(packet.destination_port.clone(), packet.destination_channel);
		let coin = {
			let mut c = data.token;
			c.denom.add_trace_prefix(prefix);
			c
		};

		let denom_trace_event = DenomTraceEvent {
			trace_hash: ctx.denom_hash_string(&coin.denom),
			denom: coin.denom.clone(),
		};
		output.emit(denom_trace_event.into());

		ctx.mint_coins(&receiver_account, &coin)
	}
}
