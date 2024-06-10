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
pub fn get_initial_authorities() -> Vec<(AccountId, BabeId, GrandpaId, ImOnlineId)> {
	vec![
		// tangle 1
		(
			hex!["f65b33a645a71be5a0953c53855ce3f8e882167d85910dc1e2cc1b9d32abe56d"].into(),
			hex!["eaea45fd67d4211b3b20706452c58850f7415c7b821c5617b42fb3373b822645"]
				.unchecked_into(),
			hex!["5b120ba5d2399de8ba3201d1c464973070934c570c539ddcad1318832353f7b0"]
				.unchecked_into(),
			hex!["1261548c9de1476e4e8e06e9dddd543ebbd7504ae502ac91a0eb8b6037334372"]
				.unchecked_into(),
		),
		// tangle 2
		(
			hex!["72db776eda47a3962a4dc987801e4762cabe38dfc7f639bd76823e09ec37e422"].into(),
			hex!["0a006e5c139ac2a46ed260a41db9bca6edbb937befe83cae167c68902d192d26"]
				.unchecked_into(),
			hex!["63e68e70b16f553476b879b5dba4b9ff587cd2a4a449df9c97698d56ea067b18"]
				.unchecked_into(),
			hex!["7c04e33631ff7ae0a453827c82c4b69179e72612f982c4d9b45393f25b4f5f48"]
				.unchecked_into(),
		),
		// tangle 3
		(
			hex!["a21d680b5daed8ca06772666e4245340d2fb6d363f7edab5cf78baa1456a5650"].into(),
			hex!["c851a63384fa6d5f7708a233e7de8c4bc81c07cacd0a8f329cb294d47aa64b78"]
				.unchecked_into(),
			hex!["a2948fb6f455ac9252ff39c59d46203dded2b0a2685e71a641942d58afc372a5"]
				.unchecked_into(),
			hex!["7842e1623d6a19cd267ad9b926ddb81a27c6aedc4cab58e99cd0ac3e2a53a762"]
				.unchecked_into(),
		),
		// tangle 4
		(
			hex!["3079d12fd7978cee5ff87fad722e3ef2156dea0daaaa1c6f73c28b69cf9a9201"].into(),
			hex!["000a4209bd78a7c1ff2eb8093b095be0d2776c3d5d6392ef4edb3d05f41d7e2e"]
				.unchecked_into(),
			hex!["7363e73123e3fd7600d136652ac763751baf65589a4c5cd6e45f49934f32058f"]
				.unchecked_into(),
			hex!["c0fc962bcbc623e343d0dddf9915f9c269dced366283e76e6302612d5172281d"]
				.unchecked_into(),
		),
		// tangle 5
		(
			hex!["22e254b686c1f38823d9d4012c75b9afe44a51ed802485f189d8df2f99053472"].into(),
			hex!["30fe2a18dbd35278c804267c61e00bebdc07d912b900d3af5b6df3bb6a35ed77"]
				.unchecked_into(),
			hex!["dcaef0b1c2e2d8fdeac4f07d60b3eb0d93564dba0c2e6bc5d0a63d775a4318da"]
				.unchecked_into(),
			hex!["cebebb5bf58a2a989bf8827d7360f04057835e42141162bfa17f3d12dde96a59"]
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
		),
		// trident
		(
			hex!["76aea5c2b3054df24605401596995d4425f6f82d842125f1db2e4ceeeade5934"].into(),
			hex!["82e0be1db35aa2bea192a24d86e296b50eaa0c1ee6b9bbc15332684df883277e"]
				.unchecked_into(),
			hex!["d9ceb6f89ac6fee1bb709e3e979a8d0da5e43a68debdc5fcb88a9e11705a6ae7"]
				.unchecked_into(),
			hex!["7c623a93a6691ecc211ea709f8eb606e2c9af5ac413b2cab88e54c2d4391872a"]
				.unchecked_into(),
		),
	]
}
