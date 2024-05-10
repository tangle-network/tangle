// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev Created by the service blueprint designer (gadget developer)
contract RequestHook {
    /// @dev Only allow the runtime to call this function.
    modifier onlyRuntime() {
      require(msg.sender == address(0x6D6F646c70792F73657276730000000000000000), "RegistrationHook: Only Runtime");
      _;
    }

    /// @dev A Hook that gets called by the runtime when a User tries to request a service.
    /// @param requestInputs The inputs that the user provided during the service request.
    ///
    /// Unless this function reverts, the service will be created using this blueprint.
    /// @custom:hook
    function onRequest(
      uint64 serviceId,
      bytes[] calldata participants,
      bytes calldata requestInputs
    ) public virtual payable onlyRuntime {}

    /// @dev A Hook that gets called by the runtime when a User call a job on a the service.
    /// @param serviceId The id of the service.
    /// @param job The index of the job.
    /// @param jobCallId The id of the job call.
    /// @param inputs The inputs that the user provided during the job call.
    ///
    /// Unless this function reverts, the job will be executed using this service.
    function onJobCall(
      uint64 serviceId,
      uint8 job,
      uint64 jobCallId,
      bytes calldata inputs
    ) public virtual payable onlyRuntime {}
}
