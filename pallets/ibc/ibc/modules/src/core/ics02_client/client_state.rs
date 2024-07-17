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
	core::{
		ics02_client::client_def::ClientDef,
		ics24_host::identifier::{ChainId, ClientId},
		ics26_routing::context::ReaderContext,
	},
	prelude::*,
	Height,
};
use alloc::string::String;
use core::{
	fmt::Debug,
	marker::{Send, Sync},
	time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
	Active,
	Frozen,
	Expired,
	Unknown,
	Unauthorized,
}

pub trait ClientState: Clone + Debug + Send + Sync {
	/// Client-specific options for upgrading the client
	type UpgradeOptions;
	type ClientDef: ClientDef<ClientState = Self>;

	/// Return the chain identifier which this client is serving (i.e., the client is verifying
	/// consensus states from this chain).
	fn chain_id(&self) -> ChainId;

	/// Type of client associated with this state (eg. Tendermint)
	fn client_def(&self) -> Self::ClientDef;

	/// Returns one of the prefixes that should be present in any client identifiers.
	/// The prefix is deterministic for a given chain type, hence all clients for a Tendermint-type
	/// chain, for example, will have the prefix '07-tendermint'.
	fn client_type(&self) -> ClientType;

	/// Latest height of consensus state
	fn latest_height(&self) -> Height;

	/// Status of the client
	fn status<Ctx: ReaderContext>(&self, _ctx: &Ctx, _client_id: &ClientId) -> Status {
		if self.frozen_height().is_some() {
			Status::Frozen
		} else {
			Status::Active
		}
	}

	/// Freeze status of the client
	fn is_frozen<Ctx: ReaderContext>(&self, ctx: &Ctx, client_id: &ClientId) -> bool {
		self.status(ctx, client_id) == Status::Frozen
	}

	/// Frozen height of the client
	fn frozen_height(&self) -> Option<Height>;

	/// Helper function to verify the upgrade client procedure.
	/// Resets all fields except the blockchain-specific ones,
	/// and updates the given fields.
	fn upgrade(
		self,
		upgrade_height: Height,
		upgrade_options: Self::UpgradeOptions,
		chain_id: ChainId,
	) -> Self;

	/// Helper function to verify the upgrade client procedure.
	fn expired(&self, elapsed: Duration) -> bool;

	/// Performs downcast of the client state from an "AnyClientState" type to T, otherwise
	/// panics. Downcast from `T` to `T` is always successful.
	fn downcast<T: Clone + 'static>(self) -> Option<T>
	where
		Self: 'static,
	{
		<dyn core::any::Any>::downcast_ref(&self).cloned()
	}

	fn wrap(sub_state: &dyn core::any::Any) -> Option<Self>
	where
		Self: 'static,
	{
		sub_state.downcast_ref::<Self>().cloned()
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error>;
}

/// Type of the client, depending on the specific consensus algorithm.
pub type ClientType = String;
