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

mod content;
mod inspect;

use ethereum::TransactionV2 as EthereumTransaction;
use ethereum_types::{H160, H256, U256};
use serde::Serialize;
use std::collections::HashMap;

pub use self::{content::Transaction, inspect::Summary};

pub type TransactionMap<T> = HashMap<H160, HashMap<U256, T>>;

#[derive(Debug, Serialize, Clone)]
pub struct TxPoolResult<T: Serialize> {
	pub pending: T,
	pub queued: T,
}

pub trait Get {
	fn get(hash: H256, from_address: H160, txn: &EthereumTransaction) -> Self;
}
