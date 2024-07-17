use alloc::boxed::Box;
use core::{borrow::Borrow, mem};

use sha2::{Digest, Sha256};
use tendermint::hash::Hash;

use crate::avl::{as_bytes::AsBytes, proof::LEAF_PREFIX, HASH_ALGO};

pub type NodeRef<T, V> = Option<Box<AvlNode<T, V>>>;

/// A node in the AVL Tree.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct AvlNode<K: Ord, V> {
	pub key: K,
	pub value: V,
	pub hash: Hash,
	pub merkle_hash: Hash,
	pub height: u32,
	pub left: NodeRef<K, V>,
	pub right: NodeRef<K, V>,
}

/// Wrap a key + value couple into a `NodeRef`.
#[allow(clippy::unnecessary_wraps)]
pub fn as_node_ref<K: Ord + AsBytes, V>(key: K, value: V) -> NodeRef<K, V>
where
	V: Borrow<[u8]>,
{
	Some(Box::new(AvlNode::new(key, value)))
}

impl<K: Ord + AsBytes, V> AvlNode<K, V>
where
	V: Borrow<[u8]>,
{
	fn new(key: K, value: V) -> Self {
		let mut sha = Sha256::new();
		sha.update(LEAF_PREFIX);
		sha.update(key.as_bytes().as_ref());
		sha.update(value.borrow());
		let hash = sha.finalize();
		let merkle_hash = Hash::from_bytes(HASH_ALGO, &Sha256::digest(hash)).unwrap();
		let hash = Hash::from_bytes(HASH_ALGO, &hash).unwrap();

		AvlNode { key, value, hash, merkle_hash, height: 0, left: None, right: None }
	}

	/// Set the value of the current node.
	pub(crate) fn set_value(&mut self, value: V) -> V {
		let hash = Self::local_hash(&self.key, &value);
		self.hash = hash;
		mem::replace(&mut self.value, value)
	}

	/// The left height, or `None` if there is no left child.
	fn left_height(&self) -> Option<u32> {
		self.left.as_ref().map(|left| left.height)
	}

	/// The right height, or `None` if there is no right child.
	fn right_height(&self) -> Option<u32> {
		self.right.as_ref().map(|right| right.height)
	}

	/// Compute the local hash for a given key and value.
	fn local_hash(key: &K, value: &V) -> Hash {
		let mut sha = Sha256::new();
		sha.update(LEAF_PREFIX);
		sha.update(key.as_bytes());
		sha.update(value.borrow());
		let hash = sha.finalize();
		Hash::from_bytes(HASH_ALGO, &hash).unwrap()
	}

	/// The left merkle hash, if any
	pub fn left_hash(&self) -> Option<&[u8]> {
		Some(self.left.as_ref()?.merkle_hash.as_bytes())
	}

	/// The right merkle hash, if any
	pub fn right_hash(&self) -> Option<&[u8]> {
		Some(self.right.as_ref()?.merkle_hash.as_bytes())
	}

	/// Update the height of this node by looking at the height of its two children.
	/// The height of this node is computed as the maximum among the height of its two children, and
	/// incremented by 1.
	fn update_height(&mut self) {
		match &self.right {
			None => match &self.left {
				None => self.height = 0,
				Some(left) => self.height = left.height + 1,
			},
			Some(right) => match &self.left {
				None => self.height = right.height + 1,
				Some(left) => self.height = core::cmp::max(left.height, right.height) + 1,
			},
		}
	}

	/// Update the node's merkle hash by looking at the hashes of its two children.
	fn update_hashes(&mut self) {
		let mut sha = Sha256::new();
		if let Some(left) = &self.left {
			sha.update(left.merkle_hash.as_bytes());
		}
		sha.update(self.hash.as_bytes());
		if let Some(right) = &self.right {
			sha.update(right.merkle_hash.as_bytes())
		}
		self.merkle_hash = Hash::from_bytes(HASH_ALGO, sha.finalize().as_slice()).unwrap();
	}

	/// Update node meta data, such as its height and merkle hash, by looking at its two
	/// children.
	pub fn update(&mut self) {
		self.update_hashes();
		self.update_height();
	}

	/// Returns the node's balance factor (left_height - right_height).
	pub fn balance_factor(&self) -> i32 {
		match (self.left_height(), self.right_height()) {
			(None, None) => 0,
			(None, Some(h)) => -(h as i32),
			(Some(h), None) => h as i32,
			(Some(h_l), Some(h_r)) => (h_l as i32) - (h_r as i32),
		}
	}
}
