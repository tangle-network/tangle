// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/// @title ServicesPrecompile Interface
/// @dev This interface is meant to interact with the ServicesPrecompile in the Tangle network.
interface ServicesPrecompile {
    /// @notice Create a new service blueprint
    /// @param blueprint_data The blueprint data encoded as bytes
    function createBlueprint(bytes calldata blueprint_data) external;

    /// @notice Register an operator for a specific blueprint
    /// @param blueprint_id The ID of the blueprint to register for
    /// @param preferences The operator's preferences encoded as bytes
    /// @param registration_args The registration arguments encoded as bytes
    function registerOperator(
        uint256 blueprint_id,
        bytes calldata preferences,
        bytes calldata registration_args
    ) external payable;

    /// @notice Unregister an operator from a specific blueprint
    /// @param blueprint_id The ID of the blueprint to unregister from
    function unregisterOperator(uint256 blueprint_id) external;

    /// @notice Request a service from a specific blueprint
    /// @param blueprint_id The ID of the blueprint
    /// @param assets The list of assets to use for the service
    /// @param permitted_callers_data The permitted callers for the service encoded as bytes
    /// @param service_providers_data The service providers encoded as bytes
    /// @param request_args_data The request arguments encoded as bytes
    function requestService(
        uint256 blueprint_id,
        uint256[] calldata assets,
        bytes calldata permitted_callers_data,
        bytes calldata service_providers_data,
        bytes calldata request_args_data
    ) external;

    /// @notice Terminate a service
    /// @param service_id The ID of the service to terminate
    function terminateService(uint256 service_id) external;

    /// @notice Approve a service request
    /// @param request_id The ID of the service request to approve
    /// @param restaking_percent The amount of your restake to be exposed to the service in percentage [0, 100]
    function approve(uint256 request_id, uint8 restaking_percent) external;

    /// @notice Reject a service request
    /// @param request_id The ID of the service request to reject
    function reject(uint256 request_id) external;

    /// @notice Call a job in the service
    /// @param service_id The ID of the service
    /// @param job The job index (as uint8)
    /// @param args_data The arguments of the job encoded as bytes
    function callJob(
        uint256 service_id,
        uint8 job,
        bytes calldata args_data
    ) external;

    /// @notice Submit the result of a job call
    /// @param service_id The ID of the service
    /// @param call_id The ID of the call
    /// @param result_data The result data encoded as bytes
    function submitResult(
        uint256 service_id,
        uint256 call_id,
        bytes calldata result_data
    ) external;

    /// @notice Slash an operator (offender) for a service id with a given percent of their exposed stake for that service.
    ///
    /// The caller needs to be an authorized Slash Origin for this service.
    /// Note that this does not apply the slash directly, but instead schedules a deferred call to apply the slash
    /// by another entity.
    /// @param offender The operator to be slashed encoded as bytes
    /// @param service_id The ID of the service to slash for
    /// @param percent The percent of the offender's exposed stake to slash
    function slash(
        bytes calldata offender,
        uint256 service_id,
        uint8 percent
    ) external;

    /// @notice Dispute an Unapplied Slash for a service id.
    ///
    /// The caller needs to be an authorized Dispute Origin for this service.
    /// @param era The era of the unapplied slash.
    /// @param slash_index The index of the unapplied slash in the era.
    function dispute(uint32 era, uint32 slash_index) external;
}
