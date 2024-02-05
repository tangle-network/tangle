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

use blake2b_simd::{Params, State};

/// Provides H^star, the hash-to-scalar function used by RedDSA.
pub struct HStar {
	pub(crate) state: State,
}

impl Default for HStar {
	fn default() -> Self {
		let state = Params::new().hash_length(64).personal(b"Zcash_RedJubjubH").to_state();
		Self { state }
	}
}

impl HStar {
	// Only used by FROST code
	#[allow(unused)]
	pub(crate) fn new(personalization_string: &[u8]) -> Self {
		let state = Params::new().hash_length(64).personal(personalization_string).to_state();
		Self { state }
	}

	/// Add `data` to the hash, and return `Self` for chaining.
	pub fn update(&mut self, data: impl AsRef<[u8]>) -> &mut Self {
		self.state.update(data.as_ref());
		self
	}

	/// Consume `self` to compute the hash output.
	pub fn finalize(&self) -> jubjub::Scalar {
		jubjub::Scalar::from_bytes_wide(self.state.finalize().as_array())
	}
}
