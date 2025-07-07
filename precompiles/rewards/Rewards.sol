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
    /// @notice Claims delegation rewards for the caller and the specified asset
    /// @param assetId The ID of the asset (0 for ERC20 tokens)
    /// @param tokenAddress The EVM address of the token (zero for custom assets)
    function claimRewards(uint256 assetId, address tokenAddress) external;
}