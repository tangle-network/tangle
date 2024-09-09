// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The ServiceBlueprint precompile contract's address.
address constant SERVICE_BLUEPRINT = 0x0000000000000000000000000000000000000819;

/// @dev The ServiceBlueprint contract's instance.
ServiceBlueprint constant SERVICE_BLUEPRINT_CONTRACT = ServiceBlueprint(SERVICE_BLUEPRINT);

/// @author Your Team
/// @title Pallet ServiceBlueprint Interface
/// @title The interface through which Solidity contracts will interact with the ServiceBlueprint pallet
/// @custom:address 0x000000000000000000000000000000000000080A
interface ServiceBlueprint {
    
    /// @dev Create a new blueprint.
    /// @param blueprintData The data related to the service blueprint.
    function createBlueprint(bytes calldata blueprintData) external returns (uint8);

    /// @dev Register as an operator for a specific blueprint.
    /// @param blueprintId The ID of the blueprint to register as an operator.
    /// @param preferences The preferences of the operator.
    /// @param registrationArgs Additional registration arguments.
    function register(
        uint256 blueprintId, 
        bytes calldata preferences, 
        bytes[] calldata registrationArgs
    ) external returns (uint8);

    /// @dev Unregister as an operator for a specific blueprint.
    /// @param blueprintId The ID of the blueprint from which to unregister.
    function unregister(uint256 blueprintId) external returns (uint8);

    /// @dev Request a new service using a blueprint.
    /// @param blueprintId The ID of the blueprint to use for the service.
    /// @param permittedCallers A list of accounts permitted to call the service.
    /// @param serviceProviders A list of service providers for the service.
    /// @param ttl The time-to-live for the service in blocks.
    /// @param requestArgs Additional arguments for the service request.
    function requestService(
        uint256 blueprintId, 
        address[] calldata permittedCallers, 
        address[] calldata serviceProviders, 
        uint256 ttl, 
        bytes[] calldata requestArgs
    ) external returns (uint8);

    /// @dev Call a job in the service.
    /// @param serviceId The ID of the service.
    /// @param job The ID of the job to execute.
    /// @param args The arguments for the job call.
    function callServiceJob(
        uint256 serviceId, 
        uint256 job, 
        bytes[] calldata args
    ) external returns (uint8);

    /// @dev Terminate an existing service.
    /// @param serviceId The ID of the service to terminate.
    function terminateService(uint256 serviceId) external returns (uint8);

    /// @dev Update operator preferences for a blueprint.
    /// @param blueprintId The ID of the blueprint to update preferences for.
    /// @param approvalPreference The new approval preferences.
    function updateApprovalPreference(
        uint256 blueprintId, 
        bytes calldata approvalPreference
    ) external returns (uint8);

    /// @dev Update the price targets for a blueprint.
    /// @param blueprintId The ID of the blueprint to update price targets for.
    /// @param priceTargets The new price targets.
    function updatePriceTargets(
        uint256 blueprintId, 
        bytes calldata priceTargets
    ) external returns (uint8);
}
