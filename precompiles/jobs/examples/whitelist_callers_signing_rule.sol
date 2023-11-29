// Example contract to allow only whitelisted callers to use existing saved DKG keys to sign inputs
// -- This is an example contract that has not been audited, do not use in production ---
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

// Import the Jobs interface
import "./Jobs.sol";

/// @title JobsCallerContract
/// @dev A contract that interacts with the Jobs contract and checks for whitelisted callers before submitting a job.
contract JobsCallerContract {

    // Mapping to store whitelisted callers
    mapping(address => bool) public whitelistedCallers;

    /// @dev Add an address to the whitelist.
    function addToWhitelist(address caller) external {
        whitelistedCallers[caller] = true;
    }

    /// @dev Remove an address from the whitelist.
    function removeFromWhitelist(address caller) external {
        whitelistedCallers[caller] = false;
    }

    /// @dev Check if an address is whitelisted.
    function isCallerWhitelisted(address caller) external view returns (bool) {
        return whitelistedCallers[caller];
    }

    // Modifier to check if the caller is whitelisted
    modifier onlyWhitelisted() {
        require(this.isCallerWhitelisted(msg.sender), "Caller is not whitelisted");
        _;
    }

    /// @dev Submit a data to be signed by dkg
    function signWithDkg(
        uint64 expiry,
        uint32 phase_one_id,
        bytes memory submission
    ) external onlyWhitelisted {
        // Call the submitDkgPhaseTwoJob function from the Jobs contract
        JOBS_CONTRACT.submitDkgPhaseTwoJob(expiry, phase_one_id, submission);
    }

}