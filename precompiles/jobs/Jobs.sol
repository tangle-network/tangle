// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Jobs contract's address.
address constant JOBS_ADDRESS = 0x0000000000000000000000000000000000000814;

/// @dev The Jobs contract's instance.
Jobs constant JOBS_CONTRACT = Jobs(JOBS_ADDRESS);

/// @author Webb Inc
/// @title Pallet Jobs Interface
/// @title The interface through which solidity contracts will interact with the jobs pallet
/// @custom:address 0x0000000000000000000000000000000000000814
interface Jobs {

    /// Submit a DKG phase one job
    /// @custom:selector <selector_hash>
    ///
    /// @notice Submits a job for the first phase of the Distributed Key Generation (DKG) process.
    ///
    /// @param expiry The expiration timestamp for the submitted job.
    /// @param ttl The time-to-live for the submitted job.
    /// @param participants An array of Ethereum addresses representing participants in the DKG.
    /// @param threshold The minimum number of participants required for the DKG to succeed.
    /// @param roleType The role type identifier.
    /// @param permittedCaller The Ethereum address of the permitted caller initiating the job submission.
    /// @param hdWallet A boolean indicating whether the job is for an HD wallet.
    ///
    /// @dev This function initiates the first phase of a DKG process, allowing participants to collaborate
    /// in generating cryptographic keys. The submitted job includes information such as the expiration time,
    /// the list of participants, the threshold for successful completion, and the permitted caller's address.
    /// It is crucial for the caller to ensure that the specified parameters align with the intended DKG process.
    ///
    function submitDkgPhaseOneJob(
        uint64 expiry,
        uint64 ttl,
        address[] memory participants,
        uint8 threshold,
        uint16 roleType,
        address permittedCaller,
        bool hdWallet
    ) external;

    /// @custom:selector <selector_hash>
    ///
    /// @notice Submits a job for the second phase of the Distributed Key Generation (DKG) process.
    ///
    /// @param expiry The expiration timestamp for the submitted job.
    /// @param ttl The time-to-live for the submitted job.
    /// @param phaseOneId The identifier of the corresponding phase one DKG job.
    /// @param submission The byte array containing the data submission for the DKG phase two.
    /// @param derivationPath The byte array containing the derivation path for the DKG phase two.
    ///
    /// @dev This function initiates the second phase of a Distributed Key Generation process, building upon
    /// the results of a prior phase one submission. The submitted job includes an expiration time, the identifier
    /// of the phase one DKG job, and the byte array representing the participant's data contribution for phase two.
    /// It is important for the caller to ensure that the provided parameters align with the ongoing DKG process.
    ///
    function submitDkgPhaseTwoJob(
        uint64 expiry,
        uint64 ttl,
        uint64 phaseOneId,
        bytes memory submission,
        bytes memory derivationPath
    ) external;

    /// @custom:selector <selector_hash>
    ///
    /// @notice Sets a new permitted caller for a specific job type identified by the given key and job ID.
    ///
    /// @param roleType An identifier specifying the role type to update the permitted caller for.
    /// @param jobId The unique identifier of the job for which the permitted caller is being updated.
    /// @param newPermittedCaller The Ethereum address of the new permitted caller.
    ///
    /// @dev This function provides flexibility in managing permitted callers for different job types.
    /// The caller can specify the job key, job ID, and the new Ethereum address that will be granted permission
    /// to initiate the specified job. It is important for the caller to ensure that the provided parameters
    /// align with the ongoing processes and permissions within the contract.
    ///
    function setPermittedCaller(
        uint16 roleType,
        uint32 jobId,
        address newPermittedCaller
    ) external;
}
