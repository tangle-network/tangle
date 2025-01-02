//! Runs only when `e2e` feature is enabled.
#![cfg(feature = "e2e")]

use alloy::network::Ethereum;
use alloy::providers::fillers::{FillProvider, RecommendedFillers, WalletFiller};
use alloy::providers::Provider;
use alloy::sol;
use sp_tracing::info;

mod common;

use common::*;

sol! {
	#[sol(rpc)]
	MockERC20,
	"tests/fixtures/MockERC20.json",
}

sol! {
	#[sol(rpc)]
	"../precompiles/multi-asset-delegation/MultiAssetDelegation.sol",
}

#[test]
fn it_works() {
	run_e2e_test(async move {
		let alice = TestAccount::Alice;
		let wallet = alice.evm_wallet();
		let provider = alloy_provider().await;
		let alice_provider = FillProvider::new(provider.clone(), Ethereum::recommended_fillers())
			.join_with(WalletFiller::new(wallet));
		// Check the wallet balance
		let balance = provider.get_balance(alice.address()).await.unwrap();
		info!("Balance: {}", balance);
		let mut current_block = provider.get_block_number().await.unwrap();
		info!("Current block: {}", current_block);
		while current_block < 2 {
			info!("Waiting for block 2...");
			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			current_block = provider.get_block_number().await.unwrap();
		}
		// Deploy the contract
		let mock_erc20 = MockERC20::deploy(&alice_provider).await.unwrap();
		info!("Deployed contract at address: {}", mock_erc20.address());
	});
}
