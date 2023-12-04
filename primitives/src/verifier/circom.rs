use core::convert::{TryFrom, TryInto};

use ark_bn254::{Bn254, Fr, G1Affine, G2Affine};
use ark_crypto_primitives::{Error, snark::SNARK};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{
	Groth16, Proof as ArkProof, VerifyingKey as ArkVerifyingKey,
};
use ark_serialize::CanonicalDeserialize;
use ark_ff::Zero;
use ethabi::{ethereum_types::U256, ParamType};
use sp_std::prelude::*;

pub struct CircomVerifierBn254;

#[derive(Debug)]
pub enum CircomError {
	InvalidVerifyingKeyBytes,
	InvalidProofBytes,
	InvalidBuilderConfig,
	ProvingFailure,
	VerifyingFailure,
	ParameterGenerationFailure,
}

impl ark_std::error::Error for CircomError {}

impl core::fmt::Display for CircomError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			CircomError::InvalidVerifyingKeyBytes => write!(f, "Invalid verifying key bytes"),
			CircomError::InvalidProofBytes => write!(f, "Invalid proof bytes"),
			CircomError::InvalidBuilderConfig => write!(f, "Invalid builder config"),
			CircomError::ProvingFailure => write!(f, "Proving failure"),
			CircomError::VerifyingFailure => write!(f, "Verifying failure"),
			CircomError::ParameterGenerationFailure => write!(f, "Parameter generation failure"),
		}
	}
}

pub fn verify_groth16<E: Pairing>(
	vk: &ArkVerifyingKey<E>,
	public_inputs: &[E::ScalarField],
	proof: &ArkProof<E>,
) -> Result<bool, Error> {
	let res = Groth16::<E>::verify(vk, public_inputs, proof)?;
	Ok(res)
}

impl super::InstanceVerifier for CircomVerifierBn254 {
	fn verify(public_inp_bytes: &[u8], proof_bytes: &[u8], vk_bytes: &[u8]) -> Result<bool, Error> {
		let public_input_field_elts = match super::to_field_elements::<Fr>(public_inp_bytes) {
			Ok(v) => v,
			Err(e) => {
				frame_support::log::error!(
					"Failed to convert public input bytes to field elements: {e:?}",
				);
				return Err(e)
			},
		};
		let vk = match ArkVerifyingKey::deserialize_compressed(vk_bytes) {
			Ok(v) => v,
			Err(e) => {
				frame_support::log::error!("Failed to deserialize verifying key: {e:?}");
				return Err(e.into())
			},
		};
		let proof = match Proof::decode(proof_bytes).and_then(|v| v.try_into()) {
			Ok(v) => v,
			Err(e) => {
				frame_support::log::error!("Failed to deserialize proof: {e:?}");
				return Err(e)
			},
		};
		let res = match verify_groth16(&vk, &public_input_field_elts, &proof) {
			Ok(v) => v,
			Err(e) => {
				frame_support::log::error!("Failed to verify proof: {e:?}");
				return Err(e)
			},
		};

		Ok(res)
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct G1 {
	pub x: U256,
	pub y: U256,
}

impl TryFrom<G1> for G1Affine {
	type Error = Error;
	fn try_from(src: G1) -> Result<Self, Self::Error> {
		let x: ark_bn254::Fq = u256_to_point(src.x)?;
		let y: ark_bn254::Fq = u256_to_point(src.y)?;
		let inf = x.is_zero() && y.is_zero();
		Ok(Self { x, y, infinity: inf })
	}
}

impl TryFrom<&G1Affine> for G1 {
	type Error = Error;
	fn try_from(p: &G1Affine) -> Result<Self, Self::Error> {
		Ok(Self { x: point_to_u256(p.x)?, y: point_to_u256(p.y)? })
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct G2 {
	pub x: [U256; 2],
	pub y: [U256; 2],
}

impl TryFrom<G2> for G2Affine {
	type Error = Error;
	fn try_from(src: G2) -> Result<Self, Self::Error> {
		let c0 = u256_to_point(src.x[0])?;
		let c1 = u256_to_point(src.x[1])?;
		let x = ark_bn254::Fq2::new(c0, c1);

		let c0 = u256_to_point(src.y[0])?;
		let c1 = u256_to_point(src.y[1])?;
		let y = ark_bn254::Fq2::new(c0, c1);

		let inf = x.is_zero() && y.is_zero();
		Ok(Self { x, y, infinity: inf })
	}
}

impl TryFrom<&G2Affine> for G2 {
	type Error = Error;
	fn try_from(p: &G2Affine) -> Result<Self, Self::Error> {
		Ok(Self {
			x: [point_to_u256(p.x.c0)?, point_to_u256(p.x.c1)?],
			y: [point_to_u256(p.y.c0)?, point_to_u256(p.y.c1)?],
		})
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Proof {
	pub a: G1,
	pub b: G2,
	pub c: G1,
}

impl Proof {
	pub fn decode(input: &[u8]) -> Result<Self, Error> {
		// (uint[2] a,uint[2][2] b,uint[2] c)
		let mut decoded = ethabi::decode(
			&[ParamType::Tuple(sp_std::vec![
				ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
				ParamType::FixedArray(
					Box::new(ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2)),
					2,
				),
				ParamType::FixedArray(Box::new(ParamType::Uint(256)), 2),
			])],
			input,
		)
		.map_err(|e| {
			frame_support::log::error!("Failed to decode proof: {:?}", e);
			CircomError::InvalidProofBytes
		})?;
		// Unwrap the decoded tuple
		let decoded = decoded.pop().ok_or(CircomError::InvalidProofBytes)?;
		let decoded = match decoded {
			ethabi::Token::Tuple(v) => v,
			_ => return Err(CircomError::InvalidProofBytes.into()),
		};
		let a = decoded[0].clone().into_fixed_array().ok_or(CircomError::InvalidProofBytes)?;
		let a_x = a[0].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		let a_y = a[1].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;

		let b = decoded[1].clone().into_fixed_array().ok_or(CircomError::InvalidProofBytes)?;
		let b_x = b[0].clone().into_fixed_array().ok_or(CircomError::InvalidProofBytes)?;
		let b_y = b[1].clone().into_fixed_array().ok_or(CircomError::InvalidProofBytes)?;
		let b_x_0 = b_x[0].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		let b_x_1 = b_x[1].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		let b_y_0 = b_y[0].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		let b_y_1 = b_y[1].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;

		let c = decoded[2].clone().into_fixed_array().ok_or(CircomError::InvalidProofBytes)?;
		let c_x = c[0].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		let c_y = c[1].clone().into_uint().ok_or(CircomError::InvalidProofBytes)?;
		Ok(Self {
			a: G1 { x: a_x, y: a_y },
			b: G2 { x: [b_x_1, b_x_0], y: [b_y_1, b_y_0] },
			c: G1 { x: c_x, y: c_y },
		})
	}

	pub fn encode(&self) -> Result<Vec<u8>, Error> {
		let a_x = self.a.x;
		let a_y = self.a.y;
		let b_x_0 = self.b.x[0];
		let b_x_1 = self.b.x[1];
		let b_y_0 = self.b.y[0];
		let b_y_1 = self.b.y[1];
		let c_x = self.c.x;
		let c_y = self.c.y;
		let encoded = ethabi::encode(&[ethabi::Token::Tuple(vec![
			ethabi::Token::FixedArray(vec![ethabi::Token::Uint(a_x), ethabi::Token::Uint(a_y)]),
			ethabi::Token::FixedArray(vec![
				ethabi::Token::FixedArray(vec![
					ethabi::Token::Uint(b_x_1),
					ethabi::Token::Uint(b_x_0),
				]),
				ethabi::Token::FixedArray(vec![
					ethabi::Token::Uint(b_y_1),
					ethabi::Token::Uint(b_y_0),
				]),
			]),
			ethabi::Token::FixedArray(vec![ethabi::Token::Uint(c_x), ethabi::Token::Uint(c_y)]),
		])]);
		Ok(encoded)
	}
}

impl TryFrom<ArkProof<Bn254>> for Proof {
	type Error = Error;
	fn try_from(proof: ArkProof<Bn254>) -> Result<Self, Self::Error> {
		Ok(Self {
			a: G1::try_from(&proof.a)?,
			b: G2::try_from(&proof.b)?,
			c: G1::try_from(&proof.c)?,
		})
	}
}

impl TryFrom<Proof> for ArkProof<Bn254> {
	type Error = Error;
	fn try_from(src: Proof) -> Result<Self, Self::Error> {
		Ok(Self { a: src.a.try_into()?, b: src.b.try_into()?, c: src.c.try_into()? })
	}
}

// Helper for converting a PrimeField to its U256 representation for Ethereum compatibility
fn u256_to_point<F: PrimeField>(point: U256) -> Result<F, Error> {
	let mut buf = [0; 32];
	point.to_little_endian(&mut buf);
	Ok(F::from_le_bytes_mod_order(&buf[..]))
}

// Helper for converting a PrimeField to its U256 representation for Ethereum compatibility
// (U256 reads data as big endian)
fn point_to_u256<F: PrimeField>(point: F) -> Result<U256, Error> {
	let point = point.into_bigint();
	let point_bytes = point.to_bytes_be();
	if point_bytes.len() != 32 {
		return Err(CircomError::InvalidProofBytes.into())
	}
	Ok(U256::from(&point_bytes[..]))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_js_solidity_proof_to_arkworks() {
		let js_proof_bytes = hex::decode("283214454fd3acd78dd7d83e2e7ff187918f93c83a7a29c65e9d84c5b796e2f4165dedc98635cbb7226bca867c4b3454cc002902d74684b63bbba33bfbfe0b9e27f8c215f3b5574fa8c4cef8b4eacfe2577a17c37f60f0f037dec244d5f6d31401c2f126b04cb69727b8c273612659a3dd6cddb96891c2c2ebea6c313956ff700ebb472ecead76346d13468cf9eea1269b5a94b3c847840d5a5bb9dba50c39f029801c58394e18719ffacc6752e803b2e3fade1219f423c38618799bd954e9b910b3936beafe6bd89c38fe0f297a0c2387d20df79e9f20b4f04b3ae59ce9a22a0c08e7eae8e0b4f5234c040436720e5c44326034e69f4b0e5236958571b5f216").unwrap();
		let eth_proof = Proof::decode(&js_proof_bytes[..]).unwrap();
		eprintln!("eth_proof: {eth_proof:#?}");
		let ark_proof: ArkProof<Bn254> = eth_proof.try_into().unwrap();
		let eth_proof2: Proof = ark_proof.clone().try_into().unwrap();
		assert_eq!(eth_proof, eth_proof2);
		let ark_proof2: ArkProof<Bn254> = eth_proof2.try_into().unwrap();
		assert_eq!(ark_proof, ark_proof2);
	}
}
