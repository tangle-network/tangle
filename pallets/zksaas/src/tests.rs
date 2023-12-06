// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.
use crate::{mock::*, types::FeeInfo, FeeInfo as FeeInfoStorage};
use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_std::{
	rand::{Rng, RngCore, SeedableRng},
	test_rng,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use tangle_primitives::{
	jobs::{
		ArkworksProofResult, CircomProofResult, Groth16ProveRequest, Groth16System, HyperData,
		JobResult, JobType, JobWithResult, ZkSaasCircuitJobType, ZkSaasProofResult,
		ZkSaasProveJobType, ZkSaasProveRequest, ZkSaasSystem,
	},
	verifier::{self, from_field_elements},
};

type E = ark_bn254::Bn254;
type F = ark_bn254::Fr;

#[test]
fn set_fees_works() {
	new_test_ext().execute_with(|| {
		let new_fee = FeeInfo { base_fee: 10, circuit_fee: 5, prove_fee: 5 };

		// should fail for non update origin
		assert_noop!(ZKSaaS::set_fee(RuntimeOrigin::signed(10), new_fee.clone()), BadOrigin);

		// Dispatch a signed extrinsic.
		assert_ok!(ZKSaaS::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		assert_eq!(FeeInfoStorage::<Runtime>::get(), new_fee);
	});
}

#[test]
fn proof_verification_works() {
	new_test_ext().execute_with(|| {
		let new_fee = FeeInfo { base_fee: 10, circuit_fee: 5, prove_fee: 5 };
		// Dispatch a signed extrinsic.
		assert_ok!(ZKSaaS::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());

		// Generate the MiMC round constants
		let constants = (0..mimc::MIMC_ROUNDS).map(|_| rng.gen()).collect::<Vec<_>>();

		// Create parameters for our circuit
		let (pk, vk) = {
			let c = mimc::MiMCDemo::<F> { xl: None, xr: None, constants: &constants };
			Groth16::<E>::setup(c, &mut rng).unwrap()
		};

		let mut pk_bytes = Vec::new();
		pk.serialize_compressed(&mut pk_bytes).unwrap();

		let mut vk_bytes = Vec::new();
		vk.serialize_compressed(&mut vk_bytes).unwrap();

		// Prepare the verification key (for proof verification)
		let pvk = Groth16::<E>::process_vk(&vk).unwrap();

		// Generate a random preimage and compute the image
		let xl = rng.gen();
		let xr = rng.gen();
		let image = mimc::mimc(xl, xr, &constants);

		// Create an instance of our circuit (with the
		// witness)
		let c = mimc::MiMCDemo { xl: Some(xl), xr: Some(xr), constants: &constants };

		// Create a groth16 proof with our parameters.
		let proof = Groth16::<E>::prove(&pk, c, &mut rng).unwrap();
		// Verifiy Locally
		assert!(Groth16::<E>::verify_with_processed_vk(&pvk, &[image], &proof).unwrap());

		let mut proof_bytes = Vec::new();
		proof.serialize_compressed(&mut proof_bytes).unwrap();

		// Phase1
		let phase_one = JobType::<AccountId>::ZkSaasCircuit(ZkSaasCircuitJobType {
			participants: vec![1, 2, 3, 4, 5, 6, 7, 8],
			system: ZkSaasSystem::Groth16(Groth16System {
				circuit: HyperData::Raw(vec![]),
				proving_key: HyperData::Raw(pk_bytes),
				verifying_key: vk_bytes,
				wasm: HyperData::Raw(vec![]),
			}),
		});

		let phase_two = JobType::<AccountId>::ZkSaasProve(ZkSaasProveJobType {
			phase_one_id: 0,
			request: ZkSaasProveRequest::Groth16(Groth16ProveRequest {
				public_input: from_field_elements(&[image]).unwrap(),
				a_shares: Default::default(),
				ax: Default::default(),
				qap_shares: Default::default(),
			}),
		});

		// Arkworks
		let result = JobResult::ZkSaasProve(ZkSaasProofResult::Arkworks(ArkworksProofResult {
			proof: proof_bytes,
		}));

		let data = JobWithResult::<AccountId> {
			job_type: phase_two.clone(),
			phase_one_job_type: Some(phase_one.clone()),
			result,
		};

		assert_ok!(ZKSaaS::verify(data));

		// Circom
		let circom_proof = verifier::circom::Proof::try_from(proof).unwrap();

		let result = JobResult::ZkSaasProve(ZkSaasProofResult::Circom(CircomProofResult {
			proof: circom_proof.encode().unwrap(),
		}));

		let data = JobWithResult::<AccountId> {
			job_type: phase_two,
			phase_one_job_type: Some(phase_one),
			result,
		};

		assert_ok!(ZKSaaS::verify(data));
	});
}

/// Simple circuit for testing.
mod mimc {
	use ark_ff::Field;

	// We'll use these interfaces to construct our circuit.
	use ark_relations::{
		lc, ns,
		r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
	};

	pub const MIMC_ROUNDS: usize = 322;

	/// This is an implementation of MiMC, specifically a
	/// variant named `LongsightF322p3` for BLS12-377.
	/// See http://eprint.iacr.org/2016/492 for more
	/// information about this construction.
	///
	/// ```
	/// function LongsightF322p3(xL ⦂ Fp, xR ⦂ Fp) {
	///     for i from 0 up to 321 {
	///         xL, xR := xR + (xL + Ci)^3, xL
	///     }
	///     return xL
	/// }
	/// ```
	pub fn mimc<F: Field>(mut xl: F, mut xr: F, constants: &[F]) -> F {
		assert_eq!(constants.len(), MIMC_ROUNDS);

		(0..MIMC_ROUNDS).for_each(|i| {
			let mut tmp1 = xl;
			tmp1.add_assign(&constants[i]);
			let mut tmp2 = tmp1;
			tmp2.square_in_place();
			tmp2.mul_assign(&tmp1);
			tmp2.add_assign(&xr);
			xr = xl;
			xl = tmp2;
		});

		xl
	}

	/// This is our demo circuit for proving knowledge of the
	/// preimage of a MiMC hash invocation.
	pub struct MiMCDemo<'a, F: Field> {
		pub xl: Option<F>,
		pub xr: Option<F>,
		pub constants: &'a [F],
	}

	/// Our demo circuit implements this `Circuit` trait which
	/// is used during paramgen and proving in order to
	/// synthesize the constraint system.
	impl<'a, F: Field> ConstraintSynthesizer<F> for MiMCDemo<'a, F> {
		fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
			assert_eq!(self.constants.len(), MIMC_ROUNDS);

			// Allocate the first component of the preimage.
			let mut xl_value = self.xl;
			let mut xl =
				cs.new_witness_variable(|| xl_value.ok_or(SynthesisError::AssignmentMissing))?;

			// Allocate the second component of the preimage.
			let mut xr_value = self.xr;
			let mut xr =
				cs.new_witness_variable(|| xr_value.ok_or(SynthesisError::AssignmentMissing))?;

			for i in 0..MIMC_ROUNDS {
				// xL, xR := xR + (xL + Ci)^3, xL
				let ns = ns!(cs, "round");
				let cs = ns.cs();

				// tmp = (xL + Ci)^2
				let tmp_value = xl_value.map(|mut e| {
					e.add_assign(&self.constants[i]);
					e.square_in_place();
					e
				});
				let tmp =
					cs.new_witness_variable(|| tmp_value.ok_or(SynthesisError::AssignmentMissing))?;

				cs.enforce_constraint(
					lc!() + xl + (self.constants[i], Variable::One),
					lc!() + xl + (self.constants[i], Variable::One),
					lc!() + tmp,
				)?;

				// new_xL = xR + (xL + Ci)^3
				// new_xL = xR + tmp * (xL + Ci)
				// new_xL - xR = tmp * (xL + Ci)
				let new_xl_value = xl_value.map(|mut e| {
					e.add_assign(&self.constants[i]);
					e.mul_assign(&tmp_value.unwrap());
					e.add_assign(&xr_value.unwrap());
					e
				});

				let new_xl = if i == (MIMC_ROUNDS - 1) {
					// This is the last round, xL is our image and so
					// we allocate a public input.
					cs.new_input_variable(|| new_xl_value.ok_or(SynthesisError::AssignmentMissing))?
				} else {
					cs.new_witness_variable(|| {
						new_xl_value.ok_or(SynthesisError::AssignmentMissing)
					})?
				};

				cs.enforce_constraint(
					lc!() + tmp,
					lc!() + xl + (self.constants[i], Variable::One),
					lc!() + new_xl - xr,
				)?;

				// xR = xL
				xr = xl;
				xr_value = xl_value;

				// xL = new_xL
				xl = new_xl;
				xl_value = new_xl_value;
			}

			Ok(())
		}
	}
}
