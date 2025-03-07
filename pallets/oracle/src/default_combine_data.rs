use crate::{Config, MomentOf, TimestampedValueOf};
use frame_support::traits::{Get, Time};
use sp_runtime::traits::Saturating;
use sp_std::{marker, prelude::*};
use tangle_primitives::traits::CombineData;

/// Sort by value and returns median timestamped value.
/// Returns prev_value if not enough valid values.
pub struct DefaultCombineData<T, MinimumCount, ExpiresIn, I = ()>(
	marker::PhantomData<(T, I, MinimumCount, ExpiresIn)>,
);

impl<T, I, MinimumCount, ExpiresIn>
	CombineData<<T as Config<I>>::OracleKey, TimestampedValueOf<T, I>>
	for DefaultCombineData<T, MinimumCount, ExpiresIn, I>
where
	T: Config<I>,
	I: 'static,
	MinimumCount: Get<u32>,
	ExpiresIn: Get<MomentOf<T, I>>,
{
	fn combine_data(
		_key: &<T as Config<I>>::OracleKey,
		mut values: Vec<TimestampedValueOf<T, I>>,
		prev_value: Option<TimestampedValueOf<T, I>>,
	) -> Option<TimestampedValueOf<T, I>> {
		let expires_in = ExpiresIn::get();
		let now = T::Time::now();

		values.retain(|x| x.timestamp.saturating_add(expires_in) > now);

		let count = values.len() as u32;
		let minimum_count = MinimumCount::get();
		if count < minimum_count || count == 0 {
			return prev_value;
		}

		let mid_index = count / 2;
		// Won't panic as `values` ensured not empty.
		let (_, value, _) =
			values.select_nth_unstable_by(mid_index as usize, |a, b| a.value.cmp(&b.value));
		Some(value.clone())
	}
}
