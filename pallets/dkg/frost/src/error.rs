use core::fmt::{Debug, Display};

/// An error related to a scalar Field.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FieldError {
	/// The encoding of a group scalar was malformed.
	MalformedScalar,
	/// This scalar MUST NOT be zero.
	InvalidZeroScalar,
}

/// An error related to a Group (usually an elliptic curve or constructed from one) or one of its
/// Elements.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GroupError {
	/// The encoding of a group element was malformed.
	MalformedElement,
	/// This element MUST NOT be the identity.
	InvalidIdentityElement,
	/// This element MUST have (large) prime order.
	InvalidNonPrimeOrderElement,
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Error {
	/// An error related to a scalar Field.
	Field(FieldError),
	/// An error related to a Group (usually an elliptic curve or constructed from one) or one of
	/// its Elements.
	Group(GroupError),
	/// Serialization error
	SerializationError,
	/// Deserialization error
	DeserializationError,
	IdentifierDerivationNotSupported,
	/// An error related to a Malformed Signature.
	MalformedSignature,
	/// An error related to an invalid signature verification
	InvalidSignature,
	/// An error related to a VerifyingKey.
	MalformedVerifyingKey,
	/// An error related to a SigningKey
	MalformedSigningKey,
	/// Missing commitment
	MissingCommitment,
	/// Invalid signature share
	InvalidSignatureShare,
	/// Duplicated identifier
	DuplicatedIdentifier,
	/// Unknown identifier
	UnknownIdentifier,
	/// Incorrect number of identifiers
	IncorrectNumberOfIdentifiers,
	/// Identity commitment error
	IdentityCommitment,
}

impl Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Error::Field(e) => write!(f, "Field error: {:?}", e),
			Error::Group(e) => write!(f, "Group error: {:?}", e),
			Error::MalformedSignature => write!(f, "Malformed Signature error"),
			Error::InvalidSignature => write!(f, "Invalid Signature error"),
			Error::MalformedVerifyingKey => write!(f, "Malformed VerifyingKey"),
			Error::MalformedSigningKey => write!(f, "Malformed SigningKey"),
			Error::SerializationError => write!(f, "Serialization error"),
			Error::DeserializationError => write!(f, "Deserialization error"),
			Error::IdentifierDerivationNotSupported =>
				write!(f, "Identifier derivation not supported"),
			Error::MissingCommitment => write!(f, "Missing commitment"),
			Error::InvalidSignatureShare => write!(f, "Invalid signature share"),
			Error::DuplicatedIdentifier => write!(f, "Duplicated identifier"),
			Error::UnknownIdentifier => write!(f, "Unknown identifier"),
			Error::IncorrectNumberOfIdentifiers => write!(f, "Incorrect number of identifiers"),
			Error::IdentityCommitment => write!(f, "Identity commitment error"),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		Debug::fmt(self, f)
	}
}

impl From<FieldError> for Error {
	fn from(e: FieldError) -> Self {
		Error::Field(e)
	}
}

impl From<GroupError> for Error {
	fn from(e: GroupError) -> Self {
		Error::Group(e)
	}
}
