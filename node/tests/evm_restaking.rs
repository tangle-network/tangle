//! Runs only when `e2e` feature is enabled.
#![cfg(feature = "e2e")]

use alloy::network::EthereumWallet;
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::{providers::ProviderBuilder, sol};
use common::run_e2e_test;
use futures::prelude::*;
use sp_tracing::info;
use tangle_subxt::subxt::{self, OnlineClient};
use tangle_subxt::subxt_signer::{ecdsa, sr25519, SecretUri};

mod common;

sol! {
	#[sol(rpc)]
	MockERC20,
	"tests/fixtures/MockERC20.json",
}

#[test]
fn it_works() {
	run_e2e_test(async move {
		let alice_seed = ecdsa::dev::alice().0.secret_bytes();
		let signer = PrivateKeySigner::from_bytes((&alice_seed).into()).unwrap();
		let alice_address = signer.address();
		let wallet = EthereumWallet::from(signer);
		let provider = ProviderBuilder::new()
			.with_recommended_fillers()
			.wallet(wallet)
			.on_builtin("http://127.0.0.1:9944")
			.await
			.unwrap();

		// Check the wallet balance
		let balance = provider.get_balance(alice_address).await.unwrap();
		info!("Balance: {}", balance);
		// Deploy the contract at block #2.
		let mut current_block = provider.get_block_number().await.unwrap();
		info!("Current block: {}", current_block);
		while current_block < 2 {
			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
			current_block = provider.get_block_number().await.unwrap();
		}
		info!("Deploying contract at block #{}", current_block);
		let mock_erc20 = MockERC20::deploy(&provider).await.unwrap();
		info!("Deployed contract at address: {}", mock_erc20.address());
	});
}
