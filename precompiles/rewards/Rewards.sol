// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Rewards contract's address.
address constant REWARDS = 0x0000000000000000000000000000000000000823;

/// @dev The Rewards contract's instance.
Rewards constant REWARDS_CONTRACT = Rewards(REWARDS);

/// @author The Tangle Team
/// @title Pallet Rewards Interface
/// @title The interface through which solidity contracts will interact with the Rewards pallet
/// @custom:address 0x0000000000000000000000000000000000000823
interface Rewards {
    /// @notice Updates the rewards for a specific asset
    /// @dev Only callable by root/admin
    /// @param assetId The ID of the asset
    /// @param rewards The new rewards amount
    /// @return uint8 Returns 0 on success
    function updateAssetRewards(uint256 assetId, uint256 rewards) external returns (uint8);

    /// @notice Updates the APY for a specific asset
    /// @dev Only callable by root/admin. APY is capped at 10%
    /// @param assetId The ID of the asset
    /// @param apy The new APY value (in basis points, e.g. 1000 = 10%)
    /// @return uint8 Returns 0 on success
    function updateAssetApy(uint256 assetId, uint32 apy) external returns (uint8);

    /// @notice Calculates the reward score for given parameters
    /// @param stake The stake amount
    /// @param rewards The rewards amount
    /// @param apy The APY value
    /// @return uint256 The calculated reward score
    function calculateRewardScore(uint256 stake, uint256 rewards, uint32 apy) external view returns (uint256);

    /// @notice Calculates the total reward score for an asset
    /// @param assetId The ID of the asset
    /// @return uint256 The total reward score
    function calculateTotalRewardScore(uint256 assetId) external view returns (uint256);

    /// @notice Gets the current rewards for an asset
    /// @param assetId The ID of the asset
    /// @return uint256 The current rewards amount
    function assetRewards(uint256 assetId) external view returns (uint256);

    /// @notice Gets the current APY for an asset
    /// @param assetId The ID of the asset
    /// @return uint32 The current APY value
    function assetApy(uint256 assetId) external view returns (uint32);

    /// @notice Sets incentive APY and cap for a vault
    /// @dev Only callable by force origin. APY is capped at 10%
    /// @param vaultId The ID of the vault
    /// @param apy The APY value (in basis points, max 1000 = 10%)
    /// @param cap The cap amount for full APY distribution
    /// @return uint8 Returns 0 on success
    function setIncentiveApyAndCap(uint256 vaultId, uint32 apy, uint256 cap) external returns (uint8);

    /// @notice Whitelists a blueprint for rewards
    /// @dev Only callable by force origin
    /// @param blueprintId The ID of the blueprint to whitelist
    /// @return uint8 Returns 0 on success
    function whitelistBlueprintForRewards(uint64 blueprintId) external returns (uint8);

    /// @notice Manages assets in a vault
    /// @dev Only callable by authorized accounts
    /// @param vaultId The ID of the vault
    /// @param assetId The ID of the asset
    /// @param action 0 for Add, 1 for Remove
    /// @return uint8 Returns 0 on success
    function manageAssetInVault(uint256 vaultId, uint256 assetId, uint8 action) external returns (uint8);
}