// Copyright 2022-2025 Tangle Foundation.
// This file is part of Tangle.
// This file originated in Moonbeam's codebase.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.

use crate::GetT;
use ethereum::{TransactionAction, TransactionV3 as EthereumTransaction};
use ethereum_types::{H160, H256, U256};
use serde::{Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct Summary {
	pub to: Option<H160>,
	pub value: U256,
	pub gas: U256,
	pub gas_price: U256,
}

impl Serialize for Summary {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let res = format!(
			"0x{:x}: {} wei + {} gas x {} wei",
			self.to.unwrap_or_default(),
			self.value,
			self.gas,
			self.gas_price
		);
		serializer.serialize_str(&res)
	}
}

impl GetT for Summary {
	fn get(_hash: H256, _from_address: H160, txn: &EthereumTransaction) -> Self {
		let (action, value, gas_price, gas_limit) = match txn {
			EthereumTransaction::Legacy(t) => (t.action, t.value, t.gas_price, t.gas_limit),
			EthereumTransaction::EIP2930(t) => (t.action, t.value, t.gas_price, t.gas_limit),
			EthereumTransaction::EIP1559(t) => (t.action, t.value, t.max_fee_per_gas, t.gas_limit),
			EthereumTransaction::EIP7702(t) => (
				ethereum::TransactionAction::Create,
				Default::default(),
				t.max_fee_per_gas,
				t.gas_limit,
			),
		};

		let value_bytes = value.to_big_endian();
		let value_converted = U256::from_big_endian(&value_bytes);

		let gas_price_bytes = gas_price.to_big_endian();
		let gas_price_converted = U256::from_big_endian(&gas_price_bytes);

		let gas_limit_bytes = gas_limit.to_big_endian();
		let gas_limit_converted = U256::from_big_endian(&gas_limit_bytes);

		Self {
			to: match action {
				TransactionAction::Call(to) => {
					let to_bytes: [u8; 20] = to.0;
					Some(H160::from_slice(&to_bytes))
				},
				_ => None,
			},
			value: value_converted,
			gas_price: gas_price_converted,
			gas: gas_limit_converted,
		}
	}
}
