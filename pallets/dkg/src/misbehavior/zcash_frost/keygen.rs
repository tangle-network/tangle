// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

use crate::*;
use sp_runtime::DispatchResult;

use tangle_primitives::{
	misbehavior::{MisbehaviorSubmission, SignedRoundMessage},
	roles::ThresholdSignatureRoleType,
};

use frost_core::{
	identifier::Identifier,
	keys::VerifiableSecretSharingCommitment,
	pok_challenge,
	signature::Signature,
	traits::{Ciphersuite, Group},
};

/// Verifies the proof of knowledge of the secret coefficients used to generate the
/// public secret sharing commitment.
pub fn verify_invalid_proof_of_knowledge<T: Config, C: Ciphersuite>(
	identifier: Identifier<C>,
	commitment: &VerifiableSecretSharingCommitment<C>,
	proof_of_knowledge: Signature<C>,
) -> Result<Option<Identifier<C>>, Error<T>> {
	// Round 1, Step 5
	//
	// > Upon receiving C⃗_ℓ, σ_ℓ from participants 1 ≤ ℓ ≤ n, ℓ ≠ i, participant
	// > P_i verifies σ_ℓ = (R_ℓ, μ_ℓ), aborting on failure, by checking
	// > R_ℓ ? ≟ g^{μ_ℓ} · φ^{-c_ℓ}_{ℓ0}, where c_ℓ = H(ℓ, Φ, φ_{ℓ0}, R_ℓ).
	let ell = identifier;
	let R_ell = proof_of_knowledge.R;
	let mu_ell = proof_of_knowledge.z;
	let phi_ell0 = commitment.verifying_key().map_err(|_| Error::<T>::MissingFrostCommitment)?;
	let c_ell = pok_challenge::<C>(ell, &phi_ell0, &R_ell)
		.ok_or(Error::<T>::InvalidFrostSignatureScheme)?;
	if R_ell != <C::Group>::generator() * mu_ell - phi_ell0.element * c_ell.0 {
		Ok(Some(ell))
	} else {
		Ok(None)
	}
}

pub fn schnorr_proof<T: Config>(
	_role: ThresholdSignatureRoleType,
	data: &MisbehaviorSubmission,
	parties_including_offender: &[[u8; 33]],
	round: &SignedRoundMessage,
) -> DispatchResult {
	let _i = round.sender;
	let _n = parties_including_offender.len() as u16;
	Pallet::<T>::ensure_signed_by_offender(round, data.offender)?;

	// TODO: add slashing logic
	// Slash the offender!
	Ok(())
}
