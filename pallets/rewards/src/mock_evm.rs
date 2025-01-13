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
use crate::mock::{
	AccountId, Balances, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Timestamp,
};
use fp_evm::FeeCalculator;
use frame_support::{parameter_types, traits::FindAuthor, weights::Weight, PalletId};
use pallet_ethereum::{EthereumBlockHashMapping, IntermediateStateRoot, PostLogContent, RawOrigin};
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, HashedAddressMapping, OnChargeEVMTransaction,
};
use sp_core::{keccak_256, ConstU32, H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, DispatchInfoOf, Dispatchable},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	ConsensusEngineId,
};

use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

use precompile_utils::precompile_set::{
	AcceptDelegateCall, AddressU64, CallableByContract, CallableByPrecompile, PrecompileAt,
	PrecompileSetBuilder, PrecompilesInRangeInclusive,
};

type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

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
	(PrecompilesInRangeInclusive<(AddressU64<1>, AddressU64<2095>), DefaultPrecompiles>,),
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
	type OnChargeTransaction = ();
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
			RuntimeCall::Ethereum(call) => {
				call.pre_dispatch_self_contained(info, dispatch_info, len)
			},
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(RawOrigin::EthereumTransaction(info))))
			},
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
