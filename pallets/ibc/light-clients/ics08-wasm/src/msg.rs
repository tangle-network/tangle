// Copyright (C) 2022 ComposableFi.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Bytes;
use alloc::string::ToString;
use core::str::FromStr;
use ibc::{
	protobuf::Protobuf,
	signer::{Signer, SignerError},
};
use ibc_proto::{
	google::protobuf::Any, ibc::lightclients::wasm::v1::MsgPushNewWasmCode as RawMsgPushNewWasmCode,
};
#[cfg(feature = "cosmwasm")]
use serde::{Deserializer, Serializer};

pub const WASM_PUSH_WASM_CODE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.MsgPushNewWasmCode";

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MsgPushNewWasmCode {
	pub code: Bytes,
	pub signer: Signer,
}

impl Protobuf<RawMsgPushNewWasmCode> for MsgPushNewWasmCode {}

impl From<MsgPushNewWasmCode> for RawMsgPushNewWasmCode {
	fn from(value: MsgPushNewWasmCode) -> Self {
		Self { code: value.code, signer: value.signer.to_string() }
	}
}

impl TryFrom<RawMsgPushNewWasmCode> for MsgPushNewWasmCode {
	type Error = SignerError;

	fn try_from(value: RawMsgPushNewWasmCode) -> Result<Self, Self::Error> {
		Ok(Self { code: value.code, signer: Signer::from_str(&value.signer)? })
	}
}

impl From<MsgPushNewWasmCode> for Any {
	fn from(value: MsgPushNewWasmCode) -> Self {
		Any {
			type_url: WASM_PUSH_WASM_CODE_TYPE_URL.to_string(),
			value: value.encode_vec().expect("MsgPushNewWasmCode encoding should always succeed"),
		}
	}
}

pub struct Base64;

#[cfg(feature = "cosmwasm")]
impl Base64 {
	pub fn serialize<S: Serializer>(v: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
		ibc_proto::base64::serialize(v, serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
		ibc_proto::base64::deserialize(deserializer)
	}
}
