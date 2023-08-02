use std::str::FromStr;

use fp_evm::GenesisAccount;
use sp_core::{H160, U256};

pub fn get_distribution() -> Vec<(H160, GenesisAccount)> {
    vec![
        // H160 address of Alice dev account
        // Derived from SS58 (42 prefix) address
        // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
        // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
        // Using the full hex key, truncating to the first 20 bytes (the first 40 hex
        // chars)
        (
            H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                    .expect("internal U256 is valid; qed"),
                code: Default::default(),
                nonce: Default::default(),
                storage: Default::default(),
            }
        ),
        // H160 address for benchmark usage
        (
            H160::from_str("1000000000000000000000000000000000000001")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: vec![0x00],
            }
        ),
        // Other accounts used by relayer, bridges, and tests
        (
            H160::from_str("8712c0449d1440d24a532a17c553e7661114e6bc")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("46Bf9B20A8144BaA7C2BB76303b6a17eB8755408")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("bFAc59575FeC3d1b33C7685eE6b3a2BfC155bdF3")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("c65351122A5dc7881559DeE52e025678212C615C")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("2ecceed83d6d2908cf4d67c76984e0bbcbfebbc1")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("228B67B0e42485E21373A7BB7278a0d02C8fDb18")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("5d26a601A80E3f472C5d6C3D1EdD78860F87Ac18")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("21Add37cBA50CF92A734c3Ee02FCeaDEf3dC57D6")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        ),
        (
            H160::from_str("2DFA35bd8C59C38FB3eC4e71b0106160E130A40E")
                .expect("internal H160 is valid; qed"),
            GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: Default::default(),
            }
        )
    ]
}