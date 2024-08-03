// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev A Contract that is inherited by all runtime hooks.
contract Runtime {
    /// @dev address(keccak256(pallet_services::Config::PalletId::to_account_id())[0:20])
    address constant RUNTIME_ADDRESS = 0x6D6F646c70792F73657276730000000000000000;
    /// @dev Only allow the runtime to call this function.

    modifier onlyRuntime() {
        require(msg.sender == RUNTIME_ADDRESS, "AccessControl: Only Runtime");
        _;
    }

    /// @dev Get the runtime address.
    /// @return The address of the runtime.
    function getRuntimeAddress() public pure returns (address) {
        return RUNTIME_ADDRESS;
    }
}
