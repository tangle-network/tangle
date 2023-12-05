use super::*;
use crate::claimsPalletClaimsPallet::Call;
use frame_benchmarking::{account, benchmarks};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use secp_utils::*;
use sp_runtime::{traits::ValidateUnsigned, DispatchResult};

const SEED: u32 = 0;

const MAX_CLAIMS: u32 = 10_000;
const VALUE: u32 = 1_000_000;

fn create_claim<T: Config>(input: u32) -> DispatchResult {
	let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
	let eth_address = eth(&secret_key);
	let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
	super::Pallet::<T>::mint_claim(
		RawOrigin::Root.into(),
		eth_address,
		VALUE.into(),
		vesting,
		None,
	)?;
	Ok(())
}

fn create_claim_attest<T: Config>(input: u32) -> DispatchResult {
	let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
	let eth_address = eth(&secret_key);
	let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
	super::Pallet::<T>::mint_claim(
		RawOrigin::Root.into(),
		eth_address,
		VALUE.into(),
		vesting,
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
		let account: T::AccountId = account("user", c, SEED);
		let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
		let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
		super::Pallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, None)?;
		assert_eq!(ClaimsPallet::<T>::get(eth_address), Some(VALUE.into()));
		let source = sp_runtime::transaction_validity::TransactionSource::External;
		let call_enc = Call::<T>::claim {
			dest: account.clone(),
			ethereum_signature: signature.clone()
		}.encode();
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		super::Pallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
		call.dispatch_bypass_filter(RawOrigin::None.into())?;
	}
	verify {
		assert_eq!(ClaimsPallet::<T>::get(eth_address), None);
	}

	// Benchmark `mint_claim` when there already exists `c` claims in storage.
	mint_claim {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let eth_address = account("eth_address", 0, SEED);
		let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
		let statement = StatementKind::Regular;
	}: _(RawOrigin::Root, eth_address, VALUE.into(), vesting, Some(statement))
	verify {
		assert_eq!(ClaimsPallet::<T>::get(eth_address), Some(VALUE.into()));
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
		let account: T::AccountId = account("user", c, SEED);
		let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
		let statement = StatementKind::Regular;
		let signature = sig::<T>(&secret_key, &account.encode(), statement.to_text());
		super::Pallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, Some(statement))?;
		assert_eq!(ClaimsPallet::<T>::get(eth_address), Some(VALUE.into()));
		let call_enc = Call::<T>::claim_attest {
			dest: account.clone(),
			ethereum_signature: signature.clone(),
			statement: StatementKind::Regular.to_text().to_vec()
		}.encode();
		let source = sp_runtime::transaction_validity::TransactionSource::External;
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		super::Pallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
		call.dispatch_bypass_filter(RawOrigin::None.into())?;
	}
	verify {
		assert_eq!(ClaimsPallet::<T>::get(eth_address), None);
	}

	// Benchmark `attest` including prevalidate logic.
	attest {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let attest_c = u32::MAX - c;
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let account: T::AccountId = account("user", c, SEED);
		let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
		let statement = StatementKind::Regular;
		let signature = sig::<T>(&secret_key, &account.encode(), statement.to_text());
		super::Pallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, Some(statement))?;
		PreclaimsPalletClaimsPallet::<T>::insert(&account, eth_address);
		assert_eq!(ClaimsPallet::<T>::get(eth_address), Some(VALUE.into()));

		let call = super::Call::<T>::attest { statement: StatementKind::Regular.to_text().to_vec() };
		// We have to copy the validate statement here because of trait issues... :(
		let validate = |who: &T::AccountId, call: &super::Call<T>| -> DispatchResult {
			if let Call::attest{ statement: attested_statement } = call {
				let signer = PreclaimsPalletClaimsPallet::<T>::get(who).ok_or("signer has no claim")?;
				if let Some(s) = Signing::<T>::get(signer) {
					ensure!(&attested_statement[..] == s.to_text(), "invalid statement");
				}
			}
			Ok(())
		};
		let call_enc = call.encode();
	}: {
		let call = <Call<T> as Decode>::decode(&mut &*call_enc)
			.expect("call is encoded above, encoding must be correct");
		validate(&account, &call)?;
		call.dispatch_bypass_filter(RawOrigin::Signed(account).into())?;
	}
	verify {
		assert_eq!(ClaimsPallet::<T>::get(eth_address), None);
	}

	move_claim {
		let c = MAX_CLAIMS;

		for i in 0 .. c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let attest_c = u32::MAX - c;
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
		let eth_address = eth(&secret_key);

		let new_secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&(u32::MAX/2).encode())).unwrap();
		let new_eth_address = eth(&new_secret_key);

		let account: T::AccountId = account("user", c, SEED);
		PreclaimsPalletClaimsPallet::<T>::insert(&account, eth_address);

		assert!(ClaimsPallet::<T>::contains_key(eth_address));
		assert!(!ClaimsPallet::<T>::contains_key(new_eth_address));
	}: _(RawOrigin::Root, eth_address, new_eth_address, Some(account))
	verify {
		assert!(!ClaimsPallet::<T>::contains_key(eth_address));
		assert!(ClaimsPallet::<T>::contains_key(new_eth_address));
	}

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
		let account: T::AccountId = account("user", i, SEED);
		let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
		let data = account.using_encoded(to_ascii_hex);
		let extra = StatementKind::default().to_text();
	}: {
		for _ in 0 .. i {
			assert!(super::Pallet::<T>::eth_recover(&signature, &data, extra).is_some());
		}
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::claimsPalletClaimsPallet::tests::new_test_ext(),
		crate::claimsPalletClaimsPallet::tests::Test,
	);
}
