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

use super::{Asset, AssetSecurityCommitment, Constraints};
use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
	/// Total cost in USD with decimal precision (scaled by 10^6, i.e., 1.23 USD = 1_230_000)
	pub total_cost_rate: u64,
	/// Timestamp when quote was generated
	pub timestamp: u64,
	/// Expiry timestamp
	pub expiry: u64,
	/// Resource pricing details
	pub resources: BoundedVec<ResourcePricing<C>, C::MaxOperatorsPerService>,
	/// Security commitments for assets
	pub security_commitments:
		Option<BoundedVec<AssetSecurityCommitment<u32>, C::MaxOperatorsPerService>>,
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
	pub kind: BoundedVec<u8, C::MaxOperatorsPerService>,
	/// Quantity of the resource
	pub count: u64,
	/// Price per unit in USD with decimal precision (scaled by 10^6, i.e., 0.00005 USD = 50)
	pub price_per_unit_rate: u64,
}

/// Creates a deterministic hash of the pricing quote that can be reproduced in any language.
/// Uses a custom serialization followed by SHA-256 hashing to match the protobuf serialization.
pub fn hash_pricing_quote<C: Constraints>(
	pricing_quote: &PricingQuote<C>,
) -> sp_std::prelude::Vec<u8> {
	// Custom serialization to mimic protobuf
	let mut serialized = sp_std::prelude::Vec::new();

	// Encode blueprint_id (field 1, varint)
	encode_varint(&mut serialized, 1, pricing_quote.blueprint_id);

	// Encode ttl_blocks (field 2, varint)
	encode_varint(&mut serialized, 2, pricing_quote.ttl_blocks);

	// Encode total_cost_rate (field 3, fixed64)
	// Convert from scaled integer to double for protobuf compatibility
	let total_cost_rate_f64 = (pricing_quote.total_cost_rate as f64) / 1_000_000.0;
	encode_double(&mut serialized, 3, total_cost_rate_f64);

	// Encode timestamp (field 4, varint)
	encode_varint(&mut serialized, 4, pricing_quote.timestamp);

	// Encode expiry (field 5, varint)
	encode_varint(&mut serialized, 5, pricing_quote.expiry);

	// Encode resources (field 6, repeated message)
	for resource in &pricing_quote.resources {
		encode_message_start(&mut serialized, 6);
		let mut resource_data = sp_std::prelude::Vec::new();

		// Encode kind (field 1, string)
		encode_bytes(&mut resource_data, 1, &resource.kind);

		// Encode count (field 2, varint)
		encode_varint(&mut resource_data, 2, resource.count);

		// Encode price_per_unit_rate (field 3, fixed64)
		// Convert from scaled integer to double for protobuf compatibility
		let price_per_unit_rate_f64 = (resource.price_per_unit_rate as f64) / 1_000_000.0;
		encode_double(&mut resource_data, 3, price_per_unit_rate_f64);

		// Add length-prefixed resource data
		encode_length_delimited(&mut serialized, &resource_data);
	}

	// Encode security_commitments (field 7, optional message)
	if let Some(security_commitments) = &pricing_quote.security_commitments {
		for commitment in security_commitments {
			encode_message_start(&mut serialized, 7);
			let mut commitment_data = sp_std::prelude::Vec::new();

			// Encode asset (field 1, message)
			encode_message_start(&mut commitment_data, 1);
			let mut asset_data = sp_std::prelude::Vec::new();

			// Encode asset type (field 1, varint)
			match &commitment.asset {
				Asset::Custom(asset_id) => {
					encode_varint(&mut asset_data, 1, 0); // 0 for Custom
					encode_varint(&mut asset_data, 2, *asset_id as u64); // Asset ID
				},
				Asset::Erc20(address) => {
					encode_varint(&mut asset_data, 1, 1); // 1 for Erc20
					// Encode the H160 address as bytes
					encode_bytes(&mut asset_data, 3, address.as_fixed_bytes());
				},
			}

			// Add length-prefixed asset data
			encode_length_delimited(&mut commitment_data, &asset_data);

			// Encode exposure_percent (field 2, varint)
			encode_varint(
				&mut commitment_data,
				2,
				commitment.exposure_percent.deconstruct() as u64,
			);

			// Add length-prefixed commitment data
			encode_length_delimited(&mut serialized, &commitment_data);
		}
	}

	// Hash the serialized bytes using SHA-256
	let mut hasher = Sha256::new();
	hasher.update(&serialized);
	let result = hasher.finalize();

	result.to_vec()
}

/// Encode a varint (variable-length integer) with field number
fn encode_varint(buf: &mut sp_std::prelude::Vec<u8>, field_num: u32, value: u64) {
	// Field number and wire type (0 for varint)
	let tag = (field_num as u64) << 3;
	encode_raw_varint(buf, tag);
	encode_raw_varint(buf, value);
}

/// Encode a raw varint value
fn encode_raw_varint(buf: &mut sp_std::prelude::Vec<u8>, mut value: u64) {
	loop {
		let mut byte = (value & 0x7F) as u8;
		value >>= 7;
		if value != 0 {
			byte |= 0x80;
		}
		buf.push(byte);
		if value == 0 {
			break;
		}
	}
}

/// Encode a double (64-bit floating point) with field number
fn encode_double(buf: &mut sp_std::prelude::Vec<u8>, field_num: u32, value: f64) {
	// Field number and wire type (1 for fixed64)
	let tag = ((field_num as u64) << 3) | 1;
	encode_raw_varint(buf, tag);

	// Convert f64 to u64 bits and encode as little-endian
	let bits = value.to_bits();
	for i in 0..8 {
		buf.push(((bits >> (i * 8)) & 0xFF) as u8);
	}
}

/// Encode bytes/string with field number
fn encode_bytes(buf: &mut sp_std::prelude::Vec<u8>, field_num: u32, value: &[u8]) {
	// Field number and wire type (2 for length-delimited)
	let tag = ((field_num as u64) << 3) | 2;
	encode_raw_varint(buf, tag);

	// Length of the bytes
	encode_raw_varint(buf, value.len() as u64);

	// The bytes themselves
	buf.extend_from_slice(value);
}

/// Start encoding a message field
fn encode_message_start(buf: &mut sp_std::prelude::Vec<u8>, field_num: u32) {
	// Field number and wire type (2 for length-delimited)
	let tag = ((field_num as u64) << 3) | 2;
	encode_raw_varint(buf, tag);
}

/// Encode length-delimited data
fn encode_length_delimited(buf: &mut sp_std::prelude::Vec<u8>, data: &[u8]) {
	// Length of the data
	encode_raw_varint(buf, data.len() as u64);

	// The data itself
	buf.extend_from_slice(data);
}
