use ark_crypto_primitives::{Error, snark::SNARK};
use ark_ec::pairing::Pairing;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use sp_std::marker::PhantomData;

pub fn verify_groth16<E: Pairing>(
	vk: &VerifyingKey<E>,
	public_inputs: &[E::ScalarField],
	proof: &Proof<E>,
) -> Result<bool, Error> {
	let res = Groth16::<E>::verify(vk, public_inputs, proof)?;
	Ok(res)
}

#[derive(Default, Clone, Copy)]
pub struct ArkworksVerifierGroth16<E: Pairing>(PhantomData<E>);

impl<E: Pairing> super::InstanceVerifier for ArkworksVerifierGroth16<E> {
	fn verify(public_inp_bytes: &[u8], proof_bytes: &[u8], vk_bytes: &[u8]) -> Result<bool, Error> {
		let public_input_field_elts = super::to_field_elements::<E::ScalarField>(public_inp_bytes)?;
		let vk = VerifyingKey::<E>::deserialize_compressed(vk_bytes)?;
		let proof = Proof::<E>::deserialize_compressed(proof_bytes)?;
		let res = verify_groth16::<E>(&vk, &public_input_field_elts, &proof)?;
		Ok(res)
	}
}

use ark_bn254::Bn254;
pub type ArkworksVerifierGroth16Bn254 = ArkworksVerifierGroth16<Bn254>;
