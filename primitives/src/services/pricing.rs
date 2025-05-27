// Copyright (C) Moondance Labs Ltd.
// This file is part of Tangle.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use super::{AssetSecurityCommitment, BoundedString, Constraints};
use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::keccak_256;

/// The detailed pricing quote information for service pricing
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct PricingQuote<C: Constraints> {
	/// The blueprint ID
	pub blueprint_id: u64,
	/// Time-to-live for service in blocks
	pub ttl_blocks: u64,
	/// Total pricing rate per block
	pub total_cost_rate: u128,
	/// Timestamp when quote was generated
	pub timestamp: u64,
	/// Expiry timestamp
	pub expiry: u64,
	/// Resource pricing details
	pub resources: BoundedVec<ResourcePricing<C>, C::MaxOperatorsPerService>,
	/// Security commitments for assets
	pub security_commitments: BoundedVec<AssetSecurityCommitment<u128>, C::MaxOperatorsPerService>,
}

/// Pricing for a specific resource type
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct ResourcePricing<C: Constraints> {
	/// Resource kind (CPU, Memory, GPU, etc.)
	pub kind: BoundedString<C::MaxResourceNameLength>,
	/// Quantity of the resource
	pub count: u64,
	/// Price per unit per block
	pub price_per_unit_rate: u128,
}

/// Creates a deterministic hash of the pricing quote that can more easily be reproduced in other
/// languages.
pub fn hash_pricing_quote<C: Constraints>(pricing_quote: &PricingQuote<C>) -> [u8; 32] {
	let encoded = pricing_quote.encode();
	keccak_256(&encoded)
}
