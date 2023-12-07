// Copyright 2022 Webb Technologies Inc.
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
pub mod jobs;
use sp_runtime::AccountId32;

pub mod profile;
pub mod roles;
/// Reputation type
pub type Reputation = u128;
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
/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, Index>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Moment
pub type Moment = u64;

pub const EPOCH_DURATION_IN_BLOCKS: u64 = 10 * crate::time::MINUTES;

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
