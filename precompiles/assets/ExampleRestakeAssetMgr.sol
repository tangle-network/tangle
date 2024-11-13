// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./IAssets.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract AssetManager is Ownable {
    // Interface to the Assets precompile
    IAssets public immutable assetsPrecompile;
    
    // Mapping from ERC20 address to asset ID
    mapping(address => uint256) public erc20ToAssetId;
    // Mapping to track the next available asset ID
    uint256 public nextAssetId = 1;
    
    // Events
    event AssetCreated(address indexed erc20Token, uint256 indexed assetId);
    event Deposited(address indexed erc20Token, uint256 indexed assetId, address indexed user, uint256 amount);

    constructor(address _assetsPrecompile) {
        assetsPrecompile = IAssets(_assetsPrecompile);
    }

    /**
     * @notice Deposits ERC20 tokens and mints corresponding native assets
     * @param erc20Token The address of the ERC20 token to deposit
     * @param amount The amount of tokens to deposit
     * @return assetId The ID of the native asset that was minted
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
     * @notice Internal function to get existing or create new asset ID
     * @param erc20Token The ERC20 token address
     * @return assetId The asset ID (either existing or newly created)
     */
    function getOrCreateAssetId(address erc20Token) internal returns (uint256) {
        uint256 assetId = erc20ToAssetId[erc20Token];
        
        // If asset doesn't exist, create it
        if (assetId == 0) {
            assetId = nextAssetId++;
            
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
     * @notice View function to check if an ERC20 token has a corresponding asset
     * @param erc20Token The ERC20 token address
     * @return The asset ID if it exists, 0 otherwise
     */
    function getAssetId(address erc20Token) external view returns (uint256) {
        return erc20ToAssetId[erc20Token];
    }

    /**
     * @notice Allows the owner to manually set an asset ID for an ERC20 token
     * @param erc20Token The ERC20 token address
     * @param assetId The corresponding asset ID
     */
    function setAssetId(address erc20Token, uint256 assetId) external onlyOwner {
        require(erc20ToAssetId[erc20Token] == 0, "Asset ID already exists");
        require(assetId > 0, "Invalid asset ID");
        erc20ToAssetId[erc20Token] = assetId;
        emit AssetCreated(erc20Token, assetId);
    }

    /**
     * @notice Emergency function to recover stuck ERC20 tokens
     * @param token The ERC20 token to recover
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
