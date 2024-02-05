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
	/// An error related to a Malformed Signature.
	MalformedSignature,
	/// An error related to an invalid signature verification
	InvalidSignature,
	/// An error related to a VerifyingKey.
	MalformedVerifyingKey,
}

impl Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Error::Field(e) => write!(f, "Field error: {:?}", e),
			Error::Group(e) => write!(f, "Group error: {:?}", e),
			Error::MalformedSignature => write!(f, "Malformed Signature error"),
			Error::InvalidSignature => write!(f, "Invalid Signature error"),
			Error::MalformedVerifyingKey => write!(f, "Malformed VerifyingKey"),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		Debug::fmt(self, f)
	}
}
