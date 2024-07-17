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

use core::str::FromStr;

use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData as RawPacketData;
use serde::{Deserialize, Serialize};

use super::{error::Error, Amount, PrefixedCoin, PrefixedDenom};
use crate::signer::Signer;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PacketData {
	#[serde(flatten)]
	pub token: PrefixedCoin,
	pub sender: Signer,
	pub receiver: Signer,
	#[serde(default)]
	pub memo: String,
}

impl TryFrom<RawPacketData> for PacketData {
	type Error = Error;

	fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
		// This denom may be prefixed or unprefixed.
		let denom = PrefixedDenom::from_str(&raw_pkt_data.denom)?;
		let amount = Amount::from_str(&raw_pkt_data.amount)?;
		Ok(Self {
			token: PrefixedCoin { denom, amount },
			sender: raw_pkt_data.sender.parse().map_err(Error::signer)?,
			receiver: raw_pkt_data.receiver.parse().map_err(Error::signer)?,
			memo: raw_pkt_data.memo,
		})
	}
}

impl From<PacketData> for RawPacketData {
	fn from(pkt_data: PacketData) -> Self {
		Self {
			denom: pkt_data.token.denom.to_string(),
			amount: pkt_data.token.amount.to_string(),
			sender: pkt_data.sender.to_string(),
			receiver: pkt_data.receiver.to_string(),
			memo: pkt_data.memo,
		}
	}
}
