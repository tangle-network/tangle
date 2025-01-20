// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The MultiAssetDelegation contract's address.
address constant MULTI_ASSET_DELEGATION = 0x0000000000000000000000000000000000000822;

/// @dev The MultiAssetDelegation contract's instance.
MultiAssetDelegation constant MULTI_ASSET_DELEGATION_CONTRACT = MultiAssetDelegation(MULTI_ASSET_DELEGATION);

/// @author The Tangle Team
/// @title Pallet MultiAssetDelegation Interface
/// @title The interface through which solidity contracts will interact with the MultiAssetDelegation pallet
/// @custom:address 0x0000000000000000000000000000000000000822
interface MultiAssetDelegation {
    /// @dev Join as an operator with a bond amount.
    /// @param bondAmount The amount to bond as an operator.
    /// @custom:selector de883d74
    function joinOperators(uint256 bondAmount) external;

    /// @dev Schedule to leave as an operator.
    /// @custom:selector ce3edd76
    function scheduleLeaveOperators() external;

    /// @dev Cancel the scheduled leave as an operator.
    /// @custom:selector 9b1300c1
    function cancelLeaveOperators() external;

    /// @dev Execute the leave as an operator.
    /// @custom:selector 0de1fc17
    function executeLeaveOperators() external;

    /// @dev Bond more as an operator.
    /// @param additionalBond The additional amount to bond.
    /// @custom:selector eede281b
    function operatorBondMore(uint256 additionalBond) external;

    /// @dev Schedule to unstake as an operator.
    /// @param unstakeAmount The amount to unstake.
    /// @custom:selector 44aff252
    function scheduleOperatorUnstake(uint256 unstakeAmount) external;

    /// @dev Execute the unstake as an operator.
    /// @custom:selector b0dfce06
    function executeOperatorUnstake() external;

    /// @dev Cancel the scheduled unstake as an operator.
    /// @custom:selector ac359f2b
    function cancelOperatorUnstake() external;

    /// @dev Go offline as an operator.
    /// @custom:selector a6485ccd
    function goOffline() external;

    /// @dev Go online as an operator.
    /// @custom:selector 6e5b676b
    function goOnline() external;

    /// @dev Deposit an amount of an asset.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to deposit.
    /// @param lockMultiplier The lock multiplier.
    /// @custom:selector b3c11395
    function deposit(uint256 assetId, address tokenAddress, uint256 amount, uint8 lockMultiplier) external;

    /// @dev Schedule a withdrawal of an amount of an asset.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to withdraw.
    /// @custom:selector a125b146
    function scheduleWithdraw(uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Execute the scheduled withdrawal.
    /// @custom:selector f8fd9795
    function executeWithdraw() external;

    /// @dev Cancel the scheduled withdrawal.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to cancel withdrawal.
    /// @custom:selector 098d2a20
    function cancelWithdraw(uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Delegate an amount of an asset to an operator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to delegate.
    /// @param blueprintSelection The blueprint selection.
    /// @custom:selector a12de0ba
    function delegate(
        bytes32 operator,
        uint256 assetId,
        address tokenAddress,
        uint256 amount,
        uint64[] memory blueprintSelection
    ) external;

    /// @dev Schedule an unstake of an amount of an asset as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to unstake.
    /// @custom:selector 5bea61c6
    function scheduleDelegatorUnstake(bytes32 operator, uint256 assetId, address tokenAddress, uint256 amount)
        external;

    /// @dev Execute the scheduled unstake as a delegator.
    /// @custom:selector 007910d0
    function executeDelegatorUnstake() external;

    /// @dev Cancel the scheduled unstake as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to cancel unstake.
    /// @custom:selector 504aff13
    function cancelDelegatorUnstake(bytes32 operator, uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Get the total balance of the delegator (including the delegated balance).
    /// @param who The address of the account.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @return The total balance of the delegator.
    /// @custom:selector 467f4cb9
    function balanceOf(address who, uint256 assetId, address tokenAddress) external view returns (uint256);

    /// @dev Get the delegated balance of the delegator.
    /// @param who The address of the delegator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @return The delegated balance of the delegator.
    /// @custom:selector aabd20df
    function delegatedBalanceOf(address who, uint256 assetId, address tokenAddress) external view returns (uint256);
}
