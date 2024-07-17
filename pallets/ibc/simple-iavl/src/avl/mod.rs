//! # AVL Tree
//!
//! This module hosts a simple implementation of an AVL Merkle Tree that support the `get` and
//! `insert` instructions (no delete yet, it's not needed as the on-chain store is supposed to be
//! immutable).
//!
//! Proof of existence are supported using [ICS23](https://github.com/cosmos/ics23), but proof of
//! non-existence are not yet implemented.
//!
//! Keys needs to implement `Ord` and `AsBytes` (see `as_bytes` module), while values are required
//! to implement `Borrow<[u8]>`.
//!
//! For more info, see [AVL Tree on wikipedia](https://en.wikipedia.org/wiki/AVL_tree),

use tendermint::hash::Algorithm;

pub use as_bytes::{AsBytes, ByteSlice};
pub use node::AvlNode;
pub use proof::get_proof_spec;
pub use tree::AvlTree;

mod as_bytes;
mod node;
mod proof;
mod tree;

#[cfg(test)]
mod tests;

const HASH_ALGO: Algorithm = Algorithm::Sha256;
