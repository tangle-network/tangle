// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! # FROST no_std primitives
//!
//! A no_std copy of FROST primitives from https://github.com/LIT-Protocol/frost.
//! Needed in order to properly verify Schnorr threshold signatures based on FROST
//! protocol from this library, since the original library was not no_std compatible.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
extern crate alloc;

pub mod challenge;
pub mod const_crc32;
pub mod error;
pub mod identifier;
pub mod keys;
pub mod serialization;
pub mod signature;
pub mod signing_key;
pub mod traits;
pub mod util;
pub mod verifying_key;

use core::marker::PhantomData;

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use traits::{Ciphersuite, Field, Group, Scalar};

/// Generates a random nonzero scalar.
///
/// It assumes that the Scalar Eq/PartialEq implementation is constant-time.
#[cfg(feature = "std")]
pub fn random_nonzero<C: Ciphersuite, R: RngCore + CryptoRng>(rng: &mut R) -> Scalar<C> {
	loop {
		let scalar = <<C::Group as Group>::Field>::random(rng);

		if scalar != <<C::Group as Group>::Field>::zero() {
			return scalar
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header<C: Ciphersuite> {
	/// Format version
	pub version: u8,
	/// Ciphersuite ID
	pub ciphersuite: (),
	#[serde(skip)]
	pub phantom: PhantomData<C>,
}
