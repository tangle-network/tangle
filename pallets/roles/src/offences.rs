use super::*;
use sp_runtime::{Perbill, Saturating};
use sp_staking::{
	offence::{Kind, Offence},
	SessionIndex,
};
use tangle_primitives::jobs::ValidatorOffenceType;

#[derive(PartialEq, Clone, Debug, Encode, Decode)]
pub struct ValidatorOffence<Offender> {
	/// The current session index in which we report the validators that submitted an invalid
	/// ethereum log.
	pub session_index: SessionIndex,
	/// The size of the validator set in current session/era.
	pub validator_set_count: u32,
	/// Authorities that validated the invalid log.
	pub offenders: Vec<Offender>,
	/// The different types of the offence.
	pub offence_type: ValidatorOffenceType,
}

impl<Offender: Clone> Offence<Offender> for ValidatorOffence<Offender> {
	const ID: Kind = *b"validator:offenc";
	type TimeSlot = SessionIndex;

	fn offenders(&self) -> Vec<Offender> {
		self.offenders.clone()
	}

	fn session_index(&self) -> SessionIndex {
		self.session_index
	}

	fn validator_set_count(&self) -> u32 {
		self.validator_set_count
	}

	fn time_slot(&self) -> Self::TimeSlot {
		self.session_index
	}

	fn slash_fraction(&self, offenders: u32) -> Perbill {
		// the formula is min((3 * (k - (n / 10 + 1))) / n, 1) * 0.07
		// basically, 10% can be offline with no slash, but after that, it linearly climbs up to 7%
		// when 13/30 are offline (around 5% when 1/3 are offline).
		if let Some(threshold) = offenders.checked_sub(self.validator_set_count / 10 + 1) {
			let x = Perbill::from_rational(3 * threshold, self.validator_set_count);
			x.saturating_mul(Perbill::from_percent(7))
		} else {
			Perbill::default()
		}
	}
}
