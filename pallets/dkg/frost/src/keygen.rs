pub mod round1 {
	use crate::{
		keys::VerifiableSecretSharingCommitment, signature::Signature, traits::Ciphersuite, Header,
	};

	/// The package that must be broadcast by each participant to all other participants
	/// between the first and second parts of the DKG protocol (round 1).
	#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
	#[serde(bound = "C: Ciphersuite")]
	#[serde(deny_unknown_fields)]
	pub struct Package<C: Ciphersuite> {
		/// Serialization header
		pub header: Header<C>,
		/// The public commitment from the participant (C_i)
		pub commitment: VerifiableSecretSharingCommitment<C>,
		/// The proof of knowledge of the temporary secret (σ_i = (R_i, μ_i))
		pub proof_of_knowledge: Signature<C>,
	}

	impl<C> Package<C>
	where
		C: Ciphersuite,
	{
		/// Create a new [`Package`] instance.
		pub fn new(
			commitment: VerifiableSecretSharingCommitment<C>,
			proof_of_knowledge: Signature<C>,
		) -> Self {
			Self { header: Header::default(), commitment, proof_of_knowledge }
		}
	}
}

pub mod round2 {
	use crate::{keys::SigningShare, traits::Ciphersuite, Header};

	/// A package that must be sent by each participant to some other participants
	/// in Round 2 of the DKG protocol. Note that there is one specific package
	/// for each specific recipient, in contrast to Round 1.
	///
	/// # Security
	///
	/// The package must be sent on an *confidential* and *authenticated* channel.
	#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
	#[serde(bound = "C: Ciphersuite")]
	#[serde(deny_unknown_fields)]
	pub struct Package<C: Ciphersuite> {
		/// Serialization header
		pub header: Header<C>,
		/// The secret share being sent.
		pub signing_share: SigningShare<C>,
	}

	impl<C> Package<C>
	where
		C: Ciphersuite,
	{
		/// Create a new [`Package`] instance.
		pub fn new(signing_share: SigningShare<C>) -> Self {
			Self { header: Header::default(), signing_share }
		}
	}
}
