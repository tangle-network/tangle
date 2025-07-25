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
	Serialize,
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
		asset_id: tangle_primitives::services::Asset<AssetId>,
		at: Option<BlockHash>,
	) -> RpcResult<Balance>;
}

/// Provides RPC methods to query a dispatchable's class, weight and fee.
pub struct RewardsClient<C, P> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> RewardsClient<C, P> {
	/// Creates a new instance of the RewardsClient Rpc helper.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, AssetId> RewardsApiServer<<Block as BlockT>::Hash, AccountId, AssetId>
	for RewardsClient<C, Block>
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
		asset_id: tangle_primitives::services::Asset<AssetId>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Balance> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_user_rewards(at, account_id, asset_id) {
			Ok(Ok(res)) => Ok(res),
			Ok(Err(e)) => Err(map_err(format!("{:?}", e), "Unable to query user rewards")),
			Err(e) => Err(map_err(format!("{:?}", e), "Unable to query user rewards")),
		}
	}
}

fn map_err(error: impl ToString, desc: &'static str) -> ErrorObjectOwned {
	ErrorObject::owned(Error::RuntimeError.into(), desc, Some(error.to_string()))
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
