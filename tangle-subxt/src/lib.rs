#![deny(
	trivial_casts,
	trivial_numeric_casts,
	stable_features,
	non_shorthand_field_patterns,
	renamed_and_removed_lints,
	unsafe_code,
	clippy::exhaustive_enums
)]
#![allow(clippy::all, clippy::exhaustive_enums)]
// pub mod tangle_mainnet_runtime;
// pub mod tangle_testnet_runtime;
pub use parity_scale_codec;
pub use scale_info;
pub use subxt;
