// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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
use crate::LstPoolAssetsInstance;
use frame_support::parameter_types;
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_batch::BatchPrecompile;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_call_permit::CallPermitPrecompile;
use pallet_evm_precompile_credits::CreditsPrecompile;
use pallet_evm_precompile_curve25519::{Curve25519Add, Curve25519ScalarMul};
use pallet_evm_precompile_democracy::DemocracyPrecompile;
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_multi_asset_delegation::MultiAssetDelegationPrecompile;
use pallet_evm_precompile_preimage::PreimagePrecompile;
use pallet_evm_precompile_registry::PrecompileRegistry;
use pallet_evm_precompile_rewards::RewardsPrecompile;
use pallet_evm_precompile_services::ServicesPrecompile;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_staking::StakingPrecompile;
use pallet_evm_precompile_tangle_lst::TangleLstPrecompile;
use pallet_evm_precompile_verify_bls381_signature::Bls381Precompile;
use pallet_evm_precompile_verify_ecdsa_secp256k1_signature::EcdsaSecp256k1Precompile;
use pallet_evm_precompile_verify_ecdsa_secp256r1_signature::EcdsaSecp256r1Precompile;
use pallet_evm_precompile_verify_ecdsa_stark_signature::EcdsaStarkPrecompile;
use pallet_evm_precompile_verify_schnorr_signatures::*;
use pallet_evm_precompile_vesting::VestingPrecompile;
use pallet_evm_precompileset_assets_erc20::Erc20AssetsPrecompileSet;
use precompile_utils::precompile_set::*;
use tangle_primitives::precompiles_constants::*;
type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"Tangle Testnet Network Token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"tTNT"
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
	// Address: 0x0000000000000000000000000000000000000001 - PRECOMPILE_ECRECOVER
	PrecompileAt<AddressU64<1>, ECRecover, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000002 - PRECOMPILE_SHA256
	PrecompileAt<AddressU64<2>, Sha256, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000003 - PRECOMPILE_RIPEMD160
	PrecompileAt<AddressU64<3>, Ripemd160, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000004 - PRECOMPILE_IDENTITY
	PrecompileAt<AddressU64<4>, Identity, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000005 - PRECOMPILE_MODEXP
	PrecompileAt<AddressU64<5>, Modexp, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000006 - PRECOMPILE_BN128_ADD
	PrecompileAt<AddressU64<6>, Bn128Add, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000007 - PRECOMPILE_BN128_MUL
	PrecompileAt<AddressU64<7>, Bn128Mul, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000008 - PRECOMPILE_BN128_PAIRING
	PrecompileAt<AddressU64<8>, Bn128Pairing, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000009 - PRECOMPILE_BLAKE2F
	PrecompileAt<AddressU64<9>, Blake2F, EthereumPrecompilesChecks>,
	// Address: 0x0000000000000000000000000000000000000400 - PRECOMPILE_SHA3FIPS256
	PrecompileAt<AddressU64<1024>, Sha3FIPS256, (CallableByContract, CallableByPrecompile)>,
	// Address: 0x0000000000000000000000000000000000000401 - PRECOMPILE_DISPATCH
	PrecompileAt<AddressU64<1025>, Dispatch<R>, (CallableByContract, CallableByPrecompile)>,
	// Address: 0x0000000000000000000000000000000000000402 - PRECOMPILE_ECRECOVER_PUBKEY
	PrecompileAt<AddressU64<1026>, ECRecoverPublicKey, (CallableByContract, CallableByPrecompile)>,
	// Address: 0x0000000000000000000000000000000000000403 - PRECOMPILE_CURVE25519_ADD
	PrecompileAt<AddressU64<1027>, Curve25519Add, (CallableByContract, CallableByPrecompile)>,
	// Address: 0x0000000000000000000000000000000000000404 - PRECOMPILE_CURVE25519_SCALAR_MUL
	PrecompileAt<AddressU64<1028>, Curve25519ScalarMul, (CallableByContract, CallableByPrecompile)>,
	// Address: 0x0000000000000000000000000000000000000405 - PRECOMPILE_ED25519_VERIFY
	PrecompileAt<AddressU64<1029>, Ed25519Verify, (CallableByContract, CallableByPrecompile)>,
	// Tangle precompiles
	// Address: 0x0000000000000000000000000000000000000800 - PRECOMPILE_STAKING (2048)
	PrecompileAt<
		AddressU64<2048>,
		StakingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000801 - PRECOMPILE_VESTING (2049)
	PrecompileAt<
		AddressU64<2049>,
		VestingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000802 - PRECOMPILE_ERC20_BALANCES (2050)
	PrecompileAt<
		AddressU64<2050>,
		Erc20BalancesPrecompile<R, NativeErc20Metadata>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000803 - PRECOMPILE_DEMOCRACY (2051)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_DEMOCRACY }>,
		DemocracyPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000804 - PRECOMPILE_BATCH (2052)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_BATCH }>,
		BatchPrecompile<R>,
		(
			SubcallWithMaxNesting<2>,
			// Batch is the only precompile allowed to call Batch.
			CallableByPrecompile<OnlyFrom<AddressU64<{ PRECOMPILE_BATCH }>>>,
		),
	>,
	// Address: 0x0000000000000000000000000000000000000805 - PRECOMPILE_CALL_PERMIT (2053)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_CALL_PERMIT }>,
		CallPermitPrecompile<R>,
		(SubcallWithMaxNesting<0>, CallableByContract),
	>,
	// Address: 0x0000000000000000000000000000000000000806 - PRECOMPILE_PREIMAGE (2054)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_PREIMAGE }>,
		PreimagePrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000807 - PRECOMPILE_REGISTRY (2055)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_REGISTRY }>,
		PrecompileRegistry<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000816 - PRECOMPILE_ECDSA_SECP256K1 (2070)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_ECDSA_SECP256K1 }>,
		EcdsaSecp256k1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000817 - PRECOMPILE_ECDSA_SECP256R1 (2071)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_ECDSA_SECP256R1 }>,
		EcdsaSecp256r1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000818 - PRECOMPILE_ECDSA_STARK (2072)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_ECDSA_STARK }>,
		EcdsaStarkPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000819 - PRECOMPILE_SCHNORR_SR25519 (2073)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_SR25519 }>,
		SchnorrSr25519Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081a - PRECOMPILE_SCHNORR_SECP256K1 (2074)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_SECP256K1 }>,
		SchnorrSecp256k1Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081b - PRECOMPILE_SCHNORR_ED25519 (2075)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_ED25519 }>,
		SchnorrEd25519Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081c - PRECOMPILE_SCHNORR_ED448 (2076)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_ED448 }>,
		SchnorrEd448Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081d - PRECOMPILE_SCHNORR_P256 (2077)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_P256 }>,
		SchnorrP256Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081e - PRECOMPILE_SCHNORR_P384 (2078)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_P384 }>,
		SchnorrP384Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x000000000000000000000000000000000000081f - PRECOMPILE_SCHNORR_RISTRETTO255 (2079)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_RISTRETTO255 }>,
		SchnorrRistretto255Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000820 - PRECOMPILE_SCHNORR_TAPROOT (2080)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SCHNORR_TAPROOT }>,
		SchnorrTaprootPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000821 - PRECOMPILE_BLS381 (2081)
	PrecompileAt<
		AddressU64<{ PRECOMPILE_BLS381 }>,
		Bls381Precompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000822
	PrecompileAt<
		AddressU64<{ PRECOMPILE_MULTI_ASSET_DELEGATION }>,
		MultiAssetDelegationPrecompile<R>,
		(CallableByContract, CallableByPrecompile, SubcallWithMaxNesting<1>),
	>,
	// Address: 0x0000000000000000000000000000000000000823
	PrecompileAt<
		AddressU64<{ PRECOMPILE_SERVICES }>,
		ServicesPrecompile<R>,
		(CallableByContract, CallableByPrecompile, SubcallWithMaxNesting<1>),
	>,
	// Address: 0x0000000000000000000000000000000000000824
	PrecompileAt<
		AddressU64<{ PRECOMPILE_TANGLE_LST }>,
		TangleLstPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000825
	PrecompileAt<
		AddressU64<{ PRECOMPILE_REWARDS }>,
		RewardsPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	// Address: 0x0000000000000000000000000000000000000826
	PrecompileAt<
		AddressU64<{ PRECOMPILE_CREDITS }>,
		CreditsPrecompile<R>,
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
			Erc20AssetsPrecompileSet<R, LstPoolAssetsInstance>,
			CallableByContract,
		>,
	),
>;
