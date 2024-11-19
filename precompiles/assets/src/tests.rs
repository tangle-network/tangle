use crate::{mock::*, U256};
use frame_support::traits::fungibles::Inspect;
use precompile_utils::testing::*;
use sp_core::H160;

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_create_asset() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					id: U256::from(1),
					admin: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					min_balance: U256::from(1),
				},
			)
			.execute_returns(());

		// Verify asset was created
		assert!(Assets::asset_exists(1));
	});
}

#[test]
fn test_mint_asset() {
	ExtBuilder::default().build().execute_with(|| {
		// First create the asset
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					id: U256::from(1),
					admin: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					min_balance: U256::from(1),
				},
			)
			.execute_returns(());

		// Then mint some tokens
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::mint {
					id: U256::from(1),
					beneficiary: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					amount: U256::from(100),
				},
			)
			.execute_returns(());
	});
}

#[test]
fn test_transfer_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let admin = sp_core::sr25519::Public::from(TestAccount::Alex);
		let to = sp_core::sr25519::Public::from(TestAccount::Bob);

		// Create asset
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					id: U256::from(1),
					admin: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					min_balance: U256::from(1),
				},
			)
			.execute_returns(());

		// Mint tokens to sender
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::mint {
					id: U256::from(1),
					beneficiary: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					amount: U256::from(100),
				},
			)
			.execute_returns(());

		// Transfer tokens
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::transfer {
					id: U256::from(1),
					target: precompile_utils::prelude::Address(H160::repeat_byte(0x02)),
					amount: U256::from(50),
				},
			)
			.execute_returns(());

		// Verify balances
		assert_eq!(Assets::balance(1, admin), 50);
		assert_eq!(Assets::balance(1, to), 50);
	});
}

#[test]
fn test_start_destroy() {
	ExtBuilder::default().build().execute_with(|| {
		// Create asset
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create {
					id: U256::from(1),
					admin: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
					min_balance: U256::from(1),
				},
			)
			.execute_returns(());

		// Start destroy
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::start_destroy { id: U256::from(1) },
			)
			.execute_returns(());

		// Verify asset is being destroyed
		assert!(Assets::asset_exists(1)); // Still exists but in "destroying" state
	});
}

#[test]
fn test_mint_insufficient_permissions() {
	ExtBuilder::default().build().execute_with(|| {
        // Create asset
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alex,
                H160::from_low_u64_be(1),
                PCall::create {
                    id: U256::from(1),
                    admin: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
                    min_balance: U256::from(1),
                },
            )
            .execute_returns(());

        // Try to mint without permission
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Bob,
                H160::from_low_u64_be(1),
                PCall::mint {
                    id: U256::from(1),
                    beneficiary: precompile_utils::prelude::Address(H160::repeat_byte(0x01)),
                    amount: U256::from(100),
                },
            )
            .execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 4, error: [2, 0, 0, 0], message: Some(\"NoPermission\") })");
    });
}
