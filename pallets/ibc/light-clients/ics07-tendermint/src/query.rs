#![allow(unused)]
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

use ibc::core::{
	ics02_client::client_consensus::QueryClientEventRequest,
	ics04_channel::channel::QueryPacketEventDataRequest,
};
use tendermint::Hash;

/// Used for queries and not yet standardized in channel's query.proto
#[derive(Clone, Debug)]
pub enum QueryTxRequest {
	Packet(QueryPacketEventDataRequest),
	Client(QueryClientEventRequest),
	Transaction(QueryTxHash),
}

#[derive(Clone, Debug)]
pub enum QueryBlockRequest {
	Packet(QueryPacketEventDataRequest),
}

#[derive(Clone, Debug)]
pub struct QueryTxHash(pub Hash);
