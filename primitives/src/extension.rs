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

//! Extensions for Tangle runtime.

/// Macro to create a CheckNominatedRestaked SignedExtension.
/// This extension checks for nominated tokens that are being restaked and prevents unbonding
/// when tokens are delegated through the multi-asset-delegation system.
#[macro_export]
macro_rules! impl_check_nominated_restaked {
	($runtime:ident) => {
		use frame_support::pallet_prelude::*;
		use parity_scale_codec::{Decode, Encode};
		use scale_info::TypeInfo;
		use sp_runtime::traits::{DispatchInfoOf, SignedExtension};
		use $crate::types::Balance;

		/// Extension that checks for nominated tokens that are being restaked.
		/// Prevents unbonding when tokens are delegated through the multi-asset-delegation system.
		#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
		#[scale_info(skip_type_params(T))]
		pub struct CheckNominatedRestaked<T>(core::marker::PhantomData<T>);

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

		impl CheckNominatedRestaked<$runtime> {
			/// Checks if unbonding is allowed based on delegated nominations
			pub fn can_unbound(
				who: &<$runtime as frame_system::Config>::AccountId,
				amount: Balance,
			) -> bool {
				pallet_multi_asset_delegation::Pallet::<$runtime>::can_unbound(who, amount)
			}
		}

		impl<T> Default for CheckNominatedRestaked<T> {
			fn default() -> Self {
				CheckNominatedRestaked(core::marker::PhantomData)
			}
		}

		impl SignedExtension for CheckNominatedRestaked<$runtime> {
			const IDENTIFIER: &'static str = "CheckNominatedRestaked";

			type AccountId = <$runtime as frame_system::Config>::AccountId;

			type Call = <$runtime as frame_system::Config>::RuntimeCall;

			type AdditionalSigned = ();

			type Pre = ();

			fn additional_signed(
				&self,
			) -> Result<Self::AdditionalSigned, TransactionValidityError> {
				Ok(())
			}

			fn validate(
				&self,
				who: &Self::AccountId,
				call: &Self::Call,
				_info: &DispatchInfoOf<Self::Call>,
				_len: usize,
			) -> TransactionValidity {
				match call {
					<$runtime as frame_system::Config>::RuntimeCall::Staking(
						pallet_staking::Call::unbond { value },
					) => {
						if Self::can_unbound(who, *value) {
							Ok(ValidTransaction::default())
						} else {
							Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
						}
					},
					<$runtime as frame_system::Config>::RuntimeCall::Proxy(
						pallet_proxy::Call::proxy { ref call, real, .. },
					) => self.validate(real, call, _info, _len),
					<$runtime as frame_system::Config>::RuntimeCall::Utility(
						pallet_utility::Call::batch { ref calls },
					)
					| <$runtime as frame_system::Config>::RuntimeCall::Utility(
						pallet_utility::Call::batch_all { ref calls },
					)
					| <$runtime as frame_system::Config>::RuntimeCall::Utility(
						pallet_utility::Call::force_batch { ref calls },
					) => {
						for call in calls {
							self.validate(who, call, _info, _len)?;
						}
						Ok(ValidTransaction::default())
					},
					_ => Ok(ValidTransaction::default()),
				}
			}

			fn pre_dispatch(
				self,
				who: &Self::AccountId,
				call: &Self::Call,
				info: &DispatchInfoOf<Self::Call>,
				len: usize,
			) -> Result<Self::Pre, TransactionValidityError> {
				self.validate(who, call, info, len).map(|_| ())
			}
		}
	};
}
