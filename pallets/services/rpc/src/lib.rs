// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::type_complexity)]
use jsonrpsee::{
	core::RpcResult,
	proc_macros::rpc,
	types::error::{ErrorObject, ErrorObjectOwned},
};
pub use pallet_services_rpc_runtime_api::ServicesApi as ServicesRuntimeApi;
use parity_scale_codec::Codec;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	traits::{Block as BlockT, MaybeDisplay},
	DispatchError, Serialize,
};
use std::sync::Arc;
use tangle_primitives::services::{Constraints, RpcServicesWithBlueprint};

type BlockNumberOf<Block> =
	<<Block as sp_runtime::traits::HeaderProvider>::HeaderT as sp_runtime::traits::Header>::Number;

/// ServicesClient RPC methods.
#[rpc(client, server)]
pub trait ServicesApi<BlockHash, X, AccountId, BlockNumber, AssetId>
where
	X: Constraints,
	AccountId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	BlockNumber: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	AssetId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
{
	#[method(name = "services_queryServicesWithBlueprintsByOperator")]
	fn query_services_with_blueprints_by_operator(
		&self,
		operator: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<Vec<RpcServicesWithBlueprint<X, AccountId, BlockNumber, AssetId>>>;
}

/// A struct that implements the `ServicesApi`.
pub struct ServicesClient<C, M, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<(M, P)>,
}

impl<C, M, P> ServicesClient<C, M, P> {
	/// Create new `JobsClient` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, X, Block, AccountId, AssetId>
	ServicesApiServer<<Block as BlockT>::Hash, X, AccountId, BlockNumberOf<Block>, AssetId>
	for ServicesClient<C, Block, AccountId>
where
	Block: BlockT,
	AccountId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	AssetId: Codec + MaybeDisplay + core::fmt::Debug + Send + Sync + 'static + Serialize,
	X: Constraints,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: ServicesRuntimeApi<Block, X, AccountId, AssetId>,
{
	fn query_services_with_blueprints_by_operator(
		&self,
		operator: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<RpcServicesWithBlueprint<X, AccountId, BlockNumberOf<Block>, AssetId>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_services_with_blueprints_by_operator(at, operator) {
			Ok(Ok(res)) => Ok(res),
			Ok(Err(e)) => Err(custom_error_into_rpc_err(Error::CustomDispatchError(e))),
			Err(e) => Err(custom_error_into_rpc_err(Error::RuntimeError(e))),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError(ApiError),
	/// Custom pallet error.
	CustomDispatchError(DispatchError),
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError(_) => 1,
			Error::DecodeError => 2,
			Error::CustomDispatchError(_) => 3,
		}
	}
}

fn custom_error_into_rpc_err(err: Error) -> ErrorObjectOwned {
	match err {
		Error::RuntimeError(e) => {
			ErrorObject::owned(RUNTIME_ERROR, "Runtime error", Some(format!("{e}")))
		},
		Error::DecodeError => {
			ErrorObject::owned(2, "Decode error", Some("Transaction was not decodable"))
		},
		Error::CustomDispatchError(msg) => ErrorObject::owned(3, "Dispatch error", Some(msg)),
	}
}

const RUNTIME_ERROR: i32 = 1;
