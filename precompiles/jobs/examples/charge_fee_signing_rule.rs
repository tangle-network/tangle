// Example contract to allow anyone to use existing saved DKG keys to sign inputs by paying a fee
// -- This is an example contract that has not been audited, do not use in production ---
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

// Import the Jobs interface
import "./Jobs.sol";

/// @title JobsWithFeeContract
/// @dev A contract that allows submitting a job after charging a fee.
contract JobsWithFeeContract {

    // Fee amount required to submit a job
    uint256 public jobSubmissionFee;

    // address to get fee
    address payable feeRecipient;

    /// @dev Constructor to set the initial job submission fee.
    constructor(uint256 _jobSubmissionFee, address payable _feeRecipient) {
        jobSubmissionFee = _jobSubmissionFee;
        feeRecipient = _feeRecipient;
    }

    /// @dev Modifier to check if the caller has paid the required fee.
    modifier requireFee() {
        require(msg.value >= jobSubmissionFee, "Insufficient fee");
        _;
    }

    /// @dev Submit a job after charging the required fee.
    function submitJob(uint64 expiry,
        uint32 phase_one_id,
        bytes memory submission) external payable requireFee {
        // Forward the fee to the designated wallet
        feeRecipient.transfer(msg.value);

        // Call the submitDkgPhaseOneJob function from the Jobs contract
        JOBS_CONTRACT.submitDkgPhaseTwoJob(expiry, phase_one_id, submission);
    }

}
