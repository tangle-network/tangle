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
    function registerOperator(uint256 blueprint_id, bytes calldata preferences, bytes calldata registration_args) external;

    /// @notice Unregister an operator from a specific blueprint
    /// @param blueprint_id The ID of the blueprint to unregister from
    function unregisterOperator(uint256 blueprint_id) external;

    /// @notice Request a service from a specific blueprint
    /// @param blueprint_id The ID of the blueprint
    /// @param assets The list of assets to use for the service
    /// @param permitted_callers_data The permitted callers for the service encoded as bytes
    /// @param service_providers_data The service providers encoded as bytes
    /// @param request_args_data The request arguments encoded as bytes
    function requestService(uint256 blueprint_id, uint256[] assets, bytes calldata permitted_callers_data, bytes calldata service_providers_data, bytes calldata request_args_data) external;

    /// @notice Terminate a service
    /// @param service_id The ID of the service to terminate
    function terminateService(uint256 service_id) external;

    /// @notice Approve a service request
    /// @param request_id The ID of the service request to approve
    function approve(uint256 request_id) external;

    /// @notice Reject a service request
    /// @param request_id The ID of the service request to reject
    function reject(uint256 request_id) external;

    /// @notice Call a job in the service
    /// @param service_id The ID of the service
    /// @param job The job index (as uint8)
    /// @param args_data The arguments of the job encoded as bytes
    function callJob(uint256 service_id, uint8 job, bytes calldata args_data) external;

    /// @notice Submit the result of a job call
    /// @param service_id The ID of the service
    /// @param call_id The ID of the call
    /// @param result_data The result data encoded as bytes
    function submitResult(uint256 service_id, uint256 call_id, bytes calldata result_data) external;
}
