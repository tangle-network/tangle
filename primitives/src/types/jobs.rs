use super::*;


/// Represents a job submission with specified `AccountId` and `BlockNumber`.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct JobSubmission<AccountId, BlockNumber> {
    /// The owner's account ID.
    pub owner: AccountId,

    /// The expiry block number.
    pub expiry: BlockNumber,

    /// The type of the job submission.
    pub job_type: JobType<AccountId>,
}

/// Enum representing different types of jobs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum JobType<AccountId> {
    /// Distributed Key Generation (DKG) job type.
    DKG(DKGJobType<AccountId>),
    /// DKG Signature job type.
    DKGSignature(DKGSignatureJobType),
    /// (zk-SNARK) Phase One job type.
    ZkSaasPhaseOne(ZkSaasPhaseOneJobType<AccountId>),
    /// (zk-SNARK) Phase Two job type.
    ZkSaasPhaseTwo(ZkSaasPhaseTwoJobType),
}

/// Represents the Distributed Key Generation (DKG) job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct DKGJobType<AccountId> {
    /// List of participants' account IDs.
    pub participants: Vec<AccountId>,

    /// The threshold value for the DKG.
    pub threshold: u8,
}

/// Represents the DKG Signature job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct DKGSignatureJobType {
    /// The phase one ID.
    pub phase_one_id: u8,

    /// The submission data as a vector of bytes.
    pub submission: Vec<u8>,
}

/// Represents the (zk-SNARK) Phase One job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct ZkSaasPhaseOneJobType<AccountId> {
    /// List of participants' account IDs.
    pub participants: Vec<AccountId>,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct ZkSaasPhaseTwoJobType {
    /// The phase one ID.
    pub phase_one_id: u8,

    /// The submission data as a vector of bytes.
    pub submission: Vec<u8>,
}

/// Enum representing different states of a job.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum JobState {
    /// The job is active.
    Active,
    /// The job is pending.
    Pending,
    /// The job has been terminated.
    Terminated,
}