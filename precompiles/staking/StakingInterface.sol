// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Staking contract's address.
address constant STAKING_ADDRESS = 0x0000000000000000000000000000000000000800;

/// @dev The Staking contract's instance.
Staking constant STAKING_CONTRACT = Staking(STAKING_ADDRESS);

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

    /// @dev Nominate a set of validators.
    /// @param targets Array of validators' addresses to nominate.
    function nominate(bytes32[] calldata targets) external;

    /// @dev Bond tokens for staking.
    /// @param value Amount of tokens to bond.
    /// @param payee Address to receive staking rewards.
    function bond(uint256 value, bytes32 payee) external;

    /// @dev Bond additional tokens for staking.
    /// @param maxAdditional Amount of additional tokens to bond.
    function bondExtra(uint256 maxAdditional) external;

    /// @dev Unbond a portion of the staked tokens.
    /// @param value Amount of tokens to unbond.
    function unbond(uint256 value) external;

    /// @dev Withdraw unbonded tokens after the unbonding period.
    /// @param numSlashingSpans Number of slashing spans for a validator.
    function withdrawUnbonded(uint32 numSlashingSpans) external;

    /// @dev Stop nominating and become inactive in staking.
    function chill() external;

    /// @dev Set the reward destination for staking rewards.
    /// @param payee The reward destination type (0-Staked, 1-Stash, 2-Controller).
    function setPayee(uint8 payee) external;

    /// @dev (Re-)set the controller to the stash.
    function setController() external;

    /// @dev Trigger payout for a validator and era.
    /// @param validatorStash The stash address of the validator.
    /// @param era The era index for which to trigger the payout.
    function payoutStakers(bytes32 validatorStash, uint32 era) external;

    /// @dev Rebond a portion of the unbonded tokens.
    /// @param value Amount of tokens to rebond.
    function rebond(uint256 value) external;
}
