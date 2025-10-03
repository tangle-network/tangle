use frame_support::weights::Weight;
use mock::{Runtime, RuntimeCall};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, TransactionExtension},
	transaction_validity::{InvalidTransaction, TransactionValidityError, ValidTransaction},
};
use types::BalanceOf;

use super::*;

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

impl<T: Config> CheckNominatedRestaked<T> {
	/// See [`crate::Pallet::can_unbound`]
	pub fn can_unbound(who: &T::AccountId, amount: BalanceOf<T>) -> bool {
		crate::Pallet::<T>::can_unbound(who, amount)
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
		
		match call {
			RuntimeCall::Staking(pallet_staking::Call::unbond { value }) => {
				if Self::can_unbound(&who, *value) {
					Ok((ValidTransaction::default(), (), origin))
				} else {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
				}
			},
			RuntimeCall::Proxy(pallet_proxy::Call::proxy { call, real, .. }) => {
				let real_who = real.clone();
				match call.as_ref() {
					RuntimeCall::Staking(pallet_staking::Call::unbond { value }) => {
						if Self::can_unbound(&real_who, *value) {
							Ok((ValidTransaction::default(), (), origin))
						} else {
							Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
						}
					},
					_ => Ok((ValidTransaction::default(), (), origin)),
				}
			},
			RuntimeCall::Utility(pallet_utility::Call::batch { calls }) |
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) |
			RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => {
				for call in calls {
					match call {
						RuntimeCall::Staking(pallet_staking::Call::unbond { value }) => {
							if !Self::can_unbound(&who, *value) {
								return Err(TransactionValidityError::Invalid(
									InvalidTransaction::Custom(1),
								));
							}
						},
						_ => {},
					}
				}
				Ok((ValidTransaction::default(), (), origin))
			},
			_ => Ok((ValidTransaction::default(), (), origin)),
		}
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
