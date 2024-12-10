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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::RuntimeDebug;

/// Represents an asset type that can be either a custom asset or an ERC20 token.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Eq)]
pub enum Asset<AssetId> {
    /// Use the specified AssetId.
    #[codec(index = 0)]
    Custom(AssetId),

    /// Use an ERC20-like token with the specified contract address.
    #[codec(index = 1)]
    Erc20(H160),
}

impl<AssetId: Default> Default for Asset<AssetId> {
    fn default() -> Self {
        Asset::Custom(AssetId::default())
    }
}

impl<AssetId: PartialEq> Asset<AssetId> {
    /// Returns true if the asset is an ERC20 token.
    pub fn is_erc20(&self) -> bool {
        matches!(self, Asset::Erc20(_))
    }

    /// Returns true if the asset is a custom asset.
    pub fn is_custom(&self) -> bool {
        matches!(self, Asset::Custom(_))
    }

    /// Returns the ERC20 address if this is an ERC20 token.
    pub fn erc20_address(&self) -> Option<H160> {
        match self {
            Asset::Erc20(address) => Some(*address),
            _ => None,
        }
    }

    /// Returns the custom asset ID if this is a custom asset.
    pub fn custom_id(&self) -> Option<&AssetId> {
        match self {
            Asset::Custom(id) => Some(id),
            _ => None,
        }
    }
}
