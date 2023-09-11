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
//! Testnet fixtures
use dkg_runtime_primitives::crypto::AuthorityId as DKGId;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_network::config::MultiaddrWithPeerId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use tangle_runtime::AccountId;

/// Testnet root key
pub fn get_testnet_root_key() -> AccountId {
	// Standalone sudo key: 5CDZpRSZ14TmXorHTsTeksY7223FzsaLXPbpTPBUV6NaZSr1
	hex!["06c225d97d596c57e620aba15e1a8a69c7b334ffdab175788c6553f7dd181a56"].into()
}

/// Standalone alpha bootnodes
pub fn get_standalone_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/3.22.222.30/tcp/30333/p2p/12D3KooWRdvZ3PRteq8DC78Z3z5ZiehipKrKhHDRpgvCjc8XSeQx"
			.parse()
			.unwrap(),
		"/ip4/18.119.14.21/tcp/30333/p2p/12D3KooWJP5NbEjEK1YihofJm3MMSJWrbRWjeEkRf3LtKvkj6mr9"
			.parse()
			.unwrap(),
		"/ip4/18.188.183.185/tcp/30333/p2p/12D3KooWDL3KiR6CpnEbgUgheje1cMGQtwH4euxGMPQBkwU5cZdu"
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

/// Standalone initial authorities
pub fn get_standalone_initial_authorities(
) -> Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId, DKGId)> {
	vec![
		// standalone 1
		(
			hex!["4e85271af1330e5e9384bd3ac5bdc04c0f8ef5a8cc29c1a8ae483d674164745c"].into(),
			hex!["804808fb75d16340dc250871138a1a6f1dfa3cab9cc1fbd6f42960f1c39a950d"].into(),
			hex!["16be9647f91aa5441e300acb8f0d6ccc63e72850202a7947df6a646c1bb4071a"]
				.unchecked_into(),
			hex!["71bf01524c555f1e0f6b7dc7243caf00851d3afc543422f98d3eb6bca78acd8c"]
				.unchecked_into(),
			hex!["16be9647f91aa5441e300acb8f0d6ccc63e72850202a7947df6a646c1bb4071a"]
				.unchecked_into(),
			hex!["028a4c0781f8369fdd873f8531491f24e2e806fd11a13d828cb4099e6c1045103e"]
				.unchecked_into(),
		),
		// standalone 2
		(
			hex!["587c2ef00ec0a1b98af4c655763acd76ece690fccbb255f01663660bc274960d"].into(),
			hex!["cc195602a63bbdcf2ef4773c86fdbfefe042cb9aa8e3059d02e59a062d9c3138"].into(),
			hex!["f4e206607ffffcd389c4c60523de5dda5a411d1435f8540b6b6bc181553bd65a"]
				.unchecked_into(),
			hex!["61f771ebfdb0a6de08b8e0ca7a39a01f24e7eaa3d1e7f1001e6503490c25c044"]
				.unchecked_into(),
			hex!["f4e206607ffffcd389c4c60523de5dda5a411d1435f8540b6b6bc181553bd65a"]
				.unchecked_into(),
			hex!["02427a6cf7f1d7538d9e3e4df834e27db337fd6ef0f530aab4e9799ff865e843fc"]
				.unchecked_into(),
		),
		// standalone 3
		(
			hex!["368ea402dbd9c9888ae999d6a799cf36e08673ee53c001dfb4529c149fc2c13b"].into(),
			hex!["a24f729f085de51eebaeaeca97d6d499761b8f6daeca9b99d754a06ef8bcec3f"].into(),
			hex!["8e92157e55a72fe0ee78c251a7553af341635bec0aafee1e4189cf8ce52cdd71"]
				.unchecked_into(),
			hex!["a41a815db90b9bd3d9ec462f90ba77ba1d627a9fccc9f7847e34c9e9e9b57c90"]
				.unchecked_into(),
			hex!["8e92157e55a72fe0ee78c251a7553af341635bec0aafee1e4189cf8ce52cdd71"]
				.unchecked_into(),
			hex!["036aec5853fba2662f31ba89e859ac100daa6c58dc8fdaf0555565663f2b99f8f2"]
				.unchecked_into(),
		),
		// standalone 4
		(
			hex!["2c7f3cc085da9175414d1a9d40aa3aa161c8584a9ca62a938684dfbe90ae9d74"].into(),
			hex!["0a55e5245382700f35d16a5ea6d60a56c36c435bef7204353b8c36871f347857"].into(),
			hex!["9a457869037b3e7643db0b71e7340d5f319ec5b53be0bffbc8280fe9a6d6bd68"]
				.unchecked_into(),
			hex!["b0f002333f4fd657155dfcb4ac5c6ce04d0b2c68b64befa178d4357ceb05fe2d"]
				.unchecked_into(),
			hex!["9a457869037b3e7643db0b71e7340d5f319ec5b53be0bffbc8280fe9a6d6bd68"]
				.unchecked_into(),
			hex!["0297579c2b3896c65bf556e710ba361d76bff80827e30d70bc8f1d39049005c509"]
				.unchecked_into(),
		),
		// standalone 5
		(
			hex!["e0948453e7acbc6ac937e124eb01580191e99f4262d588d4524994deb6134349"].into(),
			hex!["6c73e5ee9f8614e7c9f23fd8f7257d12e061e75fcbeb3b50ed70eb87ba91f500"].into(),
			hex!["4eddfb7cdb385617475a383929a3f129acad452d5789f27ca94373a1fa877b15"]
				.unchecked_into(),
			hex!["d2eb206f8c7a64ce47828b33314806ac6cb915d464990eaff9f6435880c6e54f"]
				.unchecked_into(),
			hex!["4eddfb7cdb385617475a383929a3f129acad452d5789f27ca94373a1fa877b15"]
				.unchecked_into(),
			hex!["020d672a9e42b74d47f6280f8a5ea04f11f8ef53d9bcfba8a7c652ad0131a4d2f3"]
				.unchecked_into(),
		),
	]
}
