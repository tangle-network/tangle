// This file is part of Webb.

// Copyright (C) 2021 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::NegativeImbalance;
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
	R: pallet_balances::Config + pallet_authorship::Config,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
			let _numeric_amount = amount.peek();
			<pallet_balances::Pallet<R>>::resolve_creating(&author, amount);
		}
	}
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_treasury::Config + pallet_authorship::Config,
	pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(80, 20);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut split.1);
			}

			<pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(split.0);
			<ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{
		pallet_prelude::Weight, parameter_types, traits::FindAuthor, weights::DispatchClass,
		PalletId,
	};
	use frame_system::limits;
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;
	pub type AccountId = u64;
	const TEST_ACCOUNT: AccountId = 1_u64;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
			.base_block(Weight::from_ref_time(10))
			.for_class(DispatchClass::all(), |weight| {
				weight.base_extrinsic = Weight::from_ref_time(100);
			})
			.for_class(DispatchClass::non_mandatory(), |weight| {
				weight.max_total = Some(Weight::from_ref_time(1024));
			})
			.build_or_panic();
		pub BlockLength: limits::BlockLength = limits::BlockLength::max(2 * 1024);
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = Call;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type BlockLength = BlockLength;
		type BlockWeights = BlockWeights;
		type DbWeight = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_balances::Config for Test {
		type Balance = u64;
		type Event = Event;
		type DustRemoval = ();
		type ExistentialDeposit = ();
		type AccountStore = System;
		type MaxLocks = ();
		type MaxReserves = ();
		type ReserveIdentifier = [u8; 8];
		type WeightInfo = ();
	}

	parameter_types! {
		pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
		pub const MaxApprovals: u32 = 100;
	}

	impl pallet_treasury::Config for Test {
		type Currency = pallet_balances::Pallet<Test>;
		type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u64>;
		type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
		type RejectOrigin = frame_system::EnsureRoot<AccountId>;
		type Event = Event;
		type OnSlash = ();
		type ProposalBond = ();
		type ProposalBondMinimum = ();
		type ProposalBondMaximum = ();
		type SpendPeriod = ();
		type Burn = ();
		type BurnDestination = ();
		type PalletId = TreasuryPalletId;
		type SpendFunds = ();
		type MaxApprovals = MaxApprovals;
		type WeightInfo = ();
	}

	parameter_types! {
		pub const CollatorRewardsId: PalletId = PalletId(*b"tnt/clrw");
	}

	pub struct OneAuthor;
	impl FindAuthor<AccountId> for OneAuthor {
		fn find_author<'a, I>(_: I) -> Option<AccountId>
		where
			I: 'a,
		{
			Some(TEST_ACCOUNT)
		}
	}

	impl pallet_authorship::Config for Test {
		type FindAuthor = OneAuthor;
		type UncleGenerations = ();
		type FilterUncle = ();
		type EventHandler = ();
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		pallet_balances::GenesisConfig::<Test>::default()
			.assimilate_storage(&mut t)
			.unwrap();
		t.into()
	}

	#[test]
	fn test_fees_and_tip_and_reward_distribution() {
		new_test_ext().execute_with(|| {
			let fee = Balances::issue(10);
			let tip = Balances::issue(20);

			assert_eq!(Balances::free_balance(Treasury::account_id()), 0);
			assert_eq!(Balances::free_balance(TEST_ACCOUNT), 0);

			// lets set the collator reward amount to 10
			CollatorRewards::force_set_reward_amount(Origin::root(), 10).unwrap();

			DealWithFees::on_unbalanceds(vec![fee, tip].into_iter());

			// Author gets 100% of tip and 20% of fee = 22
			// Author also gets collator reward amount of 10
			assert_eq!(Balances::free_balance(TEST_ACCOUNT), 32);

			// Treasury gets 80% of fee
			assert_eq!(Balances::free_balance(Treasury::account_id()), 8);

			// Since we did not allot any amount to the collator rewards pallet, new issuance was
			// created This should now be fee + tip + reward
			assert_eq!(Balances::total_issuance(), 40);
		});
	}
}
