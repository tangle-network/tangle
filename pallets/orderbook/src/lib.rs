#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod functions;
pub mod types;

use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect, Mutate},
		Currency, LockableCurrency, ReservableCurrency,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	Percent,
};
use sp_std::{fmt::Debug, prelude::*, vec::Vec};
use tangle_primitives::services::Asset;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use tangle_primitives::services::AssetIdT;

	use super::*;
	use crate::types::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism for deposits and fees
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The asset ID type.
		type AssetId: AssetIdT;

		/// The fungibles trait for managing assets
		type Fungibles: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ Mutate<Self::AccountId>;

		/// Origin that can force actions
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Maximum number of orders per user
		#[pallet::constant]
		type MaxOrdersPerUser: Get<u32> + Default + Parameter;

		/// Maximum number of orders per resource type
		#[pallet::constant]
		type MaxOrdersPerResource: Get<u32> + Default + Parameter;

		/// Maximum number of resource types per order
		#[pallet::constant]
		type MaxResourcesPerOrder: Get<u32> + Default + Parameter;

		/// Minimum duration for service requests
		#[pallet::constant]
		type MinServiceDuration: Get<BlockNumberFor<Self>> + Default + Parameter;

		/// Minimum collateral required for operators
		#[pallet::constant]
		type MinOperatorCollateral: Get<BalanceOf<Self>> + Default + Parameter;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // owner
		Blake2_128Concat,
		T::Hash, // order id
		Order<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn resource_orders)]
	pub type ResourceOrders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ResourceType,
		Blake2_128Concat,
		PricePoint<T>,
		BoundedVec<T::Hash, T::MaxOrdersPerResource>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_orders)]
	pub type UserOrders<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxOrdersPerUser>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_offerings)]
	pub type OperatorOfferings<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, ResourceOffering<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new order was placed
		OrderPlaced { owner: T::AccountId, order_id: T::Hash, order_type: OrderType },
		/// An order was cancelled
		OrderCancelled { owner: T::AccountId, order_id: T::Hash },
		/// An order was filled (partially or completely)
		OrderFilled {
			maker: T::AccountId,
			taker: T::AccountId,
			order_id: T::Hash,
			amount: BalanceOf<T>,
			remaining: BalanceOf<T>,
		},
		/// A match was made between orders
		OrdersMatched {
			maker_id: T::Hash,
			taker_id: T::Hash,
			resource_type: ResourceType,
			amount: BalanceOf<T>,
			price: BalanceOf<T>,
		},
		/// An operator registered their resource offering
		OperatorRegistered { operator_id: T::AccountId, collateral: BalanceOf<T> },
		/// An operator updated their resource offering
		OperatorUpdated { operator_id: T::AccountId, collateral: BalanceOf<T> },
		/// An operator was unregistered
		OperatorUnregistered { operator_id: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Order not found
		OrderNotFound,
		/// Invalid order parameters
		InvalidOrderParameters,
		/// Order already exists
		OrderAlreadyExists,
		/// Insufficient balance
		InsufficientBalance,
		/// Too many orders for this user
		TooManyOrdersForUser,
		/// Too many orders for this resource type
		TooManyOrdersForResource,
		/// Too many resources in order
		TooManyResources,
		/// Order owner mismatch
		NotOrderOwner,
		/// Order cannot be filled
		OrderCannotBeFilled,
		/// Price mismatch between orders
		PriceMismatch,
		/// Resource type mismatch
		ResourceTypeMismatch,
		/// Invalid service duration
		InvalidServiceDuration,
		/// Insufficient collateral
		InsufficientCollateral,
		/// Operator not registered
		OperatorNotRegistered,
		/// Operator already registered
		OperatorAlreadyRegistered,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Place a new bid order for compute resources
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn place_bid(
			origin: OriginFor<T>,
			resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
			total_amount: BalanceOf<T>,
			operator_id: Option<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_place_order(who, OrderType::Bid, resources, total_amount, operator_id)?;

			Ok(())
		}

		/// Place a new ask order for compute resources
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn place_ask(
			origin: OriginFor<T>,
			resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
			total_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure operator is registered
			ensure!(OperatorOfferings::<T>::contains_key(&who), Error::<T>::OperatorNotRegistered);

			Self::do_place_order(who.clone(), OrderType::Ask, resources, total_amount, Some(who))?;

			Ok(())
		}

		/// Cancel an existing order
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>, order_id: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_cancel_order(who, order_id)
		}

		/// Register as an operator with resource offering
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn register_operator(
			origin: OriginFor<T>,
			resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
			min_duration: BlockNumberFor<T>,
			collateral: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate parameters
			ensure!(
				!OperatorOfferings::<T>::contains_key(&who),
				Error::<T>::OperatorAlreadyRegistered
			);
			ensure!(
				min_duration >= T::MinServiceDuration::get(),
				Error::<T>::InvalidServiceDuration
			);
			ensure!(
				collateral >= T::MinOperatorCollateral::get(),
				Error::<T>::InsufficientCollateral
			);

			// Ensure all resource types are provided
			let mut provided_resources = sp_std::collections::btree_map::BTreeMap::new();
			for resource in resources.iter() {
				provided_resources.insert(resource.resource_type.clone(), true);
			}
			for required_type in ResourceType::all() {
				ensure!(
					provided_resources.contains_key(&required_type),
					Error::<T>::InvalidOrderParameters
				);
			}

			// Reserve collateral
			T::Currency::reserve(&who, collateral)?;

			// Store operator offering
			let offering =
				ResourceOffering { operator_id: who.clone(), resources, min_duration, collateral };
			OperatorOfferings::<T>::insert(&who, offering);

			Self::deposit_event(Event::OperatorRegistered { operator_id: who, collateral });

			Ok(())
		}

		/// Update operator resource offering
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn update_operator(
			origin: OriginFor<T>,
			resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
			min_duration: BlockNumberFor<T>,
			new_collateral: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate operator exists
			let mut offering =
				OperatorOfferings::<T>::get(&who).ok_or(Error::<T>::OperatorNotRegistered)?;

			// Validate parameters
			ensure!(
				min_duration >= T::MinServiceDuration::get(),
				Error::<T>::InvalidServiceDuration
			);

			// Ensure all resource types are provided
			let mut provided_resources = sp_std::collections::btree_map::BTreeMap::new();
			for resource in resources.iter() {
				provided_resources.insert(resource.resource_type.clone(), true);
			}
			for required_type in ResourceType::all() {
				ensure!(
					provided_resources.contains_key(&required_type),
					Error::<T>::InvalidOrderParameters
				);
			}

			// Handle collateral update if provided
			if let Some(new_amount) = new_collateral {
				ensure!(
					new_amount >= T::MinOperatorCollateral::get(),
					Error::<T>::InsufficientCollateral
				);

				// Unreserve old collateral and reserve new amount
				T::Currency::unreserve(&who, offering.collateral);
				T::Currency::reserve(&who, new_amount)?;
				offering.collateral = new_amount;
			}

			// Update offering
			offering.resources = resources;
			offering.min_duration = min_duration;
			OperatorOfferings::<T>::insert(&who, offering.clone());

			Self::deposit_event(Event::OperatorUpdated {
				operator_id: who,
				collateral: offering.collateral,
			});

			Ok(())
		}

		/// Unregister operator and withdraw collateral
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn unregister_operator(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get and remove offering
			let offering =
				OperatorOfferings::<T>::take(&who).ok_or(Error::<T>::OperatorNotRegistered)?;

			// Unreserve collateral
			T::Currency::unreserve(&who, offering.collateral);

			Self::deposit_event(Event::OperatorUnregistered { operator_id: who });

			Ok(())
		}
	}
}
