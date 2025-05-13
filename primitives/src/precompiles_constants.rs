// Copyright 2022-2025 Tangle Foundation.
// This file is part of Tangle.
// This file originated in Moonbeam's codebase.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.

// Ethereum standard precompile IDs
pub const PRECOMPILE_ECRECOVER: u64 = 1;
pub const PRECOMPILE_SHA256: u64 = 2;
pub const PRECOMPILE_RIPEMD160: u64 = 3;
pub const PRECOMPILE_IDENTITY: u64 = 4;
pub const PRECOMPILE_MODEXP: u64 = 5;
pub const PRECOMPILE_BN128_ADD: u64 = 6;
pub const PRECOMPILE_BN128_MUL: u64 = 7;
pub const PRECOMPILE_BN128_PAIRING: u64 = 8;
pub const PRECOMPILE_BLAKE2F: u64 = 9;
pub const PRECOMPILE_SHA3FIPS256: u64 = 1024;
pub const PRECOMPILE_DISPATCH: u64 = 1025;
pub const PRECOMPILE_ECRECOVER_PUBKEY: u64 = 1026;
pub const PRECOMPILE_CURVE25519_ADD: u64 = 1027;
pub const PRECOMPILE_CURVE25519_SCALAR_MUL: u64 = 1028;
pub const PRECOMPILE_ED25519_VERIFY: u64 = 1029;

// Tangle precompile IDs
pub const PRECOMPILE_STAKING: u64 = 2048;
pub const PRECOMPILE_VESTING: u64 = 2049;
pub const PRECOMPILE_ERC20_BALANCES: u64 = 2050;
pub const PRECOMPILE_DEMOCRACY: u64 = 2051;
pub const PRECOMPILE_BATCH: u64 = 2052;
pub const PRECOMPILE_CALL_PERMIT: u64 = 2053;
pub const PRECOMPILE_PREIMAGE: u64 = 2054;
pub const PRECOMPILE_REGISTRY: u64 = 2055;
pub const PRECOMPILE_ECDSA_SECP256K1: u64 = 2070;
pub const PRECOMPILE_ECDSA_SECP256R1: u64 = 2071;
pub const PRECOMPILE_ECDSA_STARK: u64 = 2072;
pub const PRECOMPILE_SCHNORR_SR25519: u64 = 2073;
pub const PRECOMPILE_SCHNORR_SECP256K1: u64 = 2074;
pub const PRECOMPILE_SCHNORR_ED25519: u64 = 2075;
pub const PRECOMPILE_SCHNORR_ED448: u64 = 2076;
pub const PRECOMPILE_SCHNORR_P256: u64 = 2077;
pub const PRECOMPILE_SCHNORR_P384: u64 = 2078;
pub const PRECOMPILE_SCHNORR_RISTRETTO255: u64 = 2079;
pub const PRECOMPILE_SCHNORR_TAPROOT: u64 = 2080;
pub const PRECOMPILE_BLS381: u64 = 2081;
pub const PRECOMPILE_MULTI_ASSET_DELEGATION: u64 = 2082;
pub const PRECOMPILE_SERVICES: u64 = 2083;
pub const PRECOMPILE_TANGLE_LST: u64 = 2084;
pub const PRECOMPILE_REWARDS: u64 = 2085;
pub const PRECOMPILE_CREDITS: u64 = 2086;

#[test]
fn test_precompile_addresses_match() {
	use precompile_utils::precompile_set::AddressU64;
	use sp_core::{Get, H160};

	// Helper function to create H160 address from hex string
	fn h160_from_hex(hex_str: &str) -> H160 {
		H160::from_slice(&hex::decode(hex_str).unwrap())
	}

	// Test all Ethereum standard precompiles
	assert_eq!(
		AddressU64::<PRECOMPILE_ECRECOVER>::get(),
		h160_from_hex("0000000000000000000000000000000000000001")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SHA256>::get(),
		h160_from_hex("0000000000000000000000000000000000000002")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_RIPEMD160>::get(),
		h160_from_hex("0000000000000000000000000000000000000003")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_IDENTITY>::get(),
		h160_from_hex("0000000000000000000000000000000000000004")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_MODEXP>::get(),
		h160_from_hex("0000000000000000000000000000000000000005")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BN128_ADD>::get(),
		h160_from_hex("0000000000000000000000000000000000000006")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BN128_MUL>::get(),
		h160_from_hex("0000000000000000000000000000000000000007")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BN128_PAIRING>::get(),
		h160_from_hex("0000000000000000000000000000000000000008")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BLAKE2F>::get(),
		h160_from_hex("0000000000000000000000000000000000000009")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SHA3FIPS256>::get(),
		h160_from_hex("0000000000000000000000000000000000000400")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_DISPATCH>::get(),
		h160_from_hex("0000000000000000000000000000000000000401")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_ECRECOVER_PUBKEY>::get(),
		h160_from_hex("0000000000000000000000000000000000000402")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_CURVE25519_ADD>::get(),
		h160_from_hex("0000000000000000000000000000000000000403")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_CURVE25519_SCALAR_MUL>::get(),
		h160_from_hex("0000000000000000000000000000000000000404")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_ED25519_VERIFY>::get(),
		h160_from_hex("0000000000000000000000000000000000000405")
	);

	// Test all Tangle precompiles
	assert_eq!(
		AddressU64::<PRECOMPILE_STAKING>::get(),
		h160_from_hex("0000000000000000000000000000000000000800")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_VESTING>::get(),
		h160_from_hex("0000000000000000000000000000000000000801")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_ERC20_BALANCES>::get(),
		h160_from_hex("0000000000000000000000000000000000000802")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_DEMOCRACY>::get(),
		h160_from_hex("0000000000000000000000000000000000000803")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BATCH>::get(),
		h160_from_hex("0000000000000000000000000000000000000804")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_CALL_PERMIT>::get(),
		h160_from_hex("0000000000000000000000000000000000000805")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_PREIMAGE>::get(),
		h160_from_hex("0000000000000000000000000000000000000806")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_REGISTRY>::get(),
		h160_from_hex("0000000000000000000000000000000000000807")
	);

	assert_eq!(
		AddressU64::<PRECOMPILE_ECDSA_SECP256K1>::get(),
		h160_from_hex("0000000000000000000000000000000000000816")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_ECDSA_SECP256R1>::get(),
		h160_from_hex("0000000000000000000000000000000000000817")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_ECDSA_STARK>::get(),
		h160_from_hex("0000000000000000000000000000000000000818")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_SR25519>::get(),
		h160_from_hex("0000000000000000000000000000000000000819")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_SECP256K1>::get(),
		h160_from_hex("000000000000000000000000000000000000081a")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_ED25519>::get(),
		h160_from_hex("000000000000000000000000000000000000081b")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_ED448>::get(),
		h160_from_hex("000000000000000000000000000000000000081c")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_P256>::get(),
		h160_from_hex("000000000000000000000000000000000000081d")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_P384>::get(),
		h160_from_hex("000000000000000000000000000000000000081e")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_RISTRETTO255>::get(),
		h160_from_hex("000000000000000000000000000000000000081f")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SCHNORR_TAPROOT>::get(),
		h160_from_hex("0000000000000000000000000000000000000820")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_BLS381>::get(),
		h160_from_hex("0000000000000000000000000000000000000821")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_MULTI_ASSET_DELEGATION>::get(),
		h160_from_hex("0000000000000000000000000000000000000822")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_SERVICES>::get(),
		h160_from_hex("0000000000000000000000000000000000000823")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_TANGLE_LST>::get(),
		h160_from_hex("0000000000000000000000000000000000000824")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_REWARDS>::get(),
		h160_from_hex("0000000000000000000000000000000000000825")
	);
	assert_eq!(
		AddressU64::<PRECOMPILE_CREDITS>::get(),
		h160_from_hex("0000000000000000000000000000000000000826")
	);
}
