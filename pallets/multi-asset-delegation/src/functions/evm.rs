use super::*;
use crate::types::BalanceOf;
use ethabi::{Function, StateMutability, Token};
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::pallet_prelude::Weight;
use sp_core::{H160, U256};
use tangle_primitives::EvmAddressMapping;

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
		let info = Self::evm_call(from, erc20, U256::zero(), data, gas_limit)?;
		let weight = Self::weight_from_call_info(&info);

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let success = if let Some(data) = maybe_value {
			let result = transfer_fn.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let success = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
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
}
