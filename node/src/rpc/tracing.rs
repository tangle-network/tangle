// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use std::time::Duration;

use crate::{
	eth::{EthApi, FrontierBackend, RpcConfig},
	service::{FullBackend, FullClient},
};

use super::*;

use fc_storage::StorageOverride;
use rpc_debug::{DebugHandler, DebugRequester};
use rpc_trace::{CacheRequester as TraceFilterCacheRequester, CacheTask};
use sc_service::TaskManager;
use substrate_prometheus_endpoint::Registry as PrometheusRegistry;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct RpcRequesters {
	pub debug: Option<DebugRequester>,
	pub trace: Option<TraceFilterCacheRequester>,
}

// Spawn the tasks that are required to run a tracing node.
pub fn spawn_tracing_tasks(
	task_manager: &TaskManager,
	client: Arc<FullClient>,
	backend: Arc<FullBackend>,
	frontier_backend: Arc<FrontierBackend>,
	overrides: Arc<dyn StorageOverride<Block>>,
	rpc_config: &RpcConfig,
	prometheus: Option<PrometheusRegistry>,
) -> RpcRequesters {
	let permit_pool = Arc::new(Semaphore::new(rpc_config.ethapi_max_permits as usize));

	let (trace_filter_task, trace_filter_requester) = if rpc_config.ethapi.contains(&EthApi::Trace)
	{
		let (trace_filter_task, trace_filter_requester) = CacheTask::create(
			Arc::clone(&client),
			Arc::clone(&backend),
			Duration::from_secs(rpc_config.ethapi_trace_cache_duration),
			Arc::clone(&permit_pool),
			Arc::clone(&overrides),
			prometheus,
		);
		(Some(trace_filter_task), Some(trace_filter_requester))
	} else {
		(None, None)
	};

	let (debug_task, debug_requester) = if rpc_config.ethapi.contains(&EthApi::Debug) {
		let (debug_task, debug_requester) = DebugHandler::task(
			Arc::clone(&client),
			Arc::clone(&backend),
			match *frontier_backend {
				fc_db::Backend::KeyValue(ref b) => b.clone(),
				fc_db::Backend::Sql(ref b) => b.clone(),
			},
			Arc::clone(&permit_pool),
			Arc::clone(&overrides),
			rpc_config.tracing_raw_max_memory_usage,
		);
		(Some(debug_task), Some(debug_requester))
	} else {
		(None, None)
	};

	// `trace_filter` cache task. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(trace_filter_task) = trace_filter_task {
		task_manager.spawn_essential_handle().spawn(
			"trace-filter-cache",
			Some("eth-tracing"),
			trace_filter_task,
		);
	}

	// `debug` task if enabled. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(debug_task) = debug_task {
		task_manager.spawn_essential_handle().spawn(
			"ethapi-debug",
			Some("eth-tracing"),
			debug_task,
		);
	}

	RpcRequesters { debug: debug_requester, trace: trace_filter_requester }
}
