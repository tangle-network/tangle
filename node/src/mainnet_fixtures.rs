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
		"/ip4/213.199.45.84/tcp/30333/p2p/12D3KooWHtMDPXL4NJzdVBDKQmUrLBDqDFjLwXm6xnTMgcmnfgTz"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.83/tcp/30333/p2p/12D3KooWSxrtFrdT4wQQqafGYGaS5Z6UfGiwpbHs2Gsqt9Pc1i7L"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.82/tcp/30333/p2p/12D3KooWExCWbJHKDdX6NSMwXRQK4AtwnCwdYMPTzBfqCYmRkuqH"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.81/tcp/30333/p2p/12D3KooWRhiprU2Nck47vtSpYs1K9718WyDYx8UQKXWqo9zFccFu"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.80/tcp/30333/p2p/12D3KooWSc5zpr9cuFqfXqCFS8G4FVSEELHwrb5uP2mCdrsW9XTC"
			.parse()
			.unwrap(),
		"/ip4/213.199.45.79/tcp/30333/p2p/12D3KooWGwU6web5sEVKzSFgFroJFbmuTcB42PEAHx6QPm86Cg9z"
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
			hex!["22e254b686c1f38823d9d4012c75b9afe44a51ed802485f189d8df2f99053472"].into(),
			hex!["30fe2a18dbd35278c804267c61e00bebdc07d912b900d3af5b6df3bb6a35ed77"]
				.unchecked_into(),
			hex!["dcaef0b1c2e2d8fdeac4f07d60b3eb0d93564dba0c2e6bc5d0a63d775a4318da"]
				.unchecked_into(),
			hex!["cebebb5bf58a2a989bf8827d7360f04057835e42141162bfa17f3d12dde96a59"]
				.unchecked_into(),
			hex!["02596bcb7dfc93455d1e7bf93aa874d481590b37af0d43598506f9ed8609086ba0"]
				.unchecked_into(),
		),
		// tangle 3
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
		// tangle 4
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
		// tangle 5
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
		// // tangle 6
		// (
		// 	hex!["5effe50292a6a8932efde21623ea0a8c2acaffe85cf30827907dba715c04137f"].into(),
		// 	hex!["fe77de52662d43eafc069b81538e57bbecc266b21f783ceba9302df95c2b6453"]
		// 		.unchecked_into(),
		// 	hex!["5922a3e86519cb61ab1e4eae6d4feee5c9bee4007a002c1cb4d4f99b69eedd63"]
		// 		.unchecked_into(),
		// 	hex!["4482f8fa27f9c9ec566554d2cf6ae3f7430da66fdeb486074a80237f6a7ca55c"]
		// 		.unchecked_into(),
		// 	hex!["028b4b49f4ab47aaf8583b98c800779397234cb23810856eeca151cc39c7e4637a"]
		// 		.unchecked_into(),
		// ),
		// // tangle 7
		// (
		// 	hex!["f65b33a645a71be5a0953c53855ce3f8e882167d85910dc1e2cc1b9d32abe56d"].into(),
		// 	hex!["eaea45fd67d4211b3b20706452c58850f7415c7b821c5617b42fb3373b822645"]
		// 		.unchecked_into(),
		// 	hex!["5b120ba5d2399de8ba3201d1c464973070934c570c539ddcad1318832353f7b0"]
		// 		.unchecked_into(),
		// 	hex!["1261548c9de1476e4e8e06e9dddd543ebbd7504ae502ac91a0eb8b6037334372"]
		// 		.unchecked_into(),
		// 	hex!["02320ab70b215ed1d37748d2aadae924ddb1e6042598ae9c2479684f0913cc15ae"]
		// 		.unchecked_into(),
		// ),
		// // tangle 8
		// (
		// 	hex!["72db776eda47a3962a4dc987801e4762cabe38dfc7f639bd76823e09ec37e422"].into(),
		// 	hex!["0a006e5c139ac2a46ed260a41db9bca6edbb937befe83cae167c68902d192d26"]
		// 		.unchecked_into(),
		// 	hex!["63e68e70b16f553476b879b5dba4b9ff587cd2a4a449df9c97698d56ea067b18"]
		// 		.unchecked_into(),
		// 	hex!["7c04e33631ff7ae0a453827c82c4b69179e72612f982c4d9b45393f25b4f5f48"]
		// 		.unchecked_into(),
		// 	hex!["026fa903a87d7e0e262e5c70d6dffecca3f5262f05096a1c5bcd477e8ea63813ad"]
		// 		.unchecked_into(),
		// ),
		// // tangle 9
		// (
		// 	hex!["a21d680b5daed8ca06772666e4245340d2fb6d363f7edab5cf78baa1456a5650"].into(),
		// 	hex!["c851a63384fa6d5f7708a233e7de8c4bc81c07cacd0a8f329cb294d47aa64b78"]
		// 		.unchecked_into(),
		// 	hex!["a2948fb6f455ac9252ff39c59d46203dded2b0a2685e71a641942d58afc372a5"]
		// 		.unchecked_into(),
		// 	hex!["7842e1623d6a19cd267ad9b926ddb81a27c6aedc4cab58e99cd0ac3e2a53a762"]
		// 		.unchecked_into(),
		// 	hex!["02c876f99aa4361e7e81eb96ff325833452efb6eed6ae43abfbbed4106ac556829"]
		// 		.unchecked_into(),
		// ),
		// // tangle 10
		// (
		// 	hex!["3079d12fd7978cee5ff87fad722e3ef2156dea0daaaa1c6f73c28b69cf9a9201"].into(),
		// 	hex!["000a4209bd78a7c1ff2eb8093b095be0d2776c3d5d6392ef4edb3d05f41d7e2e"]
		// 		.unchecked_into(),
		// 	hex!["7363e73123e3fd7600d136652ac763751baf65589a4c5cd6e45f49934f32058f"]
		// 		.unchecked_into(),
		// 	hex!["c0fc962bcbc623e343d0dddf9915f9c269dced366283e76e6302612d5172281d"]
		// 		.unchecked_into(),
		// 	hex!["03d1198468f5ba4f67d155a576d5b2b2823265616b7669000eab08bca538d8c34c"]
		// 		.unchecked_into(),
		// ),
	]
}
