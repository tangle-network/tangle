use core::fmt::Debug;

use super::*;
use frame_support::pallet_prelude::DispatchError;
use hex_literal::hex;
use parity_scale_codec::Encode;
use secp_utils::*;
use sp_core::ConstU32;
use sp_core::Pair;
use sp_runtime::{BoundedVec, TokenError::Frozen};

// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
use crate::{mock::new_test_ext, pallet::Call as ClaimsCall};
use frame_support::{
	assert_err, assert_noop, assert_ok, pallet_prelude::DispatchError::BadOrigin,
	traits::ExistenceRequirement,
};
use sp_runtime::transaction_validity::TransactionLongevity;

use crate::mock::*;

fn total_claims() -> u64 {
	100 + 200 + 300 + 400 + 500 + 600
}

pub fn get_bounded_vec<T: Debug>(elt: T) -> BoundedVec<T, ConstU32<8>> {
	BoundedVec::<T, ConstU32<8>>::try_from(vec![elt]).unwrap()
}

#[test]
fn basic_setup_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(ClaimsPallet::total(), total_claims());
		assert_eq!(ClaimsPallet::claims(&eth(&alice())), Some(100));
		assert_eq!(ClaimsPallet::claims(&eth(&dave())), Some(200));
		assert_eq!(ClaimsPallet::claims(&eth(&eve())), Some(300));
		assert_eq!(ClaimsPallet::claims(&eth(&frank())), Some(400));
		assert_eq!(ClaimsPallet::claims(&MultiAddress::EVM(EthereumAddress::default())), None);
		assert_eq!(ClaimsPallet::vesting(&eth(&alice())), Some(get_bounded_vec((50, 10, 1))));
	});
}

#[test]
fn eth_signature_works() {
	new_test_ext().execute_with(|| {
		let data = Some(get_multi_address_account_id(42)).encode();
		let s = sig::<Test>(&alice(), &data, &[][..]);
		let sig = match s {
			MultiAddressSignature::EVM(s) => s,
			_ => panic!("should be evm signature"),
		};

		assert_eq!(
			ClaimsPallet::eth_recover(&sig, &to_ascii_hex(&data), &[][..]),
			Some(eth(&alice()))
		);
		assert!(
			ClaimsPallet::eth_recover(
				&sig,
				&Some(get_multi_address_account_id(43)).encode(),
				&[][..]
			) != Some(eth(&alice()))
		);
	});
}

#[test]
fn serde_works() {
	let x = EthereumAddress(hex!["0123456789abcdef0123456789abcdef01234567"]);
	let y = serde_json::to_string(&x).unwrap();
	assert_eq!(y, "\"0x0123456789abcdef0123456789abcdef01234567\"");
	let z: EthereumAddress = serde_json::from_str(&y).unwrap();
	assert_eq!(x, z);
}

#[test]
fn claiming_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&alice(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(
			Balances::free_balance(&get_multi_address_account_id(42).to_account_id_32()),
			100
		);
		assert_eq!(
			VestingPallet::vesting_balance(&get_multi_address_account_id(42).to_account_id_32()),
			Some(50)
		);
		assert_eq!(ClaimsPallet::total(), total_claims() - 100);
	});
}

#[test]
fn basic_claim_moving_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_noop!(
			ClaimsPallet::move_claim(
				RuntimeOrigin::signed(get_multi_address_account_id(1).to_account_id_32()),
				eth(&alice()),
				eth(&bob()),
			),
			BadOrigin
		);
		assert_ok!(ClaimsPallet::move_claim(
			RuntimeOrigin::signed(get_multi_address_account_id(6).to_account_id_32()),
			eth(&alice()),
			eth(&bob()),
		));
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&alice(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim
		);
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&bob(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(
			Balances::free_balance(&get_multi_address_account_id(42).to_account_id_32()),
			100
		);
		assert_eq!(
			VestingPallet::vesting_balance(&get_multi_address_account_id(42).to_account_id_32()),
			Some(50)
		);
		assert_eq!(ClaimsPallet::total(), total_claims() - 100);
	});
}

#[test]
fn claim_attest_moving_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ClaimsPallet::move_claim(
			RuntimeOrigin::signed(get_multi_address_account_id(6).to_account_id_32()),
			eth(&dave()),
			eth(&bob()),
		));
		let s = sig::<Test>(
			&bob(),
			&get_multi_address_account_id(42).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);
		assert_ok!(ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s,
			StatementKind::Regular.to_text().to_vec()
		));
		assert_eq!(
			Balances::free_balance(&get_multi_address_account_id(42).to_account_id_32()),
			200
		);
	});
}

#[test]
fn claiming_does_not_bypass_signing() {
	new_test_ext().execute_with(|| {
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&alice(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&dave(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::InvalidStatement,
		);
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&eve(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::InvalidStatement,
		);
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&frank(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
	});
}

#[test]
fn attest_claiming_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		let data = get_multi_address_account_id(42).to_account_id_32().encode();
		let s = sig::<Test>(&dave(), &data, StatementKind::Safe.to_text());
		let r = ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s.clone(),
			StatementKind::Safe.to_text().to_vec(),
		);
		assert_noop!(r, Error::<Test>::InvalidStatement);

		let r = ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s,
			StatementKind::Regular.to_text().to_vec(),
		);
		assert_noop!(r, Error::<Test>::SignerHasNoClaim);
		// ^^^ we use ecdsa_recover, so an invalid signature just results in a random signer id
		// being recovered, which realistically will never have a claim.

		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(42).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);
		assert_ok!(ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s,
			StatementKind::Regular.to_text().to_vec()
		));
		assert_eq!(
			Balances::free_balance(&get_multi_address_account_id(42).to_account_id_32()),
			200
		);
		assert_eq!(ClaimsPallet::total(), total_claims() - 200);

		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(42).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);
		let r = ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s,
			StatementKind::Regular.to_text().to_vec(),
		);
		assert_noop!(r, Error::<Test>::SignerHasNoClaim);
	});
}

#[test]
fn cannot_bypass_attest_claiming() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(42).to_account_id_32().encode(),
			&[],
		);
		let r = ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			s.clone(),
		);
		assert_noop!(r, Error::<Test>::InvalidStatement);
	});
}

#[test]
fn add_claim_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			ClaimsPallet::mint_claim(
				RuntimeOrigin::signed(get_multi_address_account_id(42).to_account_id_32()),
				eth(&bob()),
				200,
				None,
				None
			),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(69)),
				None,
				sig::<Test>(
					&bob(),
					&get_multi_address_account_id(69).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim,
		);
		assert_ok!(ClaimsPallet::mint_claim(RuntimeOrigin::root(), eth(&bob()), 200, None, None));
		assert_eq!(ClaimsPallet::total(), total_claims() + 200);
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(69)),
			None,
			sig::<Test>(
				&bob(),
				&get_multi_address_account_id(69).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(69).to_account_id_32()),
			200
		);
		assert_eq!(
			VestingPallet::vesting_balance(&get_multi_address_account_id(69).to_account_id_32()),
			None
		);
		assert_eq!(ClaimsPallet::total(), total_claims());
	});
}

#[test]
fn add_claim_with_vesting_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			ClaimsPallet::mint_claim(
				RuntimeOrigin::signed(get_multi_address_account_id(42).to_account_id_32()),
				eth(&bob()),
				200,
				Some(get_bounded_vec((50, 10, 1))),
				None
			),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(69)),
				None,
				sig::<Test>(
					&bob(),
					&get_multi_address_account_id(69).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim,
		);
		assert_ok!(ClaimsPallet::mint_claim(
			RuntimeOrigin::root(),
			eth(&bob()),
			200,
			Some(get_bounded_vec((50, 10, 1))),
			None
		));
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(69)),
			None,
			sig::<Test>(
				&bob(),
				&get_multi_address_account_id(69).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(69).to_account_id_32()),
			200
		);
		assert_eq!(
			VestingPallet::vesting_balance(&get_multi_address_account_id(69).to_account_id_32()),
			Some(50)
		);

		// Make sure we can not transfer the vested balance.
		assert_err!(
			<Balances as Currency<_>>::transfer(
				&get_multi_address_account_id(69).to_account_id_32(),
				&get_multi_address_account_id(80).to_account_id_32(),
				180,
				ExistenceRequirement::AllowDeath
			),
			DispatchError::Token(Frozen),
		);
	});
}

#[test]
fn add_claim_with_statement_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			ClaimsPallet::mint_claim(
				RuntimeOrigin::signed(get_multi_address_account_id(42).to_account_id_32()),
				eth(&bob()),
				200,
				None,
				Some(StatementKind::Regular)
			),
			sp_runtime::traits::BadOrigin,
		);
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		let signature = sig::<Test>(
			&bob(),
			&get_multi_address_account_id(69).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);

		assert_noop!(
			ClaimsPallet::claim_attest(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(69)),
				None,
				signature.clone(),
				StatementKind::Regular.to_text().to_vec()
			),
			Error::<Test>::SignerHasNoClaim
		);
		assert_ok!(ClaimsPallet::mint_claim(
			RuntimeOrigin::root(),
			eth(&bob()),
			200,
			None,
			Some(StatementKind::Regular)
		));
		assert_noop!(
			ClaimsPallet::claim_attest(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(69)),
				None,
				signature.clone(),
				vec![],
			),
			Error::<Test>::SignerHasNoClaim
		);
		assert_ok!(ClaimsPallet::claim_attest(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(69)),
			None,
			signature.clone(),
			StatementKind::Regular.to_text().to_vec()
		));
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(69).to_account_id_32()),
			200
		);
	});
}

#[test]
fn origin_signed_claiming_fail() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_err!(
			ClaimsPallet::claim(
				RuntimeOrigin::signed(get_multi_address_account_id(42).to_account_id_32()),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&alice(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			sp_runtime::traits::BadOrigin,
		);
	});
}

#[test]
fn double_claiming_doesnt_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&alice(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&alice(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim
		);
	});
}

#[test]
fn claiming_while_vested_works() {
	new_test_ext().execute_with(|| {
		CurrencyOf::<Test>::make_free_balance_be(
			&get_multi_address_account_id(69).to_account_id_32(),
			total_claims(),
		);
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(69).to_account_id_32()),
			total_claims()
		);
		// A user is already vested
		assert_ok!(<Test as Config>::VestingSchedule::add_vesting_schedule(
			&get_multi_address_account_id(69).to_account_id_32(),
			total_claims(),
			100,
			10
		));

		assert_eq!(
			Balances::total_balance(&get_multi_address_account_id(69).to_account_id_32()),
			total_claims()
		);

		assert_ok!(ClaimsPallet::mint_claim(
			RuntimeOrigin::root(),
			eth(&bob()),
			200,
			Some(get_bounded_vec((50, 10, 1))),
			None
		));
		// New total
		assert_eq!(ClaimsPallet::total(), total_claims() + 200);

		// They should be able to claim
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(69)),
			None,
			sig::<Test>(
				&bob(),
				&get_multi_address_account_id(69).to_account_id_32().encode(),
				&[][..]
			)
		));

		assert_eq!(
			Balances::free_balance(&get_multi_address_account_id(69).to_account_id_32()),
			total_claims() + 200
		);
	});
}

#[test]
fn non_sender_sig_doesnt_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&alice(),
					&get_multi_address_account_id(69).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim
		);
	});
}

#[test]
fn non_claimant_doesnt_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()), 0);
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&bob(),
					&get_multi_address_account_id(69).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim
		);
	});
}

#[test]
fn real_eth_sig_works() {
	new_test_ext().execute_with(|| {
        // "Pay RUSTs to the TEST account:2a00000000000000"
        let sig = hex!["444023e89b67e67c0562ed0305d252a5dd12b2af5ac51d6d3cb69a0b486bc4b3191401802dc29d26d586221f7256cd3329fe82174bdf659baea149a40e1c495d1c"];
        let sig = EcdsaSignature(sig);
        let who = 42u64.using_encoded(to_ascii_hex);
        let signer = ClaimsPallet::eth_recover(&sig, &who, &[][..]).unwrap();
        assert_eq!(signer.to_ethereum_address().unwrap().0, hex!["6d31165d5d932d571f3b44695653b46dcc327e84"]);
    });
}

#[test]
fn validate_unsigned_works() {
	use sp_runtime::traits::ValidateUnsigned;
	let source = sp_runtime::transaction_validity::TransactionSource::External;

	new_test_ext().execute_with(|| {
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(
				source,
				&ClaimsCall::claim {
					dest: Some(get_multi_address_account_id(1)),
					signer: None,
					signature: sig::<Test>(
						&alice(),
						&get_multi_address_account_id(1).to_account_id_32().encode(),
						&[][..]
					)
				}
			),
			Ok(ValidTransaction {
				priority: 100,
				requires: vec![],
				provides: vec![("claims", eth(&alice())).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		);
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(
				source,
				&ClaimsCall::claim {
					dest: Some(get_multi_address_account_id(0)),
					signer: None,
					signature: MultiAddressSignature::EVM(EcdsaSignature([0; 65]))
				}
			),
			InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
		);
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(
				source,
				&ClaimsCall::claim {
					dest: Some(get_multi_address_account_id(1)),
					signer: None,
					signature: sig::<Test>(
						&bob(),
						&get_multi_address_account_id(1).to_account_id_32().encode(),
						&[][..]
					)
				}
			),
			InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
		);
		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(1).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);
		let call = ClaimsCall::claim_attest {
			dest: Some(get_multi_address_account_id(1)),
			signer: None,
			signature: s,
			statement: StatementKind::Regular.to_text().to_vec(),
		};
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(source, &call),
			Ok(ValidTransaction {
				priority: 100,
				requires: vec![],
				provides: vec![("claims", eth(&dave())).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		);
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(
				source,
				&ClaimsCall::claim_attest {
					dest: Some(get_multi_address_account_id(1)),
					signer: None,
					signature: MultiAddressSignature::EVM(EcdsaSignature([0; 65])),
					statement: StatementKind::Regular.to_text().to_vec()
				}
			),
			InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
		);

		let s = sig::<Test>(
			&bob(),
			&get_multi_address_account_id(1).to_account_id_32().encode(),
			StatementKind::Regular.to_text(),
		);
		let call = ClaimsCall::claim_attest {
			dest: Some(get_multi_address_account_id(1)),
			signer: None,
			signature: s,
			statement: StatementKind::Regular.to_text().to_vec(),
		};
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(source, &call),
			InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
		);

		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(1).to_account_id_32().encode(),
			StatementKind::Safe.to_text(),
		);
		let call = ClaimsCall::claim_attest {
			dest: Some(get_multi_address_account_id(1)),
			signer: None,
			signature: s,
			statement: StatementKind::Regular.to_text().to_vec(),
		};
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(source, &call),
			InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
		);

		let s = sig::<Test>(
			&dave(),
			&get_multi_address_account_id(1).to_account_id_32().encode(),
			StatementKind::Safe.to_text(),
		);
		let call = ClaimsCall::claim_attest {
			dest: Some(get_multi_address_account_id(1)),
			signer: None,
			signature: s,
			statement: StatementKind::Safe.to_text().to_vec(),
		};
		assert_eq!(
			<Pallet<Test>>::validate_unsigned(source, &call),
			InvalidTransaction::Custom(ValidityError::InvalidStatement.into()).into(),
		);
	});
}

#[test]
fn test_unclaimed_returned_to_destination() {
	new_test_ext().execute_with(|| {
		let original_total_claims = Total::<Test>::get();
		let claim_of_alice = 100;
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			None,
			sig::<Test>(
				&alice(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(Total::<Test>::get(), original_total_claims - claim_of_alice);

		// force set the expiry config
		assert_ok!(ClaimsPallet::force_set_expiry_config(
			RuntimeOrigin::root(),
			5,
			get_multi_address_account_id(100)
		));

		// run to after expiry block
		run_to_block(7);
		assert_eq!(Total::<Test>::get(), 0);
		// the dest account should receive the remaining pot balance
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(100).to_account_id_32()),
			original_total_claims - claim_of_alice
		);

		// all further claims should fail with PotUnderflow error since the funds have been
		// emptied
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sig::<Test>(
					&frank(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::PotUnderflow
		);
	});
}

#[test]
fn test_claim_from_substrate_address_to_evm() {
	new_test_ext().execute_with(|| {
		let original_total_claims = Total::<Test>::get();
		let claim_of_sub_alice = 500;
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			Some(sr25519_utils::sub(&alice_sr25519())),
			sr25519_utils::sig::<Test>(
				&alice_sr25519(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(Total::<Test>::get(), original_total_claims - claim_of_sub_alice);
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()),
			500
		);
		// Claim bob without providing signer value (needed for sr25519 signatures)
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				None,
				sr25519_utils::sig::<Test>(
					&bob_sr25519(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::InvalidNativeAccount
		);
		// force set the expiry config
		assert_ok!(ClaimsPallet::force_set_expiry_config(
			RuntimeOrigin::root(),
			5,
			get_multi_address_account_id(100)
		));

		// run to after expiry block
		run_to_block(7);
		assert_eq!(Total::<Test>::get(), 0);
		// the dest account should receive the remaining pot balance
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(100).to_account_id_32()),
			original_total_claims - claim_of_sub_alice
		);

		// all further claims should fail with PotUnderflow error since the funds have been
		// emptied
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				Some(sr25519_utils::sub(&bob_sr25519())),
				sr25519_utils::sig::<Test>(
					&bob_sr25519(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::PotUnderflow
		);
	});
}

#[test]
fn test_claim_alice_signed_extrinsic() {
	new_test_ext().execute_with(|| {
		let original_total_claims = Total::<Test>::get();
		let claim_of_alice = 500;
		let alice_account_id_32 = AccountId32::from(alice_sr25519().public());
		assert_ok!(ClaimsPallet::claim_signed(
			RuntimeOrigin::signed(alice_account_id_32.clone()),
			None,
		));
		assert_eq!(Total::<Test>::get(), original_total_claims - claim_of_alice);
		assert_eq!(Balances::free_balance(alice_account_id_32), 500);
	});
}

#[test]
fn test_double_claim_fails_for_substrate_account() {
	new_test_ext().execute_with(|| {
		let original_total_claims = Total::<Test>::get();
		let claim_of_sub_alice = 500;
		assert_ok!(ClaimsPallet::claim(
			RuntimeOrigin::none(),
			Some(get_multi_address_account_id(42)),
			Some(sr25519_utils::sub(&alice_sr25519())),
			sr25519_utils::sig::<Test>(
				&alice_sr25519(),
				&get_multi_address_account_id(42).to_account_id_32().encode(),
				&[][..]
			)
		));
		assert_eq!(Total::<Test>::get(), original_total_claims - claim_of_sub_alice);
		assert_eq!(
			Balances::free_balance(get_multi_address_account_id(42).to_account_id_32()),
			500
		);
		// Claim for Alice again and expect the proper error
		assert_noop!(
			ClaimsPallet::claim(
				RuntimeOrigin::none(),
				Some(get_multi_address_account_id(42)),
				Some(sr25519_utils::sub(&alice_sr25519())),
				sr25519_utils::sig::<Test>(
					&alice_sr25519(),
					&get_multi_address_account_id(42).to_account_id_32().encode(),
					&[][..]
				)
			),
			Error::<Test>::SignerHasNoClaim
		);
	});
}
