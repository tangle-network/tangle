use super::*;
use frame_support::traits::Contains;

// Custom call filter for runtime calls
pub struct TangleFilter;
impl Contains<RuntimeCall> for TangleFilter {
	fn contains(call: &RuntimeCall) -> bool {
		if matches!(call, RuntimeCall::Timestamp(_) | RuntimeCall::System(_)) {
			// always allow core call
			// pallet-timestamp and parachainSystem could not be filtered because they are used in
			// communication between releychain and parachain.
			return true
		}

		if pallet_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call) {
			// no paused call
			return false
		}

		#[allow(clippy::match_like_matches_macro)]
		// keep CallFilter with explicit true/false for documentation
		match call {
			// Explicitly ALLOWED calls
			// Sudo also cannot be filtered because it is used in runtime upgrade.
			| RuntimeCall::Sudo(_) |
			RuntimeCall::Balances(_) |
			RuntimeCall::Preimage(_) |
			RuntimeCall::TransactionPause(_) |
			RuntimeCall::Utility(_) => true,

			// DISALLOWED list
			| RuntimeCall::Democracy(_) | RuntimeCall::Claims(_) | RuntimeCall::Council(_) => false,

			// everything else is allowed
			| _ => true,
		}
	}
}
