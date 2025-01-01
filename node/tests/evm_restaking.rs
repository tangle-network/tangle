//! Runs only when `e2e` feature is enabled.
#![cfg(feature = "e2e")]

use common::run_e2e_test;
use futures::prelude::*;
use sp_tracing::info;
use tangle_subxt::subxt::{self, OnlineClient};

mod common;

#[test]
fn it_works() {
	run_e2e_test(async move {
		let client = OnlineClient::<subxt::PolkadotConfig>::new().await.unwrap();
		let stream = client.blocks().subscribe_best().await.unwrap();
		let mut stream = stream.take(5);
		while let Some(Ok(block)) = stream.next().await {
			info!("Block: #{:?}", block.number());
		}

		info!("It works!");
	});
}
