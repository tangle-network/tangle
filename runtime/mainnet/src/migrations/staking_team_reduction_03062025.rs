use crate::RuntimeOrigin;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use frame_system::RawOrigin;
use pallet_vesting::{MaxVestingSchedulesGet, Vesting, VestingInfo};
use sp_runtime::traits::StaticLookup;
use sp_runtime::{
	Percent, Saturating,
	traits::{Convert, EnsureDiv, Header, Zero},
};
use sp_std::{vec, vec::Vec};
use tangle_primitives::Balance;
pub const BLOCK_TIME: u128 = 6;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;
use log::info;
use sp_staking::StakingInterface;

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
		114383561650000000000000, // Amount to unstake and send back to team
	),
	// TB
	(
		[
			140, 62, 57, 135, 164, 41, 178, 70, 246, 25, 233, 31, 140, 164, 85, 53, 161, 191, 95,
			135, 14, 199, 197, 207, 246, 169, 16, 169, 148, 151, 139, 65,
		],
		121947842700000000000000, // Amount to unstake and send back to team
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

pub type StakingBalanceOf<T> =
	<<T as pallet_staking::Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

pub type VestingBalanceOf<T> =
	<<T as pallet_vesting::Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

pub type BalanceOf<T> = <T as pallet_balances::Config>::Balance;

pub type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as Header>::Number;

impl<T: pallet_staking::Config + pallet_vesting::Config + pallet_balances::Config> OnRuntimeUpgrade
	for UpdateTeamMemberAllocation<T>
where
	T: frame_system::Config<RuntimeOrigin = RuntimeOrigin>,
{
	fn on_runtime_upgrade() -> Weight {
		let mut reads = 0u64;
		let mut writes = 0u64;

		#[allow(clippy::type_complexity)]
		let mut nominated_validators: Vec<(
			T::AccountId,
			StakingBalanceOf<T>,
			Vec<T::AccountId>,
		)> = vec![];

		// Remove staking records from team accounts
		for (account, amount) in TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE.iter() {
			let account_id = match T::AccountId::decode(&mut account.as_ref()) {
				Ok(id) => id,
				Err(_e) => {
					info!("Failed to decode account ID");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

			let _controller = match pallet_staking::Bonded::<T>::get(account_id.clone()) {
				Some(controller) => controller,
				None => {
					info!("Controller not found for account");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

			let ledger = match pallet_staking::Ledger::<T>::get(_controller.clone()) {
				Some(l) => l,
				None => {
					info!("Ledger not found for controller");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

			let nominations = match pallet_staking::Nominators::<T>::get(account_id.clone()) {
				Some(n) => n,
				None => {
					info!("Nominations not found for account");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

			let amount_encoded = match StakingBalanceOf::<T>::decode(&mut amount.encode().as_ref())
			{
				Ok(a) => a,
				Err(_e) => {
					info!("Failed to decode amount");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

			nominated_validators.push((
				account_id,
				ledger.active - amount_encoded,
				nominations.targets.iter().cloned().collect(),
			));

			if let Err(_e) = pallet_staking::Pallet::<T>::force_unstake(
				T::RuntimeOrigin::from(RawOrigin::Root),
				_controller,
				100,
			) {
				info!("Failed to force unstake");
				return T::DbWeight::get().reads_writes(reads, writes);
			}
		}

		// Send back balance from team member account with no vesting change
		let team_account_id = match T::AccountId::decode(&mut TEAM_ACCOUNT.as_ref()) {
			Ok(id) => id,
			Err(_e) => {
				info!("Failed to decode team account ID");
				return T::DbWeight::get().reads_writes(reads, writes);
			},
		};

		let source_account_id =
			match T::AccountId::decode(&mut TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE[1].0.as_ref()) {
				Ok(id) => id,
				Err(_e) => {
					info!("Failed to decode source account ID");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

		let amount = match T::Balance::decode(
			&mut TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE[1].1.encode().as_ref(),
		) {
			Ok(a) => a,
			Err(_e) => {
				info!("Failed to decode amount");
				return T::DbWeight::get().reads_writes(reads, writes);
			},
		};

		// Check free balance before transfer
		let free_balance = pallet_balances::Pallet::<T>::free_balance(&source_account_id);
		info!("Source account free balance before transfer: {:?}", free_balance);
		info!("Amount to transfer: {:?}", amount);

		// Only transfer if sufficient free balance exists
		let transfer_amount = BalanceOf::<T>::from(amount);

		if free_balance >= transfer_amount {
			if let Err(_e) = pallet_balances::Pallet::<T>::force_transfer(
				T::RuntimeOrigin::from(RawOrigin::Root),
				T::Lookup::unlookup(source_account_id),
				T::Lookup::unlookup(team_account_id.clone()),
				transfer_amount,
			) {
				info!("Failed to transfer balance: {:?}", _e);
				return T::DbWeight::get().reads_writes(reads, writes);
			} else {
				info!("Successfully transferred balance to team account");
			}
		} else {
			info!(
				"Insufficient free balance for transfer: needed {:?}, had {:?}",
				transfer_amount, free_balance
			);
			return T::DbWeight::get().reads_writes(reads, writes);
		}

		// Update vesting record and balance from team account with vesting change
		let vesting_amount = match VestingBalanceOf::<T>::decode(
			&mut TEAM_ACCOUNT_VESTING_UPDATE.1.encode().as_ref(),
		) {
			Ok(a) => a,
			Err(_e) => {
				info!("Failed to decode vesting amount");
				return T::DbWeight::get().reads_writes(reads, writes);
			},
		};

		let vesting_account_id =
			match T::AccountId::decode(&mut TEAM_ACCOUNT_VESTING_UPDATE.0.as_ref()) {
				Ok(id) => id,
				Err(_e) => {
					info!("Failed to decode vesting account ID");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};

		update_account_vesting::<T>(
			&vesting_account_id,
			vesting_amount,
			&team_account_id,
			&mut reads,
			&mut writes,
		);

		// Nominate the same validators as before
		for (account_id, amount, targets) in nominated_validators {
			// Bond the stash accounts
			if let Err(_e) = pallet_staking::Pallet::<T>::bond(
				T::RuntimeOrigin::from(RawOrigin::Root),
				amount,
				pallet_staking::RewardDestination::Staked,
			) {
				info!("Failed to bond");
				return T::DbWeight::get().reads_writes(reads, writes);
			}

			let _ = <pallet_staking::Pallet<T> as StakingInterface>::nominate(
				&account_id,
				targets.clone(),
			);
		}

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		for (account, _) in TEAM_MEMBER_ACCOUNTS_STAKING_UPDATE {
			let account_id = match account.parse() {
				Ok(id) => id,
				Err(_) => {
					info!("Invalid account ID in post_upgrade");
					return Err("Invalid account ID");
				},
			};
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

// Update account vesting schedule
fn update_account_vesting<
	T: pallet_staking::Config + pallet_vesting::Config + pallet_balances::Config,
>(
	account_id: &T::AccountId,
	amount_to_change_to: VestingBalanceOf<T>,
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
	amount_to_change_to: VestingBalanceOf<T>,
	team_account_id: &T::AccountId,
	schedules: Vec<VestingInfo<VestingBalanceOf<T>, BlockNumberOf<T>>>,
) {
	// Calculate total vested amount
	let total_vested = schedules
		.iter()
		.map(|schedule| schedule.locked())
		.fold(Zero::zero(), |acc: VestingBalanceOf<T>, val: VestingBalanceOf<T>| {
			acc.saturating_add(val)
		});

	// Check if there is a difference to transfer
	let mut difference_to_transfer = BalanceOf::<T>::zero();

	// If total_vested > amount_to_change_to, we need to transfer the difference back to team
	if total_vested > amount_to_change_to {
		// Calculate the difference safely without relying on decode
		let vesting_diff = total_vested.saturating_sub(amount_to_change_to);

		match BalanceOf::<T>::try_from(vesting_diff.saturated_into::<u128>()) {
			Ok(diff) => difference_to_transfer = diff,
			Err(_) => {
				info!("Failed to convert difference amount");
				return;
			},
		}

		info!("Difference to transfer: {:?}", difference_to_transfer);
	} else {
		// No need to transfer anything if the new amount is larger or equal
		info!("No difference to transfer or new amount is larger");
	}

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
	let per_block =
		match remaining_amount.ensure_div(T::BlockNumberToBalance::convert(three_year_blocks)) {
			Ok(pb) => pb,
			Err(_e) => {
				info!("Failed to calculate per_block amount");
				return;
			},
		};

	let mut bounded_new_schedules: BoundedVec<
		VestingInfo<VestingBalanceOf<T>, BlockNumberOf<T>>,
		MaxVestingSchedulesGet<T>,
	> = BoundedVec::new();

	if let Err(_e) = bounded_new_schedules.try_push(VestingInfo::new(
		cliff_amount,
		Zero::zero(),
		one_year_blocks,
	)) {
		info!("Failed to push first vesting schedule");
		return;
	}

	if let Err(_e) = bounded_new_schedules.try_push(VestingInfo::new(
		remaining_amount,
		per_block,
		one_year_blocks,
	)) {
		info!("Failed to push second vesting schedule");
		return;
	}

	// Update storage
	Vesting::<T>::insert(account_id, bounded_new_schedules);

	// Only attempt transfer if there's an actual difference to transfer
	if !difference_to_transfer.is_zero() {
		info!("Attempting to transfer difference");

		// Check free balance before transfer
		let free_balance = pallet_balances::Pallet::<T>::free_balance(account_id);
		info!("Account free balance before transfer: {:?}", free_balance);

		// Only transfer if sufficient free balance exists
		if free_balance >= difference_to_transfer {
			if let Err(_e) = pallet_balances::Pallet::<T>::force_transfer(
				T::RuntimeOrigin::from(RawOrigin::Root),
				T::Lookup::unlookup(account_id.clone()),
				T::Lookup::unlookup(team_account_id.clone()),
				difference_to_transfer,
			) {
				info!("Failed to force transfer difference: {:?}", _e);
			} else {
				info!("Successfully transferred difference to team account");
			}
		} else {
			info!(
				"Insufficient free balance for transfer: needed {:?}, had {:?}",
				difference_to_transfer, free_balance
			);
		}
	}
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
		.fold(Zero::zero(), |acc: VestingBalanceOf<T>, val: VestingBalanceOf<T>| {
			acc.saturating_add(val)
		});
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
