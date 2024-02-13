pub mod bls12_381;
pub mod ecdsa;
pub mod schnorr_frost;
pub mod schnorr_sr25519;

/// Utility function to create slice of fixed size
pub fn to_slice_33(val: &[u8]) -> Option<[u8; 33]> {
	if val.len() == 33 {
		let mut key = [0u8; 33];
		key[..33].copy_from_slice(val);

		return Some(key)
	}
	None
}

/// Utility function to create slice of fixed size
pub fn to_slice_32(val: &[u8]) -> Option<[u8; 32]> {
	if val.len() == 32 {
		let mut key = [0u8; 32];
		key[..32].copy_from_slice(val);

		return Some(key)
	}
	None
}
