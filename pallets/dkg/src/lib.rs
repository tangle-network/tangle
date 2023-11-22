#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

mod functions;
#[cfg(test)]
mod tests;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::{BalanceOf, FeeInfoOf};
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The origin which may set filter.
		type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn fee_info)]
	pub type FeeInfo<T> = StorageValue<_, FeeInfoOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FeeUpdated(FeeInfoOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_fee(origin: OriginFor<T>, fee_info: FeeInfoOf<T>) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;

			// Update storage.
			<FeeInfo<T>>::put(fee_info.clone());

			// Emit an event.
			Self::deposit_event(Event::FeeUpdated(fee_info));
			Ok(().into())
		}
	}
}
