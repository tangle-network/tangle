use ark_crypto_primitives::Error;
use ark_ff::{BigInteger, PrimeField};

pub mod arkworks;
pub mod circom;

/// bytes field is assumed to contain concatenated, Big-endian encoded chunks.
pub fn to_field_elements<F: PrimeField>(bytes: &[u8]) -> Result<Vec<F>, Error> {
	let max_size_bytes = F::BigInt::NUM_LIMBS * 8;

	// Pad the input with zeros to prevent crashes in arkworks
	let padding_len = (max_size_bytes - (bytes.len() % max_size_bytes)) % max_size_bytes;
	let padded_input: Vec<u8> =
		bytes.iter().cloned().chain(core::iter::repeat(0u8).take(padding_len)).collect();

	// Reverse all chunks so the values are formatted in little-endian.
	// This is necessary because arkworks assumes little-endian.
	let mut reversed_chunks: Vec<u8> = Vec::with_capacity(bytes.len() + padding_len);

	for chunk in padded_input.chunks(max_size_bytes) {
		reversed_chunks.extend(chunk.iter().rev());
	}

	// Read the chunks into arkworks to convert into field elements.
	let res = reversed_chunks
		.chunks(max_size_bytes)
		.map(F::from_le_bytes_mod_order)
		.collect::<Vec<_>>();
	Ok(res)
}

/// Convert a vector of field elements into a vector of bytes.
pub fn from_field_elements<F: PrimeField>(elts: &[F]) -> Result<Vec<u8>, Error> {
	let res = elts.iter().fold(vec![], |mut acc, prev| {
		acc.extend_from_slice(&prev.into_bigint().to_bytes_be());
		acc
	});

	Ok(res)
}

// A trait meant to be implemented over a zero-knowledge verifier function.
pub trait InstanceVerifier {
	fn verify(pub_inps: &[u8], proof: &[u8], params: &[u8]) -> Result<bool, Error>;
}

impl<V1, V2> InstanceVerifier for (V1, V2)
where
	V1: InstanceVerifier,
	V2: InstanceVerifier,
{
	fn verify(pub_inps: &[u8], proof: &[u8], params: &[u8]) -> Result<bool, Error> {
		// Sequance flow,
		// if the first failed, we will try the other verifier.
		V1::verify(pub_inps, proof, params).or_else(|_| V2::verify(pub_inps, proof, params))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	type F = ark_bn254::Fr;

	#[test]
	fn should_convert_from_and_to_bytes() {
		let x = vec![F::from(1u8), F::from(2u8), F::from(3u8), F::from(4u8)];
		let bytes = from_field_elements::<F>(&x).unwrap();
		let y = to_field_elements::<F>(&bytes).unwrap();
		assert_eq!(x, y);
	}
}
