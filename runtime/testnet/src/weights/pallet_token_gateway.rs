//! Autogenerated weights for pallet_token_gateway
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2025-02-03, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `192.168.1.5`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/cere
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet
// pallet_token_gateway
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output=./runtime/cere-dev/src/weights/pallet_token_gateway.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;


/// Weights for pallet_token_gateway using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_token_gateway::WeightInfo for SubstrateWeight<T> {
    // Storage: `Hyperbridge::HostParams` (r:1 w:0)
    // Proof: `Hyperbridge::HostParams` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: `Ismp::Nonce` (r:1 w:1)
    // Proof: `Ismp::Nonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: `TokenGateway::Precisions` (r:0 w:100)
    // Proof: `TokenGateway::Precisions` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `TokenGateway::NativeAssets` (r:0 w:1)
    // Proof: `TokenGateway::NativeAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `TokenGateway::LocalAssets` (r:0 w:1)
    // Proof: `TokenGateway::LocalAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `TokenGateway::SupportedAssets` (r:0 w:1)
    // Proof: `TokenGateway::SupportedAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e747356aa03068021c89766e6f0dc869c` (r:1 w:1)
    // Proof: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e747356aa03068021c89766e6f0dc869c` (r:1 w:1)
    // Storage: UNKNOWN KEY `0x526571756573745061796d656e7456aa03068021c89766e6f0dc869cadd7cfc2` (r:0 w:1)
    // Proof: UNKNOWN KEY `0x526571756573745061796d656e7456aa03068021c89766e6f0dc869cadd7cfc2` (r:0 w:1)
    /// The range of component `x` is `[1, 100]`.
    fn create_erc6160_asset(x: u32, ) -> Weight {
        Weight::from_parts(22_256_047_u64, 0)
            // Standard Error: 3_063
            .saturating_add(Weight::from_parts(1_733_865_u64, 0).saturating_mul(x as u64))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(6_u64))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x as u64)))
    }
    // Storage: `TokenGateway::SupportedAssets` (r:1 w:0)
    // Proof: `TokenGateway::SupportedAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `TokenGateway::NativeAssets` (r:1 w:0)
    // Proof: `TokenGateway::NativeAssets` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `System::Account` (r:1 w:1)
    // Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
    // Storage: `TokenGateway::Precisions` (r:1 w:0)
    // Proof: `TokenGateway::Precisions` (`max_values`: None, `max_size`: None, mode: `Measured`)
    // Storage: `Hyperbridge::HostParams` (r:1 w:0)
    // Proof: `Hyperbridge::HostParams` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: `Ismp::Nonce` (r:1 w:1)
    // Proof: `Ismp::Nonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e74733d1fa1ad95382c59b20b8fe912a8` (r:1 w:1)
    // Proof: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e74733d1fa1ad95382c59b20b8fe912a8` (r:1 w:1)
    // Storage: UNKNOWN KEY `0x526571756573745061796d656e743d1fa1ad95382c59b20b8fe912a8853bf587` (r:0 w:1)
    // Proof: UNKNOWN KEY `0x526571756573745061796d656e743d1fa1ad95382c59b20b8fe912a8853bf587` (r:0 w:1)
    fn teleport() -> Weight {
        Weight::from_parts(62_000_000_u64, 0)
            .saturating_add(T::DbWeight::get().reads(7_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    // Storage: `TokenGateway::TokenGatewayAddresses` (r:0 w:1)
    // Proof: `TokenGateway::TokenGatewayAddresses` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `x` is `[1, 100]`.
    fn set_token_gateway_addresses(_x: u32, ) -> Weight {
        Weight::from_parts(3_897_505_u64, 0)
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    // Storage: `Hyperbridge::HostParams` (r:1 w:0)
    // Proof: `Hyperbridge::HostParams` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: `Ismp::Nonce` (r:1 w:1)
    // Proof: `Ismp::Nonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    // Storage: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e747399c5cd93560a9d5b3112b189670f` (r:1 w:1)
    // Proof: UNKNOWN KEY `0x52657175657374436f6d6d69746d656e747399c5cd93560a9d5b3112b189670f` (r:1 w:1)
    // Storage: UNKNOWN KEY `0x526571756573745061796d656e7499c5cd93560a9d5b3112b189670f2bf0f746` (r:0 w:1)
    // Proof: UNKNOWN KEY `0x526571756573745061796d656e7499c5cd93560a9d5b3112b189670f2bf0f746` (r:0 w:1)
    fn update_erc6160_asset() -> Weight {
        Weight::from_parts(21_000_000_u64, 0)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    // Storage: `TokenGateway::Precisions` (r:0 w:100)
    // Proof: `TokenGateway::Precisions` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `x` is `[1, 100]`.
    fn update_asset_precision(x: u32, ) -> Weight {
        Weight::from_parts(1_255_030_u64, 0)
            // Standard Error: 2_019
            .saturating_add(Weight::from_parts(1_723_090_u64, 0).saturating_mul(x as u64))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x as u64)))
    }
}