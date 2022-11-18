#![cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{RuntimeEvent, *};
use sp_runtime::traits::BadOrigin;

const BALANCE_TRANSFER: &<Runtime as frame_system::Config>::RuntimeCall =
	&mock::RuntimeCall::Balances(pallet_balances::Call::transfer { dest: ALICE, value: 10 });
const TOKENS_TRANSFER: &<Runtime as frame_system::Config>::RuntimeCall =
	&mock::RuntimeCall::Tokens(orml_tokens::Call::transfer {
		dest: ALICE,
		currency_id: STABLE,
		amount: 10,
	});

#[test]
fn pause_transaction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			TransactionPause::pause_transaction(
				RuntimeOrigin::signed(5),
				b"Balances".to_vec(),
				b"transfer".to_vec()
			),
			BadOrigin
		);

		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			None
		);
		assert_ok!(TransactionPause::pause_transaction(
			RuntimeOrigin::signed(1),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(RuntimeEvent::TransactionPause(
			crate::Event::TransactionPaused {
				pallet_name_bytes: b"Balances".to_vec(),
				function_name_bytes: b"transfer".to_vec(),
			},
		));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			Some(())
		);

		assert_noop!(
			TransactionPause::pause_transaction(
				RuntimeOrigin::signed(1),
				b"TransactionPause".to_vec(),
				b"pause_transaction".to_vec()
			),
			Error::<Runtime>::CannotPause
		);
		assert_noop!(
			TransactionPause::pause_transaction(
				RuntimeOrigin::signed(1),
				b"TransactionPause".to_vec(),
				b"some_other_call".to_vec()
			),
			Error::<Runtime>::CannotPause
		);
		assert_ok!(TransactionPause::pause_transaction(
			RuntimeOrigin::signed(1),
			b"OtherPallet".to_vec(),
			b"pause_transaction".to_vec()
		));
	});
}

#[test]
fn unpause_transaction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TransactionPause::pause_transaction(
			RuntimeOrigin::signed(1),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			Some(())
		);

		assert_noop!(
			TransactionPause::unpause_transaction(
				RuntimeOrigin::signed(5),
				b"Balances".to_vec(),
				b"transfer".to_vec()
			),
			BadOrigin
		);

		assert_ok!(TransactionPause::unpause_transaction(
			RuntimeOrigin::signed(1),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		System::assert_last_event(RuntimeEvent::TransactionPause(
			crate::Event::TransactionUnpaused {
				pallet_name_bytes: b"Balances".to_vec(),
				function_name_bytes: b"transfer".to_vec(),
			},
		));
		assert_eq!(
			TransactionPause::paused_transactions((b"Balances".to_vec(), b"transfer".to_vec())),
			None
		);
	});
}

#[test]
fn paused_transaction_filter_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!PausedTransactionFilter::<Runtime>::contains(BALANCE_TRANSFER));
		assert!(!PausedTransactionFilter::<Runtime>::contains(TOKENS_TRANSFER));
		assert_ok!(TransactionPause::pause_transaction(
			RuntimeOrigin::signed(1),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert_ok!(TransactionPause::pause_transaction(
			RuntimeOrigin::signed(1),
			b"Tokens".to_vec(),
			b"transfer".to_vec()
		));
		assert!(PausedTransactionFilter::<Runtime>::contains(BALANCE_TRANSFER));
		assert!(PausedTransactionFilter::<Runtime>::contains(TOKENS_TRANSFER));
		assert_ok!(TransactionPause::unpause_transaction(
			RuntimeOrigin::signed(1),
			b"Balances".to_vec(),
			b"transfer".to_vec()
		));
		assert_ok!(TransactionPause::unpause_transaction(
			RuntimeOrigin::signed(1),
			b"Tokens".to_vec(),
			b"transfer".to_vec()
		));
		assert!(!PausedTransactionFilter::<Runtime>::contains(BALANCE_TRANSFER));
		assert!(!PausedTransactionFilter::<Runtime>::contains(TOKENS_TRANSFER));
	});
}
