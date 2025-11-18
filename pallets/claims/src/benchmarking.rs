use super::*;
use crate::{Call, Config, Pallet as ClaimsPallet};
use frame_benchmarking::{BenchmarkError, account, v2::*};
use frame_support::{BoundedVec, traits::UnfilteredDispatchable};
use frame_system::{RawOrigin, pallet_prelude::*};
use secp_utils::*;
use sp_runtime::{AccountId32, DispatchResult, traits::ValidateUnsigned};

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

#[benchmarks]
mod benchmarks {
	use super::*;

	// Benchmark `claim` including `validate_unsigned` logic.
	#[benchmark]
	fn claim() -> Result<(), BenchmarkError> {
		let c = MAX_CLAIMS;

		for _ in 0..c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let account: AccountId32 = account("user", c, SEED);
		let vesting = Some(get_bounded_vec::<T>());
		let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
		ClaimsPallet::<T>::mint_claim(
			RawOrigin::Root.into(),
			eth_address.clone(),
			VALUE.into(),
			vesting.into(),
			None,
		)?;
		assert_eq!(Claims::<T>::get(eth_address.clone()), Some(VALUE.into()));
		let source = sp_runtime::transaction_validity::TransactionSource::External;
		let call_enc = Call::<T>::claim {
			dest: Some(MultiAddress::Native(account.clone().into())),
			signer: None,
			signature: signature.clone(),
		}
		.encode();

		#[block]
		{
			let call = <Call<T> as Decode>::decode(&mut &*call_enc)
				.expect("call is encoded above, encoding must be correct");
			ClaimsPallet::<T>::validate_unsigned(source, &call)
				.map_err(|e| -> &'static str { e.into() })?;
			call.dispatch_bypass_filter(RawOrigin::None.into())?;
		}

		// Verify
		assert_eq!(Claims::<T>::get(eth_address), None);

		Ok(())
	}

	// Benchmark `mint_claim` when there already exists `c` claims in storage.
	#[benchmark]
	fn mint_claim() -> Result<(), BenchmarkError> {
		let c = MAX_CLAIMS;

		for _ in 0..c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let vesting = Some(get_bounded_vec::<T>());
		let statement = StatementKind::Regular;

		#[extrinsic_call]
		mint_claim(RawOrigin::Root, eth_address.clone(), VALUE.into(), vesting, Some(statement));

		// Verify
		assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));

		Ok(())
	}

	// Benchmark `claim_attest` including `validate_unsigned` logic.
	#[benchmark]
	fn claim_attest() -> Result<(), BenchmarkError> {
		let c = MAX_CLAIMS;

		for _ in 0..c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		// Create signature
		let attest_c = u32::MAX - c;
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let account: AccountId32 = account("user", c, SEED);
		let vesting = Some(get_bounded_vec::<T>());
		let statement = StatementKind::Regular;
		let signature = sig::<T>(&secret_key, &account.encode(), statement.to_text());
		ClaimsPallet::<T>::mint_claim(
			RawOrigin::Root.into(),
			eth_address.clone(),
			VALUE.into(),
			vesting,
			Some(statement),
		)?;
		assert_eq!(Claims::<T>::get(eth_address.clone()), Some(VALUE.into()));
		let call_enc = Call::<T>::claim_attest {
			dest: Some(MultiAddress::Native(account.clone())),
			signer: None,
			signature: signature.clone(),
			statement: StatementKind::Regular.to_text().to_vec(),
		}
		.encode();
		let source = sp_runtime::transaction_validity::TransactionSource::External;

		#[block]
		{
			let call = <Call<T> as Decode>::decode(&mut &*call_enc)
				.expect("call is encoded above, encoding must be correct");
			ClaimsPallet::<T>::validate_unsigned(source, &call)
				.map_err(|e| -> &'static str { e.into() })?;
			call.dispatch_bypass_filter(RawOrigin::None.into())?;
		}

		// Verify
		assert_eq!(Claims::<T>::get(eth_address), None);

		Ok(())
	}

	#[benchmark]
	fn move_claim() -> Result<(), BenchmarkError> {
		let c = MAX_CLAIMS;

		for _ in 0..c / 2 {
			create_claim::<T>(c)?;
			create_claim_attest::<T>(u32::MAX - c)?;
		}

		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
		let eth_address = eth(&secret_key);

		let new_secret_key =
			libsecp256k1::SecretKey::parse(&keccak_256(&(u32::MAX / 2).encode())).unwrap();
		let new_eth_address = eth(&new_secret_key);

		assert!(Claims::<T>::contains_key(&eth_address));
		assert!(!Claims::<T>::contains_key(&new_eth_address));

		#[extrinsic_call]
		move_claim(RawOrigin::Root, eth_address.clone(), new_eth_address.clone());

		// Verify
		assert!(!Claims::<T>::contains_key(eth_address));
		assert!(Claims::<T>::contains_key(new_eth_address));

		Ok(())
	}

	// Benchmark `force_set_expiry_config` logic.
	#[benchmark]
	fn force_set_expiry_config() -> Result<(), BenchmarkError> {
		let new_expiry = 1000u32;
		let account: AccountId32 = account("user", 0, SEED);

		#[extrinsic_call]
		force_set_expiry_config(RawOrigin::Root, new_expiry.into(), MultiAddress::Native(account));

		Ok(())
	}

	// Benchmark the time it takes to do `repeat` number of keccak256 hashes
	#[benchmark]
	fn keccak256(i: Linear<0, 10_000>) -> Result<(), BenchmarkError> {
		let bytes = (i).encode();
		#[block]
		{
			for _ in 0..i {
				let _hash = keccak_256(&bytes);
			}
		}
		Ok(())
	}

	// Benchmark the time it takes to do `repeat` number of `eth_recover`
	#[benchmark]
	fn eth_recover(i: Linear<0, 1_000>) -> Result<(), BenchmarkError> {
		// Create signature
		let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&i.encode())).unwrap();
		let eth_address = eth(&secret_key);
		let signature = sig::<T>(&secret_key, &eth_address.encode(), &[][..]);
		let signature = match signature {
			MultiAddressSignature::EVM(s) => s,
			_ => panic!("should be evm signature"),
		};
		let extra = StatementKind::default().to_text();
		#[block]
		{
			for _ in 0..i {
				assert!(
					ClaimsPallet::<T>::eth_recover(
						&signature,
						&to_ascii_hex(&eth_address.encode()),
						extra
					)
					.is_some()
				);
			}
		}
		Ok(())
	}

	impl_benchmark_test_suite!(ClaimsPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
