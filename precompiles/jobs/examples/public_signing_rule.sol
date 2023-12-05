// Example contract to allow anyone to use existing saved DKG keys to sign inputs
// -- This is an example contract that has not been audited, do not use in production ---
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

// Import the Jobs interface
import "../Jobs.sol";

/// @title JobsCallerContract
/// @dev A contract that interacts with the Jobs contract and checks for whitelisted callers before submitting a job.
contract JobsCallerContract {
    /// @dev Submit a data to be signed by dkg
    function signWithDkg(
        uint64 expiry,
        uint32 phase_one_id,
        bytes memory submission
    ) external {
        // Call the submitDkgPhaseTwoJob function from the Jobs contract
        JOBS_CONTRACT.submitDkgPhaseTwoJob(expiry, phase_one_id, submission);
    }
}
