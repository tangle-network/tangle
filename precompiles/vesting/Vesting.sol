// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Vesting contract's address.
address constant VESTING_ADDRESS = 0x0000000000000000000000000000000000000801;

/// @dev The Vesting contract's instance.
Vesting constant VESTING_CONTRACT = Vesting(VESTING_ADDRESS);

/// @author The Tangle Team
/// @title Pallet Vesting Interface
/// @title The interface through which solidity contracts will interact with the Vesting pallet
/// @custom:address 0x0000000000000000000000000000000000000801
interface Vesting {
    /// @dev Unlock any vested funds of the sender account.
    function vest() external returns (uint8);

    /// @dev Unlock any vested funds of a `target` account.
    /// @param target The address of the account to unlock vested funds for.
    function vestOther(bytes32 target) external returns (uint8);

    /// @dev Create a vested transfer.
    /// @param target The address of the account to transfer funds to.
    /// @param index The index of the vesting schedule to transfer.
    function vestedTransfer(bytes32 target, uint8 index) external returns (uint8);

    /// @dev Merge two vesting schedules together.
    /// @param schedule1Index The index of the first vesting schedule.
    /// @param schedule2Index The index of the second vesting schedule.
    function mergeSchedules(uint32 schedule1Index, uint32 schedule2Index) external returns (uint8);
}
