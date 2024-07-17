use super::*;
use ibc::core::ics26_routing::error::{Error as RoutingError, ErrorDetail};

#[derive(PartialEq, Eq, Clone, Debug, scale_info::TypeInfo, Encode, Decode)]
pub enum IbcError {
	/// ICS02 client error
	Ics02Client { message: Vec<u8> },
	/// ICS03 connection error
	Ics03Connection { message: Vec<u8> },
	/// ICS04 channel error
	Ics04Channel { message: Vec<u8> },
	/// ICS20 fungible token transfer error
	Ics20FungibleTokenTransfer { message: Vec<u8> },
	/// Unknown message type URL
	UnknownMessageTypeUrl { message: Vec<u8> },
	/// The message is malformed and cannot be decoded
	MalformedMessageBytes { message: Vec<u8> },
}

impl From<RoutingError> for IbcError {
	fn from(err: RoutingError) -> Self {
		match err.0 {
			ErrorDetail::Ics03Connection(e) =>
				IbcError::Ics03Connection { message: format!("{e:?}").as_bytes().to_vec() },
			ErrorDetail::Ics02Client(e) =>
				IbcError::Ics02Client { message: format!("{e:?}").as_bytes().to_vec() },
			ErrorDetail::Ics04Channel(e) =>
				IbcError::Ics04Channel { message: format!("{e:?}").as_bytes().to_vec() },
			ErrorDetail::Ics20FungibleTokenTransfer(e) => IbcError::Ics20FungibleTokenTransfer {
				message: format!("{e:?}").as_bytes().to_vec(),
			},
			ErrorDetail::UnknownMessageTypeUrl(e) =>
				IbcError::UnknownMessageTypeUrl { message: format!("{e:?}").as_bytes().to_vec() },
			ErrorDetail::MalformedMessageBytes(e) =>
				IbcError::MalformedMessageBytes { message: format!("{e:?}").as_bytes().to_vec() },
		}
	}
}

impl<T: Config> From<Vec<RoutingError>> for Event<T> {
	fn from(errors: Vec<RoutingError>) -> Self {
		let errors: Vec<Result<_, IbcError>> =
			errors.into_iter().map(|err| Err(err.into())).collect();
		Event::<T>::Events { events: errors }
	}
}
