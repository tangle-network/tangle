//! FROST Error types

use thiserror_nostd_notrait::Error;

use crate::{Ciphersuite, Identifier};

#[derive(Error, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ParticipantError<C: Ciphersuite>(Identifier<C>);

/// An error related to FROST.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error<C: Ciphersuite> {
	/// min_signers is invalid
	#[error("min_signers must be at least 2 and not larger than max_signers")]
	InvalidMinSigners,
	/// max_signers is invalid
	#[error("max_signers must be at least 2")]
	InvalidMaxSigners,
	/// max_signers is invalid
	#[error("coefficients must have min_signers-1 elements")]
	InvalidCoefficients,
	/// This identifier is unserializable.
	#[error("Malformed identifier is unserializable.")]
	MalformedIdentifier,
	/// This identifier is duplicated.
	#[error("Duplicated identifier.")]
	DuplicatedIdentifier,
	/// This identifier does not belong to a participant in the signing process.
	#[error("Unknown identifier.")]
	UnknownIdentifier,
	/// Incorrect number of identifiers.
	#[error("Incorrect number of identifiers.")]
	IncorrectNumberOfIdentifiers,
	/// The encoding of a signing key was malformed.
	#[error("Malformed signing key encoding.")]
	MalformedSigningKey,
	/// The encoding of a verifying key was malformed.
	#[error("Malformed verifying key encoding.")]
	MalformedVerifyingKey,
	/// The encoding of a signature was malformed.
	#[error("Malformed signature encoding.")]
	MalformedSignature,
	/// Signature verification failed.
	#[error("Invalid signature.")]
	InvalidSignature,
	/// Duplicated shares provided
	#[error("Duplicated shares provided.")]
	DuplicatedShares,
	/// Incorrect number of shares.
	#[error("Incorrect number of shares.")]
	IncorrectNumberOfShares,
	/// Commitment equals the identity
	#[error("Commitment equals the identity.")]
	IdentityCommitment,
	/// The participant's commitment is missing from the Signing Package
	#[error("The Signing Package must contain the participant's Commitment.")]
	MissingCommitment,
	/// The participant's commitment is incorrect
	#[error("The participant's commitment is incorrect.")]
	IncorrectCommitment,
	/// Incorrect number of commitments.
	#[error("Incorrect number of commitments.")]
	IncorrectNumberOfCommitments,
	/// Signature share verification failed.
	#[error("Invalid signature share.")]
	InvalidSignatureShare {
		/// The identifier of the signer whose share validation failed.
		culprit: Identifier<C>,
	},
	/// Secret share verification failed.
	#[error("Invalid secret share.")]
	InvalidSecretShare {
		/// The identifier of the signer whose secret share validation failed,
		/// if possible to identify.
		culprit: Option<Identifier<C>>,
	},
	/// Round 1 package not found for Round 2 participant.
	#[error("Round 1 package not found for Round 2 participant.")]
	PackageNotFound,
	/// Incorrect number of packages.
	#[error("Incorrect number of packages.")]
	IncorrectNumberOfPackages,
	/// The incorrect package was specified.
	#[error("The incorrect package was specified.")]
	IncorrectPackage,
	/// The ciphersuite does not support DKG.
	#[error("The ciphersuite does not support DKG.")]
	DKGNotSupported,
	/// The proof of knowledge is not valid.
	#[error("The proof of knowledge is not valid.")]
	InvalidProofOfKnowledge {
		/// The identifier of the signer whose share validation failed.
		culprit: Identifier<C>,
	},
	/// Error in scalar Field.
	#[error("Error in scalar Field.")]
	FieldError(#[from] FieldError),
	/// Error in elliptic curve Group.
	#[error("Error in elliptic curve Group.")]
	GroupError(#[from] GroupError),
	/// Error in coefficient commitment deserialization.
	#[error("Invalid coefficient")]
	InvalidCoefficient,
	/// The ciphersuite does not support deriving identifiers from strings.
	#[error("The ciphersuite does not support deriving identifiers from strings.")]
	IdentifierDerivationNotSupported,
	/// Error serializing value.
	#[error("Error serializing value.")]
	SerializationError,
	/// Error deserializing value.
	#[error("Error deserializing value.")]
	DeserializationError,
}

impl<C> Error<C>
where
	C: Ciphersuite,
{
	/// Return the identifier of the participant that caused the error.
	/// Returns None if not applicable for the error.
	///
	/// This can be used to penalize a participant that does not follow the
	/// protocol correctly, e.g. removing them from further signings.
	pub fn culprit(&self) -> Option<Identifier<C>> {
		// Use an exhaustive match to make sure that if we add new enum items
		// then we will explicitly check if they should be added here.
		match self {
			Error::InvalidSignatureShare { culprit: identifier }
			| Error::InvalidProofOfKnowledge { culprit: identifier } => Some(*identifier),
			Error::InvalidSecretShare { culprit: identifier } => *identifier,
			Error::InvalidMinSigners
			| Error::InvalidMaxSigners
			| Error::InvalidCoefficients
			| Error::MalformedIdentifier
			| Error::MalformedSigningKey
			| Error::MalformedVerifyingKey
			| Error::MalformedSignature
			| Error::InvalidSignature
			| Error::DuplicatedShares
			| Error::IncorrectNumberOfShares
			| Error::IdentityCommitment
			| Error::MissingCommitment
			| Error::IncorrectCommitment
			| Error::PackageNotFound
			| Error::IncorrectNumberOfPackages
			| Error::IncorrectPackage
			| Error::DKGNotSupported
			| Error::FieldError(_)
			| Error::GroupError(_)
			| Error::DuplicatedIdentifier
			| Error::InvalidCoefficient
			| Error::UnknownIdentifier
			| Error::IncorrectNumberOfIdentifiers
			| Error::IncorrectNumberOfCommitments
			| Error::SerializationError
			| Error::DeserializationError
			| Error::IdentifierDerivationNotSupported => None,
		}
	}
}

/// An error related to a scalar Field.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum FieldError {
	/// The encoding of a group scalar was malformed.
	#[error("Malformed scalar encoding.")]
	MalformedScalar,
	/// This scalar MUST NOT be zero.
	#[error("Invalid for this scalar to be zero.")]
	InvalidZeroScalar,
}

/// An error related to a Group (usually an elliptic curve or constructed from one) or one of its Elements.
#[non_exhaustive]
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum GroupError {
	/// The encoding of a group element was malformed.
	#[error("Malformed group element encoding.")]
	MalformedElement,
	/// This element MUST NOT be the identity.
	#[error("Invalid for this element to be the identity.")]
	InvalidIdentityElement,
	/// This element MUST have (large) prime order.
	#[error("Invalid for this element to not have large prime order.")]
	InvalidNonPrimeOrderElement,
}
