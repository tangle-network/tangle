// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Staking contract's address.
address constant STAKING_ADDRESS = 0x0000000000000000000000000000000000000800;

/// @dev The Staking contract's instance.
Staking constant STAKING_CONTRACT = Staking(
    STAKING_ADDRESS
);

/// @title Pallet Staking Interface
/// @dev The interface through which solidity contracts will interact with Staking pALLET
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000800
interface Staking {
    /// @dev Get current era
    /// @return era 
    function currentEra() external view returns (uint32);

    /// @dev Get min nominator bond
    /// @return min nominator bond 
    function minNominatorBond() external view returns (uint256);

    /// @dev Get min validator bond
    /// @return min validator bond  
    function minValidatorBond() external view returns (uint256);

    /// @dev Min Active stake
    /// @return min active stake  
    function minActiveStake() external view returns (uint256);

    /// @dev Validator count
    /// @return Validator count
    function validatorCount() external view returns (uint32);

    /// @dev Max validator count
    /// @return Max validator count
    function maxValidatorCount() external view returns (uint32);

    /// @dev Check whether the specified address is a validator
    /// @param stash the address that we want to confirm is a validator
    /// @return A boolean confirming whether the address is a validator
    function isValidator(address stash) external view returns (bool);

    /// @dev Check whether the specified address is a nominator
    /// @param stash the address that we want to confirm is a nominator
    /// @return A boolean confirming whether the address is a nominator
    function isNominator(address stash) external view returns (bool);

    /// @dev Max Nominator count
    /// @return Max Nominator count
    function maxNominatorCount() external view returns (uint32);

    /// @dev Total stake in era.
    /// @param eraIndex the address that we want to confirm is a nominator
    /// @return Total stake in era.
    function erasTotalStake(uint32 eraIndex) external view returns (uint256);

}
