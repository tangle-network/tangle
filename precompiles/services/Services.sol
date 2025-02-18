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
    /// @custom:selector c45a6865
    function createBlueprint(bytes calldata blueprint_data) external;

    /// @notice Request a service from a specific blueprint
    /// @param blueprint_id The ID of the blueprint
    /// @param security_requirements The security requirements for each asset
    /// @param permitted_callers_data The permitted callers for the service encoded as bytes
    /// @param service_providers_data The service providers encoded as bytes
    /// @param request_args_data The request arguments encoded as bytes
    /// @param ttl The time-to-live of the service.
    /// @param payment_asset_id The ID of the asset to use for payment (0 for native asset)
    /// @param payment_token_address The address of the token to use for payment (0x0 for using the value of payment_asset_id)
    /// @param payment_amount The amount to pay for the service (use msg.value if payment_asset_id is 0)
    /// @param min_operators The minimum number of operators required
    /// @param max_operators The maximum number of operators allowed
    /// @custom:selector 7f3c0ee4
    function requestService(
        uint256 blueprint_id,
        bytes[] calldata security_requirements,
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

    /// @dev Terminate a service.
    /// @param service_id The service ID.
    /// @custom:selector b997a71e
    function terminateService(uint256 service_id) external;

    /// @dev Call a job in the service.
    /// @param service_id The service ID.
    /// @param job The job ID.
    /// @param args_data The job arguments in SCALE-encoded format.
    /// @custom:selector fce65e13
    function callJob(uint256 service_id, uint8 job, bytes calldata args_data) external;

    /// @dev Slash an operator for a service.
    /// @param offender The offender in SCALE-encoded format.
    /// @param service_id The service ID.
    /// @param percent The slash percentage (0-100).
    /// @custom:selector 64a798ac
    function slash(bytes calldata offender, uint256 service_id, uint8 percent) external;

    /// @dev Dispute an unapplied slash.
    /// @param era The era number.
    /// @param index The index of the slash.
    /// @custom:selector fac9efa3
    function dispute(uint32 era, uint32 index) external;

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
