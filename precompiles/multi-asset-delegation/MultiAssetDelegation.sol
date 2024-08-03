// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The MultiAssetDelegation contract's address.
address constant MULTI_ASSET_DELEGATION = 0x0000000000000000000000000000000000000809;

/// @dev The MultiAssetDelegation contract's instance.
MultiAssetDelegation constant MULTI_ASSET_DELEGATION_CONTRACT = MultiAssetDelegation(MULTI_ASSET_DELEGATION);

/// @author The Tangle Team
/// @title Pallet MultiAssetDelegation Interface
/// @title The interface through which solidity contracts will interact with the MultiAssetDelegation pallet
/// @custom:address 0x0000000000000000000000000000000000000809
interface MultiAssetDelegation {
    /// @dev Join as an operator with a bond amount.
    /// @param bondAmount The amount to bond as an operator.
    function joinOperators(uint256 bondAmount) external returns (uint8);

    /// @dev Schedule to leave as an operator.
    function scheduleLeaveOperators() external returns (uint8);

    /// @dev Cancel the scheduled leave as an operator.
    function cancelLeaveOperators() external returns (uint8);

    /// @dev Execute the leave as an operator.
    function executeLeaveOperators() external returns (uint8);

    /// @dev Bond more as an operator.
    /// @param additionalBond The additional amount to bond.
    function operatorBondMore(uint256 additionalBond) external returns (uint8);

    /// @dev Schedule to unstake as an operator.
    /// @param unstakeAmount The amount to unstake.
    function scheduleOperatorUnstake(uint256 unstakeAmount) external returns (uint8);

    /// @dev Execute the unstake as an operator.
    function executeOperatorUnstake() external returns (uint8);

    /// @dev Cancel the scheduled unstake as an operator.
    function cancelOperatorUnstake() external returns (uint8);

    /// @dev Go offline as an operator.
    function goOffline() external returns (uint8);

    /// @dev Go online as an operator.
    function goOnline() external returns (uint8);

    /// @dev Deposit an amount of an asset.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to deposit.
    function deposit(uint256 assetId, uint256 amount) external returns (uint8);

    /// @dev Schedule a withdrawal of an amount of an asset.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to withdraw.
    function scheduleWithdraw(uint256 assetId, uint256 amount) external returns (uint8);

    /// @dev Execute the scheduled withdrawal.
    function executeWithdraw() external returns (uint8);

    /// @dev Cancel the scheduled withdrawal.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to cancel withdrawal.
    function cancelWithdraw(uint256 assetId, uint256 amount) external returns (uint8);

    /// @dev Delegate an amount of an asset to an operator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to delegate.
    function delegate(bytes32 operator, uint256 assetId, uint256 amount) external returns (uint8);

    /// @dev Schedule an unstake of an amount of an asset as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to unstake.
    function scheduleDelegatorUnstake(bytes32 operator, uint256 assetId, uint256 amount) external returns (uint8);

    /// @dev Execute the scheduled unstake as a delegator.
    function executeDelegatorUnstake() external returns (uint8);

    /// @dev Cancel the scheduled unstake as a delegator.
    /// @param operator The address of the operator.
    /// @param assetId The ID of the asset.
    /// @param amount The amount to cancel unstake.
    function cancelDelegatorUnstake(bytes32 operator, uint256 assetId, uint256 amount) external returns (uint8);
}