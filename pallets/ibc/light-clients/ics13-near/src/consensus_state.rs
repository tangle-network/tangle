// Copyright (C) 2022 ComposableFi.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::error::Error;
use ibc::{
	core::{
		ics02_client::client_consensus::{self},
		ics23_commitment::commitment::CommitmentRoot,
	},
	timestamp::Timestamp,
};
use serde::Serialize;
use tendermint_proto::Protobuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConsensusState {
	commitment_root: CommitmentRoot,
}

impl client_consensus::ConsensusState for ConsensusState {
	type Error = Error;

	fn root(&self) -> &CommitmentRoot {
		&self.commitment_root
	}

	fn timestamp(&self) -> Timestamp {
		todo!()
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		todo!("Encode to vec")
	}
}

impl Protobuf<()> for ConsensusState {}

impl From<ConsensusState> for () {
	fn from(_: ConsensusState) -> Self {
		todo!()
	}
}

impl From<()> for ConsensusState {
	fn from(_: ()) -> Self {
		todo!()
	}
}
