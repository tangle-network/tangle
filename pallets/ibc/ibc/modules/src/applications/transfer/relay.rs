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

//! This module implements the processing logic for ICS20 (token transfer) message.
use crate::{
	applications::transfer::{
		context::Ics20Context, error::Error as Ics20Error, is_sender_chain_source,
		packet::PacketData,
	},
	core::ics04_channel::packet::Packet,
	prelude::*,
};

pub mod on_ack_packet;
pub mod on_recv_packet;
pub mod on_timeout_packet;
pub mod send_transfer;

fn refund_packet_token(
	ctx: &mut impl Ics20Context,
	packet: &Packet,
	data: &PacketData,
) -> Result<(), Ics20Error> {
	let sender = data
		.sender
		.clone()
		.try_into()
		.map_err(|_| Ics20Error::parse_account_failure())?;

	if is_sender_chain_source(packet.source_port.clone(), packet.source_channel, &data.token.denom)
	{
		// unescrow tokens back to sender
		let escrow_address =
			ctx.get_channel_escrow_address(&packet.source_port, packet.source_channel)?;

		ctx.send_coins(&escrow_address, &sender, &data.token)
	}
	// mint vouchers back to sender
	else {
		ctx.mint_coins(&sender, &data.token)
	}
}
