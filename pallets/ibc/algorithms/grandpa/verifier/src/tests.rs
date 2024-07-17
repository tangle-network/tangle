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

use grandpa_light_client_primitives::justification::GrandpaJustification;
use polkadot_core_primitives::Header;
use serde::{Deserialize, Serialize};

pub type Justification = GrandpaJustification<Header>;

/// An encoded justification proving that the given header has been finalized
#[derive(Clone, Serialize, Deserialize)]
pub struct JustificationNotification(sp_core::Bytes);

// #[tokio::test]
// async fn follow_grandpa_justifications() {
// 	env_logger::builder()
// 		.filter_module("grandpa", log::LevelFilter::Trace)
// 		.format_module_path(false)
// 		.init();

// 	let relay = std::env::var("RELAY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
// 	let para = std::env::var("PARA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

// 	let relay_ws_url = format!("ws://{relay}:9944");
// 	let para_ws_url = format!("ws://{para}:9188");

// 	let prover = GrandpaProver::<PolkadotConfig>::new(
// 		&relay_ws_url,
// 		&para_ws_url,
// 		2000,
// 		Duration::from_millis(100),
// 	)
// 	.await
// 	.unwrap();

// 	println!("Waiting for grandpa proofs to become available");
// 	let session_length = prover.session_length().await.unwrap();
// 	prover
// 		.relay_client
// 		.blocks()
// 		.subscribe_finalized()
// 		.await
// 		.unwrap()
// 		.filter_map(|result| futures::future::ready(result.ok()))
// 		.skip_while(|h| futures::future::ready(h.number() < (session_length * 2) + 10))
// 		.take(1)
// 		.collect::<Vec<_>>()
// 		.await;

// 	let mut subscription = prover
// 		.relay_client
// 		.subscribe::<JustificationNotification>(
// 			"grandpa_subscribeJustifications",
// 			rpc_params![],
// 			"grandpa_unsubscribeJustifications",
// 		)
// 		.await
// 		.unwrap()
// 		.take((2 * session_length).try_into().unwrap());

// 	let mut client_state = prover.initialize_client_state().await.unwrap();
// 	println!("Grandpa proofs are now available");
// 	while let Some(Ok(JustificationNotification(sp_core::Bytes(_)))) = subscription.next().await {
// 		let next_relay_height = client_state.latest_relay_height + 1;

// 		let encoded = sc_consensus_grandpa_rpc::GrandpaApiClient::<
// 			JustificationNotification,
// 			H256,
// 			u32,
// 		>::prove_finality(&*prover.relay_ws_client.clone(), next_relay_height)
// 		.await
// 		.unwrap()
// 		.unwrap()
// 		.0;

// 		let finality_proof = FinalityProof::<Header>::decode(&mut &encoded[..]).unwrap();

// 		let justification = Justification::decode(&mut &finality_proof.justification[..]).unwrap();

// 		let finalized_para_header = prover
// 			.query_latest_finalized_parachain_header(justification.commit.target_number)
// 			.await
// 			.expect("Failed to fetch finalized parachain headers");

// 		// notice the inclusive range
// 		let header_numbers = ((client_state.latest_para_height + 1)..=finalized_para_header.number)
// 			.collect::<Vec<_>>();

// 		if header_numbers.len() == 0 {
// 			continue
// 		}

// 		println!("current_set_id: {}", client_state.current_set_id);
// 		println!("latest_relay_height: {}", client_state.latest_relay_height);
// 		println!(
// 			"For relay chain header: Hash({:?}), Number({})",
// 			justification.commit.target_hash, justification.commit.target_number
// 		);

// 		let proof = prover
// 			.query_finalized_parachain_headers_with_proof::<SubstrateHeader<u32, BlakeTwo256>>(
// 				client_state.latest_relay_height,
// 				justification.commit.target_number,
// 				Some(justification.encode()),
// 				header_numbers,
// 			)
// 			.await
// 			.expect("Failed to fetch finalized parachain headers with proof");

// 		let proof = proof.encode();
// 		let proof = ParachainHeadersWithFinalityProof::<Header>::decode(&mut &*proof).unwrap();

// 		let new_client_state = verify_parachain_headers_with_grandpa_finality_proof::<
// 			Header,
// 			HostFunctionsProvider,
// 		>(client_state.clone(), proof.clone())
// 		.expect("Failed to verify parachain headers with grandpa finality_proof");

// 		if !proof.parachain_headers.is_empty() {
// 			assert!(new_client_state.latest_para_height > client_state.latest_para_height);
// 		}

// 		client_state = new_client_state;
// 		println!("========= Successfully verified grandpa justification =========");
// 	}
// }
