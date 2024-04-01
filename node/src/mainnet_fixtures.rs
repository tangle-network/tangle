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
//! Mainnet fixtures
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_network::config::MultiaddrWithPeerId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use tangle_crypto_primitives::crypto::AuthorityId as RoleKeyId;
use tangle_runtime::AccountId;

/// Mainnet root key
pub fn get_root_key() -> AccountId {
	hex!["dc15b770b3cab4c43ab92173516482891c5f8ef4d42967f364bd2c52e0aebc0f"].into()
}

/// Mainnet bootnodes
pub fn get_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/213.199.45.84/tcp/30333/ws/p2p/12D3KooWHtMDPXL4NJzdVBDKQmUrLBDqDFjLwXm6xnTMgcmnfgTz"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.83/tcp/30333/ws/p2p/12D3KooWSxrtFrdT4wQQqafGYGaS5Z6UfGiwpbHs2Gsqt9Pc1i7L"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.82/tcp/30333/ws/p2p/12D3KooWExCWbJHKDdX6NSMwXRQK4AtwnCwdYMPTzBfqCYmRkuqH"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.81/tcp/30333/ws/p2p/12D3KooWRhiprU2Nck47vtSpYs1K9718WyDYx8UQKXWqo9zFccFu"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.80/tcp/30333/ws/p2p/12D3KooWSc5zpr9cuFqfXqCFS8G4FVSEELHwrb5uP2mCdrsW9XTC"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.79/tcp/30333/ws/p2p/12D3KooWGwU6web5sEVKzSFgFroJFbmuTcB42PEAHx6QPm86Cg9z"
			.parse()
			.unwrap(),
	]
}

/// Tangle runtime initial authorities
pub fn get_initial_authorities() -> Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, RoleKeyId)> {
	vec![
		// tangle 1
		(
			hex!["e4ee912e81516a503285ae59c0990ba5ef93941b3c185b456b84e7bc42c91013"].into(),
			hex!["fc7094e63e424d4ca53438646b86cd402c3dbe6a958da834d1933715afe05e31"]
				.unchecked_into(),
			hex!["e2442a31fed9cc5cda6b70e15959cad5a9fbfb14e8784233374deabb05b6fa11"]
				.unchecked_into(),
			hex!["be408915b6cf6a5843cef444bbf66609125b251f0ff9d06d059bcc43a631737f"]
				.unchecked_into(),
			hex!["0208cca30a428925678d695ba02d8968e8a9f6798615a7a0093f8a4c2cd1b5a9fc"]
				.unchecked_into(),
		),
		// tangle 2
		(
			hex!["c083df88dca627b873c972875d3570677ddc8ef7582041ca7c8c5bf6f04b1155"].into(),
			hex!["ee9da837e817f7d87eab1df000f92257811a21b895d09653b4bcd44f7e08ff7e"]
				.unchecked_into(),
			hex!["2f9ea304466841d597eb02f2bdd4dc9f1d6c8a862f0e345220c01a1b2670b51c"]
				.unchecked_into(),
			hex!["401d823b728892f280ca65daefdcadb37cdc6e42c04be780a6bedde80906df6d"]
				.unchecked_into(),
			hex!["0301760025504e5f9bec992e1ef79c83c126b372658d676c2c9f51be4bca55c859"]
				.unchecked_into(),
		),
		// tangle 3
		(
			hex!["ec012c87ebfac36ab4a56fb19d7ef5561785fc5bc9a1fa380185ae6440f0196f"].into(),
			hex!["66621351db6c9ddd0836df712e8838ad03c11d1703401f5f4b4d71a5c14d9b1e"]
				.unchecked_into(),
			hex!["6386c777558cfe531ca9bb86acc3479ebe8e301ec187f9ea2293a3c1ed309394"]
				.unchecked_into(),
			hex!["de11737a302be3afcae7492c381b5aaaa4ca9449cba9042ffe9cada7b7b37e21"]
				.unchecked_into(),
			hex!["03c7f43daff671e51b56c51b0029903f41c7a4b149b43dc5115c7dbfeba5e9ac3c"]
				.unchecked_into(),
		),
		// tangle 4
		(
			hex!["dc6d32ff27a13a3be0c6cebe857c31e17ab8a12da6aecea79541b30a909c8539"].into(),
			hex!["a085f963e31e1fd9592a8e3b1106cc7ea5571df53e2bbedc303d244274d24b09"]
				.unchecked_into(),
			hex!["4d416faaa28fd1d32744e00e40aa783dd30779e7e49f9fc7026923103b460895"]
				.unchecked_into(),
			hex!["48bccc462321ccf5806ad45d35755e9ac3e76813ad64c5f15e3f15a5b25f1437"]
				.unchecked_into(),
			hex!["02150142402731bc289cd6edd27536f7fb419e312f25e2c8d0f67c5e76f2a2a9fd"]
				.unchecked_into(),
		),
		// tangle 5
		(
			hex!["5effe50292a6a8932efde21623ea0a8c2acaffe85cf30827907dba715c04137f"].into(),
			hex!["fe77de52662d43eafc069b81538e57bbecc266b21f783ceba9302df95c2b6453"]
				.unchecked_into(),
			hex!["5922a3e86519cb61ab1e4eae6d4feee5c9bee4007a002c1cb4d4f99b69eedd63"]
				.unchecked_into(),
			hex!["4482f8fa27f9c9ec566554d2cf6ae3f7430da66fdeb486074a80237f6a7ca55c"]
				.unchecked_into(),
			hex!["028b4b49f4ab47aaf8583b98c800779397234cb23810856eeca151cc39c7e4637a"]
				.unchecked_into(),
		),
		// snowflake
		(
			hex!["34a02cae42b4455427d5ef200c6755b860c8d83bb5a923e2a4e9d9d9c051b678"].into(),
			hex!["b468a293b5696b48e3b4b1661f1d5a80447f578e7786785a54791be484376638"]
				.unchecked_into(),
			hex!["93bbed0d6a788adaac3a9f8c47264897e3d1b57fac3e58bca9764908f41f0c68"]
				.unchecked_into(),
			hex!["12fd766a9916d23c125c8b2f61b3fb9d68ecaa0dea2c4119aa5d1980dbe0c442"]
				.unchecked_into(),
			hex!["03ace29a9b62f4aee605b74a7b44624b9d96570cd70aa5d4d6f771d5824889038b"]
				.unchecked_into(),
		),
	]
}
