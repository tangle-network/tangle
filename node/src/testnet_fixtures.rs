// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
//! Testnet fixtures
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_network::config::MultiaddrWithPeerId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use tangle_testnet_runtime::AccountId;

/// Testnet root key
pub fn get_testnet_root_key() -> AccountId {
	hex!["9c2c928c3b9ac62c6dc4caaf5777c16ce28984dedc92687ef427f8f4f6d61d2f"].into()
}

/// Standalone alpha bootnodes
pub fn get_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/3.22.222.30/tcp/30333/p2p/12D3KooWRdvZ3PRteq8DC78Z3z5ZiehipKrKhHDRpgvCjc8XSeQx"
			.parse()
			.unwrap(),
		"/ip4/18.119.14.21/tcp/30333/p2p/12D3KooWJP5NbEjEK1YihofJm3MMSJWrbRWjeEkRf3LtKvkj6mr9"
			.parse()
			.unwrap(),
		"/ip4/3.15.186.160/tcp/30333/p2p/12D3KooWDL3KiR6CpnEbgUgheje1cMGQtwH4euxGMPQBkwU5cZdu"
			.parse()
			.unwrap(),
		"/ip4/3.137.213.159/tcp/30333/p2p/12D3KooWS4aniCJTz2RiNfNUka8TTa3gXak63FJgdAgfAWLCnsAi"
			.parse()
			.unwrap(),
		"/ip4/3.144.191.66/tcp/30333/p2p/12D3KooWM8RYTbVygshTJAbiM5YqvTwWPbZrF8iQ9WS96nEE2Ebr"
			.parse()
			.unwrap(),
	]
}

/// Tangle testnet authorities
pub fn get_initial_authorities() -> Vec<(AccountId, BabeId, GrandpaId, ImOnlineId)> {
	vec![
		// tangle-testnet 1
		(
			hex!["4e85271af1330e5e9384bd3ac5bdc04c0f8ef5a8cc29c1a8ae483d674164745c"].into(),
			hex!["82a764a9835b2ac7aacd8ec96479d872e91256fba11cc0393b0e98dd320bbf38"]
				.unchecked_into(),
			hex!["71bf01524c555f1e0f6b7dc7243caf00851d3afc543422f98d3eb6bca78acd8c"]
				.unchecked_into(),
			hex!["2c7d0dbc639d8d52d02e9e03b6dd19b50f44a6a491ca75c9ebd6a1a29782e743"]
				.unchecked_into(),
		),
		// tangle-testnet 2
		(
			hex!["587c2ef00ec0a1b98af4c655763acd76ece690fccbb255f01663660bc274960d"].into(),
			hex!["020af75377d0b400946938970c47055a2a48ad4fd728a427b7a1dd96e75db65f"]
				.unchecked_into(),
			hex!["61f771ebfdb0a6de08b8e0ca7a39a01f24e7eaa3d1e7f1001e6503490c25c044"]
				.unchecked_into(),
			hex!["ecb58b3d1eaaad8ef42c27e183e41830628ca5fee5bb06cdf13883e57c4a0770"]
				.unchecked_into(),
		),
		// tangle-testnet 3
		(
			hex!["a24f729f085de51eebaeaeca97d6d499761b8f6daeca9b99d754a06ef8bcec3f"].into(),
			hex!["a8f14e015ced0b21e680e577231519c33484d1b63f5529bf4fbb137a920bb82e"]
				.unchecked_into(),
			hex!["a41a815db90b9bd3d9ec462f90ba77ba1d627a9fccc9f7847e34c9e9e9b57c90"]
				.unchecked_into(),
			hex!["2ef4f718a407e0b4d86913a989f749b7edfe836bae1d726b07f9c419ad94c42c"]
				.unchecked_into(),
		),
		// tangle-testnet 4
		(
			hex!["0a55e5245382700f35d16a5ea6d60a56c36c435bef7204353b8c36871f347857"].into(),
			hex!["2296eb6f84cafd066c07e19fd127e7f5ca0d6e26ce305f9facc0e0221c7ecb1a"]
				.unchecked_into(),
			hex!["b0f002333f4fd657155dfcb4ac5c6ce04d0b2c68b64befa178d4357ceb05fe2d"]
				.unchecked_into(),
			hex!["dc8be4d69084d5bb8a6a0cd48c4d3ab82def85e996b87c63176b49f70c028c35"]
				.unchecked_into(),
		),
		// tangle-testnet 5
		(
			hex!["e0948453e7acbc6ac937e124eb01580191e99f4262d588d4524994deb6134349"].into(),
			hex!["0e08bb5a9cc4e7dbd279ba53bd5aecd78956c34a2dae678c5490c52bc754521e"]
				.unchecked_into(),
			hex!["d2eb206f8c7a64ce47828b33314806ac6cb915d464990eaff9f6435880c6e54f"]
				.unchecked_into(),
			hex!["363e33c500396cc52ef58924a005227eed67801bd376ca886fd6432ba7070711"]
				.unchecked_into(),
		),
	]
}
