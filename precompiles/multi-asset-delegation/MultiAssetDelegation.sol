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
    function joinOperators(uint256 bondAmount) external;

    /// @dev Schedule to leave as an operator.
    function scheduleLeaveOperators() external;

    /// @dev Cancel the scheduled leave as an operator.
    function cancelLeaveOperators() external;

    /// @dev Execute the leave as an operator.
    function executeLeaveOperators() external;

    /// @dev Bond more as an operator.
    /// @param additionalBond The additional amount to bond.
    function operatorBondMore(uint256 additionalBond) external;

    /// @dev Schedule to unstake as an operator.
    /// @param unstakeAmount The amount to unstake.
    function scheduleOperatorUnstake(uint256 unstakeAmount) external;

    /// @dev Execute the unstake as an operator.
    function executeOperatorUnstake() external;

    /// @dev Cancel the scheduled unstake as an operator.
    function cancelOperatorUnstake() external;

    /// @dev Go offline as an operator.
    function goOffline() external;

    /// @dev Go online as an operator.
    function goOnline() external;

    /// @dev Deposit an amount of an asset.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to deposit.
    /// @param lockMultiplier The lock multiplier.
    function deposit(uint256 assetId, address tokenAddress, uint256 amount, uint8 lockMultiplier) external;

    /// @dev Schedule a withdrawal of an amount of an asset.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to withdraw.
    function scheduleWithdraw(uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Execute the scheduled withdrawal.
    function executeWithdraw() external;

    /// @dev Cancel the scheduled withdrawal.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to cancel withdrawal.
    function cancelWithdraw(uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Delegate an amount of an asset to an operator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to delegate.
    /// @param blueprintSelection The blueprint selection.
    function delegate(bytes32 operator, uint256 assetId, address tokenAddress, uint256 amount, uint64[] memory blueprintSelection) external;

    /// @dev Schedule an unstake of an amount of an asset as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to unstake.
    function scheduleDelegatorUnstake(bytes32 operator, uint256 assetId, address tokenAddress, uint256 amount) external;

    /// @dev Execute the scheduled unstake as a delegator.
    function executeDelegatorUnstake() external;

    /// @dev Cancel the scheduled unstake as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset (0 for ERC20).
    /// @param tokenAddress The address of the ERC20 token (if assetId is 0).
    /// @param amount The amount to cancel unstake.
    function cancelDelegatorUnstake(bytes32 operator, uint256 assetId, address tokenAddress, uint256 amount) external;
}