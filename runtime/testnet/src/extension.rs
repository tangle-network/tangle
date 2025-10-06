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

//! Runtime extension implementations for testnet.

use frame_support::{pallet_prelude::*, weights::Weight};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, TransactionExtension},
	transaction_validity::{InvalidTransaction, TransactionValidityError, ValidTransaction},
};

use crate::{Balance, Runtime, RuntimeCall};

/// Extension that checks for nominated tokens that are being restaked.
/// Prevents unbonding when tokens are delegated through the multi-asset-delegation system.
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct CheckNominatedRestaked<T>(core::marker::PhantomData<T>);

impl<T> parity_scale_codec::DecodeWithMemTracking for CheckNominatedRestaked<T> {}

impl<T> sp_std::fmt::Debug for CheckNominatedRestaked<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckNominatedRestaked")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T> CheckNominatedRestaked<T> {
	pub fn new() -> Self {
		CheckNominatedRestaked(core::marker::PhantomData)
	}
}

impl CheckNominatedRestaked<Runtime> {
	/// Checks if unbonding is allowed based on delegated nominations
	pub fn can_unbound(
		who: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> bool {
		pallet_multi_asset_delegation::Pallet::<Runtime>::can_unbound(who, amount)
	}
}

impl<T> Default for CheckNominatedRestaked<T> {
	fn default() -> Self {
		CheckNominatedRestaked(core::marker::PhantomData)
	}
}

impl TransactionExtension<RuntimeCall> for CheckNominatedRestaked<Runtime> {
	const IDENTIFIER: &'static str = "CheckNominatedRestaked";
	type Implicit = ();
	type Pre = ();
	type Val = ();

	fn weight(&self, _call: &RuntimeCall) -> Weight {
		Weight::zero()
	}

	fn validate(
		&self,
		origin: <Runtime as frame_system::Config>::RuntimeOrigin,
		call: &RuntimeCall,
		_info: &DispatchInfoOf<RuntimeCall>,
		_len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl Encode,
		_source: sp_runtime::transaction_validity::TransactionSource,
	) -> Result<
		(ValidTransaction, Self::Val, <Runtime as frame_system::Config>::RuntimeOrigin),
		TransactionValidityError,
	> {
		let who = frame_system::ensure_signed(origin.clone())
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))?;

		let result = match call {
			RuntimeCall::Staking(pallet_staking::Call::unbond { value }) => {
				if Self::can_unbound(&who, *value) {
					Ok(ValidTransaction::default())
				} else {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
				}
			},
			RuntimeCall::Proxy(pallet_proxy::Call::proxy { call, real, .. }) =>
				if let sp_runtime::MultiAddress::Id(account_id) = real {
					match call.as_ref() {
						RuntimeCall::Staking(pallet_staking::Call::unbond { value }) =>
							if Self::can_unbound(account_id, *value) {
								Ok(ValidTransaction::default())
							} else {
								Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(
									1,
								)))
							},
						_ => Ok(ValidTransaction::default()),
					}
				} else {
					Ok(ValidTransaction::default())
				},
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) |
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) |
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => {
				for call in calls {
					match call {
						RuntimeCall::Staking(pallet_staking::Call::unbond { value }) =>
							if !Self::can_unbound(&who, *value) {
								return Err(TransactionValidityError::Invalid(
									InvalidTransaction::Custom(1),
								));
							},
						_ => {},
					}
				}
				Ok(ValidTransaction::default())
			},
			_ => Ok(ValidTransaction::default()),
		};

		result.map(|v| (v, (), origin))
	}

	fn prepare(
		self,
		_val: Self::Val,
		_origin: &<Runtime as frame_system::Config>::RuntimeOrigin,
		_call: &RuntimeCall,
		_info: &DispatchInfoOf<RuntimeCall>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		Ok(())
	}
}
