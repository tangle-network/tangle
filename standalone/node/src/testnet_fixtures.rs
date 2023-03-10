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
use sc_network_common::config::MultiaddrWithPeerId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use tangle_runtime::AccountId;

/// Testnet root key
pub fn get_testnet_root_key() -> AccountId {
	// Standalone sudo key: 5CDZpRSZ14TmXorHTsTeksY7223FzsaLXPbpTPBUV6NaZSr1
	hex!["06c225d97d596c57e620aba15e1a8a69c7b334ffdab175788c6553f7dd181a56"].into()
}

/// Standalone alpha bootnodes
pub fn get_standalone_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/18.191.185.238/tcp/30333/p2p/12D3KooWPfrrLpP7rzHdkQ1bpPebq62TaUaEoA4qmGbyGUNySD8h"
			.parse()
			.unwrap(),
		"/ip4/3.16.148.122/tcp/30333/p2p/12D3KooWA1EpUAKGHsgJc4ZfZnXS9wHzESMDDq8T7WzeHzP7pcj8"
			.parse()
			.unwrap(),
		"/ip4/3.143.253.232/tcp/30333/p2p/12D3KooWLGKC7vivZiSw8k82ANH5yzYvWcLJRa2n1u3Ym3WMg3K9"
			.parse()
			.unwrap(),
	]
}

/// Standalone initial authorities
pub fn get_standalone_initial_authorities(
) -> Vec<(AccountId, AccountId, AuraId, GrandpaId, ImOnlineId, DKGId)> {
	vec![
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
		// standalone 6
		(
			hex!["541dc9dd9cd9b47ff19c77c3b14fab50ab0774e19abe438719cd09e4f4861166"].into(),
			hex!["607e948bad733780eda6c0bd9b084243276332823ca8481fc20cd01e1a2ef36f"].into(),
			hex!["c09ca0a13acb3bca703195c5c452d64f9ead47dec540f601e9db42bf97ec8025"]
				.unchecked_into(),
			hex!["bb9b1375992318bd2171bb22766129855864881cf79689440bf1da4f35964e06"]
				.unchecked_into(),
			hex!["c09ca0a13acb3bca703195c5c452d64f9ead47dec540f601e9db42bf97ec8025"]
				.unchecked_into(),
			hex!["0278f0bd16e3ff51540b044844fcc0dba9a374bc9824305ec4d6687dcf7058d220"]
				.unchecked_into(),
		),
	    // standalone 7
		(
			hex!["b2c09cb1b78c3afd2b1ea4316dfb1be9065e070db948477248e4f3e0f1a2d850"].into(),
			hex!["fc156f082d789f94149f8b52b191672fbf202ef1b92b487c3cec9bca2d1fbe72"].into(),
			hex!["c8243df041c48137e16b5152bad01ce99441b3f2907915d569fa3b58ad83495f"]
				.unchecked_into(),
			hex!["142635c658c9fd7685ef466e41948557ecedaec76587307bdc939467781f9d58"]
				.unchecked_into(),
			hex!["c8243df041c48137e16b5152bad01ce99441b3f2907915d569fa3b58ad83495f"]
				.unchecked_into(),
			hex!["03f41e4f76c2abe5e28fd16eaee36153c953759689712eb3dd2c666b62484b2567"]
				.unchecked_into(),
		),
		// standalone 8
		(
			hex!["0e87759b6eeb6891743900cba17b8b5f31b2fa9c28536d9bcf76468d6e455b23"].into(),
			hex!["48cea44ac6dd245572272dc6d4d33908586fb80886bf3207344388eac279cc25"].into(),
			hex!["52edde82f4e7af3c9c526b8a8eea08f9d70eec0da9246f6c34708ca1bde52904"]
				.unchecked_into(),
			hex!["c167d6596d3406a92efe10cad755073dd835caaf982b07ef8e5f8217b59721a9"]
				.unchecked_into(),
			hex!["52edde82f4e7af3c9c526b8a8eea08f9d70eec0da9246f6c34708ca1bde52904"]
				.unchecked_into(),
			hex!["02d1f1d1c6b02514cbb345d98bab19be55a42345573b24c7c1ac8b8da0955f0efd"]
				.unchecked_into(),
		),
		// standalone 9
		(
			hex!["fa2c711c82661a761cf200421b9a5ef3257aa977a3a33acad0722d7d6993f03b"].into(),
			hex!["daf7985bfa22b5060a4eb212fbeddb7c47f7c29db5a356ed9500b34d2944eb3d"].into(),
			hex!["bc95a464f528a8450b2a7c6d6aa8177c9a61c2f0b5c853a39f36818a6a811b00"]
				.unchecked_into(),
			hex!["e04d7783447a6b3c825d346bb0ed0365251f05ac339710b03191320539bc988c"]
				.unchecked_into(),
			hex!["bc95a464f528a8450b2a7c6d6aa8177c9a61c2f0b5c853a39f36818a6a811b00"]
				.unchecked_into(),
			hex!["025fdcd9601c99675868f742ffdec4aebf894951a41e2002b6f7c484dee3392aad"]
				.unchecked_into(),
		),
		// standalone 10
		(
			hex!["4ec0389ae623884a68234fd84d85af833633668aa382007e6515020e8cc29532"].into(),
			hex!["48bb70f924e7362ee55817a6628a79e522a08a31735b0129e47ac435215d6c4e"].into(),
			hex!["f0be719939c8f1b68f2aa21bcc38cc51befecd9d35db5cf2581a297877cb1530"]
				.unchecked_into(),
			hex!["d521e36ab465ca96408685cded42592867600a30ae18284675b00dad525461fe"]
				.unchecked_into(),
			hex!["f0be719939c8f1b68f2aa21bcc38cc51befecd9d35db5cf2581a297877cb1530"]
				.unchecked_into(),
			hex!["03f1f37c1c54dde71a0a122de2326c0d74c6c84bc415bcb7020ea0c6772b8d8650"]
				.unchecked_into(),
		),
	]
}
