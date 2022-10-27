use crate::{currency::DOLLAR, Balance};
use pallet_parachain_staking::{BalanceOf, InflationInfo};

pub const NORMAL_COLLATOR_MINIMUM_STAKE: Balance = 400 * DOLLAR;
pub const EARLY_COLLATOR_MINIMUM_STAKE: Balance = 400 * DOLLAR;
pub const MIN_BOND_TO_BE_CONSIDERED_COLLATOR: Balance = EARLY_COLLATOR_MINIMUM_STAKE;

pub fn inflation_config<T: frame_system::Config + pallet_parachain_staking::Config>(
) -> InflationInfo<BalanceOf<T>> {
	use pallet_parachain_staking::inflation::Range;
	use sp_arithmetic::Perbill;
	use sp_runtime::{traits::UniqueSaturatedInto, PerThing};

	fn to_round_inflation(annual: Range<Perbill>) -> Range<Perbill> {
		use pallet_parachain_staking::inflation::{
			perbill_annual_to_perbill_round, BLOCKS_PER_YEAR,
		};
		perbill_annual_to_perbill_round(
			annual,
			// rounds per year
			BLOCKS_PER_YEAR / crate::SESSION_PERIOD_BLOCKS,
		)
	}
	let annual = Range {
		min: Perbill::from_rational_with_rounding(5u32, 200u32, sp_arithmetic::Rounding::Down)
			.expect("constant denom is not 0. qed"), // = 2.5%
		ideal: Perbill::from_percent(3),
		max: Perbill::from_percent(5),
	};
	InflationInfo::<BalanceOf<T>> {
		// staking expectations **per round**
		expect: Range {
			min: (170_000 * DOLLAR).unique_saturated_into(),
			ideal: (205_479 * DOLLAR).unique_saturated_into(), /* annual inflation / number of
			                                                    * rounds */
			max: (210_000 * DOLLAR).unique_saturated_into(),
		},
		// annual inflation
		annual,
		round: to_round_inflation(annual),
	}
}
