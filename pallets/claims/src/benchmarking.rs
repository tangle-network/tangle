use super::*;
use crate::{Call, Config, Pallet as ClaimsPallet};
use frame_benchmarking::{account, benchmarks};
use frame_support::{traits::UnfilteredDispatchable, BoundedVec};
use frame_system::{pallet_prelude::*, RawOrigin};
use secp_utils::*;
use sp_runtime::{traits::ValidateUnsigned, AccountId32, DispatchResult};

const SEED: u32 = 0;

const MAX_CLAIMS: u32 = 10_000;
const VALUE: u32 = 1_000_000;

type VestingScheduleType<T> = (BalanceOf<T>, BalanceOf<T>, BlockNumberFor<T>);

pub fn get_bounded_vec<T: Config>() -> BoundedVec<VestingScheduleType<T>, T::MaxVestingSchedules> {
	BoundedVec::try_from(vec![(100_000u32.into(), 1_000u32.into(), 100u32.into())])
		.unwrap()
		.into()
}

fn create_claim<T: Config>(input: u32) -> DispatchResult {
	let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
	let eth_address = eth(&secret_key);
	let vesting = Some(get_bounded_vec::<T>());
	ClaimsPallet::<T>::mint_claim(
		RawOrigin::Root.into(),
		eth_address,
		VALUE.into(),
		vesting.into(),
		None,
	)?;
	Ok(())
}

fn create_claim_attest<T: Config>(input: u32) -> DispatchResult {
	let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
	let eth_address = eth(&secret_key);
	let vesting = Some(get_bounded_vec::<T>());
	ClaimsPallet::<T>::mint_claim(
		RawOrigin::Root.into(),
		eth_address,
		VALUE.into(),
		vesting.into(),
		Some(Default::default()),
	)?;
	Ok(())
}

benchmarks! {
	// Benchmark `claim` including `validate_unsigned` logic.
	claim {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let account: AccountId32 = account("user", c, SEED);
		let vesting =  Some(get_bounded_vec::<T>());
		let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
		ClaimsPallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address.clone(), VALUE.into(),vesting.into(), None)?;
		assert_eq!(Claims::<T>::get(eth_address.clone()), Some(VALUE.into()));
		let source = sp_runtime::transaction_validity::TransactionSource::External;
		let call_enc = Call::<T>::claim {
			dest: Some(MultiAddress::Native(account.clone().into())),
			signer: None,
			signature: signature.clone()
		}.encode();
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		ClaimsPallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
		call.dispatch_bypass_filter(RawOrigin::None.into())?;
	}
	verify {
		assert_eq!(Claims::<T>::get(eth_address), None);
	}

	// Benchmark `mint_claim` when there already exists `c` claims in storage.
	mint_claim {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let vesting =  Some(get_bounded_vec::<T>());
		let statement = StatementKind::Regular;
	}: _(RawOrigin::Root, eth_address.clone(), VALUE.into(), vesting, Some(statement))
	verify {
		assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
	}

	// Benchmark `claim_attest` including `validate_unsigned` logic.
	claim_attest {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		// Crate signature
		let attest_c = u32::MAX - c;
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let account: AccountId32 = account("user", c, SEED);
		let vesting =  Some(get_bounded_vec::<T>());
		let statement = StatementKind::Regular;
		let signature = sig::<T>(&secret_key, &account.encode(), statement.to_text());
		ClaimsPallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address.clone(), VALUE.into(), vesting, Some(statement))?;
		assert_eq!(Claims::<T>::get(eth_address.clone()), Some(VALUE.into()));
		let call_enc = Call::<T>::claim_attest {
			dest:Some(MultiAddress::Native(account.clone())),
			signer: None,
			signature: signature.clone(),
			statement: StatementKind::Regular.to_text().to_vec()
		}.encode();
		let source = sp_runtime::transaction_validity::TransactionSource::External;
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		ClaimsPallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
		call.dispatch_bypass_filter(RawOrigin::None.into())?;
	}
	verify {
		assert_eq!(Claims::<T>::get(eth_address), None);
	}

	move_claim {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);

		let new_secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&(u32::MAX/2).encode())).unwrap();
		let new_eth_address = eth(&new_secret_key);

		assert!(Claims::<T>::contains_key(&eth_address));
		assert!(!Claims::<T>::contains_key(&new_eth_address));
	}: _(RawOrigin::Root, eth_address.clone(), new_eth_address.clone())
	verify {
		assert!(!Claims::<T>::contains_key(eth_address));
		assert!(Claims::<T>::contains_key(new_eth_address));
	}

	// Benchmark `force_set_expiry_config` logic.
	force_set_expiry_config {
		let new_expiry = 1000u32;
		let account: AccountId32 = account("user", 0, SEED);

	}: _(RawOrigin::Root, new_expiry.into(), MultiAddress::Native(account) )

	// Benchmark the time it takes to do `repeat` number of keccak256 hashes
	#[extra]
	keccak256 {
		let i in 0 .. 10_000;
		let bytes = (i).encode();
	}: {
		for index in 0 .. i {
			let _hash = keccak_256(&bytes);
		}
	}

	// Benchmark the time it takes to do `repeat` number of `eth_recover`
	#[extra]
	eth_recover {
		let i in 0 .. 1_000;
		// Crate signature
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&i.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let signature = sig::<T>(&secret_key, &eth_address.encode(), &[][..]);
		let signature = match signature {
			MultiAddressSignature::EVM(s) => s,
			_ => panic!("should be evm signature"),
		};
		let extra = StatementKind::default().to_text();
	}: {
		for _ in 0 .. i {
			assert!(ClaimsPallet::<T>::eth_recover(&signature, &to_ascii_hex(&eth_address.encode()), extra).is_some());
		}
	}

	impl_benchmark_test_suite!(ClaimsPallet, crate::mock::new_test_ext(), crate::mock::Test,);

}
