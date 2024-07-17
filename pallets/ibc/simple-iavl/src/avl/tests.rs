//! # Test suite of tendermock AVL Tree.

use ics23::{commitment_proof::Proof, verify_membership, HostFunctionsManager};
use sha2::{Digest, Sha256};

use crate::avl::{
	node::{as_node_ref, NodeRef},
	tree::AvlTree,
	*,
};

#[test]
fn insert() {
	let data = [42];
	let mut tree = AvlTree::new();
	let target =
		AvlTree { root: build_node([1], data, as_node_ref([0], data), as_node_ref([2], data)) };
	tree.insert([1], data);
	tree.insert([0], data);
	tree.insert([2], data);
	assert_eq!(tree, target);
}

#[test]
fn get() {
	let mut tree = AvlTree::new();
	tree.insert([1], [1]);
	tree.insert([2], [2]);
	tree.insert([0], [0]);
	tree.insert([5], [5]);

	assert_eq!(tree.get(&[0]), Some(&[0]));
	assert_eq!(tree.get(&[1]), Some(&[1]));
	assert_eq!(tree.get(&[2]), Some(&[2]));
	assert_eq!(tree.get(&[5]), Some(&[5]));
	assert_eq!(tree.get(&[4]), None);
}

#[test]
fn rotate_right() {
	let mut before = AvlTree {
		root: build_node(
			[5],
			[5],
			build_node([3], [3], as_node_ref([2], [2]), as_node_ref([4], [4])),
			as_node_ref([6], [6]),
		),
	};
	let after = AvlTree {
		root: build_node(
			[3],
			[3],
			as_node_ref([2], [2]),
			build_node([5], [5], as_node_ref([4], [4]), as_node_ref([6], [6])),
		),
	};
	AvlTree::rotate_right(&mut before.root);
	assert_eq!(before, after);
}

#[test]
fn rotate_left() {
	let mut before = AvlTree {
		root: build_node(
			[1],
			[1],
			as_node_ref([0], [0]),
			build_node([3], [3], as_node_ref([2], [2]), as_node_ref([4], [4])),
		),
	};
	let after = AvlTree {
		root: build_node(
			[3],
			[3],
			build_node([1], [1], as_node_ref([0], [0]), as_node_ref([2], [2])),
			as_node_ref([4], [4]),
		),
	};
	AvlTree::rotate_left(&mut before.root);
	assert_eq!(before, after);
}

#[test]
fn proof() {
	let mut tree = AvlTree::new();
	tree.insert("A", [0]);
	tree.insert("B", [1]);
	let node_a = tree.root.as_ref().unwrap();
	let node_b = node_a.right.as_ref().unwrap();
	let root = tree.root_hash().expect("Unable to retrieve root hash");
	let ics_proof = tree.get_proof("B").expect("Unable to retrieve proof for 'B'");
	let proof = match &ics_proof.proof.as_ref().unwrap() {
		Proof::Exist(proof) => proof,
		_ => panic!("Should return an existence proof"),
	};
	assert_eq!(proof.path.len(), 2);
	// Apply leaf transformations
	let leaf = proof.leaf.as_ref().expect("There should be a leaf in the proof");
	let mut sha = Sha256::new();
	sha.update(&leaf.prefix);
	sha.update("B".as_bytes());
	sha.update([1]);
	let child_hash = sha.finalize();
	// Apply first inner node transformations
	let inner_b = &proof.path[0];
	let mut sha = Sha256::new();
	sha.update(&inner_b.prefix);
	sha.update(child_hash);
	sha.update(&inner_b.suffix);
	let inner_hash_b = sha.finalize();
	assert_eq!(inner_hash_b.as_slice(), node_b.merkle_hash.as_bytes());
	// Apply second inner node transformations
	let inner_a = &proof.path[1];
	let mut sha = Sha256::new();
	sha.update(&inner_a.prefix);
	sha.update(inner_hash_b);
	sha.update(&inner_a.suffix);
	let inner_hash_a = sha.finalize();
	assert_eq!(inner_hash_a.as_slice(), node_a.merkle_hash.as_bytes());
	// Check with ics32
	let spec = get_proof_spec();
	assert!(verify_membership::<HostFunctionsManager>(
		&ics_proof,
		&spec,
		&root.as_bytes().to_vec(),
		"B".as_bytes(),
		&[1]
	));
}

#[test]
fn integration() {
	let mut tree = AvlTree::new();
	tree.insert("M", [0]);
	tree.insert("N", [0]);
	tree.insert("O", [0]);
	tree.insert("L", [0]);
	tree.insert("K", [0]);
	tree.insert("Q", [0]);
	tree.insert("P", [0]);
	tree.insert("H", [0]);
	tree.insert("I", [0]);
	tree.insert("A", [0]);
	assert!(check_integrity(&tree.root));

	let root = tree.root_hash().expect("Unable to retrieve root hash").as_bytes().to_vec();
	let proof = tree.get_proof("K").expect("Unable to retrieve a proof for 'K'");
	let spec = get_proof_spec();
	assert!(verify_membership::<HostFunctionsManager>(&proof, &spec, &root, "K".as_bytes(), &[0]));
}

/// Check that nodes are ordered, heights are correct and that balance factors are in {-1, 0, 1}.
fn check_integrity<T: Ord, V>(node_ref: &NodeRef<T, V>) -> bool {
	if let Some(node) = node_ref {
		let mut left_height = 0;
		let mut right_height = 0;
		let mut is_leaf = true;
		if let Some(ref left) = node.left {
			if left.key >= node.key {
				println!("[AVL]: Left child should have a smaller key");
				return false;
			}
			left_height = left.height;
			is_leaf = false;
		}
		if let Some(ref right) = node.right {
			if right.key <= node.key {
				println!("[AVL]: Right child should have a bigger key");
				return false;
			}
			right_height = right.height;
			is_leaf = false;
		}
		let balance_factor = (left_height as i32) - (right_height as i32);
		if balance_factor <= -2 {
			println!("[AVL] Balance factor <= -2");
			return false;
		} else if balance_factor >= 2 {
			println!("[AVL] Balance factor >= 2");
			return false;
		}
		let bonus_height = if is_leaf { 0 } else { 1 };
		if node.height != std::cmp::max(left_height, right_height) + bonus_height {
			println!("[AVL] Heights are inconsistent");
			return false;
		}
		check_integrity(&node.left) && check_integrity(&node.right)
	} else {
		true
	}
}

/// An helper function to build simple AvlNodes.
#[allow(clippy::unnecessary_wraps)]
fn build_node<T: Ord + AsBytes>(
	key: T,
	value: [u8; 1],
	left: NodeRef<T, [u8; 1]>,
	right: NodeRef<T, [u8; 1]>,
) -> NodeRef<T, [u8; 1]> {
	let mut node = as_node_ref(key, value).unwrap();
	node.left = left;
	node.right = right;
	node.update();
	Some(node)
}
