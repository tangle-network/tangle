// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

use crate as pallet_services;
use crate::mock::{
	AccountId, Balances, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Timestamp,
};
use ethereum::TransactionAction;
use fp_ethereum::Transaction;
use fp_evm::FeeCalculator;
use frame_support::traits::{
	Currency, ExistenceRequirement, Imbalance, OnUnbalanced, SignedImbalance, WithdrawReasons,
};
use frame_support::{parameter_types, traits::FindAuthor, weights::Weight, PalletId};
use pallet_ethereum::{EthereumBlockHashMapping, IntermediateStateRoot, PostLogContent, RawOrigin};
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, HashedAddressMapping,
	OnChargeEVMTransaction,
};
use sp_core::{keccak_256, ConstU32, H160, H256, U256};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::{
	traits::{BlakeTwo256, DispatchInfoOf, Dispatchable},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	ConsensusEngineId,
};

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
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
		who: &H160,
		fee: U256,
	) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
		Ok(())
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		already_withdrawn
	}

	fn pay_priority_fee(tip: Self::LiquidityInfo) {}
}

/// Type alias for negative imbalance during fees
type NegativeImbalanceOf<C> =
	<C as Currency<<Runtime as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Implements the transaction payment for a pallet implementing the `Currency`
/// trait (eg. the pallet_balances) using an unbalance handler (implementing
/// `OnUnbalanced`).
/// Similar to `CurrencyAdapter` of `pallet_transaction_payment`
pub struct CustomEVMCurrencyAdapter<C, OU>(sp_std::marker::PhantomData<(C, OU)>);

// See:
// https://github.com/polkadot-evm/frontier/blob/c5d6daa9ffd46c0a85915526aa26d200fd635e30/frame/evm/src/lib.rs#L946
impl<C, OU> OnChargeEVMTransaction<Runtime> for CustomEVMCurrencyAdapter<C, OU>
where
	C: Currency<<Runtime as frame_system::Config>::AccountId>,
	C::PositiveImbalance: Imbalance<
		<C as Currency<<Runtime as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as Currency<<Runtime as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnUnbalanced<NegativeImbalanceOf<C>>,
	U256:
		UniqueSaturatedInto<<C as Currency<<Runtime as frame_system::Config>::AccountId>>::Balance>,
{
	// Kept type as Option to satisfy bound of Default
	type LiquidityInfo = Option<NegativeImbalanceOf<C>>;

	fn withdraw_fee(
		who: &H160,
		fee: U256,
	) -> Result<Self::LiquidityInfo, pallet_evm::Error<Runtime>> {
		if fee.is_zero() {
			return Ok(None);
		}
		let account_id = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(*who);
		let imbalance = C::withdraw(
			&account_id,
			fee.unique_saturated_into(),
			WithdrawReasons::FEE,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| pallet_evm::Error::<Runtime>::BalanceLow)?;
		Ok(Some(imbalance))
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		if let Some(paid) = already_withdrawn {
			let account_id = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(*who);

			// Calculate how much refund we should return
			let refund_amount = paid.peek().saturating_sub(corrected_fee.unique_saturated_into());
			// refund to the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = C::deposit_into_existing(&account_id, refund_amount)
				.unwrap_or_else(|_| C::PositiveImbalance::zero());

			// Make sure this works with 0 ExistentialDeposit
			// https://github.com/paritytech/substrate/issues/10117
			// If we tried to refund something, the account still empty and the ED is set to 0,
			// we call `make_free_balance_be` with the refunded amount.
			let refund_imbalance = if C::minimum_balance().is_zero()
				&& refund_amount > C::Balance::zero()
				&& C::total_balance(&account_id).is_zero()
			{
				// Known bug: Substrate tried to refund to a zeroed AccountData, but
				// interpreted the account to not exist.
				match C::make_free_balance_be(&account_id, refund_amount) {
					SignedImbalance::Positive(p) => p,
					_ => C::PositiveImbalance::zero(),
				}
			} else {
				refund_imbalance
			};

			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid = paid
				.offset(refund_imbalance)
				.same()
				.unwrap_or_else(|_| C::NegativeImbalance::zero());

			let (base_fee, tip) = adjusted_paid.split(base_fee.unique_saturated_into());
			// Handle base fee. Can be either burned, rationed, etc ...
			OU::on_unbalanced(base_fee);
			return Some(tip);
		}
		None
	}

	fn pay_priority_fee(tip: Self::LiquidityInfo) {
		// Default Ethereum behaviour: issue the tip to the block author.
		if let Some(tip) = tip {
			let account_id = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
				<pallet_evm::Pallet<Runtime>>::find_author(),
			);
			let _ = C::deposit_into_existing(&account_id, tip.peek());
		}
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
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = FreeEVMExecution;
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

impl pallet_services::EvmRunner<Runtime> for MockedEvmRunner {
	type Error = pallet_evm::Error<Runtime>;

	fn call(
		source: sp_core::H160,
		target: sp_core::H160,
		input: Vec<u8>,
		value: sp_core::U256,
		gas_limit: u64,
		is_transactional: bool,
		validate: bool,
	) -> Result<fp_evm::CallInfo, pallet_services::traits::RunnerError<Self::Error>> {
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
		.map_err(|o| pallet_services::traits::RunnerError { error: o.error, weight: o.weight })
	}
}

pub struct AccountInfo {
	pub address: H160,
	pub account_id: AccountId,
	pub private_key: H256,
}

pub fn address_build(seed: u8) -> AccountInfo {
	let private_key = H256::from_slice(&[(seed + 1); 32]); //H256::from_low_u64_be((i + 1) as u64);
	let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
	let public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let address = H160::from(H256::from(keccak_256(public_key)));

	AccountInfo {
		private_key,
		account_id: HashedAddressMapping::<BlakeTwo256>::into_account_id(address),
		address,
	}
}

pub struct EIP1559UnsignedTransaction {
	pub nonce: U256,
	pub max_priority_fee_per_gas: U256,
	pub max_fee_per_gas: U256,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

impl EIP1559UnsignedTransaction {
	pub fn sign(&self, secret: &H256, chain_id: Option<u64>) -> Transaction {
		let secret = {
			let mut sk: [u8; 32] = [0u8; 32];
			sk.copy_from_slice(&secret[0..]);
			libsecp256k1::SecretKey::parse(&sk).unwrap()
		};
		let chain_id = chain_id.unwrap_or(ChainId::get());
		let msg = ethereum::EIP1559TransactionMessage {
			chain_id,
			nonce: self.nonce,
			max_priority_fee_per_gas: self.max_priority_fee_per_gas,
			max_fee_per_gas: self.max_fee_per_gas,
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.clone(),
			access_list: vec![],
		};
		let signing_message = libsecp256k1::Message::parse_slice(&msg.hash()[..]).unwrap();

		let (signature, recid) = libsecp256k1::sign(&signing_message, &secret);
		let rs = signature.serialize();
		let r = H256::from_slice(&rs[0..32]);
		let s = H256::from_slice(&rs[32..64]);
		Transaction::EIP1559(ethereum::EIP1559Transaction {
			chain_id: msg.chain_id,
			nonce: msg.nonce,
			max_priority_fee_per_gas: msg.max_priority_fee_per_gas,
			max_fee_per_gas: msg.max_fee_per_gas,
			gas_limit: msg.gas_limit,
			action: msg.action,
			value: msg.value,
			input: msg.input.clone(),
			access_list: msg.access_list,
			odd_y_parity: recid.serialize() != 0,
			r,
			s,
		})
	}
}
