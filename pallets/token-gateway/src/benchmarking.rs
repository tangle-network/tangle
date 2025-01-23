#![cfg(feature = "runtime-benchmarks")]

use crate::{types::*, *};
use frame_benchmarking::v2::*;
use frame_support::{
	traits::{fungible, fungibles, Currency},
	BoundedVec,
};
use frame_system::RawOrigin;
use ismp::host::StateMachine;
use scale_info::prelude::collections::BTreeMap;
use sp_runtime::AccountId32;
use token_gateway_primitives::{GatewayAssetRegistration, GatewayAssetUpdate};

#[benchmarks(
    where
    <<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance: From<u128>,
    <T as frame_system::Config>::AccountId: From<[u8; 32]>,
    u128: From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
    T::Balance: From<u128>,
    <T as pallet_ismp::Config>::Balance: From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
    <<T as Config>::Assets as fungibles::Inspect<T::AccountId>>::Balance: From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
    <<T as Config>::Assets as fungibles::Inspect<T::AccountId>>::Balance: From<u128>,
    [u8; 32]: From<<T as frame_system::Config>::AccountId>,
    <T as frame_system::Config>::RuntimeOrigin: From<frame_system::RawOrigin<AccountId32>>,
)]
mod benches {
	use super::*;

	#[benchmark]
	fn create_erc6160_asset(x: Linear<1, 100>) -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let asset_details = GatewayAssetRegistration {
			name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
			symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
			chains: vec![StateMachine::Evm(100)],
			minimum_balance: Some(10),
		};

		let mut precision = BTreeMap::new();
		for i in 0..x {
			precision.insert(StateMachine::Evm(i as u32), 18);
		}

		let asset = AssetRegistration {
			local_id: T::NativeAssetId::get(),
			reg: asset_details,
			native: true,
			precision,
		};

		<T::Currency as fungible::Mutate<T::AccountId>>::set_balance(&account, u128::MAX.into());

		#[extrinsic_call]
		_(RawOrigin::Signed(account), asset);

		Ok(())
	}

	#[benchmark]
	fn teleport() -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let asset_id = T::NativeAssetId::get();

		Pallet::<T>::create_erc6160_asset(
			RawOrigin::Signed(account.clone()).into(),
			AssetRegistration {
				local_id: asset_id.clone(),
				reg: GatewayAssetRegistration {
					name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
					symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
					chains: vec![StateMachine::Evm(100)],
					minimum_balance: None,
				},
				native: true,
				precision: vec![(StateMachine::Evm(100), 18)].into_iter().collect(),
			},
		)?;

		let _ = T::NativeCurrency::deposit_creating(&account, u128::MAX.into());
		let teleport_params = TeleportParams {
			asset_id,
			destination: StateMachine::Evm(100),
			recepient: H256::from([1u8; 32]),
			amount: 10_000_000_000_000u128.into(),
			timeout: 0,
			token_gateway: vec![1, 2, 3, 4, 5],
			relayer_fee: 0u128.into(),
			call_data: None,
			redeem: false,
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(account), teleport_params);
		Ok(())
	}

	#[benchmark]
	fn set_token_gateway_addresses(x: Linear<1, 100>) -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let mut addresses = BTreeMap::new();
		for i in 0..x {
			let addr = i.to_string().as_bytes().to_vec();
			addresses.insert(StateMachine::Evm(100), addr);
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(account), addresses);
		Ok(())
	}

	#[benchmark]
	fn update_erc6160_asset() -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let local_id = T::NativeAssetId::get();

		Pallet::<T>::create_erc6160_asset(
			RawOrigin::Signed(account.clone()).into(),
			AssetRegistration {
				local_id,
				reg: GatewayAssetRegistration {
					name: BoundedVec::try_from(b"Spectre".to_vec()).unwrap(),
					symbol: BoundedVec::try_from(b"SPC".to_vec()).unwrap(),
					chains: vec![StateMachine::Evm(100)],
					minimum_balance: None,
				},
				native: true,
				precision: Default::default(),
			},
		)?;

		let asset_update = GatewayAssetUpdate {
			asset_id: sp_io::hashing::keccak_256(b"SPC".as_ref()).into(),
			add_chains: BoundedVec::try_from(vec![StateMachine::Evm(200)]).unwrap(),
			remove_chains: BoundedVec::try_from(Vec::new()).unwrap(),
			new_admins: BoundedVec::try_from(Vec::new()).unwrap(),
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(account), asset_update);
		Ok(())
	}

	#[benchmark]
	fn update_asset_precision(x: Linear<1, 100>) -> Result<(), BenchmarkError> {
		let account: T::AccountId = whitelisted_caller();

		let mut precisions = BTreeMap::new();
		for i in 0..x {
			precisions.insert(StateMachine::Evm(i as u32), 18);
		}

		let update = PrecisionUpdate { asset_id: T::NativeAssetId::get(), precisions };

		#[extrinsic_call]
		_(RawOrigin::Signed(account), update);
		Ok(())
	}
}
