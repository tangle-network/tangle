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

use crate::{events::IbcEvent, prelude::*};
use core::marker::PhantomData;

pub type HandlerResult<T, E> = Result<HandlerOutput<T>, E>;

#[derive(Clone, Debug)]
pub struct HandlerOutput<T, Event = IbcEvent> {
	pub result: T,
	pub log: Vec<String>,
	pub events: Vec<Event>,
}

impl<T, E> HandlerOutput<T, E> {
	pub fn builder() -> HandlerOutputBuilder<T, E> {
		HandlerOutputBuilder::new()
	}
}

#[derive(Clone, Debug, Default)]
pub struct HandlerOutputBuilder<T, E = IbcEvent> {
	log: Vec<String>,
	events: Vec<E>,
	marker: PhantomData<T>,
}

impl<T, E> HandlerOutputBuilder<T, E> {
	pub fn new() -> Self {
		Self { log: Vec::new(), events: Vec::new(), marker: PhantomData }
	}

	pub fn with_log(mut self, log: impl Into<Vec<String>>) -> Self {
		self.log.append(&mut log.into());
		self
	}

	pub fn log(&mut self, log: impl Into<String>) {
		self.log.push(log.into());
	}

	pub fn with_events(mut self, mut events: Vec<E>) -> Self {
		self.events.append(&mut events);
		self
	}

	pub fn emit(&mut self, event: E) {
		self.events.push(event);
	}

	pub fn with_result(self, result: T) -> HandlerOutput<T, E> {
		HandlerOutput { result, log: self.log, events: self.events }
	}

	pub fn merge<Event: Into<E>>(&mut self, other: HandlerOutputBuilder<(), Event>) {
		let HandlerOutputBuilder { mut log, events, .. } = other;
		self.log.append(&mut log);
		self.events.append(&mut events.into_iter().map(Into::into).collect());
	}

	pub fn merge_output<Event: Into<E>>(&mut self, other: HandlerOutput<(), Event>) {
		let HandlerOutput { mut log, events, .. } = other;
		self.log.append(&mut log);
		self.events.append(&mut events.into_iter().map(Into::into).collect());
	}
}
