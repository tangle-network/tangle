// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
use super::*;
pub mod ordered_set;
pub mod rewards;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::{generic, AccountId32, OpaqueExtrinsic};

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// A custom AccountId32 type that exposes the underlying 32-byte array.
pub struct WrappedAccountId32(pub [u8; 32]);

/// The type for looking up accounts.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u64;

/// Round index for sessions
pub type RoundIndex = u32;

/// Blueprint ID
pub type BlueprintId = u64;

/// Service request ID
pub type ServiceRequestId = u64;

/// Service instance ID
pub type InstanceId = u64;

/// Job call ID
pub type JobCallId = u64;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, Index>;

// Moment
pub type Moment = u64;

impl From<WrappedAccountId32> for AccountId32 {
	fn from(x: WrappedAccountId32) -> Self {
		AccountId32::new(x.0)
	}
}

impl From<WrappedAccountId32> for sp_core::sr25519::Public {
	fn from(x: WrappedAccountId32) -> Self {
		sp_core::sr25519::Public::from_raw(x.0)
	}
}

/// Different Account kinds
#[derive(
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Copy,
	Clone,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Account<AccountId> {
	#[codec(index = 0)]
	Id(AccountId),
	#[codec(index = 1)]
	Address(sp_core::H160),
}

impl<AccountId> Default for Account<AccountId> {
	fn default() -> Self {
		Account::<AccountId>::Address(sp_core::H160([0u8; 20]))
	}
}

impl<AccountId> Account<AccountId> {
	/// Create a new account from an AccountId.
	pub fn id(account_id: AccountId) -> Self {
		Self::Id(account_id)
	}

	/// Create a new account from an EVM address.
	pub fn address(address: sp_core::H160) -> Self {
		Self::Address(address)
	}
	/// Returns `true` if the account is native (a la [`Id`]).
	///
	/// [`Id`]: Account::Id
	#[must_use]
	#[doc(alias = "is_id", alias = "is_account_id")]
	pub fn is_native(&self) -> bool {
		matches!(self, Self::Id(..))
	}

	/// Returns `true` if the account is [`Address`].
	///
	/// [`Address`]: Account::Address
	#[must_use]
	#[doc(alias = "is_evm")]
	pub fn is_address(&self) -> bool {
		matches!(self, Self::Address(..))
	}

	/// Try to convert into an EVM address.
	#[doc(alias = "try_into_evm")]
	pub fn try_into_address(self) -> Result<sp_core::H160, Self> {
		if let Self::Address(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Try to convert into an AccountId.
	#[doc(alias = "try_into_native")]
	pub fn try_into_account_id(self) -> Result<AccountId, Self> {
		if let Self::Id(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}
}

impl<AccountId> From<sp_core::H160> for Account<AccountId> {
	fn from(v: sp_core::H160) -> Self {
		Self::Address(v)
	}
}
