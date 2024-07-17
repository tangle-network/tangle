// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ibc::prelude::*;

use alloc::string::ToString;
use core::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	time::Duration,
};
use ibc_proto::{
	google::protobuf::Any, ibc::lightclients::tendermint::v1::ClientState as RawClientState,
};
use serde::{Deserialize, Serialize};
use tendermint_light_client_verifier::options::Options;
use tendermint_proto::Protobuf;

use crate::{
	client_def::TendermintClient, client_message::Header, error::Error, HostFunctionsProvider,
};
use ibc::{
	core::{
		ics02_client::{
			client_state::ClientType, error::Error as Ics02Error, trust_threshold::TrustThreshold,
		},
		ics23_commitment::specs::ProofSpecs,
		ics24_host::identifier::ChainId,
	},
	timestamp::Timestamp,
	Height,
};

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct ClientState<H> {
	pub chain_id: ChainId,
	pub trust_level: TrustThreshold,
	pub trusting_period: Duration,
	pub unbonding_period: Duration,
	pub max_clock_drift: Duration,
	pub latest_height: Height,
	pub proof_specs: ProofSpecs,
	pub upgrade_path: Vec<String>,
	pub frozen_height: Option<Height>,
	pub _phantom: PhantomData<H>,
}

impl<H: Clone> Protobuf<RawClientState> for ClientState<H> {}

impl<H: Clone> ClientState<H> {
	pub fn to_any(&self) -> Any {
		Any {
			type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
			value: self.encode_vec().unwrap(),
		}
	}
}

impl<H> ClientState<H> {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		chain_id: ChainId,
		trust_level: TrustThreshold,
		trusting_period: Duration,
		unbonding_period: Duration,
		max_clock_drift: Duration,
		latest_height: Height,
		proof_specs: ProofSpecs,
		upgrade_path: Vec<String>,
	) -> Result<ClientState<H>, Error> {
		// Basic validation of trusting period and unbonding period: each should be non-zero.
		if trusting_period <= Duration::new(0, 0) {
			return Err(Error::invalid_trusting_period(format!(
				"ClientState trusting period ({:?}) must be greater than zero",
				trusting_period
			)))
		}

		if unbonding_period <= Duration::new(0, 0) {
			return Err(Error::invalid_unbonding_period(format!(
				"ClientState unbonding period ({:?}) must be greater than zero",
				unbonding_period
			)))
		}

		if trusting_period >= unbonding_period {
			return Err(Error::invalid_trusting_period(format!(
				"ClientState trusting period ({:?}) must be smaller than unbonding period ({:?})",
				trusting_period, unbonding_period,
			)))
		}

		// Basic validation for the latest_height parameter.
		if latest_height <= Height::zero() {
			return Err(Error::validation(
				"ClientState latest height must be greater than zero".to_string(),
			))
		}

		// `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
		// value is invalid in this context
		if trust_level == TrustThreshold::ZERO {
			return Err(Error::validation("ClientState trust-level cannot be zero".to_string()))
		}

		// Disallow empty proof-specs
		if proof_specs.is_empty() {
			return Err(Error::validation("ClientState proof-specs cannot be empty".to_string()))
		}

		Ok(Self {
			chain_id,
			trust_level,
			trusting_period,
			unbonding_period,
			max_clock_drift,
			latest_height,
			proof_specs,
			upgrade_path,
			frozen_height: None,
			_phantom: Default::default(),
		})
	}

	pub fn latest_height(&self) -> Height {
		self.latest_height
	}

	pub fn with_header(self, h: Header) -> Self {
		// TODO: Clarify which fields should update.
		ClientState {
			latest_height: self
				.latest_height
				.with_revision_height(u64::from(h.signed_header.header.height)),
			..self
		}
	}

	pub fn with_frozen_height(self, h: Height) -> Result<Self, Error> {
		if h == Height::zero() {
			return Err(Error::validation(
				"ClientState frozen height must be greater than zero".to_string(),
			))
		}
		Ok(Self { frozen_height: Some(h), ..self })
	}

	/// Get the refresh time to ensure the state does not expire
	pub fn refresh_time(&self) -> Option<Duration> {
		Some(2 * self.trusting_period / 3)
	}

	pub fn chain_id(&self) -> ChainId {
		self.chain_id.clone()
	}

	pub fn client_type() -> ClientType {
		"07-tendermint".to_string()
	}

	pub fn frozen_height(&self) -> Option<Height> {
		self.frozen_height
	}

	pub fn upgrade(
		mut self,
		upgrade_height: Height,
		upgrade_options: UpgradeOptions,
		chain_id: ChainId,
	) -> Self {
		// Upgrade the client state
		self.latest_height = upgrade_height;
		self.unbonding_period = upgrade_options.unbonding_period;
		self.chain_id = chain_id;
		self.proof_specs = upgrade_options.proof_specs;
		self.upgrade_path = upgrade_options.upgrade_path;

		self
	}

	/// Check if the state is expired when `elapsed` time has passed since the latest consensus
	/// state timestamp
	pub fn expired(&self, elapsed: Duration) -> bool {
		elapsed > self.trusting_period
	}

	/// Helper method to produce a [`Options`] struct for use in
	/// Tendermint-specific light client verification.
	pub fn as_light_client_options(&self) -> Result<Options, Error> {
		Ok(Options {
			trust_threshold: self
				.trust_level
				.try_into()
				.map_err(|e: Ics02Error| Error::invalid_trust_threshold(e.to_string()))?,
			trusting_period: self.trusting_period,
			clock_drift: self.max_clock_drift,
		})
	}

	/// Verify the time and height delays
	pub fn verify_delay_passed(
		current_time: Timestamp,
		current_height: Height,
		processed_time: u64,
		processed_height: u64,
		delay_period_time: u64,
		delay_period_blocks: u64,
	) -> Result<(), Error> {
		let earliest_time = processed_time + delay_period_time;
		// NOTE: delay time period is inclusive, so if current_time is earliest_time, then we
		// return no error https://github.com/cosmos/ibc-go/blob/9ebc2f81049869bc40c443ffb72d9f3e47afb4fc/modules/light-clients/07-tendermint/client_state.go#L306
		if current_time.nanoseconds() < earliest_time {
			return Err(Error::not_enough_time_elapsed(current_time, earliest_time))
		}

		let earliest_height = processed_height + delay_period_blocks;
		if current_height.revision_height < earliest_height {
			return Err(Error::not_enough_blocks_elapsed(current_height, earliest_height))
		}

		Ok(())
	}

	/// Verify that the client is at a sufficient height and unfrozen at the given height
	pub fn verify_height(&self, height: Height) -> Result<(), Error> {
		if self.latest_height < height {
			return Err(Error::insufficient_height(self.latest_height(), height))
		}

		match self.frozen_height {
			Some(frozen_height) if frozen_height <= height =>
				Err(Error::client_frozen(frozen_height, height)),
			_ => Ok(()),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeOptions {
	pub unbonding_period: Duration,
	pub proof_specs: ProofSpecs,
	pub upgrade_path: Vec<String>,
}

impl<H> ibc::core::ics02_client::client_state::ClientState for ClientState<H>
where
	H: HostFunctionsProvider,
{
	type UpgradeOptions = UpgradeOptions;
	type ClientDef = TendermintClient<H>;

	fn chain_id(&self) -> ChainId {
		self.chain_id()
	}

	fn client_def(&self) -> Self::ClientDef {
		TendermintClient::default()
	}

	fn client_type(&self) -> ClientType {
		Self::client_type()
	}

	fn latest_height(&self) -> Height {
		self.latest_height()
	}

	fn frozen_height(&self) -> Option<Height> {
		self.frozen_height()
	}

	fn upgrade(
		self,
		upgrade_height: Height,
		upgrade_options: UpgradeOptions,
		chain_id: ChainId,
	) -> Self {
		self.upgrade(upgrade_height, upgrade_options, chain_id)
	}

	fn expired(&self, elapsed: Duration) -> bool {
		self.expired(elapsed)
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl<H> TryFrom<RawClientState> for ClientState<H> {
	type Error = Error;

	fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
		let trust_level = raw.trust_level.clone().ok_or_else(Error::missing_trusting_period)?;

		let frozen_height = raw.frozen_height.and_then(|raw_height| {
			let height = raw_height.into();
			if height == Height::zero() {
				None
			} else {
				Some(height)
			}
		});

		Ok(Self {
			chain_id: ChainId::from_string(raw.chain_id.as_str()),
			trust_level: trust_level
				.try_into()
				.map_err(|e| Error::invalid_trust_threshold(format!("{}", e)))?,
			trusting_period: raw
				.trusting_period
				.ok_or_else(Error::missing_trusting_period)?
				.try_into()
				.map_err(|_| Error::negative_trusting_period())?,
			unbonding_period: raw
				.unbonding_period
				.ok_or_else(Error::missing_unbonding_period)?
				.try_into()
				.map_err(|_| Error::negative_unbonding_period())?,
			max_clock_drift: raw
				.max_clock_drift
				.ok_or_else(Error::missing_max_clock_drift)?
				.try_into()
				.map_err(|_| Error::negative_max_clock_drift())?,
			latest_height: raw.latest_height.ok_or_else(Error::missing_latest_height)?.into(),
			frozen_height,
			upgrade_path: raw.upgrade_path,
			proof_specs: raw.proof_specs.into(),
			_phantom: Default::default(),
		})
	}
}

impl<H> From<ClientState<H>> for RawClientState {
	fn from(value: ClientState<H>) -> Self {
		RawClientState {
			chain_id: value.chain_id.to_string(),
			trust_level: Some(value.trust_level.into()),
			trusting_period: Some(value.trusting_period.into()),
			unbonding_period: Some(value.unbonding_period.into()),
			max_clock_drift: Some(value.max_clock_drift.into()),
			frozen_height: Some(value.frozen_height.unwrap_or_else(Height::zero).into()),
			latest_height: Some(value.latest_height.into()),
			proof_specs: value.proof_specs.into(),
			upgrade_path: value.upgrade_path,
			..Default::default()
		}
	}
}

#[cfg(test)]
mod tests {
	use core::time::Duration;
	use ibc::{prelude::*, Height};
	use test_log::test;

	use ibc_proto::cosmos::ics23::v1::ProofSpec as Ics23ProofSpec;
	use tendermint_rpc::endpoint::abci_query::AbciQuery;

	use crate::{client_state::ClientState, mock::Crypto};
	use ibc::{
		core::{
			ics02_client::trust_threshold::TrustThreshold, ics23_commitment::specs::ProofSpecs,
			ics24_host::identifier::ChainId,
		},
		test::test_serialization_roundtrip,
		timestamp::{Timestamp, ZERO_DURATION},
	};

	#[derive(Clone, Debug, PartialEq)]
	struct ClientStateParams {
		id: ChainId,
		trust_level: TrustThreshold,
		trusting_period: Duration,
		unbonding_period: Duration,
		max_clock_drift: Duration,
		latest_height: Height,
		proof_specs: ProofSpecs,
		upgrade_path: Vec<String>,
	}

	#[test]
	fn serialization_roundtrip_no_proof() {
		let json_data = include_str!("mock/query/serialization/client_state.json");
		test_serialization_roundtrip::<AbciQuery>(json_data);
	}

	#[test]
	fn serialization_roundtrip_with_proof() {
		let json_data = include_str!("mock/query/serialization/client_state_proof.json");
		test_serialization_roundtrip::<AbciQuery>(json_data);
	}

	#[test]
	fn client_state_new() {
		// Define a "default" set of parameters to reuse throughout these tests.
		let default_params: ClientStateParams = ClientStateParams {
			id: ChainId::default(),
			trust_level: TrustThreshold::ONE_THIRD,
			trusting_period: Duration::new(64000, 0),
			unbonding_period: Duration::new(128000, 0),
			max_clock_drift: Duration::new(3, 0),
			latest_height: Height::new(0, 10),
			proof_specs: ProofSpecs::default(),
			upgrade_path: vec!["".to_string()],
		};

		struct Test {
			name: String,
			params: ClientStateParams,
			want_pass: bool,
		}

		let tests: Vec<Test> = vec![
			Test {
				name: "Valid parameters".to_string(),
				params: default_params.clone(),
				want_pass: true,
			},
			Test {
				name: "Invalid unbonding period".to_string(),
				params: ClientStateParams {
					unbonding_period: ZERO_DURATION,
					..default_params.clone()
				},
				want_pass: false,
			},
			Test {
				name: "Invalid (too small) trusting period".to_string(),
				params: ClientStateParams {
					trusting_period: ZERO_DURATION,
					..default_params.clone()
				},
				want_pass: false,
			},
			Test {
				name: "Invalid (too large) trusting period w.r.t. unbonding period".to_string(),
				params: ClientStateParams {
					trusting_period: Duration::new(11, 0),
					unbonding_period: Duration::new(10, 0),
					..default_params.clone()
				},
				want_pass: false,
			},
			Test {
				name: "Invalid (too small) trusting trust threshold".to_string(),
				params: ClientStateParams {
					trust_level: TrustThreshold::ZERO,
					..default_params.clone()
				},
				want_pass: false,
			},
			Test {
				name: "Invalid (too small) latest height".to_string(),
				params: ClientStateParams {
					latest_height: Height::zero(),
					..default_params.clone()
				},
				want_pass: false,
			},
			Test {
				name: "Invalid (empty) proof specs".to_string(),
				params: ClientStateParams {
					proof_specs: ProofSpecs::from(Vec::<Ics23ProofSpec>::new()),
					..default_params
				},
				want_pass: false,
			},
		]
		.into_iter()
		.collect();

		for test in tests {
			let p = test.params.clone();

			let cs_result = ClientState::<Crypto>::new(
				p.id,
				p.trust_level,
				p.trusting_period,
				p.unbonding_period,
				p.max_clock_drift,
				p.latest_height,
				p.proof_specs,
				p.upgrade_path,
			);

			assert_eq!(
				test.want_pass,
				cs_result.is_ok(),
				"ClientState::new() failed for test {}, \nmsg{:?} with error {:?}",
				test.name,
				test.params.clone(),
				cs_result.err(),
			);
		}
	}

	#[test]
	fn client_state_verify_delay_passed() {
		#[derive(Debug, Clone)]
		struct Params {
			current_time: Timestamp,
			current_height: Height,
			processed_time: Timestamp,
			processed_height: Height,
			delay_period_time: Duration,
			delay_period_blocks: u64,
		}
		struct Test {
			name: String,
			params: Params,
			want_pass: bool,
		}
		let now = Timestamp::now();

		let tests: Vec<Test> = vec![
			Test {
				name: "Successful delay verification".to_string(),
				params: Params {
					current_time: (now + Duration::from_nanos(2000)).unwrap(),
					current_height: Height::new(0, 5),
					processed_time: (now + Duration::from_nanos(1000)).unwrap(),
					processed_height: Height::new(0, 3),
					delay_period_time: Duration::from_nanos(500),
					delay_period_blocks: 2,
				},
				want_pass: true,
			},
			Test {
				name: "Delay period(time) has not elapsed".to_string(),
				params: Params {
					current_time: (now + Duration::from_nanos(1200)).unwrap(),
					current_height: Height::new(0, 5),
					processed_time: (now + Duration::from_nanos(1000)).unwrap(),
					processed_height: Height::new(0, 4),
					delay_period_time: Duration::from_nanos(500),
					delay_period_blocks: 2,
				},
				want_pass: false,
			},
			Test {
				name: "Delay period(blocks) has not elapsed".to_string(),
				params: Params {
					current_time: (now + Duration::from_nanos(2000)).unwrap(),
					current_height: Height::new(0, 5),
					processed_time: (now + Duration::from_nanos(1000)).unwrap(),
					processed_height: Height::new(0, 4),
					delay_period_time: Duration::from_nanos(500),
					delay_period_blocks: 2,
				},
				want_pass: false,
			},
		];

		for test in tests {
			let res = ClientState::<Crypto>::verify_delay_passed(
				test.params.current_time,
				test.params.current_height,
				test.params.processed_time.nanoseconds(),
				test.params.processed_height.revision_height,
				test.params.delay_period_time.as_secs(),
				test.params.delay_period_blocks,
			);

			assert_eq!(
				test.want_pass,
				res.is_ok(),
				"ClientState::verify_delay_passed() failed for test {}, \nmsg{:?} with error {:?}",
				test.name,
				test.params.clone(),
				res.err(),
			);
		}
	}

	#[test]
	fn client_state_verify_height() {
		// Define a "default" set of parameters to reuse throughout these tests.
		let default_params: ClientStateParams = ClientStateParams {
			id: ChainId::default(),
			trust_level: TrustThreshold::ONE_THIRD,
			trusting_period: Duration::new(64000, 0),
			unbonding_period: Duration::new(128000, 0),
			max_clock_drift: Duration::new(3, 0),
			latest_height: Height::new(1, 10),
			proof_specs: ProofSpecs::default(),
			upgrade_path: vec!["".to_string()],
		};

		struct Test {
			name: String,
			height: Height,
			setup: Option<Box<dyn FnOnce(ClientState<Crypto>) -> ClientState<Crypto>>>,
			want_pass: bool,
		}

		let tests = vec![
			Test {
				name: "Successful height verification".to_string(),
				height: Height::new(1, 8),
				setup: None,
				want_pass: true,
			},
			Test {
				name: "Invalid (too large)  client height".to_string(),
				height: Height::new(1, 12),
				setup: None,
				want_pass: false,
			},
			Test {
				name: "Invalid, client is frozen below current height".to_string(),
				height: Height::new(1, 6),
				setup: Some(Box::new(|client_state| {
					client_state.with_frozen_height(Height::new(1, 5)).unwrap()
				})),
				want_pass: false,
			},
		];

		for test in tests {
			let p = default_params.clone();
			let client_state = ClientState::new(
				p.id,
				p.trust_level,
				p.trusting_period,
				p.unbonding_period,
				p.max_clock_drift,
				p.latest_height,
				p.proof_specs,
				p.upgrade_path,
			)
			.unwrap();
			let client_state = match test.setup {
				Some(setup) => (setup)(client_state),
				_ => client_state,
			};
			let res = client_state.verify_height(test.height);

			assert_eq!(
				test.want_pass,
				res.is_ok(),
				"ClientState::verify_delay_height() failed for test {}, \nmsg{:?} with error {:?}",
				test.name,
				test.height,
				res.err(),
			);
		}
	}
}

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
	use core::time::Duration;
	use ibc::prelude::*;

	use tendermint::block::Header;

	use crate::{
		client_state::ClientState,
		mock::{AnyClientState, Crypto},
	};
	use ibc::core::{ics02_client::height::Height, ics24_host::identifier::ChainId};

	pub fn get_dummy_tendermint_client_state(tm_header: Header) -> AnyClientState {
		AnyClientState::Tendermint(
			ClientState::<Crypto>::new(
				ChainId::from(tm_header.chain_id.clone()),
				Default::default(),
				Duration::from_secs(64000),
				Duration::from_secs(128000),
				Duration::from_millis(3000),
				Height::new(
					ChainId::chain_version(tm_header.chain_id.as_str()),
					u64::from(tm_header.height),
				),
				Default::default(),
				vec!["".to_string()],
			)
			.unwrap(),
		)
	}
}
