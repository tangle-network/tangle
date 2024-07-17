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

use crate::{core::ics24_host::error::ValidationError, prelude::*};
use ibc_proto::google::protobuf::Any;

pub trait Msg: Clone {
	type ValidationError;
	type Raw: From<Self> + prost::Message;

	// TODO: Clarify what is this function supposed to do & its connection to ICS26 routing mod.
	fn route(&self) -> String;

	/// Unique type identifier for this message, to support encoding to/from `prost_types::Any`.
	fn type_url(&self) -> String;

	#[allow(clippy::wrong_self_convention)]
	fn to_any(self) -> Any {
		Any { type_url: self.type_url(), value: self.get_sign_bytes() }
	}

	fn get_sign_bytes(self) -> Vec<u8> {
		let mut buf = Vec::new();
		let raw_msg: Self::Raw = self.into();
		match prost::Message::encode(&raw_msg, &mut buf) {
			Ok(()) => buf,
			// Severe error that cannot be recovered.
			Err(e) => panic!(
				"Cannot encode the proto message {:?} into a buffer due to underlying error: {}",
				raw_msg, e
			),
		}
	}

	fn validate_basic(&self) -> Result<(), ValidationError> {
		Ok(())
	}
}
