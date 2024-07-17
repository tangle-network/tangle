#![cfg_attr(not(feature = "std"), no_std)]
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
#![allow(clippy::all)]

//! ICS 07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.

#[macro_use]
#[cfg(any(test, feature = "mocks"))]
extern crate serde;
#[cfg(any(test, feature = "mocks"))]
#[macro_use]
extern crate ibc_derive;
extern crate alloc;

use core::fmt::Debug;
use tendermint::{
	crypto::{signature::Verifier, Sha256},
	merkle::MerkleHash,
};
use tendermint_light_client_verifier::{
	operations::{ProdCommitValidator, ProvidedVotingPowerCalculator},
	predicates::VerificationPredicates,
	PredicateVerifier,
};

pub mod client_def;
pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod merkle;
#[cfg(any(test, feature = "mocks"))]
pub mod mock;
#[cfg(any(test, feature = "mocks"))]
mod query;

/// Host functions that allow the light client verify cryptographic proofs in native.
pub trait HostFunctionsProvider:
	ics23::HostFunctionsProvider
	+ Sha256
	+ MerkleHash
	+ Verifier
	+ Debug
	+ Clone
	+ Send
	+ Sync
	+ Default
	+ Eq
{
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProdPredicates<H: HostFunctionsProvider>(core::marker::PhantomData<H>);

impl<H: HostFunctionsProvider> VerificationPredicates for ProdPredicates<H> {
	type Sha256 = H;
}

pub type ProdVotingPowerCalculator<H> = ProvidedVotingPowerCalculator<H>;

pub type ProdVerifier<H> =
	PredicateVerifier<ProdPredicates<H>, ProdVotingPowerCalculator<H>, ProdCommitValidator>;

#[cfg(test)]
mod tests {
	use crate::client_state::{
		test_util::get_dummy_tendermint_client_state, ClientState as TendermintClientState,
		ClientState,
	};

	use crate::{
		client_message::test_util::{get_dummy_ics07_header, get_dummy_tendermint_header},
		mock::{AnyClientState, AnyConsensusState, MockClientTypes},
	};

	use crate::{client_message::ClientMessage, mock::AnyClientMessage};
	use ibc::{
		core::{
			ics02_client::{
				context::ClientReader,
				handler::{dispatch, ClientResult},
				msgs::{
					create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient, ClientMsg,
				},
				trust_threshold::TrustThreshold,
			},
			ics23_commitment::specs::ProofSpecs,
			ics24_host::identifier::ClientId,
		},
		events::IbcEvent,
		handler::HandlerOutput,
		mock::context::MockContext,
		prelude::*,
		test_utils::get_dummy_account_id,
		Height,
	};
	use ibc_proto::ibc::core::client::v1::{MsgCreateClient, MsgUpdateClient};
	use std::time::Duration;
	use test_log::test;

	#[test]
	fn msg_create_client_serialization() {
		let signer = get_dummy_account_id();

		let tm_header = get_dummy_tendermint_header();
		let tm_client_state = get_dummy_tendermint_client_state(tm_header.clone());

		let msg = MsgCreateAnyClient::<MockContext<MockClientTypes>>::new(
			tm_client_state,
			AnyConsensusState::Tendermint(tm_header.try_into().unwrap()),
			signer,
		)
		.unwrap();

		let raw = MsgCreateClient::from(msg.clone());
		let msg_back =
			MsgCreateAnyClient::<MockContext<MockClientTypes>>::try_from(raw.clone()).unwrap();
		let raw_back = MsgCreateClient::from(msg_back.clone());
		assert_eq!(msg, msg_back);
		assert_eq!(raw, raw_back);
	}

	#[test]
	fn test_tm_create_client_ok() {
		let signer = get_dummy_account_id();

		let ctx = MockContext::default();

		let tm_header = get_dummy_tendermint_header();

		let tm_client_state = AnyClientState::Tendermint(
			TendermintClientState::new(
				tm_header.chain_id.clone().into(),
				TrustThreshold::ONE_THIRD,
				Duration::from_secs(64000),
				Duration::from_secs(128000),
				Duration::from_millis(3000),
				Height::new(0, u64::from(tm_header.height)),
				ProofSpecs::default(),
				vec!["".to_string()],
			)
			.unwrap(),
		);

		let msg = MsgCreateAnyClient::<MockContext<MockClientTypes>>::new(
			tm_client_state,
			AnyConsensusState::Tendermint(tm_header.try_into().unwrap()),
			signer,
		)
		.unwrap();

		let output = dispatch(&ctx, ClientMsg::CreateClient(msg.clone()));

		match output {
			Ok(HandlerOutput { result, mut events, .. }) => {
				assert_eq!(events.len(), 1);
				let event = events.pop().unwrap();
				let expected_client_id =
					ClientId::new(&ClientState::<()>::client_type(), 0).unwrap();
				assert!(
					matches!(event, IbcEvent::CreateClient(ref e) if e.client_id() == &expected_client_id)
				);
				assert_eq!(event.height(), ctx.host_height());
				match result {
					ClientResult::Create(create_res) => {
						assert_eq!(create_res.client_type, ClientState::<()>::client_type());
						assert_eq!(create_res.client_id, expected_client_id);
						assert_eq!(create_res.client_state, msg.client_state);
						assert_eq!(create_res.consensus_state, msg.consensus_state);
					},
					_ => {
						panic!("expected result of type ClientResult::CreateResult");
					},
				}
			},
			Err(err) => {
				panic!("unexpected error: {}", err);
			},
		}
	}

	#[test]
	fn msg_update_client_serialization() {
		let client_id: ClientId = "tendermint".parse().unwrap();
		let signer = get_dummy_account_id();

		let header = get_dummy_ics07_header();

		let msg = MsgUpdateAnyClient::<MockContext<MockClientTypes>>::new(
			client_id,
			AnyClientMessage::Tendermint(ClientMessage::Header(header)),
			signer,
		);
		let raw = MsgUpdateClient::from(msg.clone());
		let msg_back = MsgUpdateAnyClient::try_from(raw.clone()).unwrap();
		let raw_back = MsgUpdateClient::from(msg_back.clone());
		assert_eq!(msg, msg_back);
		assert_eq!(raw, raw_back);
	}
}
