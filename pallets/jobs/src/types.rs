use super::*;
use tangle_primitives::jobs::*;

pub type JobSubmissionOf<T> = JobSubmission<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;
