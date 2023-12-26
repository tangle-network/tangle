// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

/// Key type for ROLE keys
pub const ROLE_KEY_TYPE: sp_application_crypto::KeyTypeId =
	sp_application_crypto::KeyTypeId(*b"role");

pub mod crypto {
	use sp_application_crypto::{app_crypto, ecdsa};
	app_crypto!(ecdsa, crate::ROLE_KEY_TYPE);

	/// Identity of a DKG authority using ECDSA as its crypto.
	pub type AuthorityId = Public;

	/// Signature for a DKG authority using ECDSA as its crypto.
	pub type AuthoritySignature = Signature;
}
