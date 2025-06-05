// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.
#![allow(clippy::all)]
use crate as pallet_services;
use crate::mock::{
	AccountId, AssetId, Balances, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Timestamp,
};
use fp_evm::{ExitReason, ExitRevert, ExitSucceed, FeeCalculator};
use frame_support::{
	PalletId, parameter_types,
	traits::{Currency, FindAuthor, OnUnbalanced},
	weights::Weight,
};
use pallet_ethereum::{EthereumBlockHashMapping, IntermediateStateRoot, PostLogContent, RawOrigin};
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, HashedAddressMapping, OnChargeEVMTransaction,
};
use sp_core::{ConstU32, H160, H256, U256, keccak_256};
use sp_runtime::{
	ConsensusEngineId,
	traits::{BlakeTwo256, DispatchInfoOf, Dispatchable},
	transaction_validity::{TransactionValidity, TransactionValidityError},
};

use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompileset_assets_erc20::{AddressToAssetId, Erc20AssetsPrecompileSet};

use precompile_utils::precompile_set::{
	AcceptDelegateCall, AddressU64, CallableByContract, CallableByPrecompile, PrecompileAt,
	PrecompileSetBuilder, PrecompileSetStartingWith, PrecompilesInRangeInclusive,
};

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
pub type DefaultPrecompiles = (
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
	PrecompileAt<AddressU64<1026>, ECRecoverPublicKey, (CallableByContract, CallableByPrecompile)>,
);

pub type TanglePrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompilesInRangeInclusive<(AddressU64<1>, AddressU64<2095>), DefaultPrecompiles>,
		PrecompileAt<
			AddressU64<2050>,
			Erc20BalancesPrecompile<R, NativeErc20Metadata>,
			(CallableByContract, CallableByPrecompile),
		>,
		// Prefixed precompile sets (XC20)
		PrecompileSetStartingWith<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R>,
			CallableByContract,
		>,
	),
>;

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;

	pub PrecompilesValue: TanglePrecompiles<Runtime> = TanglePrecompiles::<_>::new();
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

const ASSET_ID_SIZE: usize = core::mem::size_of::<AssetId>();

impl AddressToAssetId<AssetId> for Runtime {
	fn address_to_asset_id(address: H160) -> Option<AssetId> {
		let mut data = [0u8; ASSET_ID_SIZE];
		let address_bytes: [u8; 20] = address.into();
		if ASSET_PRECOMPILE_ADDRESS_PREFIX.eq(&address_bytes[0..4]) {
			data.copy_from_slice(&address_bytes[4..ASSET_ID_SIZE + 4]);
			Some(AssetId::from_be_bytes(data))
		} else {
			None
		}
	}

	fn asset_id_to_address(asset_id: AssetId) -> H160 {
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(ASSET_PRECOMPILE_ADDRESS_PREFIX);
		data[4..ASSET_ID_SIZE + 4].copy_from_slice(&asset_id.to_be_bytes());
		H160::from(data)
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		(1.into(), Weight::zero())
	}
}

pub struct FindAuthorTruncated;
impl FindAuthor<H160> for FindAuthorTruncated {
	fn find_author<'a, I>(_digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(address_build(0).address)
	}
}

const BLOCK_GAS_LIMIT: u64 = 150_000_000;
const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

parameter_types! {
	pub const TransactionByteFee: u64 = 1;
	pub const ChainId: u64 = 42;
	pub const EVMModuleId: PalletId = PalletId(*b"py/evmpa");
	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	pub const GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_POV_SIZE);
	pub const WeightPerGas: Weight = Weight::from_parts(20_000, 0);
}

parameter_types! {
	pub SuicideQuickClearLimit: u32 = 0;
}

pub struct DealWithFees;
impl OnUnbalanced<RuntimeNegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(_fees_then_tips: impl Iterator<Item = RuntimeNegativeImbalance>) {
		// whatever
	}
}
pub struct FreeEVMExecution;

impl OnChargeEVMTransaction<Runtime> for FreeEVMExecution {
	type LiquidityInfo = ();

	fn withdraw_fee(
		_who: &H160,
		_fee: U256,
	) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
		Ok(())
	}

	fn correct_and_deposit_fee(
		_who: &H160,
		_corrected_fee: U256,
		_base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		already_withdrawn
	}

	fn pay_priority_fee(_tip: Self::LiquidityInfo) {}
}

/// Type alias for negative imbalance during fees
type RuntimeNegativeImbalance =
	<Balances as Currency<<Runtime as frame_system::Config>::AccountId>>::NegativeImbalance;

/// See: [`pallet_evm::EVMCurrencyAdapter`]
pub struct CustomEVMCurrencyAdapter;

impl OnChargeEVMTransaction<Runtime> for CustomEVMCurrencyAdapter {
	type LiquidityInfo = Option<RuntimeNegativeImbalance>;

	fn withdraw_fee(
		who: &H160,
		fee: U256,
	) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
		let pallet_services_address = pallet_services::Pallet::<Runtime>::pallet_evm_account();
		// Make pallet services account free to use
		if who == &pallet_services_address {
			return Ok(None);
		}
		// fallback to the default implementation
		<pallet_evm::EVMCurrencyAdapter<Balances, DealWithFees> as OnChargeEVMTransaction<
			Runtime,
		>>::withdraw_fee(who, fee)
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		let pallet_services_address = pallet_services::Pallet::<Runtime>::pallet_evm_account();
		// Make pallet services account free to use
		if who == &pallet_services_address {
			return already_withdrawn;
		}
		// fallback to the default implementation
		<pallet_evm::EVMCurrencyAdapter<Balances, DealWithFees> as OnChargeEVMTransaction<
			Runtime,
		>>::correct_and_deposit_fee(who, corrected_fee, base_fee, already_withdrawn)
	}

	fn pay_priority_fee(tip: Self::LiquidityInfo) {
		<pallet_evm::EVMCurrencyAdapter<Balances, DealWithFees> as OnChargeEVMTransaction<
			Runtime,
		>>::pay_priority_fee(tip)
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = TanglePrecompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = CustomEVMCurrencyAdapter;
	type OnCreate = ();
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type FindAuthor = FindAuthorTruncated;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = ();
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) =>
				call.pre_dispatch_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
				Some(call.dispatch(RuntimeOrigin::from(RawOrigin::EthereumTransaction(info)))),
			_ => None,
		}
	}
}

pub struct MockedEvmRunner;

impl tangle_primitives::services::EvmRunner<Runtime> for MockedEvmRunner {
	type Error = pallet_evm::Error<Runtime>;

	fn call(
		source: sp_core::H160,
		target: sp_core::H160,
		input: Vec<u8>,
		value: sp_core::U256,
		gas_limit: u64,
		is_transactional: bool,
		validate: bool,
	) -> Result<fp_evm::CallInfo, tangle_primitives::services::RunnerError<Self::Error>> {
		// Check if this is a call to one of our mock contract addresses
		if target == crate::mock::MBSM ||
			target == crate::mock::CGGMP21_BLUEPRINT ||
			target == crate::mock::HOOKS_TEST
		{
			#[cfg(test)]
			eprintln!(
				"Mock EVM call to: {:?}, selector: 0x{}",
				target,
				if input.len() >= 4 {
					format!("{:02x}{:02x}{:02x}{:02x}", input[0], input[1], input[2], input[3])
				} else {
					"<no selector>".to_string()
				}
			);

			// Parse function selector and implement validation logic
			if input.len() >= 4 {
				let selector = &input[0..4];
				let call_data = &input[4..];

				match selector {
					// First function (selector: 0x0b6535d7) - possibly onBlueprintCreated to MBSM
					[0x0b, 0x65, 0x35, 0xd7] => {
						// Always allow MBSM calls
						return Ok(fp_evm::CallInfo {
							exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
							value: vec![],
							used_gas: fp_evm::UsedGas {
								standard: U256::from(21000),
								effective: U256::from(21000),
							},
							weight_info: None,
							logs: vec![],
						});
					},

					// onBlueprintCreated(uint64,address,ServiceBlueprint)
					[0xb8, 0x9a, 0xf9, 0x04] => {
						// Always allow blueprint creation in tests
						return Ok(fp_evm::CallInfo {
							exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
							value: vec![],
							used_gas: fp_evm::UsedGas {
								standard: U256::from(21000),
								effective: U256::from(21000),
							},
							weight_info: None,
							logs: vec![],
						});
					},

					// onRegister(uint64,OperatorPreferences,bytes)
					[0x95, 0x24, 0xcf, 0x20] => {
						// Basic validation: ensure blueprint_id is reasonable (< 1000 for test
						// purposes)
						if call_data.len() >= 32 {
							let blueprint_id_bytes = &call_data[24..32]; // uint64 is in the last 8 bytes of the 32-byte word
							let blueprint_id = u64::from_be_bytes([
								blueprint_id_bytes[0],
								blueprint_id_bytes[1],
								blueprint_id_bytes[2],
								blueprint_id_bytes[3],
								blueprint_id_bytes[4],
								blueprint_id_bytes[5],
								blueprint_id_bytes[6],
								blueprint_id_bytes[7],
							]);

							// Only reject registrations for extremely high blueprint IDs (which
							// would be invalid in tests)
							if blueprint_id > 1000 {
								return Ok(fp_evm::CallInfo {
									exit_reason: ExitReason::Revert(ExitRevert::Reverted),
									value: "Invalid blueprint ID".as_bytes().to_vec(),
									used_gas: fp_evm::UsedGas {
										standard: U256::from(21000),
										effective: U256::from(21000),
									},
									weight_info: None,
									logs: vec![],
								});
							}
						}

						// Accept valid registrations
						return Ok(fp_evm::CallInfo {
							exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
							value: vec![],
							used_gas: fp_evm::UsedGas {
								standard: U256::from(21000),
								effective: U256::from(21000),
							},
							weight_info: None,
							logs: vec![],
						});
					},

					// onRequest(uint64,RequestParams)
					[0x9a, 0x4b, 0xa5, 0x00] => {
						// Basic validation for service requests
						if call_data.len() >= 32 {
							let blueprint_id_bytes = &call_data[24..32];
							let blueprint_id = u64::from_be_bytes([
								blueprint_id_bytes[0],
								blueprint_id_bytes[1],
								blueprint_id_bytes[2],
								blueprint_id_bytes[3],
								blueprint_id_bytes[4],
								blueprint_id_bytes[5],
								blueprint_id_bytes[6],
								blueprint_id_bytes[7],
							]);

							// Only reject requests for extremely high blueprint IDs (which would be
							// invalid in tests)
							if blueprint_id > 1000 {
								return Ok(fp_evm::CallInfo {
									exit_reason: ExitReason::Revert(ExitRevert::Reverted),
									value: "Blueprint not found".as_bytes().to_vec(),
									used_gas: fp_evm::UsedGas {
										standard: U256::from(21000),
										effective: U256::from(21000),
									},
									weight_info: None,
									logs: vec![],
								});
							}
						}

						return Ok(fp_evm::CallInfo {
							exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
							value: vec![],
							used_gas: fp_evm::UsedGas {
								standard: U256::from(21000),
								effective: U256::from(21000),
							},
							weight_info: None,
							logs: vec![],
						});
					},

					// onApprove, onReject, onServiceInitialized, etc. - allow by default
					_ => {
						#[cfg(test)]
						eprintln!(
							"Unknown EVM function selector: 0x{:02x}{:02x}{:02x}{:02x}",
							selector[0], selector[1], selector[2], selector[3]
						);

						return Ok(fp_evm::CallInfo {
							exit_reason: ExitReason::Succeed(ExitSucceed::Stopped),
							value: vec![],
							used_gas: fp_evm::UsedGas {
								standard: U256::from(21000),
								effective: U256::from(21000),
							},
							weight_info: None,
							logs: vec![],
						});
					},
				}
			}

			// For calls without proper function selectors, revert
			return Ok(fp_evm::CallInfo {
				exit_reason: ExitReason::Revert(ExitRevert::Reverted),
				value: "Invalid function call".as_bytes().to_vec(),
				used_gas: fp_evm::UsedGas {
					standard: U256::from(21000),
					effective: U256::from(21000),
				},
				weight_info: None,
				logs: vec![],
			});
		}

		let max_fee_per_gas = FixedGasPrice::min_gas_price().0;
		let max_priority_fee_per_gas = max_fee_per_gas.saturating_mul(U256::from(2));
		let nonce = None;
		let access_list = Default::default();
		let weight_limit = None;
		let proof_size_base_cost = None;
		<<Runtime as pallet_evm::Config>::Runner as pallet_evm::Runner<Runtime>>::call(
			source,
			target,
			input,
			value,
			gas_limit,
			Some(max_fee_per_gas),
			Some(max_priority_fee_per_gas),
			nonce,
			access_list,
			is_transactional,
			validate,
			weight_limit,
			proof_size_base_cost,
			<Runtime as pallet_evm::Config>::config(),
		)
		.map_err(|o| tangle_primitives::services::RunnerError { error: o.error, weight: o.weight })
	}
}

pub struct AccountInfo {
	pub address: H160,
}

pub fn address_build(seed: u8) -> AccountInfo {
	let private_key = H256::from_slice(&[(seed + 1); 32]); //H256::from_low_u64_be((i + 1) as u64);
	let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
	let public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let address = H160::from(H256::from(keccak_256(public_key)));

	AccountInfo { address }
}
