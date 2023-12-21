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
//! Mainnet fixtures
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_network::config::MultiaddrWithPeerId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use tangle_testnet_runtime::AccountId;

/// Mainnet root key
pub fn get_root_key() -> AccountId {
	// Standalone sudo key: 5CDZpRSZ14TmXorHTsTeksY7223FzsaLXPbpTPBUV6NaZSr1
	hex!["06c225d97d596c57e620aba15e1a8a69c7b334ffdab175788c6553f7dd181a56"].into()
}

/// Mainnet bootnodes
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
) -> Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)> {
	vec![
		// standalone 1
		(
			hex!["d4c197403bae729fd0f219a0925c4f9274d432c7bfce5f94f2e0dae605dde407"].into(),
			hex!["6c99e8e4ae3fe7e3328d7e9d85eb98e86bdc6410695797349fa536ebb9bb0a4a"].into(),
			hex!["d8a00a2454cd7455c040e363e6e76f4abd9e4d3876253964d9f40a66ad79694b"]
				.unchecked_into(),
			hex!["5bcf983a969f8de7628b271a5bf523924856c3935b15eb3e03f20146ced2c57f"]
				.unchecked_into(),
			hex!["0297bc051d94b25787482e60fa4eba19f15af30fa244240ae4439d219ee00e78"]
				.unchecked_into(),
		),
		// standalone 2
		(
			hex!["48a705897c103ddeda7e38bdadb42dc4c429e1b542287dfb07a9837982e04d14"].into(),
			hex!["444dbfd0220eb1a993a7a2b9e1530aee1d17388ba3db34a0ee2b8ff971bfd073"].into(),
			hex!["f02ee9baa32c490cf06eabe3580a90be704618f04636b321ee599c8912392c7a"]
				.unchecked_into(),
			hex!["2d6ac10cde791863771847c035c36e13ad60e6129465e1aefad8f5fee8dff5c7"]
				.unchecked_into(),
			hex!["de6d5010678fd2175fe70c857d3eba80838e3735b1051f4aed98671800ec483f"]
				.unchecked_into(),
		),
		// standalone 3
		(
			hex!["e2629eedccc6887f78d62d4ed15becd1b791ba0c38a5c72ccd416367097d7c3c"].into(),
			hex!["2c4e648b0fbbb88ff6b92b208273eb144383b2b19edc992e91448a4371d4d97d"].into(),
			hex!["a41b35f75e5509ce96e62bc27bb9a1b5587cc3d596f8afa867962b0e03230513"]
				.unchecked_into(),
			hex!["340e5969c8dd40ff77184fa73fbdcda77dcc90dd9b68b8b28eef5f01ce42339c"]
				.unchecked_into(),
			hex!["8472336050c4e4a51ac69865a4a31c6dd0e5c2f79555d8646cafb3bd8f12d75c"]
				.unchecked_into(),
		),
		// standalone 4
		(
			hex!["3281b9311756ee35e8bd53bc05e38af78ea4211c72db0ffcd8dd317785fa1327"].into(),
			hex!["c884c8eb280327221a3ae6a45fe6c8805f09bcfc11b409c8e2daa621c0d99608"].into(),
			hex!["06e0a0d39503a101ca9c36f84b3ccf53015ee625a546bc570e550af963d13164"]
				.unchecked_into(),
			hex!["57eda010788108257f4c148cf0c3112d620b9067546777dc393a65dd34732079"]
				.unchecked_into(),
			hex!["029cb182ddb9c5560aaa299f7b445e575007b8295bd85a80a7b2eb7baa3e2b7c"]
				.unchecked_into(),
		),
		// standalone 5
		(
			hex!["34d06ae4117b82a936b81d5219a438fa7b4093a6b67ebb0899686fb4e3b79b55"].into(),
			hex!["483e0b8d6801c51115fd4b124c91e2d5dcd642b30335f6c5a1738ea18f66c251"].into(),
			hex!["ce80df4851003f6ffd4ee88d9be85966f1de8b2e494c009dbf336177485f023f"]
				.unchecked_into(),
			hex!["9027284e6cad3f73eee950695c56f87330311331139616640c9168934dba82df"]
				.unchecked_into(),
			hex!["1ea007d87f91f96c31b1062548eb77c40d47a43f1c84c36caa8586fc7c359729"]
				.unchecked_into(),
		),
	]
}
