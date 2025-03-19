// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight, BoundedVec};
use sp_runtime::{DispatchError, DispatchResult};
use sp_std::{marker::PhantomData, vec::Vec};

/// Migration to pause newly added pallets: LST, MultiAssetDelegation, Services, and Rewards
pub struct PauseNewPallets<T>(PhantomData<T>);

impl<T: pallet_tx_pause::Config> OnRuntimeUpgrade for PauseNewPallets<T> {
	fn on_runtime_upgrade() -> Weight {
		// Keep track of reads and writes
		let mut reads = 0u64;
		let mut writes = 0u64;

		// List of pallets and their calls to pause
		let pallets_to_pause = [
			// LST pallet
			b"Lst".to_vec(),
			// MultiAssetDelegation pallet
			b"MultiAssetDelegation".to_vec(),
			// Services pallet
			b"Services".to_vec(),
			// Rewards pallet
			b"Rewards".to_vec(),
		];

		// Pause all pallets
		for pallet_name in pallets_to_pause.iter() {
			// Pause all calls in the pallet
			reads += 1;
			writes += 1;
			let _ = pause_pallet::<T>(pallet_name.clone());
		}

		log::info!("Paused newly added pallets: LST, MultiAssetDelegation, Services, and Rewards");

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// No need to capture state since we're just pausing pallets
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		// Verify that all pallets are paused
		ensure!(is_pallet_paused::<T>(b"Lst".to_vec()), "LST pallet should be paused");
		ensure!(
			is_pallet_paused::<T>(b"MultiAssetDelegation".to_vec()),
			"MultiAssetDelegation pallet should be paused"
		);
		ensure!(is_pallet_paused::<T>(b"Services".to_vec()), "Services pallet should be paused");
		ensure!(is_pallet_paused::<T>(b"Rewards".to_vec()), "Rewards pallet should be paused");

		Ok(())
	}
}

/// Pause all calls in a pallet
fn pause_pallet<T: pallet_tx_pause::Config>(pallet_name: Vec<u8>) -> DispatchResult {
	// Use wildcard * to pause all calls in the pallet
	let call_name = b"*".to_vec();

	// Convert Vec<u8> to BoundedVec<u8, T::MaxNameLen>
	let bounded_pallet_name = BoundedVec::<u8, T::MaxNameLen>::try_from(pallet_name)
		.map_err(|_| DispatchError::Other("Pallet name too long"))?;
	let bounded_call_name = BoundedVec::<u8, T::MaxNameLen>::try_from(call_name)
		.map_err(|_| DispatchError::Other("Call name too long"))?;

	let full_name = (bounded_pallet_name, bounded_call_name);

	<pallet_tx_pause::Pallet<T> as frame_support::traits::TransactionPause>::pause(full_name)
		.map_err(|_| DispatchError::Other("Failed to pause pallet"))
}

/// Check if a pallet is paused
#[cfg(feature = "try-runtime")]
fn is_pallet_paused<T: pallet_tx_pause::Config>(pallet_name: Vec<u8>) -> bool {
	let call_name = b"*".to_vec();

	// Convert Vec<u8> to BoundedVec<u8, T::MaxNameLen>
	let bounded_pallet_name = match BoundedVec::<u8, T::MaxNameLen>::try_from(pallet_name) {
		Ok(bounded) => bounded,
		Err(_) => return false,
	};
	let bounded_call_name = match BoundedVec::<u8, T::MaxNameLen>::try_from(call_name) {
		Ok(bounded) => bounded,
		Err(_) => return false,
	};

	let full_name = (bounded_pallet_name, bounded_call_name);
	pallet_tx_pause::PausedCalls::<T>::contains_key(&full_name)
}
