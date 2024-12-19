use super::*;
use crate::types::BalanceOf;
use ethabi::{Function, StateMutability, Token};
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
	pallet_prelude::{Pays, Weight},
};
use parity_scale_codec::Encode;
use scale_info::prelude::string::String;
use sp_core::{H160, U256};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::{vec, vec::Vec};
use tangle_primitives::services::{EvmGasWeightMapping, EvmRunner};

impl<T: Config> Pallet<T> {
	/// Moves a `value` amount of tokens from the caller's account to `to`.
	pub fn erc20_transfer(
		erc20: H160,
		from: &H160,
		to: H160,
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		let transfer_fn = Function {
			name: String::from("transfer"),
			inputs: vec![
				ethabi::Param {
					name: String::from("to"),
					kind: ethabi::ParamType::Address,
					internal_type: None,
				},
				ethabi::Param {
					name: String::from("value"),
					kind: ethabi::ParamType::Uint(256),
					internal_type: None,
				},
			],
			outputs: vec![ethabi::Param {
				name: String::from("success"),
				kind: ethabi::ParamType::Bool,
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::NonPayable,
		};

		let args = [
			Token::Address(to),
			Token::Uint(ethabi::Uint::from(value.using_encoded(U256::from_little_endian))),
		];

		log::debug!(target: "evm", "Dispatching EVM call(0x{}): {}", hex::encode(transfer_fn.short_signature()), transfer_fn.signature());
		let data = transfer_fn.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
		let gas_limit = 300_000;
		let info = Self::evm_call(*from, erc20, U256::zero(), data, gas_limit)?;
		let weight = Self::weight_from_call_info(&info);

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let success = if let Some(data) = maybe_value {
			let result = transfer_fn.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let success = result.first().ok_or(Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Bool(val) = success {
				*val
			} else {
				false
			}
		} else {
			false
		};

		Ok((success, weight))
	}

	/// Dispatches a call to the EVM and returns the result.
	fn evm_call(
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Result<fp_evm::CallInfo, DispatchErrorWithPostInfo> {
		let transactional = true;
		let validate = false;
		let result =
			T::EvmRunner::call(from, to, data.clone(), value, gas_limit, transactional, validate);
		match result {
			Ok(info) => {
				log::debug!(
					target: "evm",
					"Call from: {:?}, to: {:?}, data: 0x{}, gas_limit: {:?}, result: {:?}",
					from,
					to,
					hex::encode(&data),
					gas_limit,
					info,
				);
				// if we have a revert reason, emit an event
				if info.exit_reason.is_revert() {
					log::debug!(
						target: "evm",
						"Call to: {:?} with data: 0x{} Reverted with reason: (0x{})",
						to,
						hex::encode(&data),
						hex::encode(&info.value),
					);
					#[cfg(test)]
					eprintln!(
						"Call to: {:?} with data: 0x{} Reverted with reason: (0x{})",
						to,
						hex::encode(&data),
						hex::encode(&info.value),
					);
					Self::deposit_event(Event::<T>::EvmReverted {
						from,
						to,
						data,
						reason: info.value.clone(),
					});
				}
				Ok(info)
			},
			Err(e) => Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo { actual_weight: Some(e.weight), pays_fee: Pays::Yes },
				error: e.error.into(),
			}),
		}
	}

	/// Convert the gas used in the call info to weight.
	pub fn weight_from_call_info(info: &fp_evm::CallInfo) -> Weight {
		let mut gas_to_weight = T::EvmGasWeightMapping::gas_to_weight(
			info.used_gas.standard.unique_saturated_into(),
			true,
		);
		if let Some(weight_info) = info.weight_info {
			if let Some(proof_size_usage) = weight_info.proof_size_usage {
				*gas_to_weight.proof_size_mut() = proof_size_usage;
			}
		}
		gas_to_weight
	}
}
