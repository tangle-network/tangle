use alloc::string::String;
use base58::{FromBase58, ToBase58};
use sp_core::{crypto::ByteArray, H256};
use sp_runtime::AccountId32;
use sp_std::prelude::*;

#[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, Debug)]
pub enum SS58CodecError {
	/// Invalid SS58 String
	InvalidString,
	/// Invalid Account id
	InvalidAccountId,
}

pub fn blake2_256_verify_non_membership_proof(root: &H256, proof: &[Vec<u8>], key: &[u8]) -> bool {
	sp_trie::verify_trie_proof::<sp_trie::LayoutV0<sp_runtime::traits::BlakeTwo256>, _, _, &[u8]>(
		root,
		proof,
		&[(key, None)],
	)
	.is_ok()
}

pub fn sha2_512(message: &[u8]) -> [u8; 64] {
	use sha2::Digest;
	let mut hasher = sha2::Sha512::new();
	hasher.update(message);
	let hash = hasher.finalize();
	let mut res = [0u8; 64];
	res.copy_from_slice(&hash);
	res
}

pub fn sha2_512_truncated(message: &[u8]) -> [u8; 32] {
	use sha2::Digest;
	let mut hasher = sha2::Sha512::new();
	hasher.update(message);
	let hash = hasher.finalize();
	let mut res = [0u8; 32];
	res.copy_from_slice(&hash[..32]);
	res
}

pub fn sha3_512(message: &[u8]) -> [u8; 64] {
	use sha3::Digest;
	let mut hasher = sha3::Sha3_512::new();
	hasher.update(message);
	let hash = hasher.finalize();
	let mut res = [0u8; 64];
	res.copy_from_slice(&hash);
	res
}

pub fn ripemd160(message: &[u8]) -> [u8; 20] {
	use ripemd::Digest;
	let mut hasher = ripemd::Ripemd160::new();
	hasher.update(message);
	let hash = hasher.finalize();
	let mut res = [0u8; 20];
	res.copy_from_slice(&hash);
	res
}

pub fn ss58_to_account_id_32(raw_str: &str) -> Result<[u8; 32], SS58CodecError> {
	from_ss58check_with_version::<AccountId32>(raw_str)
		.map(|acc| acc.into())
		.map_err(|_| SS58CodecError::InvalidString)
}

#[inline(always)]
pub fn account_id_to_ss58(bytes: [u8; 32], version: u16) -> String {
	let account_id = AccountId32::new(bytes);
	to_ss58check_with_version(account_id, version)
}

// lifted directly from sp-core
fn to_ss58check_with_version(account_id: impl ByteArray, version: u16) -> String {
	// We mask out the upper two bits of the ident - SS58 Prefix currently only supports 14-bits
	let ident: u16 = version & 0b0011_1111_1111_1111;
	let mut v = match ident {
		0..=63 => vec![ident as u8],
		64..=16_383 => {
			// upper six bits of the lower byte(!)
			let first = ((ident & 0b0000_0000_1111_1100) as u8) >> 2;
			// lower two bits of the lower byte in the high pos,
			// lower bits of the upper byte in the low pos
			let second = ((ident >> 8) as u8) | ((ident & 0b0000_0000_0000_0011) as u8) << 6;
			vec![first | 0b01000000, second]
		},
		_ => unreachable!("masked out the upper two bits; qed"),
	};
	v.extend(account_id.as_ref());
	let r = ss58hash(&v);
	v.extend(&r[0..2]);
	v.to_base58()
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum PublicError {
	BadBase58,
	BadLength,
	InvalidChecksum,
	InvalidPrefix,
	InvalidFormat,
	InvalidPath,
	FormatNotAllowed,
}

// lifted directly from sp-core
fn from_ss58check_with_version<T>(s: &str) -> Result<T, PublicError>
where
	T: ByteArray,
{
	const CHECKSUM_LEN: usize = 2;
	let body_len = T::LEN;

	let data = s.from_base58().map_err(|_| PublicError::BadBase58)?;
	if data.len() < 2 {
		return Err(PublicError::BadLength);
	}
	let (prefix_len, _) = match data[0] {
		0..=63 => (1, data[0] as u16),
		64..=127 => {
			// weird bit manipulation owing to the combination of LE encoding and missing two
			// bits from the left.
			// d[0] d[1] are: 01aaaaaa bbcccccc
			// they make the LE-encoded 16-bit value: aaaaaabb 00cccccc
			// so the lower byte is formed of aaaaaabb and the higher byte is 00cccccc
			let lower = (data[0] << 2) | (data[1] >> 6);
			let upper = data[1] & 0b00111111;
			(2, (lower as u16) | ((upper as u16) << 8))
		},
		_ => return Err(PublicError::InvalidPrefix),
	};
	if data.len() != prefix_len + body_len + CHECKSUM_LEN {
		return Err(PublicError::BadLength);
	}

	let hash = ss58hash(&data[0..body_len + prefix_len]);
	let checksum = &hash[0..CHECKSUM_LEN];
	if data[body_len + prefix_len..body_len + prefix_len + CHECKSUM_LEN] != *checksum {
		// Invalid checksum.
		return Err(PublicError::InvalidChecksum);
	}

	let result = T::from_slice(&data[prefix_len..body_len + prefix_len])
		.map_err(|()| PublicError::BadLength)?;
	Ok(result)
}

/// uses host functions for hashing
fn ss58hash(data: &[u8]) -> Vec<u8> {
	use blake2::{Blake2b512, Digest};

	let mut ctx = Blake2b512::new();
	ctx.update(b"SS58PRE");
	ctx.update(data);
	ctx.finalize().to_vec()
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{
		crypto::{Ss58AddressFormat, Ss58AddressFormatRegistry},
		ecdsa::{Pair, Public},
		Pair as _,
	};

	#[test]
	fn ss58_test() {
		// make sure that decode(encode(address)) = address
		let alice = "5yNZjX24n2eg7W6EVamaTXNQbWCwchhThEaSWB7V3GRjtHeL";
		let account_id = ss58_to_account_id_32(alice).unwrap();
		let ss58_account = account_id_to_ss58(account_id, 49);
		assert_eq!(alice, ss58_account);
	}

	#[test]
	fn ss58check_format_check_works() {
		let pair = Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();
		let format: Ss58AddressFormat = Ss58AddressFormatRegistry::Reserved46Account.into();
		let s = to_ss58check_with_version(public, u16::from(format));
		assert_eq!(from_ss58check_with_version::<Public>(&s), Ok(public));
	}

	#[test]
	fn ss58check_full_roundtrip_works() {
		let pair = Pair::from_seed(b"12345678901234567890123456789012");
		let public = pair.public();
		let format: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadotAccount.into();
		let s = to_ss58check_with_version(public, u16::from(format));
		let k = from_ss58check_with_version::<Public>(&s).unwrap();
		assert_eq!(k.as_slice(), public.as_slice());

		let format = Ss58AddressFormat::custom(64);
		let s = to_ss58check_with_version(public, u16::from(format));
		let k = from_ss58check_with_version::<Public>(&s).unwrap();
		assert_eq!(k.as_slice(), public.as_slice());
	}
}
