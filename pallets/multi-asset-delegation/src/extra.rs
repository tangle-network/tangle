use frame_support::{traits::IsSubType, weights::Weight};
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

impl<T: Config + pallet_staking::Config + Send + Sync>
	TransactionExtension<<T as frame_system::Config>::RuntimeCall> for CheckNominatedRestaked<T>
where
	<T as frame_system::Config>::RuntimeCall:
		IsSubType<Call<T>> + IsSubType<pallet_staking::Call<T>>,
{
	const IDENTIFIER: &'static str = "CheckNominatedRestaked";
	type Implicit = ();
	type Pre = ();
	type Val = ();

	fn weight(&self, _call: &<T as frame_system::Config>::RuntimeCall) -> Weight {
		Weight::zero()
	}

	fn validate(
		&self,
		origin: <T as frame_system::Config>::RuntimeOrigin,
		_call: &<T as frame_system::Config>::RuntimeCall,
		_info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		_len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl Encode,
		_source: sp_runtime::transaction_validity::TransactionSource,
	) -> Result<
		(ValidTransaction, Self::Val, <T as frame_system::Config>::RuntimeOrigin),
		TransactionValidityError,
	> {
		Ok((ValidTransaction::default(), (), origin))
	}

	fn prepare(
		self,
		_val: Self::Val,
		_origin: &<T as frame_system::Config>::RuntimeOrigin,
		_call: &<T as frame_system::Config>::RuntimeCall,
		_info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		Ok(())
	}
}
