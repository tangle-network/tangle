// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./IAssets.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title Asset Manager Contract
 * @dev This contract manages the bridging between ERC20 tokens and native assets in the Tangle Network.
 * It allows users to deposit ERC20 tokens and receive corresponding native assets, maintaining a 1:1 relationship
 * between ERC20 tokens and native assets.
 *
 * Key features:
 * - Creates and tracks native assets corresponding to ERC20 tokens
 * - Handles deposits of ERC20 tokens and mints equivalent native assets
 * - Maintains a mapping between ERC20 tokens and their corresponding asset IDs
 * - Uses the Assets precompile for native asset operations
 *
 * Security considerations:
 * - Only the contract owner can manually set asset IDs
 * - Includes emergency recovery function for stuck ERC20 tokens
 * - Asset IDs are obtained from the precompile to ensure system-wide consistency
 */
contract AssetManager is Ownable {
    // Interface to the Assets precompile that handles native asset operations
    IAssets public immutable assetsPrecompile;
    
    // Maps ERC20 token addresses to their corresponding native asset IDs
    mapping(address => uint256) public erc20ToAssetId;
    
    // Events for tracking asset creation and deposits
    event AssetCreated(address indexed erc20Token, uint256 indexed assetId);
    event Deposited(address indexed erc20Token, uint256 indexed assetId, address indexed user, uint256 amount);

    /**
     * @dev Initializes the contract with the Assets precompile address
     * @param _assetsPrecompile The address of the Assets precompile contract
     */
    constructor(address _assetsPrecompile) {
        assetsPrecompile = IAssets(_assetsPrecompile);
    }

    /**
     * @notice Deposits ERC20 tokens and mints corresponding native assets
     * @dev The function performs the following steps:
     * 1. Transfers ERC20 tokens from the user to this contract
     * 2. Gets or creates a native asset ID for the ERC20 token
     * 3. Mints an equivalent amount of native assets to the user
     *
     * @param erc20Token The address of the ERC20 token to deposit
     * @param amount The amount of tokens to deposit
     * @return assetId The ID of the native asset that was minted
     *
     * Requirements:
     * - Amount must be greater than 0
     * - ERC20 token address must not be zero address
     * - User must have approved this contract to spend their tokens
     */
    function deposit(address erc20Token, uint256 amount) external returns (uint256) {
        require(amount > 0, "Amount must be greater than 0");
        require(erc20Token != address(0), "Invalid token address");
        
        // Transfer ERC20 tokens from user to this contract
        require(
            IERC20(erc20Token).transferFrom(msg.sender, address(this), amount),
            "Token transfer failed"
        );
        
        // Get or create asset ID
        uint256 assetId = getOrCreateAssetId(erc20Token);
        
        // Mint native assets to the user
        require(
            assetsPrecompile.mint(assetId, address(this), amount),
            "Asset minting failed"
        );
        
        emit Deposited(erc20Token, assetId, msg.sender, amount);
        return assetId;
    }

    /**
     * @dev Internal function to get existing or create new asset ID for an ERC20 token
     * @param erc20Token The ERC20 token address
     * @return assetId The asset ID (either existing or newly created)
     *
     * The function follows these steps:
     * 1. Checks if an asset ID already exists for the token
     * 2. If not, gets the next available asset ID from the precompile
     * 3. Creates a new native asset with this contract as admin
     * 4. Stores the ERC20 token to asset ID mapping
     *
     * Note: The minimum balance for new assets is set to 1 to prevent dust attacks
     */
    function getOrCreateAssetId(address erc20Token) internal returns (uint256) {
        uint256 assetId = erc20ToAssetId[erc20Token];
        
        // If asset doesn't exist, create it
        if (assetId == 0) {
            // Get the next available asset ID from the precompile
            assetId = assetsPrecompile.next_asset_id();
            
            // Create the asset with this contract as admin
            require(
                assetsPrecompile.create(assetId, address(this), 1),
                "Asset creation failed"
            );
            
            // Store the mapping
            erc20ToAssetId[erc20Token] = assetId;
            
            emit AssetCreated(erc20Token, assetId);
        }
        
        return assetId;
    }

    /**
     * @notice Retrieves the native asset ID for a given ERC20 token
     * @dev Returns 0 if no asset ID exists for the token
     * @param erc20Token The ERC20 token address to query
     * @return The corresponding native asset ID, or 0 if none exists
     */
    function getAssetId(address erc20Token) external view returns (uint256) {
        return erc20ToAssetId[erc20Token];
    }

    /**
     * @notice Allows the owner to manually set an asset ID for an ERC20 token
     * @dev This function is restricted to the contract owner and can only be used
     * for tokens that don't already have an asset ID assigned
     *
     * @param erc20Token The ERC20 token address
     * @param assetId The corresponding asset ID to assign
     *
     * Requirements:
     * - Caller must be the contract owner
     * - Token must not already have an asset ID
     * - Asset ID must be greater than 0
     */
    function setAssetId(address erc20Token, uint256 assetId) external onlyOwner {
        require(erc20ToAssetId[erc20Token] == 0, "Asset ID already exists");
        require(assetId > 0, "Invalid asset ID");
        erc20ToAssetId[erc20Token] = assetId;
        emit AssetCreated(erc20Token, assetId);
    }

    /**
     * @notice Emergency function to recover accidentally sent ERC20 tokens
     * @dev This function allows the owner to recover any ERC20 tokens that were
     * accidentally sent to this contract. This is a safety measure and should
     * only be used in emergency situations.
     *
     * @param token The ERC20 token address to recover
     *
     * Requirements:
     * - Caller must be the contract owner
     * - Contract must have a non-zero balance of the specified token
     */
    function recoverERC20(address token) external onlyOwner {
        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance > 0, "No tokens to recover");
        require(
            IERC20(token).transfer(owner(), balance),
            "Token recovery failed"
        );
    }
}
