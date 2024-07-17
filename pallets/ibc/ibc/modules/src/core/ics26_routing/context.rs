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

use crate::prelude::*;

use crate::{
	core::{
		ics02_client::context::{ClientKeeper, ClientReader},
		ics03_connection::context::{ConnectionKeeper, ConnectionReader},
		ics04_channel::{
			channel::{Counterparty, Order},
			context::{ChannelKeeper, ChannelReader},
			error::Error,
			msgs::acknowledgement::Acknowledgement as GenericAcknowledgement,
			packet::Packet,
			Version,
		},
		ics05_port::context::PortReader,
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
	},
	events::IbcEvent,
	handler::HandlerOutputBuilder,
	signer::Signer,
};
use alloc::borrow::{Borrow, Cow};
use core::{any::Any, fmt, fmt::Debug, str::FromStr};
use serde::{Deserialize, Serialize};

/// This trait captures all the functional dependencies needed in light client implementations
pub trait ReaderContext: ClientKeeper + ClientReader + ConnectionReader + ChannelReader {}

/// This trait captures all the dependencies needed for module callbacks to read storage
pub trait ModuleCallbackContext: ConnectionReader + ChannelReader {}

/// This trait captures all the functional dependencies (i.e., context) which the ICS26 module
/// requires to be able to dispatch and process IBC messages. In other words, this is the
/// representation of a chain from the perspective of the IBC module of that chain.
pub trait Ics26Context:
	Clone + ConnectionKeeper + ChannelKeeper + PortReader + ReaderContext + ModuleCallbackContext
{
	type Router: Router;

	fn router(&self) -> &Self::Router;

	fn router_mut(&mut self) -> &mut Self::Router;
}

#[derive(Debug, PartialEq)]
pub struct InvalidModuleId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ModuleId(String);

impl ModuleId {
	pub fn new(s: Cow<'_, str>) -> Result<Self, InvalidModuleId> {
		if !s.trim().is_empty() && s.chars().all(char::is_alphanumeric) {
			Ok(Self(s.into_owned()))
		} else {
			Err(InvalidModuleId)
		}
	}
}

impl fmt::Display for ModuleId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl FromStr for ModuleId {
	type Err = InvalidModuleId;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(Cow::Borrowed(s))
	}
}

impl Borrow<str> for ModuleId {
	fn borrow(&self) -> &str {
		self.0.as_str()
	}
}

impl AsRef<str> for ModuleId {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}

/// Types implementing this trait are expected to implement `From<GenericAcknowledgement>`
pub trait Acknowledgement: AsRef<[u8]> {}

pub type ModuleOutputBuilder = HandlerOutputBuilder<(), IbcEvent>;

pub trait Module: Send + Sync + AsAnyMut {
	#[allow(clippy::too_many_arguments)]
	fn on_chan_open_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty: &Counterparty,
		_version: &Version,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	#[allow(clippy::too_many_arguments)]
	fn on_chan_open_try(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty: &Counterparty,
		_version: &Version,
		_counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<Version, Error>;

	fn on_chan_open_ack(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_chan_open_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_chan_close_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_chan_close_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	/// Modules can choose to write acknowledgement to storage in this callback
	fn on_recv_packet(
		&self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<GenericAcknowledgement, Error> {
		Ok(GenericAcknowledgement::from_bytes(Default::default()))
	}

	fn on_acknowledgement_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_packet: &mut Packet,
		_acknowledgement: &GenericAcknowledgement,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}

	fn on_timeout_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<(), Error> {
		Ok(())
	}
}

pub trait RouterBuilder: Sized {
	/// The `Router` type that the builder must build
	type Router: Router;

	/// Registers `Module` against the specified `ModuleId` in the `Router`'s internal map
	///
	/// Returns an error if a `Module` has already been registered against the specified `ModuleId`
	fn add_route(self, module_id: ModuleId, module: impl Module) -> Result<Self, String>;

	/// Consumes the `RouterBuilder` and returns a `Router` as configured
	fn build(self) -> Self::Router;
}

pub trait AsAnyMut: Any {
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<M: Any + Module> AsAnyMut for M {
	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}

/// A router maintains a mapping of `ModuleId`s against `Modules`. Implementations must not publicly
/// expose APIs to add new routes once constructed. Routes may only be added at the time of
/// instantiation using the `RouterBuilder`.
pub trait Router {
	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;

	/// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
	fn has_route(&self, module_id: &ModuleId) -> bool;
}
