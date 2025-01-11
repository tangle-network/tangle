// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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

#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::type_complexity)]

use jsonrpsee::{
	core::RpcResult,
	proc_macros::rpc,
	types::error::{ErrorObject, ErrorObjectOwned},
};
use parity_scale_codec::Codec;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	traits::{Block as BlockT, MaybeDisplay},
	DispatchError, Serialize,
};
use std::sync::Arc;
use tangle_primitives::Balance;

pub use pallet_rewards_rpc_runtime_api::RewardsApi as RewardsRuntimeApi;

/// RewardsClient RPC methods.
#[rpc(client, server)]
pub trait RewardsApi<BlockHash, AccountId, AssetId>
where
	AccountId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	AssetId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
{
	#[method(name = "rewards_queryUserRewards")]
	fn query_user_rewards(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<Balance>>;
}

/// A struct that implements the `RewardsApi`.
pub struct RewardsClient<C, M, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<(M, P)>,
}

impl<C, M, P> RewardsClient<C, M, P> {
	/// Create new `RewardsClient` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, AssetId> RewardsApiServer<<Block as BlockT>::Hash, AccountId, AssetId>
	for RewardsClient<C, AccountId, AssetId>
where
	Block: BlockT,
	AccountId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	AssetId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: RewardsRuntimeApi<Block, AccountId, AssetId, Balance>,
{
	fn query_user_rewards(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<Balance>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		api.query_user_rewards(at, account_id, asset_id).map_err(|e| {
			ErrorObject::owned(
				Error::RuntimeError.into(),
				"Error querying user rewards",
				Some(format!("{:?}", e)),
			)
		})
	}
}

/// Error type of this RPC api.
#[derive(Debug)]
pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}
