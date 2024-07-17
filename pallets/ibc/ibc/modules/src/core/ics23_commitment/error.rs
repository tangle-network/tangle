// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use flex_error::{define_error, TraceError};
use prost::DecodeError;

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	Error {
		InvalidRawMerkleProof
			[ TraceError<DecodeError> ]
			|_| { "invalid raw merkle proof" },

		CommitmentProofDecodingFailed
			[ TraceError<DecodeError> ]
			|_| { "failed to decode commitment proof" },

		EmptyCommitmentPrefix
			|_| { "empty commitment prefix" },

		EmptyMerkleProof
			|_| { "empty merkle proof" },

		EmptyMerkleRoot
			|_| { "empty merkle root" },

		EmptyVerifiedValue
			|_| { "empty verified value" },

		NumberOfSpecsMismatch
			|_| { "mismatch between the number of proofs with that of specs" },

		NumberOfKeysMismatch
			|_| { "mismatch between the number of proofs with that of keys" },

		InvalidMerkleProof
			|_| { "invalid merkle proof" },

		VerificationFailure
			|_| { "proof verification failed" }
	}
}
