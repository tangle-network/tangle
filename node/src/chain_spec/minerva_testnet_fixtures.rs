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
use nimbus_primitives::NimbusId;
use sc_network_common::config::MultiaddrWithPeerId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use tangle_rococo_runtime::{nimbus_session_adapter::VrfId, AccountId, ImOnlineId};

/// Testnet root key
pub fn get_testnet_root_key() -> AccountId {
	// Arana sudo key: 5F9jS22zsSzmWNXKt4kknBsrhVAokEQ9e3UcuBeg21hkzqWz
	hex!["888a3ab33eea2b827f15302cb26af0e007b067ccfbf693faff3aa7ffcfa25925"].into()
}

/// Arana alpha bootnodes
pub fn get_testnet_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/18.118.130.41/tcp/30333/p2p/12D3KooWKpvw51KPvNx86jv6jLnkmnpiZubeT77LZDFyDTs7NAGW"
			.parse()
			.unwrap(),
		"/ip4/18.119.156.80/tcp/30333/p2p/12D3KooWAca3kpZUpk5is9jSBjfz56CSq3azd8utRvtRE4DsDh3u"
			.parse()
			.unwrap(),
		"/ip4/18.191.138.127/tcp/30333/p2p/12D3KooWJLQLRhrff3E3vSCjGDoNNUzBk1HvdXGCgs2gayjcqWTY"
			.parse()
			.unwrap(),
	]
}

/// Arana initial authorities
pub fn get_testnet_initial_authorities(
) -> Vec<(AccountId, AuraId, DKGId, NimbusId, VrfId, ImOnlineId)> {
	vec![
		(
			// AccountId
			hex!["66f07ce0432d73995e3c37afb65aed10d72c872400282d87e23c7cbbf7be5a4e"].into(),
			// AuraId
			hex!["b01fc4d7a9f4af40f92329e5fb6e26cda9cb279058d4e21db4234f61fd4e3667"]
				.unchecked_into(),
			// DKGId
			hex!["03ca5523f8d8193e7689afaef8f3eda11f489c3e2bfbeed662bb1e9fb42a426720"]
				.unchecked_into(),
			// NimbusId
			hex!["983a5bf631b54f6dadb3628045294ee25fd43d570f8a036f05157019480d4f51"]
				.unchecked_into(),
			// VrfId
			hex!["724b4a909df9a749df3ecb8fe9fddafd0335db73113602cc5d55286ed800422a"]
				.unchecked_into(),
			// ImOnlineId
			hex!["7ee4ef61354cf5e4eccb575999d7ec610e0bd96b5c50b92b51db21e764fee45a"]
				.unchecked_into(),
			// relayer address
			// 0xb6806626f5e4490c27a4ccffed4fed513539b6a455b14b32f58878cf7c5c4e68
		),
		(
			// AccountId
			hex!["0cffebaeb8ba50c523ec6a8ed518d534c1e27cd6f692d4d28618e3256a880412"].into(),
			// AuraId
			hex!["e2b50d14718d578abdbf3ea05498faf6caca79f426556bc10c49a946adc2da24"]
				.unchecked_into(),
			// DKGId
			hex!["02d4df6225d731c367c788c64329d8c2cf0a4d675cf197025cb7be5e6ef32ffce1"]
				.unchecked_into(),
			// NimbusId
			hex!["1a2fdfdbf34f9bbdc4733637ac779c64af401d0f6d65d5c1229392dd22c2a15c"]
				.unchecked_into(),
			// VrfId
			hex!["1a8a5938503159bf9ef7ffd52c81b4c4b31d539f51ffb46104f30236b16cf855"]
				.unchecked_into(),
			// ImOnlineId
			hex!["b62ff60a1d245b0b87427b73db4fa6d057837a43496289274fa89a3028ecba12"]
				.unchecked_into(),
			// relayer address
			// 0x22203dbd79c7ef6ce6bd7ec9b1f4d87425b1db0ab827543d3c7ce3f6a0749005
		),
		(
			// AccountId
			hex!["3c845c875a53061c8efbe6b149966a105f95097b49280256f65fd994686ed341"].into(),
			// AuraId
			hex!["3e57249065e9f8b10ef781bc33d7664d1c774b1a4113bd28219fd0e85fb0b300"]
				.unchecked_into(),
			// DKGId
			hex!["02e09de80f861c948a89b3f5b03b6bb681b19937a70ef5e51c63443d1d382bef8e"]
				.unchecked_into(),
			// NimbusId
			hex!["b4dfce49493ec45e49b8f38182f21ce859689692b38566e560266cde38554e06"]
				.unchecked_into(),
			// VrfId
			hex!["d0f1a1b9236b82f58da8818207c0e804c2f4a081b5e4ec0512693252a7161713"]
				.unchecked_into(),
			// ImOnlineId
			hex!["fa79d17a705241ae4dbacd766c2e8c0e7b49ac55b94737d4cefbbfd7d137d47f"]
				.unchecked_into(),
			// relayer address
			// 0x6a682aa89827a4028c9f1c2612fb1caa63957a892c7b05659b4e4f46b669b10d
		),
		(
			// AccountId
			hex!["a80afbb2600998b2858e011a1a74e9aa92d8b8edc31ec54253c43d7eafef0675"].into(),
			// AuraId
			hex!["58ccb55e6bbb006b6c038e9c8a3ae44207557513b60cc23a9bbc7d4a43aee66a"]
				.unchecked_into(),
			// DKGId
			hex!["027626be18e28dc3122755b3361d48dc7d934f069cef37fc57cc7d0e15d7bf4eaa"]
				.unchecked_into(),
			// NimbusId
			hex!["1c026b74c3369aef561313f6f1468257e088685f98d870d7fb0fe5168653da42"]
				.unchecked_into(),
			// VrfId
			hex!["1cde385be609e795347e7f7b5345bfa1e597fca219ef17e6eaf303b7d6912a63"]
				.unchecked_into(),
			// ImOnlineId
			hex!["bcd6aa871b888808360bf51b84e7d90a0d0ef859a9a7a96576246d0f8f913607"]
				.unchecked_into(),
			// relayer address
			// 0x6abe9075d17ca10075c1f8c11169009334f567e12047c80712fdc499cad8e026
		),
		(
			// AccountId
			hex!["3874c16c9855de4791f363d5779dab4cd8e71f21b62494288344002e3a031265"].into(),
			// AuraId
			hex!["406c44535d620b5c029f8cf4f5754e9108c56c325d985858311346ebbad8fe05"]
				.unchecked_into(),
			// DKGId
			hex!["02607f79f4f15b54065d08b2b028708e8963bf95895d3efebea17c79d284e8b609"]
				.unchecked_into(),
			// NimbusId
			hex!["84db76f3f4ed202c7091ae77d9947e29580c396ae70d72b240ee6ff0e56a355a"]
				.unchecked_into(),
			// VrfId
			hex!["f2121bfd55136904893edffed25b68585a2754c5062c0d3a39db7963e66fa116"]
				.unchecked_into(),
			// ImOnlineId
			hex!["6c767199b70a637613a0ae26ee2a5475d8b745218c0a00367f6d3c6a4939354d"]
				.unchecked_into(),
			// relayer address
			// 0xd85cbc2e3242d5264a020cef8d577b4022e08fa3295423604d4cc2d12bfc906f
		),
	]
}
