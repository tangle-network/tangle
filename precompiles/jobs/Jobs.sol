// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Jobs contract's address.
address constant JOBS_ADDRESS = 0x0000000000000000000000000000000000000820;

/// @dev The Jobs contract's instance.
Jobs constant JOBS_CONTRACT = Jobs(JOBS_ADDRESS);

/// @author Webb Inc
/// @title Pallet Jobs Interface
/// @title The interface through which solidity contracts will interact with the jobs pallet
/// @custom:address 0x0000000000000000000000000000000000000813
interface Jobs {
    /// @dev Register Jobs on-chain.
    /// @custom:selector cb00f603
    /// @param jobSubmission The jobs to be registered on-chain
    /// @return jobsHash The hash of the jobs
    function submitJobs(bytes memory jobSubmission)
        external
        returns (bytes32 jobsHash);
}
