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
pub use pallet_jobs_rpc_runtime_api::JobsApi as JobsRuntimeApi;
use parity_scale_codec::Codec;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	traits::{Block as BlockT, Get, MaybeDisplay},
	Serialize,
};
use std::sync::Arc;
use tangle_primitives::{
	jobs::{JobId, PhaseResult, RpcResponseJobsData},
	roles::RoleType,
};

type BlockNumberOf<Block> =
	<<Block as sp_runtime::traits::HeaderProvider>::HeaderT as sp_runtime::traits::Header>::Number;

/// JobsClient RPC methods.
#[rpc(client, server)]
pub trait JobsApi<
	BlockHash,
	AccountId,
	BlockNumber,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
	MaxSignatureLen: Get<u32>,
	MaxProofLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
>
{
	#[method(name = "jobs_queryJobsByValidator")]
	fn query_jobs_by_validator(
		&self,
		at: Option<BlockHash>,
		validator: AccountId,
	) -> RpcResult<
		Option<
			Vec<
				RpcResponseJobsData<
					AccountId,
					BlockNumber,
					MaxParticipants,
					MaxSubmissionLen,
					MaxAdditionalParamsLen,
				>,
			>,
		>,
	>;

	#[method(name = "jobs_queryJobById")]
	fn query_job_by_id(
		&self,
		at: Option<BlockHash>,
		role_type: RoleType,
		job_id: JobId,
	) -> RpcResult<
		Option<
			RpcResponseJobsData<
				AccountId,
				BlockNumber,
				MaxParticipants,
				MaxSubmissionLen,
				MaxAdditionalParamsLen,
			>,
		>,
	>;

	#[method(name = "jobs_queryPhaseOneById")]
	fn query_job_result(
		&self,
		at: Option<BlockHash>,
		role_type: RoleType,
		job_id: JobId,
	) -> RpcResult<
		Option<
			PhaseResult<
				AccountId,
				BlockNumber,
				MaxParticipants,
				MaxKeyLen,
				MaxDataLen,
				MaxSignatureLen,
				MaxSubmissionLen,
				MaxProofLen,
				MaxAdditionalParamsLen,
			>,
		>,
	>;

	#[method(name = "jobs_queryNextJobId")]
	fn query_next_job_id(&self, at: Option<BlockHash>) -> RpcResult<JobId>;

	#[method(name = "jobs_queryRestakerRoleKey")]
	fn query_restaker_role_key(
		&self,
		at: Option<BlockHash>,
		address: AccountId,
	) -> RpcResult<Option<Vec<u8>>>;
}

/// A struct that implements the `JobsApi`.
pub struct JobsClient<C, M, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<(M, P)>,
}

impl<C, M, P> JobsClient<C, M, P> {
	/// Create new `JobsClient` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<
		C,
		Block,
		AccountId,
		MaxParticipants,
		MaxSubmissionLen,
		MaxKeyLen,
		MaxDataLen,
		MaxSignatureLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	>
	JobsApiServer<
		<Block as BlockT>::Hash,
		AccountId,
		BlockNumberOf<Block>,
		MaxParticipants,
		MaxSubmissionLen,
		MaxKeyLen,
		MaxDataLen,
		MaxSignatureLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	> for JobsClient<C, Block, AccountId>
where
	Block: BlockT,
	AccountId: Codec + MaybeDisplay + Send + Sync + 'static + Serialize,
	MaxParticipants: Codec + Serialize + Get<u32> + Clone,
	MaxSubmissionLen: Codec + Serialize + Get<u32>,
	MaxKeyLen: Codec + Serialize + Get<u32>,
	MaxDataLen: Codec + Serialize + Get<u32>,
	MaxSignatureLen: Codec + Serialize + Get<u32>,
	MaxProofLen: Codec + Serialize + Get<u32>,
	MaxAdditionalParamsLen: Codec + Serialize + Get<u32>,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: JobsRuntimeApi<
		Block,
		AccountId,
		MaxParticipants,
		MaxSubmissionLen,
		MaxKeyLen,
		MaxDataLen,
		MaxSignatureLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	>,
{
	fn query_jobs_by_validator(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		validator: AccountId,
	) -> RpcResult<
		Option<
			Vec<
				RpcResponseJobsData<
					AccountId,
					BlockNumberOf<Block>,
					MaxParticipants,
					MaxSubmissionLen,
					MaxAdditionalParamsLen,
				>,
			>,
		>,
	> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_jobs_by_validator(at, validator) {
			Ok(res) => Ok(res),
			Err(e) => Err(runtime_error_into_rpc_err(e)),
		}
	}

	fn query_job_by_id(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		role_type: RoleType,
		job_id: JobId,
	) -> RpcResult<
		Option<
			RpcResponseJobsData<
				AccountId,
				BlockNumberOf<Block>,
				MaxParticipants,
				MaxSubmissionLen,
				MaxAdditionalParamsLen,
			>,
		>,
	> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		match api.query_job_by_id(at, role_type, job_id) {
			Ok(res) => Ok(res),
			Err(e) => Err(runtime_error_into_rpc_err(e)),
		}
	}

	fn query_job_result(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		role_type: RoleType,
		job_id: JobId,
	) -> RpcResult<
		Option<
			PhaseResult<
				AccountId,
				BlockNumberOf<Block>,
				MaxParticipants,
				MaxKeyLen,
				MaxDataLen,
				MaxSignatureLen,
				MaxSubmissionLen,
				MaxProofLen,
				MaxAdditionalParamsLen,
			>,
		>,
	> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_job_result(at, role_type, job_id) {
			Ok(res) => Ok(res),
			Err(e) => Err(runtime_error_into_rpc_err(e)),
		}
	}

	fn query_next_job_id(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<JobId> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_next_job_id(at) {
			Ok(res) => Ok(res),
			Err(e) => Err(runtime_error_into_rpc_err(e)),
		}
	}

	fn query_restaker_role_key(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		address: AccountId,
	) -> RpcResult<Option<Vec<u8>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		match api.query_restaker_role_key(at, address) {
			Ok(res) => Ok(res),
			Err(e) => Err(runtime_error_into_rpc_err(e)),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> ErrorObjectOwned {
	ErrorObject::owned(RUNTIME_ERROR, "Runtime error", Some(format!("{:?}", err)))
}

const RUNTIME_ERROR: i32 = 1;
