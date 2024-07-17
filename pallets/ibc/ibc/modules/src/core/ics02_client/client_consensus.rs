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
	core::{ics23_commitment::commitment::CommitmentRoot, ics24_host::identifier::ClientId},
	events::WithBlockDataType,
	prelude::*,
	timestamp::Timestamp,
};
use core::{
	fmt::Debug,
	marker::{Send, Sync},
};

pub trait ConsensusState: Clone + Debug + Send + Sync {
	type Error;

	/// Commitment root of the consensus state, which is used for key-value pair verification.
	fn root(&self) -> &CommitmentRoot;

	/// Returns the timestamp of the state.
	fn timestamp(&self) -> Timestamp;

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

/// Query request for a single client event, identified by `event_id`, for `client_id`.
#[derive(Clone, Debug)]
pub struct QueryClientEventRequest {
	pub height: crate::Height,
	pub event_id: WithBlockDataType,
	pub client_id: ClientId,
	pub consensus_height: crate::Height,
}
