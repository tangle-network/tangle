// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Cloud Credits Precompile Interface
/// @notice Interface for interacting with the Cloud Credits pallet through EVM
interface Credits {
    /// @notice Burn TNT tokens for potential off-chain credits
    /// @param amount The amount of TNT to burn
    /// @return success True if the burn operation was successful
    function burn(uint256 amount) external returns (bool success);

    /// @notice Claim potential credits accrued within the allowed window
    /// @param amount_to_claim The amount of credits to claim
    /// @param offchain_account_id The off-chain account ID to associate with these credits
    /// @return success True if the claim operation was successful
    function claim_credits(uint256 amount_to_claim, bytes calldata offchain_account_id) external returns (bool success);

    /// @notice Get the current emission rate for a given staked amount
    /// @param staked_amount The amount of staked TNT
    /// @return rate The credit emission rate per block
    function get_current_rate(uint256 staked_amount) external view returns (uint256 rate);

    /// @notice Calculate the potential credits accrued within the allowed window
    /// @param account The account to calculate for
    /// @return amount The calculated potential credits
    function calculate_accrued_credits(address account) external view returns (uint256 amount);

    /// @notice Get the configured stake tiers
    /// @return thresholds Array of stake thresholds
    /// @return rates Array of corresponding rates per block
    function get_stake_tiers() external view returns (uint256[] memory thresholds, uint256[] memory rates);

    /// @notice Emitted when TNT tokens are burned for potential off-chain credits
    /// @param who The account that burned the tokens
    /// @param tnt_burned The amount of TNT burned
    /// @param credits_granted The amount of potential credits granted
    event CreditsGrantedFromBurn(address indexed who, uint256 tnt_burned, uint256 credits_granted);

    /// @notice Emitted when a user claims credits
    /// @param who The account that claimed the credits
    /// @param amount_claimed The amount of credits claimed
    /// @param offchain_account_id The off-chain account ID associated with the claim
    event CreditsClaimed(address indexed who, uint256 amount_claimed, bytes offchain_account_id);
}