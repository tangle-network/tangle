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

use crate::{
	client_message::{ClientMessage, Header, RelayChainHeader},
	client_state::ClientState,
	consensus_state::ConsensusState,
	mock::{
		AnyClientMessage, AnyClientState, AnyConsensusState, HostFunctionsManager, MockClientTypes,
	},
};
use beefy_prover::helpers::{fetch_timestamp_extrinsic_with_proof, TimeStampExtWithProof};
use futures::stream::StreamExt;
use grandpa_client_primitives::{
	justification::GrandpaJustification, parachain_header_storage_key, FinalityProof,
	ParachainHeaderProofs, ParachainHeadersWithFinalityProof,
};
use grandpa_prover::{GrandpaProver, JustificationNotification};
use ibc::{
	core::{
		ics02_client::{
			client_state::ClientState as _,
			context::{ClientKeeper, ClientReader},
			handler::{dispatch, ClientResult::Update},
			msgs::{
				create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient, ClientMsg,
			},
		},
		ics24_host::identifier::{ChainId, ClientId},
	},
	events::IbcEvent,
	handler::HandlerOutput,
	mock::{context::MockContext, host::MockHostType},
	test_utils::get_dummy_account_id,
	Height,
};
use light_client_common::config::RuntimeStorage;
use parity_scale_codec::{Decode, Encode};
use sc_consensus_grandpa_rpc::GrandpaApiClient;
use sp_core::{hexdisplay::AsBytesRef, H256};
use std::time::Duration;
use subxt::{
	backend::legacy::LegacyRpcMethods,
	config::substrate::{BlakeTwo256, SubstrateHeader},
	PolkadotConfig,
};

#[tokio::test]
async fn test_continuous_update_of_grandpa_client() {
	env_logger::builder()
		.filter_module("grandpa", log::LevelFilter::Trace)
		.format_module_path(false)
		.init();

	let client_id = ClientId::new(&ClientState::<HostFunctionsManager>::client_type(), 0).unwrap();
	let chain_start_height = Height::new(1, 11);
	let mut ctx = MockContext::<MockClientTypes>::new(
		ChainId::new("mockgaiaA".to_string(), 1),
		MockHostType::Mock,
		5,
		chain_start_height,
	);
	ctx.block_time = Duration::from_secs(600);

	let signer = get_dummy_account_id();

	let relay = std::env::var("RELAY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
	let para = std::env::var("PARA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

	let relay_ws_url = format!("ws://{relay}:9944");
	let para_ws_url = format!("ws://{para}:9188");

	let prover = GrandpaProver::<PolkadotConfig>::new(
		&relay_ws_url,
		&para_ws_url,
		2000,
		Duration::from_millis(100),
	)
	.await
	.unwrap();

	println!("Waiting for grandpa proofs to become available");
	let session_length = prover.session_length().await.unwrap();
	prover
		.relay_client
		.blocks()
		.subscribe_finalized()
		.await
		.unwrap()
		.filter_map(|result| futures::future::ready(result.ok()))
		.skip_while(|h| futures::future::ready(h.number() < (session_length * 2) + 10))
		.take(1)
		.collect::<Vec<_>>()
		.await;

	println!("Grandpa proofs are now available");

	let (client_state, consensus_state) = loop {
		let client_state = prover.initialize_client_state().await.unwrap();

		let latest_relay_header = prover
			.relay_client
			.header(Some(client_state.latest_relay_hash))
			.await
			.expect("Failed to fetch finalized header")
			.expect("Failed to fetch finalized header");

		let head_data = {
			let key = <<PolkadotConfig as light_client_common::config::Config>::Storage as RuntimeStorage>::paras_heads(
				prover.para_id,
			);
			prover
				.relay_client
				.storage()
				.at(client_state.latest_relay_hash)
				.fetch(&key)
				.await
				.unwrap()
				.unwrap()
		};
		let decoded_para_head = frame_support::sp_runtime::generic::Header::<
			u32,
			sp_runtime::traits::BlakeTwo256,
		>::decode(&mut &*head_data.0)
		.expect("Failed to decode parachain header");
		// we can't use the genesis block to construct the initial state.
		if decoded_para_head.number == 0 {
			continue;
		}
		let client_state = ClientState {
			relay_chain: Default::default(),
			latest_relay_hash: client_state.latest_relay_hash,
			latest_relay_height: latest_relay_header.number,
			frozen_height: None,
			latest_para_height: decoded_para_head.number,
			para_id: prover.para_id,
			current_set_id: client_state.current_set_id,
			current_authorities: client_state.current_authorities,
			_phantom: Default::default(),
		};
		let subxt_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
			decoded_para_head.number.into();
		let block_hash = LegacyRpcMethods::<T>::new(prover.para_rpc_client.clone())
			.chain_get_block_hash(Some(subxt_block_number))
			.await
			.unwrap();

		let TimeStampExtWithProof { ext: timestamp_extrinsic, proof: extrinsic_proof } =
			fetch_timestamp_extrinsic_with_proof(&prover.para_client, block_hash)
				.await
				.unwrap();
		let state_proof = prover
			.relay_client
			.read_proof(
				vec![parachain_header_storage_key(prover.para_id).0.as_bytes_ref()],
				Some(client_state.latest_relay_hash),
			)
			.await
			.expect("Failed to fetch state proof!")
			.proof
			.into_iter()
			.map(|bytes| bytes.0)
			.collect();

		let header_proof =
			ParachainHeaderProofs { state_proof, extrinsic: timestamp_extrinsic, extrinsic_proof };

		let (_, consensus_state) = ConsensusState::from_header::<HostFunctionsManager>(
			header_proof,
			prover.para_id,
			latest_relay_header.state_root,
		)
		.unwrap();

		break (AnyClientState::Grandpa(client_state), AnyConsensusState::Grandpa(consensus_state));
	};

	let create_client =
		MsgCreateAnyClient { client_state, consensus_state, signer: signer.clone() };

	// Create the client
	let res = dispatch(&ctx, ClientMsg::CreateClient(create_client)).unwrap();
	ctx.store_client_result(res.result).unwrap();
	let subscription =
		GrandpaApiClient::<JustificationNotification, H256, u32>::subscribe_justifications(
			&*prover.relay_ws_client.clone(),
		)
		.await
		.expect("Failed to subscribe to grandpa justifications");
	let mut subscription = subscription.take((2 * session_length).try_into().unwrap());

	while let Some(Ok(JustificationNotification(sp_core::Bytes(_)))) = subscription.next().await {
		let client_state: ClientState<HostFunctionsManager> =
			match ctx.client_state(&client_id).unwrap() {
				AnyClientState::Grandpa(client_state) => client_state,
				_ => panic!("unexpected client state"),
			};

		let next_relay_height = client_state.latest_relay_height + 1;

		let encoded = sc_consensus_grandpa_rpc::GrandpaApiClient::<
			JustificationNotification,
			H256,
			u32,
		>::prove_finality(
			// we cast between the same type but different crate versions.
			&*prover.relay_ws_client.clone(),
			next_relay_height,
		)
		.await
		.unwrap()
		.unwrap()
		.0;

		let finality_proof = FinalityProof::<RelayChainHeader>::decode(&mut &encoded[..]).unwrap();

		let justification = GrandpaJustification::<RelayChainHeader>::decode(
			&mut &finality_proof.justification[..],
		)
		.unwrap();

		let finalized_para_header = prover
			.query_latest_finalized_parachain_header(justification.commit.target_number)
			.await
			.expect("Failed to fetch finalized parachain headers");
		// notice the inclusive range
		let header_numbers = ((client_state.latest_para_height + 1)..=finalized_para_header.number)
			.collect::<Vec<_>>();

		println!("Client State: {:?}", client_state);
		println!("Finalized para header: {:?}", finalized_para_header.number);
		let proof = prover
			.query_finalized_parachain_headers_with_proof::<SubstrateHeader<u32, BlakeTwo256>>(
				client_state.latest_relay_height,
				justification.commit.target_number,
				Some(justification.encode()),
				header_numbers.clone(),
			)
			.await
			.expect("Failed to fetch finalized parachain headers with proof");
		let proof = proof.encode();
		let proof =
			ParachainHeadersWithFinalityProof::<RelayChainHeader>::decode(&mut &*proof).unwrap();
		println!("========= New Justification =========");
		println!("current_set_id: {}", client_state.current_set_id);
		println!(
			"For relay chain header: Hash({:?}), Number({})",
			justification.commit.target_hash, justification.commit.target_number
		);

		let header = Header {
			finality_proof: proof.finality_proof,
			parachain_headers: proof.parachain_headers.clone(),
			height: Height::new(prover.para_id as u64, finalized_para_header.number as u64),
		};
		let msg = MsgUpdateAnyClient {
			client_id: client_id.clone(),
			client_message: AnyClientMessage::Grandpa(ClientMessage::Header(header)),
			signer: signer.clone(),
		};

		// advance the chain
		ctx.advance_host_chain_height();
		let res = dispatch(&ctx, ClientMsg::UpdateClient(msg.clone()));

		match res {
			Ok(HandlerOutput { result, mut events, log }) => {
				assert_eq!(events.len(), 1);
				let event = events.pop().unwrap();
				assert!(
					matches!(event, IbcEvent::UpdateClient(ref e) if e.client_id() == &msg.client_id)
				);
				assert_eq!(event.height(), ctx.host_height());
				assert!(log.is_empty());
				ctx.store_client_result(result.clone()).unwrap();
				match result {
					Update(upd_res) => {
						assert_eq!(upd_res.client_id, client_id);
						assert!(!upd_res.client_state.is_frozen(&ctx, &client_id));
						assert_eq!(
							upd_res.client_state,
							ctx.latest_client_states(&client_id).clone()
						);
						for height in header_numbers {
							let cs = ctx
								.consensus_state(
									&client_id,
									Height::new(prover.para_id as u64, height as u64),
								)
								.ok();
							dbg!((height, cs.is_some()));
						}
						println!(
							"======== Successfully updated parachain client to height: {} ========",
							upd_res.client_state.latest_height(),
						);
					},
					_ => unreachable!("update handler result has incorrect type"),
				}
			},
			Err(e) => panic!("Unexpected error {:?}", e),
		}
	}
}
