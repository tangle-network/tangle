// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The TangleLst contract's address.
address constant TANGLE_LST = 0x0000000000000000000000000000000000000809;

/// @dev The TangleLst contract's instance.
TangleLst constant TANGLE_LST_CONTRACT = TangleLst(TANGLE_LST);

/// @author The Tangle Team
/// @title Pallet TangleLst Interface
/// @title The interface through which solidity contracts will interact with the TangleLst pallet
/// @custom:address 0x0000000000000000000000000000000000000809
interface TangleLst {
    /// @dev Join a pool with a specified amount.
    /// @param amount The amount to join with.
    /// @param poolId The ID of the pool to join.
    function join(uint256 amount, uint256 poolId) external returns (uint8);

    /// @dev Bond extra to a pool.
    /// @param poolId The ID of the pool.
    /// @param extraType The type of extra bond (0 for FreeBalance, 1 for Rewards).
    /// @param extra The amount of extra bond.
    function bondExtra(uint256 poolId, uint8 extraType, uint256 extra) external returns (uint8);

    /// @dev Unbond from a pool.
    /// @param memberAccount The account of the member.
    /// @param poolId The ID of the pool.
    /// @param unbondingPoints The amount of unbonding points.
    function unbond(bytes32 memberAccount, uint256 poolId, uint256 unbondingPoints) external returns (uint8);

    /// @dev Withdraw unbonded funds from a pool.
    /// @param poolId The ID of the pool.
    /// @param numSlashingSpans The number of slashing spans.
    function poolWithdrawUnbonded(uint256 poolId, uint32 numSlashingSpans) external returns (uint8);

    /// @dev Withdraw unbonded funds for a member.
    /// @param memberAccount The account of the member.
    /// @param poolId The ID of the pool.
    /// @param numSlashingSpans The number of slashing spans.
    function withdrawUnbonded(bytes32 memberAccount, uint256 poolId, uint32 numSlashingSpans) external returns (uint8);

    /// @dev Create a new pool.
    /// @param amount The initial amount to create the pool with.
    /// @param root The root account of the pool.
    /// @param nominator The nominator account of the pool.
    /// @param bouncer The bouncer account of the pool.
    function create(uint256 amount, bytes32 root, bytes32 nominator, bytes32 bouncer) external returns (uint8);

    /// @dev Create a new pool with a specific pool ID.
    /// @param amount The initial amount to create the pool with.
    /// @param root The root account of the pool.
    /// @param nominator The nominator account of the pool.
    /// @param bouncer The bouncer account of the pool.
    /// @param poolId The desired pool ID.
    function createWithPoolId(uint256 amount, bytes32 root, bytes32 nominator, bytes32 bouncer, uint256 poolId) external returns (uint8);

    /// @dev Nominate validators for a pool.
    /// @param poolId The ID of the pool.
    /// @param validators An array of validator accounts to nominate.
    function nominate(uint256 poolId, bytes32[] calldata validators) external returns (uint8);

    /// @dev Set the state of a pool.
    /// @param poolId The ID of the pool.
    /// @param state The new state (0 for Open, 1 for Blocked, 2 for Destroying).
    function setState(uint256 poolId, uint8 state) external returns (uint8);

    /// @dev Set metadata for a pool.
    /// @param poolId The ID of the pool.
    /// @param metadata The metadata to set.
    function setMetadata(uint256 poolId, bytes calldata metadata) external returns (uint8);

    /// @dev Set global configurations (only callable by root).
    /// @param minJoinBond The minimum bond required to join a pool (0 for no change).
    /// @param minCreateBond The minimum bond required to create a pool (0 for no change).
    /// @param maxPools The maximum number of pools (0 for no change).
    /// @param globalMaxCommission The global maximum commission percentage (0 for no change).
    function setConfigs(uint256 minJoinBond, uint256 minCreateBond, uint32 maxPools, uint32 globalMaxCommission) external returns (uint8);
}