use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use log::info;
use pallet_vesting::{MaxVestingSchedulesGet, Vesting, VestingInfo};
use sp_runtime::{
	Percent, Saturating,
	traits::{Convert, EnsureDiv, Header, Zero},
};
use sp_std::vec::Vec;

pub const BLOCK_TIME: u128 = 6;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;

pub const INVESTOR_ACCOUNTS: [[u8; 32]; 29] = [
	[
		138, 249, 246, 184, 248, 138, 249, 225, 200, 68, 130, 170, 83, 137, 64, 251, 106, 197, 74,
		79, 232, 191, 177, 86, 84, 71, 188, 169, 202, 64, 231, 36,
	],
	[
		225, 94, 83, 233, 185, 41, 177, 181, 248, 143, 147, 205, 180, 83, 14, 125, 47, 25, 125,
		248, 22, 125, 226, 220, 156, 78, 175, 3, 230, 244, 140, 52,
	],
	[
		226, 56, 28, 178, 235, 59, 229, 44, 201, 146, 200, 29, 38, 152, 123, 201, 70, 248, 93, 143,
		16, 158, 67, 242, 146, 71, 119, 144, 164, 118, 131, 239,
	],
	[
		83, 94, 177, 114, 5, 76, 126, 89, 124, 208, 48, 113, 197, 113, 201, 20, 143, 244, 231, 128,
		68, 163, 167, 63, 84, 227, 193, 177, 216, 112, 208, 84,
	],
	[
		226, 248, 45, 109, 239, 71, 153, 69, 230, 189, 169, 9, 81, 83, 24, 60, 250, 169, 164, 251,
		82, 166, 31, 117, 66, 164, 222, 120, 208, 80, 196, 88,
	],
	[
		72, 57, 99, 4, 212, 215, 132, 34, 128, 223, 72, 106, 143, 56, 37, 98, 103, 77, 253, 149,
		40, 92, 6, 129, 204, 248, 163, 22, 61, 149, 207, 120,
	],
	[
		165, 243, 249, 88, 179, 113, 230, 225, 100, 88, 23, 11, 87, 179, 246, 32, 54, 85, 215, 64,
		68, 146, 82, 238, 21, 108, 206, 162, 78, 144, 200, 158,
	],
	[
		70, 101, 198, 201, 165, 102, 235, 103, 142, 23, 51, 45, 209, 4, 177, 120, 143, 223, 81,
		222, 187, 212, 130, 212, 103, 129, 129, 86, 35, 141, 167, 97,
	],
	[
		158, 193, 210, 91, 210, 217, 80, 151, 219, 193, 65, 163, 149, 70, 177, 108, 77, 81, 5, 84,
		187, 146, 59, 162, 138, 78, 236, 7, 21, 109, 42, 66,
	],
	[
		73, 92, 137, 97, 116, 188, 150, 231, 162, 97, 156, 213, 207, 65, 244, 189, 115, 96, 149,
		86, 219, 50, 223, 200, 220, 213, 143, 160, 153, 53, 153, 173,
	],
	[
		196, 215, 35, 118, 232, 152, 244, 80, 42, 241, 100, 107, 224, 191, 61, 29, 64, 54, 3, 112,
		174, 240, 158, 212, 197, 115, 20, 10, 10, 92, 116, 28,
	],
	[
		26, 55, 2, 120, 183, 66, 224, 91, 238, 81, 231, 203, 154, 226, 246, 253, 26, 45, 218, 123,
		51, 5, 122, 148, 251, 110, 109, 204, 217, 81, 117, 18,
	],
	[
		20, 148, 147, 133, 5, 238, 234, 73, 51, 154, 251, 105, 92, 63, 31, 44, 131, 42, 183, 201,
		182, 242, 211, 21, 14, 99, 22, 212, 193, 29, 186, 217,
	],
	[
		50, 234, 142, 113, 250, 70, 204, 94, 20, 44, 9, 35, 5, 25, 87, 208, 53, 96, 94, 47, 51,
		135, 10, 94, 133, 29, 179, 167, 247, 226, 137, 19,
	],
	[
		20, 160, 96, 205, 27, 33, 238, 218, 236, 77, 34, 88, 25, 163, 112, 231, 10, 110, 163, 111,
		36, 5, 110, 50, 142, 76, 49, 183, 209, 147, 179, 100,
	],
	[
		180, 115, 60, 111, 163, 25, 14, 178, 121, 79, 191, 213, 179, 191, 156, 236, 25, 37, 83, 89,
		216, 107, 123, 146, 254, 218, 30, 154, 115, 101, 39, 47,
	],
	[
		110, 132, 65, 222, 55, 201, 168, 118, 191, 249, 229, 145, 52, 219, 24, 86, 203, 255, 63,
		209, 91, 34, 206, 199, 8, 240, 78, 43, 66, 93, 224, 31,
	],
	[
		158, 51, 219, 46, 28, 186, 50, 107, 225, 183, 177, 124, 244, 215, 38, 15, 44, 229, 59, 64,
		40, 246, 80, 239, 182, 237, 129, 191, 49, 173, 231, 19,
	],
	[
		240, 187, 14, 171, 122, 87, 33, 20, 39, 68, 54, 68, 44, 137, 73, 107, 151, 54, 58, 149,
		132, 130, 222, 184, 244, 65, 148, 162, 98, 23, 127, 76,
	],
	[
		102, 46, 18, 1, 115, 40, 196, 41, 159, 117, 230, 38, 72, 10, 253, 186, 224, 229, 249, 227,
		186, 62, 32, 12, 210, 131, 42, 190, 135, 233, 142, 27,
	],
	[
		72, 173, 142, 236, 101, 180, 147, 239, 64, 70, 44, 143, 198, 254, 51, 143, 104, 149, 94,
		146, 75, 159, 40, 52, 53, 225, 28, 197, 73, 84, 189, 11,
	],
	[
		0, 64, 106, 126, 60, 179, 52, 134, 195, 43, 228, 249, 202, 36, 34, 62, 137, 47, 60, 194,
		123, 253, 171, 30, 205, 154, 28, 23, 3, 169, 223, 100,
	],
	[
		244, 156, 26, 140, 205, 108, 13, 44, 139, 130, 165, 246, 58, 203, 83, 171, 189, 21, 71, 62,
		89, 81, 150, 115, 29, 175, 114, 213, 194, 91, 164, 64,
	],
	[
		252, 249, 120, 35, 115, 244, 16, 85, 11, 113, 242, 83, 208, 86, 198, 225, 23, 163, 23, 142,
		49, 124, 81, 48, 158, 202, 185, 100, 59, 31, 218, 67,
	],
	[
		4, 125, 23, 244, 85, 187, 199, 151, 55, 109, 248, 90, 16, 134, 116, 192, 43, 28, 74, 246,
		21, 111, 172, 109, 27, 122, 88, 247, 219, 172, 88, 63,
	],
	[
		58, 69, 255, 40, 206, 137, 18, 192, 97, 105, 41, 95, 157, 205, 120, 252, 232, 50, 221, 68,
		83, 187, 60, 133, 96, 189, 35, 160, 89, 121, 188, 7,
	],
	[
		128, 133, 255, 113, 171, 251, 146, 3, 232, 220, 247, 119, 166, 113, 67, 37, 228, 170, 48,
		5, 52, 240, 26, 77, 236, 13, 180, 213, 9, 26, 219, 117,
	],
	[
		74, 126, 6, 168, 58, 228, 131, 195, 50, 142, 139, 187, 204, 130, 114, 32, 107, 105, 238,
		136, 202, 91, 109, 251, 247, 98, 9, 37, 122, 202, 33, 103,
	],
	[
		14, 243, 144, 141, 108, 213, 243, 175, 130, 92, 68, 191, 148, 27, 215, 242, 160, 27, 197,
		13, 240, 118, 166, 253, 214, 93, 174, 242, 127, 123, 238, 97,
	],
];

pub const TEAM_ACCOUNTS: [[u8; 32]; 11] = [
	[
		220, 217, 183, 10, 4, 9, 183, 98, 108, 186, 26, 64, 22, 216, 218, 25, 244, 223, 92, 233,
		252, 94, 141, 22, 183, 137, 231, 27, 177, 22, 29, 115,
	],
	[
		76, 227, 164, 218, 58, 124, 28, 230, 95, 126, 222, 255, 134, 77, 195, 221, 66, 232, 244,
		126, 236, 194, 114, 109, 153, 160, 168, 1, 36, 105, 130, 23,
	],
	[
		226, 96, 78, 172, 142, 71, 162, 218, 115, 191, 193, 196, 247, 215, 108, 71, 131, 198, 47,
		143, 92, 45, 64, 78, 106, 176, 223, 167, 103, 39, 29, 33,
	],
	[
		162, 133, 55, 193, 209, 134, 98, 231, 154, 181, 108, 72, 150, 15, 166, 174, 160, 28, 203,
		238, 113, 51, 80, 242, 197, 70, 70, 237, 33, 244, 251, 67,
	],
	[
		112, 37, 90, 147, 193, 18, 157, 49, 188, 57, 43, 67, 30, 93, 62, 212, 223, 135, 97, 116,
		54, 19, 11, 84, 98, 79, 154, 98, 77, 97, 206, 66,
	],
	[
		242, 66, 123, 184, 39, 134, 15, 110, 181, 6, 187, 139, 110, 58, 126, 105, 95, 171, 231,
		171, 87, 30, 111, 88, 107, 141, 246, 112, 113, 230, 9, 127,
	],
	[
		72, 59, 70, 104, 50, 224, 148, 240, 27, 23, 121, 167, 237, 7, 2, 93, 243, 25, 196, 146,
		218, 197, 22, 10, 202, 137, 163, 190, 17, 122, 123, 109,
	],
	[
		140, 62, 57, 135, 164, 41, 178, 70, 246, 25, 233, 31, 140, 164, 85, 53, 161, 191, 95, 135,
		14, 199, 197, 207, 246, 169, 16, 169, 148, 151, 139, 65,
	],
	[
		134, 208, 142, 123, 190, 119, 188, 116, 227, 216, 142, 226, 46, 220, 83, 54, 139, 193, 61,
		97, 158, 5, 182, 111, 230, 196, 184, 226, 213, 199, 1, 90,
	],
	[
		220, 247, 191, 133, 183, 80, 119, 15, 91, 184, 117, 45, 0, 7, 150, 96, 236, 168, 190, 205,
		63, 90, 138, 73, 172, 186, 174, 240, 173, 126, 237, 8,
	],
	[
		142, 28, 43, 221, 218, 185, 87, 61, 140, 176, 148, 219, 255, 186, 36, 162, 178, 194, 27,
		126, 113, 227, 245, 182, 4, 232, 96, 116, 131, 135, 36, 67,
	],
];

/// Migration to update team and investor vesting schedules to 4 years with 1 year cliff
pub struct UpdateTeamInvestorVesting<T>(sp_std::marker::PhantomData<T>);

pub type BalanceOf<T> =
	<<T as pallet_vesting::Config>::Currency as frame_support::traits::Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

pub type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as Header>::Number;

impl<T: pallet_vesting::Config + pallet_balances::Config> OnRuntimeUpgrade
	for UpdateTeamInvestorVesting<T>
{
	fn on_runtime_upgrade() -> Weight {
		let mut reads = 0u64;
		let mut writes = 0u64;

		// Update investor vesting schedules
		for account in INVESTOR_ACCOUNTS.iter() {
			let account_id = match T::AccountId::decode(&mut account.as_ref()) {
				Ok(id) => id,
				Err(_) => {
					info!("Failed to decode investor account ID");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};
			update_account_vesting::<T>(&account_id, &mut reads, &mut writes);
		}

		// Update team vesting schedule
		for account in TEAM_ACCOUNTS.iter() {
			let account_id = match T::AccountId::decode(&mut account.as_ref()) {
				Ok(id) => id,
				Err(_) => {
					info!("Failed to decode team account ID");
					return T::DbWeight::get().reads_writes(reads, writes);
				},
			};
			update_account_vesting::<T>(&account_id, &mut reads, &mut writes);
		}

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		// Verify schedules for all accounts
		for (address, _) in INVESTOR_ACCOUNTS {
			let account_id = if address.starts_with("0x") {
				let h160 = match H160::from_str(address) {
					Ok(h) => h,
					Err(_) => {
						info!("Invalid EVM address");
						return Err("Invalid EVM address");
					},
				};
				MultiAddress::Evm(h160).to_account_id_32()
			} else {
				match address.parse() {
					Ok(id) => id,
					Err(_) => {
						info!("Invalid investor account ID");
						return Err("Invalid investor account ID");
					},
				}
			};
			verify_updated_schedule::<T>(&account_id)?;
		}
		for (address, _) in TEAM_ACCOUNT_TO_UPDATE {
			let account_id = match address.parse() {
				Ok(id) => id,
				Err(_) => {
					info!("Invalid team account ID");
					return Err("Invalid team account ID");
				},
			};
			verify_updated_schedule::<T>(&account_id)?;
		}

		Ok(())
	}
}

// Update investor vesting schedules
fn update_account_vesting<T: pallet_vesting::Config + pallet_balances::Config>(
	account_id: &T::AccountId,
	reads: &mut u64,
	writes: &mut u64,
) {
	let schedules = Vesting::<T>::get(account_id);
	*reads += 1;
	if let Some(schedules) = schedules {
		update_vesting_schedule::<T>(account_id, schedules.to_vec());
		*writes += 1;
	}
}

fn update_vesting_schedule<T: pallet_vesting::Config>(
	account_id: &T::AccountId,
	schedules: Vec<VestingInfo<BalanceOf<T>, BlockNumberOf<T>>>,
) {
	// Calculate total vested amount
	let total_vested = schedules
		.iter()
		.map(|schedule| schedule.locked())
		.fold(Zero::zero(), |acc: BalanceOf<T>, val: BalanceOf<T>| acc.saturating_add(val));

	if total_vested.is_zero() {
		return;
	}

	// New vesting parameters
	let one_year_blocks = BlockNumberOf::<T>::from(ONE_YEAR_BLOCKS as u32);
	let three_year_blocks = one_year_blocks.saturating_mul(BlockNumberOf::<T>::from(3u32));

	// At 1 year cliff, 25% unlocks
	let quarter_percentage = Percent::from_percent(25);
	let cliff_amount = quarter_percentage.mul_floor(total_vested);
	// Remaining 75% vests linearly over 3 years
	let remaining_amount = total_vested.saturating_sub(cliff_amount);
	let per_block =
		match remaining_amount.ensure_div(T::BlockNumberToBalance::convert(three_year_blocks)) {
			Ok(pb) => pb,
			Err(_) => {
				info!("Failed to calculate per_block amount");
				return;
			},
		};

	let mut bounded_new_schedules: BoundedVec<
		VestingInfo<BalanceOf<T>, BlockNumberOf<T>>,
		MaxVestingSchedulesGet<T>,
	> = BoundedVec::new();

	if bounded_new_schedules
		.try_push(VestingInfo::new(cliff_amount, cliff_amount, one_year_blocks))
		.is_err()
	{
		info!("Failed to push first vesting schedule");
		return;
	}

	if bounded_new_schedules
		.try_push(VestingInfo::new(remaining_amount, per_block, one_year_blocks))
		.is_err()
	{
		info!("Failed to push second vesting schedule");
		return;
	}

	// Update storage
	Vesting::<T>::insert(account_id, bounded_new_schedules);
}

#[cfg(feature = "try-runtime")]
fn verify_updated_schedule<T: pallet_vesting::Config>(
	account_id: &T::AccountId,
) -> Result<(), &'static str> {
	use sp_runtime::traits::Block;

	if let Some(schedules) = Vesting::<T>::get(account_id) {
		ensure!(schedules.len() >= 2, "Schedule should have at least 2 entries");
		ensure!(
			schedules[0].starting_block()
				== <<<T as frame_system::Config>::Block as Block>::Header as Header>::Number::from(
					ONE_YEAR_BLOCKS as u32
				),
			"First schedule should start at 1 year"
		);
		ensure!(
			schedules[1].starting_block()
				== <<<T as frame_system::Config>::Block as Block>::Header as Header>::Number::from(
					ONE_YEAR_BLOCKS as u32
				),
			"Second schedule should start at 1 year"
		);
		ensure!(schedules[0].per_block().is_zero(), "First schedule should have zero per_block");
	}
	Ok(())
}
