use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use pallet_vesting::{MaxVestingSchedulesGet, Vesting, VestingInfo};
use sp_runtime::{
	traits::{Convert, EnsureDiv, Header, Zero},
	Percent, Saturating,
};
use sp_runtime::traits::StaticLookup;

use sp_std::vec::Vec;
use tangle_primitives::Balance;
use frame_system::RawOrigin;
use crate::RuntimeOrigin;
pub const BLOCK_TIME: u128 = 6;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;

pub const TEAM_ACCOUNT: [u8; 32] = [
	142, 28, 43, 221, 218, 185, 87, 61, 140, 176, 148, 219, 255, 186, 36, 162, 178, 194, 27, 126,
	113, 227, 245, 182, 4, 232, 96, 116, 131, 135, 36, 67,
];

/// Team accounts and their corresponding balances to unstake and send back to team.
pub const TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE: [([u8; 32], Balance); 2] = [
	// TI
	(
		[
			76, 227, 164, 218, 58, 124, 28, 230, 95, 126, 222, 255, 134, 77, 195, 221, 66, 232,
			244, 126, 236, 194, 114, 109, 153, 160, 168, 1, 36, 105, 130, 23,
		],
		114383561650000000000000,
	),
	// TB
	(
		[
			140, 62, 57, 135, 164, 41, 178, 70, 246, 25, 233, 31, 140, 164, 85, 53, 161, 191, 95,
			135, 14, 199, 197, 207, 246, 169, 16, 169, 148, 151, 139, 65,
		],
		121947842700000000000000,
	),
];

/// Team accounts and their corresponding balances to change vesting schedule and send back to team.
pub const TEAM_ACCOUNT_VESTING_UPDATE: ([u8; 32], Balance) = (
	[
		140, 62, 57, 135, 164, 41, 178, 70, 246, 25, 233, 31, 140, 164, 85, 53, 161, 191, 95, 135,
		14, 199, 197, 207, 246, 169, 16, 169, 148, 151, 139, 65,
	],
	135616438356164400000000, // Amount to change vesting schedule to
);

/// Migration to update team members' allocations who left project.
pub struct UpdateTeamMemberAllocation<T>(sp_std::marker::PhantomData<T>);

pub type BalanceOf<T> =
	<<T as pallet_staking::Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

pub type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as Header>::Number;

impl<T: pallet_staking::Config + pallet_vesting::Config + pallet_balances::Config + pallet_staking::Config> OnRuntimeUpgrade
	for UpdateTeamMemberAllocation<T>
	where T: frame_system::Config<RuntimeOrigin = RuntimeOrigin>
{
	fn on_runtime_upgrade() -> Weight {
		let mut reads = 0u64;
		let mut writes = 0u64;

		// Remove staking records from team accounts
		for (account, amount) in TEAM_ACCOUNTS.iter() {
			let account_id: T::AccountId =
				T::AccountId::decode(&mut account.as_ref()).expect("Invalid account ID");
			let controller =
				pallet_staking::Bonded::<T>::get(account_id).expect("Controller not found");
			let ledger = pallet_staking::Ledger::<T>::get(controller).expect("Ledger not found");
			let nominations =
				pallet_staking::Nominators::<T>::get(account_id).expect("Nominations not found");
			let _ =
				pallet_staking::Pallet::<T>::force_unstake(T::RuntimeOrigin::from(RawOrigin::Root), controller, 100)
					.unwrap();
		}

		// Send back balance from team member account with no vesting change
		let team_account_id: T::AccountId =
			T::AccountId::decode(&mut TEAM_ACCOUNT.as_ref()).expect("Invalid account ID");
		let source_account_id: T::AccountId = T::AccountId::decode(&mut TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE[1].0.as_ref())
			.expect("Invalid source account ID");
		pallet_balances::Pallet::<T>::force_transfer(
			T::RuntimeOrigin::from(RawOrigin::Root),
			T::Lookup::unlookup(source_account_id),
			T::Lookup::unlookup(team_account_id.clone()),
			TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE[1].1.into(),
		).expect("Failed to transfer balance");

		// Update vesting record and balance from team account with vesting change
		update_account_vesting(
			&TEAM_ACCOUNT_VESTING_UPDATE.0,
			TEAM_ACCOUNT_VESTING_UPDATE.1,
			&team_account_id,
			&mut reads,
			&mut writes,
		);

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		for (account, _) in TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE {
			let account_id = account.parse().expect("Invalid account ID");
			verify_updated::<T>(&account_id)?;
		}

		verify_updated_schedule::<T>(&TEAM_ACCOUNT_VESTING_UPDATE.0)?;

		Ok(())
	}
}

#[cfg(feature = "try-runtime")]
fn verify_updated<T: pallet_staking::Config + pallet_balances::Config>(
	account_id: &T::AccountId,
) -> Result<(), &'static str> {
	// Check the team member accounts are not bonded nor nominating
	assert!(pallet_staking::Bonded::<T>::get(account_id).is_none());
	assert!(pallet_staking::Ledger::<T>::get(account_id).is_none());
	assert!(pallet_staking::Nominators::<T>::get(account_id).is_none());
	// Check the team member accounts have only 1 lock
	assert!(pallet_balances::Locks::<T>::get(account_id).len() == 1);

	Ok(())
}

// Update investor vesting schedules
fn update_account_vesting<T: pallet_vesting::Config + pallet_balances::Config + pallet_staking::Config>(
	account_id: &T::AccountId,
	amount_to_change_to: BalanceOf<T>,
	team_account_id: &T::AccountId,
	reads: &mut u64,
	writes: &mut u64,
) {
	let schedules = Vesting::<T>::get(account_id);
	*reads += 1;
	if let Some(schedules) = schedules {
		update_vesting_schedule::<T>(
			account_id,
			amount_to_change_to,
			team_account_id,
			schedules.to_vec(),
		);
		*writes += 1;
	}
}

fn update_vesting_schedule<
	T: pallet_staking::Config + pallet_vesting::Config + pallet_balances::Config,
>(
	account_id: &T::AccountId,
	amount_to_change_to: BalanceOf<T>,
	team_account_id: &T::AccountId,
	schedules: Vec<VestingInfo<BalanceOf<T>, BlockNumberOf<T>>>,
) {
	// Calculate total vested amount
	let total_vested = schedules
		.iter()
		.map(|schedule| schedule.locked())
		.fold(Zero::zero(), |acc: BalanceOf<T>, val: BalanceOf<T>| acc.saturating_add(val));

	// Calculate the difference between the amount to change to and the total vested amount
	// Send the difference back to team account
	let difference = amount_to_change_to.saturating_sub(total_vested);

	if total_vested.is_zero() {
		return;
	}

	// New vesting parameters
	let one_year_blocks = BlockNumberOf::<T>::from(ONE_YEAR_BLOCKS as u32);
	let three_year_blocks = one_year_blocks.saturating_mul(BlockNumberOf::<T>::from(3u32));

	// At 1 year cliff, 25% unlocks
	let quarter_percentage = Percent::from_percent(25);
	let cliff_amount = quarter_percentage.mul_floor(amount_to_change_to);
	// Remaining 75% vests linearly over 3 years
	let remaining_amount = amount_to_change_to.saturating_sub(cliff_amount);
	let per_block = remaining_amount
		.ensure_div(T::BlockNumberToBalance::convert(three_year_blocks))
		.unwrap();

	let mut bounded_new_schedules: BoundedVec<
		VestingInfo<BalanceOf<T>, BlockNumberOf<T>>,
		MaxVestingSchedulesGet<T>,
	> = BoundedVec::new();

	bounded_new_schedules
		.try_push(VestingInfo::new(cliff_amount, Zero::zero(), one_year_blocks))
		.expect("Failed to push new schedules");
	bounded_new_schedules
		.try_push(VestingInfo::new(remaining_amount, per_block, one_year_blocks))
		.expect("Failed to push new schedules");

	// Update storage
	Vesting::<T>::insert(account_id, bounded_new_schedules);

	// Send the difference back to team account
	pallet_balances::Pallet::<T>::force_transfer(
		T::RuntimeOrigin::from(RawOrigin::Root),
		account_id.into(),
		team_account_id.into(),
		difference,
	);
}

#[cfg(feature = "try-runtime")]
fn verify_updated_schedule<T: pallet_vesting::Config>(
	account_id: &T::AccountId,
) -> Result<(), &'static str> {
	use sp_runtime::traits::Block;

	let schedules = Vesting::<T>::get(account_id);
	if let Some(schedules) = schedules {
		ensure!(schedules.len() >= 2, "Schedule should have at least 2 entries");
	}

	// Ensure amount is less than double the original amount
	let total_vested = schedules
		.iter()
		.map(|schedule| schedule.locked())
		.fold(Zero::zero(), |acc: BalanceOf<T>, val: BalanceOf<T>| acc.saturating_add(val));
	let original_amount = TEAM_ACCOUNT_VESTING_UPDATE.1;
	let double_amount = original_amount.saturating_mul(2);
	ensure!(
		total_vested < double_amount,
		"Amount to change to is greater than double the original amount"
	);

	let new_balance = pallet_balances::Pallet::<T>::free_balance(account_id);
	ensure!(new_balance < double_amount, "New balance is greater than double the original amount");

	Ok(())
}
