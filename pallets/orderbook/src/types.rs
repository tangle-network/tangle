use super::*;
use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
	Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen,
)]
pub enum OrderType {
	/// A bid order from a service requester
	Bid,
	/// An ask order from a service provider
	Ask,
}

#[derive(
	Clone, Encode, Decode, Eq, PartialEq, PartialOrd, Ord, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub enum ResourceType {
	Compute,
	Memory,
	Storage,
	Network,
}

impl ResourceType {
	pub fn all() -> Vec<ResourceType> {
		vec![
			ResourceType::Compute,
			ResourceType::Memory,
			ResourceType::Storage,
			ResourceType::Network,
		]
	}
}

#[derive(
	Clone, Encode, Decode, Eq, PartialEq, PartialOrd, Ord, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct PricePoint<T: Config> {
	pub resource_type: ResourceType,
	pub price: BalanceOf<T>,
}

#[derive(
	Clone, Encode, Decode, Eq, PartialEq, PartialOrd, Ord, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct Order<T: Config> {
	pub owner: T::AccountId,
	pub order_type: OrderType,
	/// For Ask orders, must include all resource types
	pub resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
	pub total_amount: BalanceOf<T>,
	pub filled_amount: BalanceOf<T>,
	pub created_at: BlockNumberFor<T>,
	/// For Ask orders, this is the operator ID that must match across all resources
	pub operator_id: Option<T::AccountId>,
}

#[derive(
	Clone,
	Encode,
	Decode,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
	RuntimeDebugNoBound,
)]
#[scale_info(skip_type_params(T))]
pub struct ResourceRequest<T: Config> {
	pub resource_type: ResourceType,
	pub amount: BalanceOf<T>,
	pub price: BalanceOf<T>,
}

/// Represents a complete match across all resource types
#[derive(
	Clone,
	Encode,
	Decode,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	MaxEncodedLen,
	TypeInfo,
	RuntimeDebugNoBound,
)]
#[scale_info(skip_type_params(T))]
pub struct OrderMatch<T: Config> {
	pub maker_id: T::Hash,
	pub maker_owner: T::AccountId,
	pub taker_id: T::Hash,
	pub taker_owner: T::AccountId,
	pub resources: Vec<(ResourceType, BalanceOf<T>, BalanceOf<T>)>, // (type, amount, price)
	pub total_price: BalanceOf<T>,
}

/// Represents a complete service request with all required resources
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ServiceRequest<T: Config> {
	pub resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
	pub operator_id: Option<T::AccountId>,
	pub min_duration: BlockNumberFor<T>,
	pub max_price: BalanceOf<T>,
}

/// Represents an operator's resource offering
#[derive(
	Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, PartialOrd, Ord, RuntimeDebug, TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct ResourceOffering<T: Config> {
	pub operator_id: T::AccountId,
	pub resources: BoundedVec<ResourceRequest<T>, T::MaxResourcesPerOrder>,
	pub min_duration: BlockNumberFor<T>,
	pub collateral: BalanceOf<T>,
}
