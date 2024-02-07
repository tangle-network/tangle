// -*- mode: rust; -*-
//
// This file was part of reddsa.
// With updates made to support FROST.
// Copyright (c) 2019-2021 Zcash Foundation
// See LICENSE for licensing information.
//
// Authors:
// - Deirdre Connolly <deirdre@zfnd.org>
// - Henry de Valence <hdevalence@hdevalence.ca>

use blake2::{
	digest::{Mac, Update},
	Blake2bMac512,
};

/// Provides H^star, the hash-to-scalar function used by RedDSA.
pub struct HStar {
	pub(crate) state: Blake2bMac512,
}

impl Default for HStar {
	fn default() -> Self {
		let persona = b"Zcash_RedJubjubH";
		let state = Blake2bMac512::new_with_salt_and_personal(&[], &[], persona).unwrap();
		Self { state }
	}
}

impl HStar {
	// Only used by FROST code
	#[allow(unused)]
	pub(crate) fn new(personalization_string: &[u8]) -> Self {
		let state =
			Blake2bMac512::new_with_salt_and_personal(&[], &[], personalization_string).unwrap();
		Self { state }
	}

	/// Add `data` to the hash, and return `Self` for chaining.
	pub fn update(&mut self, data: impl AsRef<[u8]>) -> &mut Self {
		Update::update(&mut self.state, data.as_ref());
		self
	}

	/// Consume `self` to compute the hash output.
	pub fn finalize(&self) -> jubjub::Scalar {
		jubjub::Scalar::from_bytes_wide(
			self.state
				.clone()
				.finalize()
				.into_bytes()
				.to_vec()
				.as_slice()
				.try_into()
				.unwrap_or(&[0u8; 64]),
		)
	}
}
