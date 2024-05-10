// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "./Runtime.sol";

/// @dev Created by the service blueprint designer (gadget developer) to verify the result of a job.
contract JobResultVerifier is Runtime {
    /// @dev Verifies the result of a job call.
    /// @param serviceId The id of the service.
    /// @param jobIndex The index of the job in the service blueprint.
    /// @param jobCallId The id of the job call.
    /// @param participant The participant that produced the outputs.
    /// @param inputs The inputs that were provided during the job call.
    /// @param outputs The outputs that were produced by the job call.
    ///
    /// Unless this function reverts, the job result will be accepted.
    function verify(
        uint64 serviceId,
        uint8 jobIndex,
        uint64 jobCallId,
        bytes calldata participant,
        bytes calldata inputs,
        bytes calldata outputs
    ) public virtual onlyRuntime {}
}
