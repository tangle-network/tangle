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
use frame_support::parameter_types;
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_batch::BatchPrecompile;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_call_permit::CallPermitPrecompile;
use pallet_evm_precompile_curve25519::{Curve25519Add, Curve25519ScalarMul};
use pallet_evm_precompile_democracy::DemocracyPrecompile;
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_multi_asset_delegation::MultiAssetDelegationPrecompile;
use pallet_evm_precompile_preimage::PreimagePrecompile;
use pallet_evm_precompile_registry::PrecompileRegistry;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_staking::StakingPrecompile;
use pallet_evm_precompile_verify_bls381_signature::Bls381Precompile;
use pallet_evm_precompile_verify_ecdsa_secp256k1_signature::EcdsaSecp256k1Precompile;
use pallet_evm_precompile_verify_ecdsa_secp256r1_signature::EcdsaSecp256r1Precompile;
use pallet_evm_precompile_verify_ecdsa_stark_signature::EcdsaStarkPrecompile;
use pallet_evm_precompile_verify_schnorr_signatures::*;
use pallet_evm_precompile_vesting::VestingPrecompile;
use pallet_evm_precompileset_assets_erc20::Erc20AssetsPrecompileSet;
use precompile_utils::precompile_set::{
	AcceptDelegateCall, AddressU64, CallableByContract, CallableByPrecompile, OnlyFrom,
	PrecompileAt, PrecompileSetBuilder, PrecompileSetStartingWith, PrecompilesInRangeInclusive,
	SubcallWithMaxNesting,
};
type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"Tangle Network Token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"TNT"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as foreign
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = ASSET_PRECOMPILE_ADDRESS_PREFIX;
}

#[precompile_utils::precompile_name_from_address]
pub type TanglePrecompilesAt<R> = (
	// Ethereum precompiles:
	PrecompileAt<AddressU64<1>, ECRecover, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<2>, Sha256, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<3>, Ripemd160, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<4>, Identity, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<5>, Modexp, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<6>, Bn128Add, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<7>, Bn128Mul, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<8>, Bn128Pairing, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<9>, Blake2F, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<1024>, Sha3FIPS256, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1025>, Dispatch<R>, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1026>, ECRecoverPublicKey, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1027>, Curve25519Add, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1028>, Curve25519ScalarMul, (CallableByContract, CallableByPrecompile)>,
	PrecompileAt<AddressU64<1029>, Ed25519Verify, (CallableByContract, CallableByPrecompile)>,
	// Tangle precompiles
	PrecompileAt<
		AddressU64<2048>,
		StakingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2049>,
		VestingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2050>,
		Erc20BalancesPrecompile<R, NativeErc20Metadata>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2051>,
		DemocracyPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2052>,
		BatchPrecompile<R>,
		(
			SubcallWithMaxNesting<2>,
			// Batch is the only precompile allowed to call Batch.
			CallableByPrecompile<OnlyFrom<AddressU64<2056>>>,
		),
	>,
	PrecompileAt<
		AddressU64<2053>,
		CallPermitPrecompile<R>,
		(SubcallWithMaxNesting<0>, CallableByContract),
	>,
	PrecompileAt<
		AddressU64<2054>,
		PreimagePrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2055>,
		PrecompileRegistry<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Ecdsa-Secp256k1 signature verifier precompile
	PrecompileAt<
		AddressU64<2070>,
		EcdsaSecp256k1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Ecdsa-Secp256r1 signature verifier precompile
	PrecompileAt<
		AddressU64<2071>,
		EcdsaSecp256r1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Ecdsa-Stark signature verifier precompile
	PrecompileAt<
		AddressU64<2072>,
		EcdsaStarkPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Sr25519 signature verifier precompile
	PrecompileAt<
		AddressU64<2073>,
		SchnorrSr25519Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Secp256k1 signature verifier precompile
	PrecompileAt<
		AddressU64<2074>,
		SchnorrSecp256k1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Ed25519 signature verifier precompile
	PrecompileAt<
		AddressU64<2075>,
		SchnorrEd25519Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Ed448 signature verifier precompile
	PrecompileAt<
		AddressU64<2076>,
		SchnorrEd448Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-P256 signature verifier precompile
	PrecompileAt<
		AddressU64<2077>,
		SchnorrP256Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-P384 signature verifier precompile
	PrecompileAt<
		AddressU64<2078>,
		SchnorrP384Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Ristretto255 signature verifier precompile
	PrecompileAt<
		AddressU64<2079>,
		SchnorrRistretto255Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Schnorr-Taproot signature verifier precompile
	PrecompileAt<
		AddressU64<2080>,
		SchnorrTaprootPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<AddressU64<2081>, Bls381Precompile<R>, (CallableByContract, CallableByPrecompile)>,
	// MultiAsset Delegation precompile
	PrecompileAt<
		AddressU64<2082>,
		MultiAssetDelegationPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
);

pub type TanglePrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompilesInRangeInclusive<(AddressU64<1>, AddressU64<2095>), TanglePrecompilesAt<R>>,
		// Prefixed precompile sets (XC20)
		PrecompileSetStartingWith<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R>,
			CallableByContract,
		>,
	),
>;
