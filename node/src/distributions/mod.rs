// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use fp_evm::GenesisAccount;
use sp_core::H160;

pub mod develop;
pub mod mainnet;
pub mod testnet;

pub fn combine_distributions(
	distributions: Vec<Vec<(H160, GenesisAccount)>>,
) -> Vec<(H160, GenesisAccount)> {
	let mut combined = Vec::new();
	for distribution in distributions {
		for (address, account) in distribution {
			combined.push((address, account));
		}
	}
	combined
}
