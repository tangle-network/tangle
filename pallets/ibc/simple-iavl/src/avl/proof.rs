//! # ICS23 Proof
//!
//! This module provides the ICS23 proof spec, which can be used to verify the existence of a value
//! in the AVL Tree.
use alloc::vec;
use ics23::{HashOp, InnerSpec, LeafOp, LengthOp, ProofSpec};

pub const LEAF_PREFIX: [u8; 64] = [0; 64]; // 64 bytes of zeroes.

#[allow(dead_code)]
/// Return the `ProofSpec` of tendermock AVL Tree.
pub fn get_proof_spec() -> ProofSpec {
	ProofSpec {
		leaf_spec: Some(LeafOp {
			hash: HashOp::Sha256.into(),
			prehash_key: HashOp::NoHash.into(),
			prehash_value: HashOp::NoHash.into(),
			length: LengthOp::NoPrefix.into(),
			prefix: LEAF_PREFIX.to_vec(),
		}),
		inner_spec: Some(InnerSpec {
			child_order: vec![0, 1, 2],
			child_size: 32,
			min_prefix_length: 0,
			max_prefix_length: 64,
			empty_child: vec![0, 32],
			hash: HashOp::Sha256.into(),
		}),
		max_depth: 0,
		min_depth: 0,
		prehash_key_before_comparison: false,
	}
}
