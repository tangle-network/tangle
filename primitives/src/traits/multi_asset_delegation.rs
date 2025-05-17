use crate::{
	services::Asset,
	types::{RoundIndex, rewards::UserDepositWithLocks},
};
use sp_std::prelude::*;

/// A trait to provide information about multi-asset delegation.
///
/// This trait defines methods to retrieve information about the delegation
/// state in a multi-asset context, including current round index, operator
/// status, operator stakes, and total delegation amounts for specific assets.
///
/// # Type Parameters
///
/// * `AccountId`: The type representing an account identifier.
/// * `AssetId`: The type representing an asset identifier.
/// * `Balance`: The type representing a balance or amount.
/// * `BlockNumber`: The type representing a block number.
pub trait MultiAssetDelegationInfo<AccountId, Balance, BlockNumber, AssetId, AssetType> {
	/// Get the current round index.
	///
	/// This method returns the current round index, which may be used to track
	/// the period or phase in the delegation lifecycle.
	///
	/// # Returns
	///
	/// The current round index as a `RoundIndex`.
	fn get_current_round() -> RoundIndex;

	/// Check if the given account is an operator.
	///
	/// This method checks whether the provided account identifier corresponds
	/// to an operator.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier to check.
	///
	/// # Returns
	///
	/// `true` if the account is an operator, otherwise `false`.
	fn is_operator(operator: &AccountId) -> bool;

	/// Check if the given operator is active.
	///
	/// This method checks whether the specified operator is currently active.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// `true` if the operator is active, otherwise `false`.
	fn is_operator_active(operator: &AccountId) -> bool;

	/// Get the stake of the given operator.
	///
	/// This method retrieves the self stake amount associated with the specified
	/// operator.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// The stake amount as a `Balance`.
	fn get_operator_stake(operator: &AccountId) -> Balance;

	/// Get the total delegation amount for a specific operator and asset.
	///
	/// This method returns the total amount of delegation that a given operator
	/// has for a specific asset.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	/// * `asset`: A reference to the asset identifier for which the total delegation amount is
	///   requested.
	///
	/// # Returns
	///
	/// The total delegation amount as a `Balance`.
	fn get_total_delegation_by_asset(operator: &AccountId, asset_id: &Asset<AssetId>) -> Balance;

	/// Get all delegators for a specific operator.
	///
	/// This method returns a list of delegators for the specified operator, along
	/// with their delegation amounts and asset identifiers.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// A list of delegators as a vector of tuples, where each tuple contains the
	/// delegator account identifier, delegation amount, and asset identifier.
	fn get_delegators_for_operator(
		operator: &AccountId,
	) -> Vec<(AccountId, Balance, Asset<AssetId>)>;

	/// Get a user's deposit and associated locks for a specific asset.
	///
	/// This method retrieves information about a user's deposit for a given asset,
	/// including both the unlocked amount and any time-locked portions.
	///
	/// # Parameters
	///
	/// * `who`: A reference to the account identifier of the user.
	/// * `asset`: The asset identifier for which to get deposit information.
	///
	/// # Returns
	///
	/// An `Option` containing the user's deposit information if it exists:
	/// - `Some(UserDepositWithLocks)` containing the unlocked amount and any time-locks
	/// - `None` if no deposit exists for this user and asset
	fn get_user_deposit_with_locks(
		who: &AccountId,
		asset: Asset<AssetId>,
	) -> Option<UserDepositWithLocks<Balance, BlockNumber>>;

	/// Get a user's deposit by asset type.
	///
	/// This method retrieves information about a user's deposit for a given asset type,
	/// including both the unlocked amount and any time-locked portions.
	///
	/// # Parameters
	///
	/// * `who`: A reference to the account identifier of the user.
	/// * `asset_type`: The type of asset for which to get deposit information.
	///
	/// # Returns
	///
	/// An `Option` containing the user's deposit information if it exists:
	/// - `Some(UserDepositWithLocks)` containing the unlocked amount and any time-locks
	fn get_user_deposit_by_asset_type(who: &AccountId, asset_type: AssetType) -> Option<Balance>;
}
