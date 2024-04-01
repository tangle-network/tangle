// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Jobs contract's address.
address constant JOBS_ADDRESS = 0x0000000000000000000000000000000000000820;

/// @dev The Jobs contract's instance.
Jobs constant JOBS_CONTRACT = Jobs(JOBS_ADDRESS);

/// @author Webb Inc
/// @title Pallet Jobs Interface
/// @title The interface through which solidity contracts will interact with the jobs pallet
/// @custom:address 0x0000000000000000000000000000000000000820
interface Jobs {

    /// Submit a DKG phase one job
    /// @custom:selector <selector_hash>
    ///
    /// @notice Submits a job for the first phase of the Distributed Key Generation (DKG) process.
    ///
    /// @param expiry The expiration timestamp for the submitted job.
    /// @param participants An array of Ethereum addresses representing participants in the DKG.
    /// @param threshold The minimum number of participants required for the DKG to succeed.
    /// @param permitted_caller The Ethereum address of the permitted caller initiating the job submission.
    ///
    /// @dev This function initiates the first phase of a DKG process, allowing participants to collaborate
    /// in generating cryptographic keys. The submitted job includes information such as the expiration time,
    /// the list of participants, the threshold for successful completion, and the permitted caller's address.
    /// It is crucial for the caller to ensure that the specified parameters align with the intended DKG process.
    ///
    function submitDkgPhaseOneJob(
        uint64 expiry,
        address[] memory participants,
        uint8 threshold,
        address permitted_caller
    ) external;

    /// @custom:selector <selector_hash>
    ///
    /// @notice Submits a job for the second phase of the Distributed Key Generation (DKG) process.
    ///
    /// @param expiry The expiration timestamp for the submitted job.
    /// @param phase_one_id The identifier of the corresponding phase one DKG job.
    /// @param submission The byte array containing the data submission for the DKG phase two.
    /// @param derivation_path The byte array containing the derivation path for the DKG phase two.
    ///
    /// @dev This function initiates the second phase of a Distributed Key Generation process, building upon
    /// the results of a prior phase one submission. The submitted job includes an expiration time, the identifier
    /// of the phase one DKG job, and the byte array representing the participant's data contribution for phase two.
    /// It is important for the caller to ensure that the provided parameters align with the ongoing DKG process.
    ///
    function submitDkgPhaseTwoJob(
        uint64 expiry,
        uint32 phase_one_id,
        bytes memory submission,
        bytes memory derivation_path
    ) external;

    /// @custom:selector <selector_hash>
    ///
    /// @notice Initiates the withdrawal of accumulated rewards for the caller.
    ///
    /// @dev This function allows the caller to withdraw any rewards accumulated through participation in
    /// various activities or processes within the contract. The withdrawal process is triggered by invoking
    /// this function, and the caller receives their entitled rewards accordingly.
    ///
    function withdrawRewards(
    ) external;

    /// @custom:selector <selector_hash>
    ///
    /// @notice Sets a new permitted caller for a specific job type identified by the given key and job ID.
    ///
    /// @param role_type An identifier specifying the role type to update the permitted caller for.
    /// @param job_id The unique identifier of the job for which the permitted caller is being updated.
    /// @param new_permitted_caller The Ethereum address of the new permitted caller.
    ///
    /// @dev This function provides flexibility in managing permitted callers for different job types.
    /// The caller can specify the job key, job ID, and the new Ethereum address that will be granted permission
    /// to initiate the specified job. It is important for the caller to ensure that the provided parameters
    /// align with the ongoing processes and permissions within the contract.
    ///
    function setPermittedCaller(
        uint16 role_type,
        uint32 job_id,
        address new_permitted_caller
    ) external;
}
