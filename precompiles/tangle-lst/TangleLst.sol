// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;
/// @author The Tangle Team
/// @title Pallet TangleLst Interface
/// @title The interface through which solidity contracts will interact with the TangleLst pallet
interface TangleLst {
    /// @dev Join a pool with a specified amount.
    /// @param amount The amount to join with.
    /// @param poolId The ID of the pool to join.
    function join(uint256 amount, uint256 poolId) external;

    /// @dev Bond extra to a pool.
    /// @param poolId The ID of the pool.
    /// @param extraType The type of extra bond (0 for FreeBalance, 1 for Rewards).
    /// @param extra The amount of extra bond.
    function bondExtra(
        uint256 poolId,
        uint8 extraType,
        uint256 extra
    ) external returns (uint8);

    /// @dev Unbond from a pool.
    /// @param memberAccount The account of the member.
    /// @param poolId The ID of the pool.
    /// @param unbondingPoints The amount of unbonding points.
    function unbond(
        bytes32 memberAccount,
        uint256 poolId,
        uint256 unbondingPoints
    ) external returns (uint8);

    /// @dev Withdraw unbonded funds from a pool.
    /// @param poolId The ID of the pool.
    /// @param numSlashingSpans The number of slashing spans.
    function poolWithdrawUnbonded(
        uint256 poolId,
        uint32 numSlashingSpans
    ) external returns (uint8);

    /// @dev Withdraw unbonded funds for a member.
    /// @param memberAccount The account of the member.
    /// @param poolId The ID of the pool.
    /// @param numSlashingSpans The number of slashing spans.
    function withdrawUnbonded(
        bytes32 memberAccount,
        uint256 poolId,
        uint32 numSlashingSpans
    ) external returns (uint8);

    /// @dev Create a new pool.
    /// @param amount The initial amount to create the pool with.
    /// @param root The root account of the pool.
    /// @param nominator The nominator account of the pool.
    /// @param bouncer The bouncer account of the pool.
    /// @param name The name of the pool.
    /// @param icon The icon of the pool.
    function create(
        uint256 amount,
        bytes32 root,
        bytes32 nominator,
        bytes32 bouncer,
        bytes calldata name,
        bytes calldata icon
    ) external returns (uint8);

    /// @dev Nominate validators for a pool.
    /// @param poolId The ID of the pool.
    /// @param validators An array of validator accounts to nominate.
    function nominate(
        uint256 poolId,
        bytes32[] calldata validators
    ) external returns (uint8);

    /// @dev Set the state of a pool.
    /// @param poolId The ID of the pool.
    /// @param state The new state (0 for Open, 1 for Blocked, 2 for Destroying).
    function setState(uint256 poolId, uint8 state) external returns (uint8);

    /// @dev Set metadata for a pool.
    /// @param poolId The ID of the pool.
    /// @param metadata The metadata to set.
    function setMetadata(
        uint256 poolId,
        bytes calldata metadata
    ) external returns (uint8);

    /// @dev Set global configurations (only callable by root).
    /// @param minJoinBond The minimum bond required to join a pool (0 for no change).
    /// @param minCreateBond The minimum bond required to create a pool (0 for no change).
    /// @param maxPools The maximum number of pools (0 for no change).
    /// @param globalMaxCommission The global maximum commission percentage (0 for no change).
    function setConfigs(
        uint256 minJoinBond,
        uint256 minCreateBond,
        uint32 maxPools,
        uint32 globalMaxCommission
    ) external returns (uint8);

    /// @dev Update roles for a pool.
    /// @param poolId The ID of the pool.
    /// @param root The new root account.
    /// @param nominator The new nominator account.
    /// @param bouncer The new bouncer account.
    function updateRoles(
        uint256 poolId,
        bytes32 root,
        bytes32 nominator,
        bytes32 bouncer
    ) external returns (uint8);

    /// @dev Stop nominating for a pool
    /// @param poolId The ID of the pool to chill
    function chill(uint256 poolId) external;

    /// @dev Bond extra tokens for another account
    /// @param poolId The ID of the pool
    /// @param who The account to bond extra for
    /// @param amount The amount to bond extra
    function bondExtraOther(uint256 poolId, bytes32 who, uint256 amount) external;

    /// @dev Set commission for a pool
    /// @param poolId The ID of the pool
    /// @param newCommission The new commission value
    /// @param payee The account to receive commission payments
    function setCommission(uint256 poolId, uint256 newCommission, bytes32 payee) external;

    /// @dev Set maximum commission for a pool
    /// @param poolId The ID of the pool
    /// @param maxCommission The maximum commission value
    function setCommissionMax(uint256 poolId, uint256 maxCommission) external;

    /// @dev Set commission change rate
    /// @param poolId The ID of the pool
    /// @param maxIncrease The maximum increase in commission
    /// @param minDelay The minimum delay between changes
    function setCommissionChangeRate(uint256 poolId, uint256 maxIncrease, uint256 minDelay) external;

    /// @dev Claim commission for a pool
    /// @param poolId The ID of the pool
    function claimCommission(uint256 poolId) external;

    /// @dev Adjust pool deposit
    /// @param poolId The ID of the pool
    function adjustPoolDeposit(uint256 poolId) external;

    /// @dev Set commission claim permission
    /// @param poolId The ID of the pool
    /// @param permission The permission value (as uint8)
    function setCommissionClaimPermission(uint256 poolId, uint8 permission) external;
}
