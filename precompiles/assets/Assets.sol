// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IAssets {
    /// @notice Create a new asset with the given parameters
    /// @param id The identifier for the new asset
    /// @param admin The account that will administer the asset
    /// @param minBalance The minimum balance required for an account to exist for this asset
    /// @return success True if the operation was successful
    function create(
        uint256 id,
        address admin,
        uint256 minBalance
    ) external returns (bool success);

    /// @notice Start the process of destroying an asset
    /// @param id The identifier of the asset to destroy
    /// @return success True if the operation was successful
    function startDestroy(
        uint256 id
    ) external returns (bool success);

    /// @notice Mint new tokens for an asset
    /// @param id The identifier of the asset
    /// @param beneficiary The account that will receive the minted tokens
    /// @param amount The amount of tokens to mint
    /// @return success True if the operation was successful
    function mint(
        uint256 id,
        address beneficiary,
        uint256 amount
    ) external returns (bool success);

    /// @notice Transfer tokens from the caller to another account
    /// @param id The identifier of the asset
    /// @param target The account that will receive the tokens
    /// @param amount The amount of tokens to transfer
    /// @return success True if the operation was successful
    function transfer(
        uint256 id,
        address target,
        uint256 amount
    ) external returns (bool success);

    // Events that should be emitted by the implementation
    event Created(uint256 indexed id, address indexed admin, uint256 minBalance);
    event DestroyStarted(uint256 indexed id);
    event Minted(uint256 indexed id, address indexed beneficiary, uint256 amount);
    event Transferred(uint256 indexed id, address indexed from, address indexed to, uint256 amount);
}
