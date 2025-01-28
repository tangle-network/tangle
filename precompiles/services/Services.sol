// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Services contract's address.
address constant SERVICES_ADDRESS = 0x0000000000000000000000000000000000000900;

/// @dev The Services contract's instance.
Services constant SERVICES_CONTRACT = Services(SERVICES_ADDRESS);

/// @title Pallet Services Interface
/// @dev The interface through which solidity contracts will interact with Services pallet
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000900
interface Services {
    /// @dev Create a new blueprint.
    /// @param blueprint_data The blueprint data in SCALE-encoded format.
    function createBlueprint(bytes calldata blueprint_data) external;

    /// @dev Register as an operator for a specific blueprint.
    /// @param blueprint_id The blueprint ID.
    /// @param preferences The operator preferences in SCALE-encoded format.
    /// @param registration_args The registration arguments in SCALE-encoded format.
    function registerOperator(uint256 blueprint_id, bytes calldata preferences, bytes calldata registration_args) external payable;

<<<<<<< HEAD
	/// @notice Request a service from a specific blueprint
	/// @param blueprint_id The ID of the blueprint
	/// @param assets The list of assets to use for the service
	/// @param permitted_callers_data The permitted callers for the service encoded as bytes
	/// @param service_providers_data The service providers encoded as bytes
	/// @param request_args_data The request arguments encoded as bytes
	/// @param ttl The time-to-live of the service.
	/// @param payment_asset_id The ID of the asset to use for payment (0 for native asset)
	/// @param payment_token_address The address of the token to use for payment (0x0 for using the value of payment_asset_id)
	/// @param payment_amount The amount to pay for the service (use msg.value if payment_asset_id is 0)
	function requestService(
		uint256 blueprint_id,
		uint256[] calldata assets,
		bytes calldata permitted_callers_data,
		bytes calldata service_providers_data,
		bytes calldata request_args_data,
		uint256 ttl,
		uint256 payment_asset_id,
		address payment_token_address,
		uint256 payment_amount,
		uint32 min_operators,
		uint32 max_operators
	) external payable;
=======
    /// @dev Pre-register as an operator for a specific blueprint.
    /// @param blueprint_id The blueprint ID.
    function preRegister(uint256 blueprint_id) external;
>>>>>>> main

    /// @dev Unregister as an operator from a blueprint.
    /// @param blueprint_id The blueprint ID.
    function unregisterOperator(uint256 blueprint_id) external;

    /// @dev Request a new service.
    /// @param blueprint_id The blueprint ID.
    /// @param assets The list of asset IDs.
    /// @param permitted_callers The permitted callers in SCALE-encoded format.
    /// @param service_providers The service providers in SCALE-encoded format.
    /// @param request_args The request arguments in SCALE-encoded format.
    /// @param ttl The time-to-live for the request.
    /// @param payment_asset_id The payment asset ID.
    /// @param payment_token_address The payment token address.
    /// @param amount The payment amount.
    function requestService(
        uint256 blueprint_id,
        uint256[] calldata assets,
        bytes calldata permitted_callers,
        bytes calldata service_providers,
        bytes calldata request_args,
        uint256 ttl,
        uint256 payment_asset_id,
        address payment_token_address,
        uint256 amount
    ) external payable;

    /// @dev Terminate a service.
    /// @param service_id The service ID.
    function terminateService(uint256 service_id) external;

    /// @dev Approve a request.
    /// @param request_id The request ID.
    /// @param restaking_percent The restaking percentage.
    function approve(uint256 request_id, uint8 restaking_percent) external;

    /// @dev Reject a service request.
    /// @param request_id The request ID.
    function reject(uint256 request_id) external;

    /// @dev Call a job in the service.
    /// @param service_id The service ID.
    /// @param job The job ID.
    /// @param args_data The job arguments in SCALE-encoded format.
    function callJob(uint256 service_id, uint8 job, bytes calldata args_data) external;

    /// @dev Submit the result for a job call.
    /// @param service_id The service ID.
    /// @param call_id The call ID.
    /// @param result_data The result data in SCALE-encoded format.
    function submitResult(uint256 service_id, uint256 call_id, bytes calldata result_data) external;

    /// @dev Slash an operator for a service.
    /// @param offender The offender in SCALE-encoded format.
    /// @param service_id The service ID.
    /// @param percent The slash percentage.
    function slash(bytes calldata offender, uint256 service_id, uint8 percent) external;

    /// @dev Dispute an unapplied slash.
    /// @param era The era number.
    /// @param index The index of the slash.
    function dispute(uint32 era, uint32 index) external;

    /// @dev Update price targets for a blueprint.
    /// @param blueprint_id The blueprint ID.
    /// @param price_targets The new price targets.
    function updatePriceTargets(uint256 blueprint_id, uint256[] calldata price_targets) external;

    /// @dev Custom errors for the Services precompile
    error InvalidPermittedCallers();
    error InvalidOperatorsList();
    error InvalidRequestArguments();
    error InvalidTTL();
    error InvalidAmount();
    error ValueMustBeZeroForERC20();
    error ValueMustBeZeroForCustomAsset();
    error PaymentAssetShouldBeCustomOrERC20();
}
