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

use alloc::vec::Vec;
use ibc::core::ics23_commitment::{error::Error, merkle::MerkleProof};
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use tendermint::merkle::proof::ProofOps;

#[allow(unused)]
pub fn convert_tm_to_ics_merkle_proof<H>(tm_proof: &ProofOps) -> Result<MerkleProof<H>, Error> {
	let mut proofs = Vec::new();

	for op in &tm_proof.ops {
		let mut parsed = ibc_proto::cosmos::ics23::v1::CommitmentProof { proof: None };
		prost::Message::merge(&mut parsed, op.data.as_slice())
			.map_err(Error::commitment_proof_decoding_failed)?;

		proofs.push(parsed);
	}

	Ok(MerkleProof::from(RawMerkleProof { proofs }))
}
