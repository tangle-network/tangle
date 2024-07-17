// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::prelude::*;

use super::error::ValidationError as Error;

/// Path separator (ie. forward slash '/')
const PATH_SEPARATOR: char = '/';
const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Default validator function for identifiers.
///
/// A valid identifier only contain lowercase alphabetic characters, and be of a given min and max
/// length.
pub fn validate_identifier(id: &str, min: usize, max: usize) -> Result<(), Error> {
	assert!(max >= min);

	// Check identifier is not empty
	if id.is_empty() {
		return Err(Error::empty())
	}

	// Check identifier does not contain path separators
	if id.contains(PATH_SEPARATOR) {
		return Err(Error::contain_separator(id.to_string()))
	}

	// Check identifier length is between given min/max
	if id.len() < min || id.len() > max {
		return Err(Error::invalid_length(id.to_string(), id.len(), min, max))
	}

	// Check that the identifier comprises only valid characters:
	// - Alphanumeric
	// - `.`, `_`, `+`, `-`, `#`
	// - `[`, `]`, `<`, `>`
	if !id.chars().all(|c| c.is_alphanumeric() || VALID_SPECIAL_CHARS.contains(c)) {
		return Err(Error::invalid_character(id.to_string()))
	}

	// All good!
	Ok(())
}

/// Default validator function for Client identifiers.
///
/// A valid identifier must be between 9-64 characters and only contain lowercase
/// alphabetic characters,
pub fn validate_client_identifier(id: &str) -> Result<(), Error> {
	validate_identifier(id, 9, 64)
}

/// Default validator function for Connection identifiers.
///
/// A valid Identifier must be between 10-64 characters and only contain lowercase
/// alphabetic characters,
pub fn validate_connection_identifier(id: &str) -> Result<(), Error> {
	validate_identifier(id, 10, 64)
}

/// Default validator function for Port identifiers.
///
/// A valid Identifier must be between 2-128 characters and only contain lowercase
/// alphabetic characters,
pub fn validate_port_identifier(id: &str) -> Result<(), Error> {
	validate_identifier(id, 2, 128)
}

#[cfg(test)]
mod tests {
	use crate::core::ics24_host::validate::{
		validate_client_identifier, validate_connection_identifier, validate_identifier,
		validate_port_identifier,
	};
	use test_log::test;

	#[test]
	fn parse_invalid_port_id_min() {
		// invalid min port id
		let id = validate_port_identifier("p");
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_port_id_max() {
		// invalid max port id (test string length is 130 chars)
		let id = validate_port_identifier(
            "9anxkcme6je544d5lnj46zqiiiygfqzf8w4bjecbnyj4lj6s7zlpst67yln64tixp9anxkcme6je544d5lnj46zqiiiygfqzf8w4bjecbnyj4lj6s7zlpst67yln64tixp",
        );
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_connection_id_min() {
		// invalid min connection id
		let id = validate_connection_identifier("connect01");
		assert!(id.is_err())
	}

	#[test]
	fn parse_connection_id_max() {
		// invalid max connection id (test string length is 65)
		let id = validate_connection_identifier(
			"ihhankr30iy4nna65hjl2wjod7182io1t2s7u3ip3wqtbbn1sl0rgcntqc540r36r",
		);
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_client_id_min() {
		// invalid min client id
		let id = validate_client_identifier("client");
		assert!(id.is_err())
	}

	#[test]
	fn parse_client_id_max() {
		// invalid max client id (test string length is 65)
		let id = validate_client_identifier(
			"f0isrs5enif9e4td3r2jcbxoevhz6u1fthn4aforq7ams52jn5m48eiesfht9ckpn",
		);
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_id_chars() {
		// invalid id chars
		let id = validate_identifier("channel@01", 1, 10);
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_id_empty() {
		// invalid id empty
		let id = validate_identifier("", 1, 10);
		assert!(id.is_err())
	}

	#[test]
	fn parse_invalid_id_path_separator() {
		// invalid id with path separator
		let id = validate_identifier("id/1", 1, 10);
		assert!(id.is_err())
	}
}
