use frame_support::pallet_prelude::*;
use mock::{AccountId, Runtime, RuntimeCall};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::{DispatchInfoOf, SignedExtension};
use types::BalanceOf;

use super::*;

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

impl SignedExtension for CheckNominatedRestaked<Runtime> {
	const IDENTIFIER: &'static str = "CheckNominatedRestaked";

	type AccountId = AccountId;

	type Call = RuntimeCall;

	type AdditionalSigned = ();

	type Pre = ();

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
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
			RuntimeCall::Staking(pallet_staking::Call::unbond { value }) => {
				if Self::can_unbound(who, *value) {
					Ok(ValidTransaction::default())
				} else {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))
				}
			},
			RuntimeCall::Proxy(pallet_proxy::Call::proxy { ref call, real, .. }) => {
				self.validate(real, call, _info, _len)
			},
			RuntimeCall::Utility(pallet_utility::Call::batch { ref calls })
			| RuntimeCall::Utility(pallet_utility::Call::batch_all { ref calls })
			| RuntimeCall::Utility(pallet_utility::Call::force_batch { ref calls }) => {
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
