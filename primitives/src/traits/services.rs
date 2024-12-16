use scale_info::prelude::vec::Vec;

/// A trait to manage and query services and blueprints for operators.
///
/// This trait defines methods to retrieve information about the number of active
/// services and blueprints for a specific operator, as well as to check if an
/// operator can exit (i.e., has no active services or blueprints).
///
/// # Type Parameters
///
/// * `AccountId`: The type representing an account identifier.
/// * `Balance`: The type representing a balance or amount.
pub trait ServiceManager<AccountId, Balance> {
	/// Get the count of active services for a specific operator.
	///
	/// This method returns the number of active services associated with the
	/// specified operator.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// The count of active services as a `usize`.
	fn get_active_services_count(operator: &AccountId) -> usize;

	/// Get the count of active blueprints for a specific operator.
	///
	/// This method returns the number of active blueprints associated with the
	/// specified operator.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// The count of active blueprints as a `usize`.
	fn get_active_blueprints_count(operator: &AccountId) -> usize;

	/// Get all blueprints for a specific operator.
	///
	/// This method returns a list of blueprints associated with the specified
	/// operator.
	fn get_blueprints_by_operator(operator: &AccountId) -> Vec<crate::BlueprintId>;

	/// Check if the given account ID can exit.
	///
	/// This method checks whether the specified operator can exit, which is
	/// determined by having no active services or blueprints.
	///
	/// # Parameters
	///
	/// * `operator`: A reference to the account identifier of the operator.
	///
	/// # Returns
	///
	/// `true` if the operator can exit, otherwise `false`.
	fn can_exit(operator: &AccountId) -> bool;
}
