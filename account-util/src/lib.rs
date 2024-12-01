use core::str::FromStr;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use pallet_airdrop_claims::MultiAddress;
use pallet_vesting::{Vesting, VestingInfo};
use sp_core::H160;
use sp_runtime::{
	traits::{AccountIdConversion, Zero},
	AccountId32,
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::{AccountId, Balance};

pub const BLOCK_TIME: u128 = 6;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;

pub const INVESTOR_ACCOUNTS: [(&str, Balance); 29] = [
	("5FCviiKcJzVfdwqv451JRQc93ZTSbbC9YfgKMkN3LpNMBjS3", 100000000000000000000000),
	("0xC02ad7b9a9121fc849196E844DC869D2250DF3A6", 150000000000000000000000),
	("5HBKM8XL7sr7S7qx6ukugDJUBhm16Ubnz1KuJCdHmHQvtc7Z", 400000000000000000000000),
	("0x86f99feff4cd3268ccefc760a6bbce2e07aa4d8e", 100000000000000000000000),
	("5HCJQLeQqCf64C6uy7CSoTND35QNdi1zU66TTM6QPCG9u9BB", 500000000000000000000000),
	("5DhQMPP69vRBNvZ6w4E9qii1yj56M66YXuPkkvz12yvn5ufN", 2750000000000000000000000),
	("0x17B74Dcf1E422AF5964056eB836321aE7A820035", 200000000000000000000000),
	("5Df1Sec4ZmeidskuuPtpt4SSCL6U5St19fsv4sBAwvkwmvin", 100000000000000000000000),
	("5FerzxKQZoP9wxjRhS1PHA4v27aPQMoZpXmufcQk1JYE5xU8", 100000000000000000000000),
	("0x75afaece8cf2a7974b1e541648923afd9339b3f8", 150000000000000000000000),
	("0x5a82d0bad9995b1bfa71de79b5e524decb5bee1c", 1000000000000000000000000),
	("5Cf5SfzngS9T9fdQSsKt1GJ42BQPP7FqU4trLwgkDD1ZVBN7", 20000000000000000000000),
	("0x5da7351A4Cb03c33e11F51841bc614d985812821", 20000000000000000000000),
	("5DDTvr7P2MaUirj3rSvpcF5DLxECiAqpQegdVCzAuwxingVC", 50000000000000000000000),
	("5CXkTTyNzVuE2fwNBzSwmmtcCntnun5vMDx6Wnn5VjopGUVx", 160000000000000000000000),
	("5G9Ji4EEiTehMKa9Nfe2rGozZB3XEyRtyvJWt2wYLMRYyAyn", 180000000000000000000000),
	("5EZcQuvjvuv1K4Ugg2ofLEhAC3NXPbaCAytPR3hmUqBv1Bhe", 200000000000000000000000),
	("5Fe8pwn27TNBuM1Agz1ivCMykX795woPQkjwiWD4UoSndmQj", 220000000000000000000000),
	("5HWLv9RvMSZ6FwuQswHCo1nmYxmAqRutqh37Vapx3pmXbVU3", 240000000000000000000000),
	("5ENgRjBQue32ppyLw1u55Rbe36gDWTJgZmcbSfZwrJAyt2tZ", 781850000000000000000000),
	("5DhzryWQpJTQfdauBf7yGKpr6LW42ye4oYvEYiVbMcGN7GZt", 879590000000000000000000),
	("5C52zXiWq7BM5x55soCudf8daW12NTLtMfisBiYd5Pov1Hw1", 977320000000000000000000),
	("5HbRuwKiUw4g7yh9iNBQ6zFG63sgZyGoZ4ep61CqMYTzk4gU", 1075050000000000000000000),
	("5HnQ2onP12Vhv3dVUvmKocbWr5sMjunCmjt2MmtxMr5z5dp1", 1172780000000000000000000),
	("5CAbD6BFcATxi9jVpixNtsLKovQy6RWQMQynHtapqAtKL8vT", 338150000000000000000000),
	("5DP7RmWdPD6TWbTNgBu6iKzYsm95fHETXyy5tQvEmS6zDhPn", 380410000000000000000000),
	("5EyDntnRYxgWLuHFRmSnirioDNpMDCUTQ4bwRYV3LGHQoMxo", 422680000000000000000000),
	("5DkNqZs22mLYXKy7c2vrtstqbcQBU1ArPKw51xvC37eGsuNP", 464950000000000000000000),
	("5CQJt5A7GmEUhh7S3MtaiBGvNRTY6PFw6KHJFcTVD4ZxBty7", 507220000000000000000000),
];

pub const TEAM_ACCOUNT_TO_UPDATE: [(&str, Balance); 1] =
	[("5FH32Ro5cTpLE1FhP3skdi16UuVariyzoQfyK7vvjE2CHEtX", 28721849310000000043843584)];

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decoding_accounts_into_account_id_32_bytes() {
		for (address, _) in INVESTOR_ACCOUNTS {
			let account_id = if address.starts_with("0x") {
				MultiAddress::EVM(
					H160::from_str(address).expect("should be a valid address").into(),
				)
				.to_account_id_32()
			} else {
				let account = MultiAddress::Native(
					AccountId32::from_str(address).expect("should be a valid address"),
				)
				.to_account_id_32();

				assert_eq!(
					account,
					AccountId32::from_str(address).expect("should be a valid address")
				);

				account
			};

			let account_id_bytes: [u8; 32] = account_id.into();
			println!("{:?}", account_id_bytes);
		}

		for (address, _) in TEAM_ACCOUNT_TO_UPDATE {
			let account_id = MultiAddress::Native(
				AccountId32::from_str(address).expect("should be a valid address"),
			)
			.to_account_id_32();

			let account_id_bytes: [u8; 32] = account_id.into();
			println!("{:?}", account_id_bytes);
		}
	}
}
