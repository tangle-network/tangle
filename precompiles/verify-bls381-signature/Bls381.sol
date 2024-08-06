pragma solidity ^0.8.0;

/**
 * @title Bls-381 interface to verify isgnature.
 */
interface IBls381 {
    /**
     * @dev Verify signed message hash.
     * @return A boolean confirming whether the public key is signer for the message. 
     */
    function verify(
        bytes32 public_key,
        bytes calldata signature,
        bytes calldata message
    ) external view returns (bool);
}
