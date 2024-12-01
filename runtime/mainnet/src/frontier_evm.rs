// This file is part of Tangle.

// Copyright (C) 2022-2024 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
	precompiles::{PrecompileName, TanglePrecompiles},
	*,
};
use frame_support::{pallet_prelude::*, parameter_types, traits::FindAuthor, weights::Weight};
use sp_core::{crypto::ByteArray, H160, U256};
use sp_runtime::{traits::BlakeTwo256, ConsensusEngineId, Permill};
use sp_std::{marker::PhantomData, prelude::*};
// Frontier
use crate::precompiles::ASSET_PRECOMPILE_ADDRESS_PREFIX;
use pallet_ethereum::PostLogContent;
use pallet_evm::HashedAddressMapping;
use pallet_evm_precompileset_assets_erc20::AddressToAssetId;
use tangle_primitives::evm::WEIGHT_PER_GAS;
impl pallet_evm_chain_id::Config for Runtime {}

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

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Babe::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.0.to_raw_vec()[4..24]));
		}
		None
	}
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
)]
pub enum ProxyType {
	/// All calls can be proxied. This is the trivial/most permissive filter.
	Any = 0,
	/// Only extrinsics related to governance (democracy and collectives).
	Governance = 1,
	/// Allow to veto an announced proxy call.
	CancelProxy = 2,
	/// Allow extrinsic related to Balances.
	Balances = 3,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

fn is_governance_precompile(precompile_name: &precompiles::PrecompileName) -> bool {
	matches!(
		precompile_name,
		PrecompileName::DemocracyPrecompile | PrecompileName::PreimagePrecompile
	)
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

// Be careful: Each time this filter is modified, the substrate filter must also be modified
// consistently.
impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {
	fn is_evm_proxy_call_allowed(
		&self,
		call: &pallet_evm_precompile_proxy::EvmSubCall,
		recipient_has_code: bool,
		gas: u64,
	) -> precompile_utils::EvmResult<bool> {
		Ok(match self {
			ProxyType::Any => true,
			ProxyType::Governance => {
				call.value == U256::zero()
					&& matches!(
						PrecompileName::from_address(call.to.0),
						Some(ref precompile) if is_governance_precompile(precompile)
					)
			},
			// The proxy precompile does not contain method cancel_proxy
			ProxyType::CancelProxy => false,
			ProxyType::Balances => {
				// Allow only "simple" accounts as recipient (no code nor precompile).
				// Note: Checking the presence of the code is not enough because some precompiles
				// have no code.
				!recipient_has_code
					&& !precompile_utils::precompile_set::is_precompile_or_fail::<Runtime>(
						call.to.0, gas,
					)?
			},
		})
	}
}

parameter_types! {
	/// EVM gas limit
	pub BlockGasLimit: U256 = U256::from(
		NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS
	);
	/// The amount of gas per pov. A ratio of 4 if we convert ref_time to gas and we compare
	/// it with the pov_size for a block. E.g.
	/// ceil(
	///     (max_extrinsic.ref_time() / max_extrinsic.proof_size()) / WEIGHT_PER_GAS
	/// )
	pub const GasLimitPovSizeRatio: u64 = 4;
	pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
	pub PrecompilesValue: TanglePrecompiles<Runtime> = TanglePrecompiles::<_>::new();
	pub SuicideQuickClearLimit: u32 = 0;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = BaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = TanglePrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EVMChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction =
		pallet_evm::EVMCurrencyAdapter<Balances, impls::DealWithFees<Runtime>>;
	type OnCreate = ();
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type FindAuthor = FindAuthorTruncated<Babe>;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

parameter_types! {
	pub BoundDivision: U256 = U256::from(1024);
}

impl pallet_dynamic_fee::Config for Runtime {
	type MinGasPriceBoundDivisor = BoundDivision;
}

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = (MILLIUNIT / 1_000_000).into();
	// At the moment, we don't use dynamic fee calculation by default.
	pub DefaultElasticity: Permill = Permill::zero();
}

impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = BaseFeeThreshold;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

impl pallet_hotfix_sufficients::Config for Runtime {
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type WeightInfo = pallet_hotfix_sufficients::weights::SubstrateWeight<Runtime>;
}
