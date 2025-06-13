#![deny(stable_features, non_shorthand_field_patterns, renamed_and_removed_lints, unsafe_code)]

pub use parity_scale_codec;
pub use scale_info;
#[cfg(any(feature = "std", feature = "web"))]
pub use subxt;
#[cfg(any(feature = "std", feature = "web"))]
pub use subxt::ext::subxt_core;
#[cfg(not(any(feature = "std", feature = "web")))]
pub use subxt_core;
pub use subxt_signer;

// #[cfg_attr(rustfmt, rustfmt::skip)]
// pub mod tangle_mainnet_runtime;
#[rustfmt::skip]
pub mod tangle_testnet_runtime;

mod field_ext;
pub use field_ext::*;
