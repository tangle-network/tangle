use super::*;
use crate::types::*;
use frame_support::{ensure, traits::Currency};
use sp_runtime::{
	traits::{CheckedAdd, CheckedMul, Hash, Zero},
	ArithmeticError, Saturating,
};
use sp_std::collections::btree_map::BTreeMap;

impl<T: Config> Pallet<T> {
	/// Place a new order in the orderbook
	pub(crate) fn do_place_order(
		owner: T::AccountId,
		order_type: OrderType,
		resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
		total_amount: BalanceOf<T>,
		operator_id: Option<T::AccountId>,
	) -> Result<T::Hash, DispatchError> {
		// Validate order parameters
		ensure!(!resources.is_empty(), Error::<T>::InvalidOrderParameters);
		ensure!(!total_amount.is_zero(), Error::<T>::InvalidOrderParameters);

		// For Ask orders, ensure all resource types are provided
		if matches!(order_type, OrderType::Ask) {
			let mut provided_resources = BTreeMap::new();
			for resource in resources.iter() {
				provided_resources.insert(resource.resource_type.clone(), true);
			}

			for required_type in ResourceType::all() {
				ensure!(
					provided_resources.contains_key(&required_type),
					Error::<T>::InvalidOrderParameters
				);
			}
		}

		// Check user order limits
		let mut user_orders = UserOrders::<T>::get(&owner);
		ensure!(
			user_orders.len() < T::MaxOrdersPerUser::get() as usize,
			Error::<T>::TooManyOrdersForUser
		);

		// Create order
		let now = frame_system::Pallet::<T>::block_number();
		let order = Order {
			owner: owner.clone(),
			order_type: order_type.clone(),
			resources: resources.clone(),
			total_amount,
			filled_amount: Zero::zero(),
			created_at: now,
			operator_id,
		};

		// Generate unique order ID
		let order_id = T::Hashing::hash_of(&(owner.clone(), now, resources.clone()));

		// Store order
		Orders::<T>::insert(owner.clone(), order_id, order);

		// Update user orders
		user_orders.try_push(order_id).map_err(|_| Error::<T>::TooManyOrdersForUser)?;
		UserOrders::<T>::insert(&owner, user_orders);

		// Update resource orders
		for resource in resources.iter() {
			let price_point =
				PricePoint { resource_type: resource.resource_type.clone(), price: resource.price };

			let mut resource_orders =
				ResourceOrders::<T>::get(resource.resource_type.clone(), price_point.clone());

			ensure!(
				resource_orders.len() < T::MaxOrdersPerResource::get() as usize,
				Error::<T>::TooManyOrdersForResource
			);

			resource_orders
				.try_push(order_id)
				.map_err(|_| Error::<T>::TooManyOrdersForResource)?;
			ResourceOrders::<T>::insert(
				resource.resource_type.clone(),
				price_point,
				resource_orders,
			);
		}

		// Reserve funds for bid orders
		if matches!(order_type.clone(), OrderType::Bid) {
			T::Currency::reserve(&owner, total_amount)?;
		}

		Self::deposit_event(Event::OrderPlaced {
			owner: owner.clone(),
			order_id,
			order_type: order_type.clone(),
		});

		// Try to match orders immediately
		if matches!(order_type, OrderType::Bid) {
			Self::do_try_match_orders(owner, order_id)?;
		}

		Ok(order_id)
	}

	/// Cancel an existing order
	pub(crate) fn do_cancel_order(
		owner: T::AccountId,
		order_id: T::Hash,
	) -> Result<(), DispatchError> {
		// Get and validate order
		let order = Orders::<T>::get(&owner, order_id).ok_or(Error::<T>::OrderNotFound)?;
		ensure!(order.owner == owner, Error::<T>::NotOrderOwner);

		// Remove order from storage
		Orders::<T>::remove(&owner, order_id);

		// Update user orders
		let mut user_orders = UserOrders::<T>::get(&owner);
		user_orders.retain(|id| *id != order_id);
		UserOrders::<T>::insert(&owner, user_orders);

		// Update resource orders
		for resource in order.resources.iter() {
			let price_point =
				PricePoint { resource_type: resource.resource_type.clone(), price: resource.price };

			let mut resource_orders =
				ResourceOrders::<T>::get(resource.resource_type.clone(), price_point.clone());
			resource_orders.retain(|id| *id != order_id);
			ResourceOrders::<T>::insert(
				resource.resource_type.clone(),
				price_point,
				resource_orders,
			);
		}

		// Unreserve funds for bid orders
		if matches!(order.order_type, OrderType::Bid) {
			let remaining = order.total_amount.saturating_sub(order.filled_amount);
			T::Currency::unreserve(&owner, remaining);
		}

		Self::deposit_event(Event::OrderCancelled { owner, order_id });

		Ok(())
	}

	/// Find valid matches for all resources from the same operator
	fn find_matching_operator_orders(
		taker: &Order<T>,
		resource_type: &ResourceType,
		price_point: &PricePoint<T>,
	) -> Result<Option<OrderMatch<T>>, DispatchError> {
		let resource_orders = ResourceOrders::<T>::get(resource_type.clone(), price_point.clone());

		for maker_id in resource_orders.iter() {
			// Find maker order
			for (maker_owner, t_maker_id, maker_order) in Orders::<T>::iter() {
				if t_maker_id != *maker_id {
					continue;
				}

				// Validate order types match
				if maker_order.order_type == taker.order_type {
					continue;
				}

				// For each resource in taker's request, verify maker can provide it
				let mut all_resources_match = true;
				let mut matched_resources = Vec::new();
				let mut total_price: BalanceOf<T> = Zero::zero();

				for taker_resource in taker.resources.iter() {
					let maker_resource = maker_order
						.resources
						.iter()
						.find(|r| r.resource_type == taker_resource.resource_type);

					match maker_resource {
						Some(resource) => {
							let maker_remaining =
								maker_order.total_amount.saturating_sub(maker_order.filled_amount);
							let taker_remaining =
								taker.total_amount.saturating_sub(taker.filled_amount);
							let amount = maker_remaining.min(taker_remaining);

							if amount.is_zero() {
								all_resources_match = false;
								break;
							}

							let price: BalanceOf<T> = resource.price.into();
							total_price = total_price.saturating_add(amount.saturating_mul(price));

							matched_resources.push((resource.resource_type.clone(), amount, price));
						},
						None => {
							all_resources_match = false;
							break;
						},
					}
				}

				if all_resources_match {
					return Ok(Some(OrderMatch {
						maker_id: *maker_id,
						maker_owner: maker_owner.clone(),
						taker_id: T::Hashing::hash_of(&taker),
						taker_owner: taker.owner.clone(),
						resources: matched_resources,
						total_price,
					}));
				}
			}
		}

		Ok(None)
	}

	/// Try to match and fill orders atomically across all resources
	pub(crate) fn do_try_match_orders(
		taker_owner: T::AccountId,
		taker_id: T::Hash,
	) -> Result<(), DispatchError> {
		let taker = Orders::<T>::get(&taker_owner, taker_id).ok_or(Error::<T>::OrderNotFound)?;

		// For each resource in the taker's order
		for resource in taker.resources.iter() {
			let price_point =
				PricePoint { resource_type: resource.resource_type.clone(), price: resource.price };

			// Try to find matching orders from operators that can provide all resources
			if let Some(match_info) =
				Self::find_matching_operator_orders(&taker, &resource.resource_type, &price_point)?
			{
				// Execute the match atomically
				Self::execute_match(match_info)?;
			}
		}

		Ok(())
	}

	/// Execute a match atomically across all resources
	fn execute_match(match_info: OrderMatch<T>) -> Result<(), DispatchError> {
		let maker = Orders::<T>::get(&match_info.maker_owner, match_info.maker_id)
			.ok_or(Error::<T>::OrderNotFound)?;
		let mut taker = Orders::<T>::get(&match_info.taker_owner, match_info.taker_id)
			.ok_or(Error::<T>::OrderNotFound)?;

		// Update orders
		for (resource_type, amount, price) in match_info.resources.iter() {
			// Update maker order
			let mut updated_maker = maker.clone();
			updated_maker.filled_amount = updated_maker
				.filled_amount
				.checked_add(amount)
				.ok_or(ArithmeticError::Overflow)?;
			Orders::<T>::insert(
				match_info.maker_owner.clone(),
				match_info.maker_id,
				updated_maker.clone(),
			);

			// Update taker order
			taker.filled_amount =
				taker.filled_amount.checked_add(amount).ok_or(ArithmeticError::Overflow)?;

			// Handle currency transfers
			match taker.order_type {
				OrderType::Bid => {
					// Transfer from taker to maker
					let transfer_amount =
						amount.checked_mul(price).ok_or(ArithmeticError::Overflow)?;
					T::Currency::unreserve(&match_info.taker_owner, transfer_amount);
					T::Currency::transfer(
						&match_info.taker_owner,
						&match_info.maker_owner,
						transfer_amount,
						frame_support::traits::ExistenceRequirement::KeepAlive,
					)?;
				},
				OrderType::Ask => {
					// Transfer from maker to taker
					let transfer_amount =
						amount.checked_mul(price).ok_or(ArithmeticError::Overflow)?;
					T::Currency::unreserve(&match_info.maker_owner, transfer_amount);
					T::Currency::transfer(
						&match_info.maker_owner,
						&match_info.taker_owner,
						transfer_amount,
						frame_support::traits::ExistenceRequirement::KeepAlive,
					)?;
				},
			}

			// Emit events
			Self::deposit_event(Event::OrderFilled {
				maker: match_info.maker_owner.clone(),
				taker: match_info.taker_owner.clone(),
				order_id: match_info.maker_id,
				amount: *amount,
				remaining: maker.total_amount.saturating_sub(updated_maker.filled_amount),
			});

			Self::deposit_event(Event::OrdersMatched {
				maker_id: match_info.maker_id,
				taker_id: match_info.taker_id,
				resource_type: resource_type.clone(),
				amount: *amount,
				price: *price,
			});
		}

		// Update taker order
		Orders::<T>::insert(match_info.taker_owner.clone(), match_info.taker_id, taker.clone());

		// Clean up filled orders
		if maker.filled_amount == maker.total_amount {
			Self::do_cancel_order(match_info.maker_owner, match_info.maker_id)?;
		}
		if taker.filled_amount == taker.total_amount {
			Self::do_cancel_order(match_info.taker_owner, match_info.taker_id)?;
		}

		Ok(())
	}
}
