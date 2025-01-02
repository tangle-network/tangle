//! Runs only when `e2e` feature is enabled.
#![cfg(feature = "e2e")]

use core::future::Future;
use core::ops::{Div, Mul};
use core::time::Duration;

use alloy::network::Ethereum;
use alloy::primitives::*;
use alloy::providers::fillers::{FillProvider, RecommendedFillers, WalletFiller};
use alloy::providers::Provider;
use alloy::sol;
use sp_tracing::info;
use tangle_subxt::tangle_testnet_runtime::api;

mod common;

use common::*;

sol! {
	#[sol(rpc, all_derives)]
	MockERC20,
	"tests/fixtures/MockERC20.json",
}

sol! {
	#[sol(rpc, all_derives)]
	"../precompiles/multi-asset-delegation/MultiAssetDelegation.sol",
}

const MULTI_ASSET_DELEGATION: Address = address!("0000000000000000000000000000000000000822");

pub async fn wait_for_block(provider: &impl Provider, block_number: u64) {
	let mut current_block = provider.get_block_number().await.unwrap();
	while current_block < block_number {
		current_block = provider.get_block_number().await.unwrap();
		info!(%current_block, "Waiting for block #{}...", block_number);
		tokio::time::sleep(Duration::from_secs(1)).await;
	}
}

pub async fn wait_for_more_blocks(provider: &impl Provider, blocks: u64) {
	let current_block = provider.get_block_number().await.unwrap();
	wait_for_block(provider, current_block + blocks).await;
}

#[track_caller]
pub fn run_mad_test<F>(f: F)
where
	F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
	run_e2e_test(async move {
		let provider = alloy_provider().await;
		wait_for_block(&provider, 1).await;
		match f.await {
			Ok(_) => info!("Test passed"),
			Err(e) => {
				// wait for 1 more block to ensure that the logs are printed
				wait_for_more_blocks(&provider, 5).await;
				panic!("Test failed: {:?}", e);
			},
		}
	});
}

#[test]
fn operator_join_delegator_delegate() {
	run_mad_test(async move {
		let alice = TestAccount::Alice;
		let wallet = alice.evm_wallet();
		let provider = alloy_provider().await;
		let client = subxt_client().await;
		let alice_provider = FillProvider::new(provider.clone(), Ethereum::recommended_fillers())
			.join_with(WalletFiller::new(wallet));
		// Deploy the contract
		let usdc = MockERC20::deploy(&alice_provider).await?;
		info!("Deployed MockERC20 (USDC) contract at address: {}", usdc.address());
		let decimals = 6u8;
		usdc.initialize(String::from("USD Coin"), String::from("USDC"), decimals)
			.send()
			.await?
			.get_receipt()
			.await?;

		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &alice_provider);
		// Join operators.
		let tnt = U256::from(100_000u128);
		let join_operators_result = precompile
			.joinOperators(tnt)
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(join_operators_result.status());

		let operator_key = api::storage()
			.multi_asset_delegation()
			.operators(alice.address().to_account_id());
		let maybe_operator = client.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.map(|p| p.stake), Some(tnt.to::<u128>()));

		// Delegate assets to the operator.
		let bob = TestAccount::Bob;
		let bob_provider = FillProvider::new(provider.clone(), Ethereum::recommended_fillers())
			.join_with(WalletFiller::new(bob.evm_wallet()));
		// Mint some USDC for Bob.
		let mint_amount = U256::from(100u128).mul(U256::from(10).pow(U256::from(decimals)));
		usdc.mint(bob.address(), mint_amount).send().await?.get_receipt().await?;

		let bob_tnt_balance = bob_provider.get_balance(bob.address()).await?;
		info!("Bob TNT balance: {:?}", bob_tnt_balance);
		assert!(bob_tnt_balance > U256::ZERO);

		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		info!("Bob balance: {:?}", bob_balance._0);
		assert_eq!(bob_balance._0, mint_amount);

		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(U256::from(2));
		assert!(delegate_amount < mint_amount);
		// Deposit USDC to the MAD pallet.
		let deposit_result = precompile
			.deposit(U256::ZERO, *usdc.address(), delegate_amount)
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());
		// Bob balance should be reduced by the delegate amount.
		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		assert_eq!(bob_balance._0, mint_amount - delegate_amount);
		let delegate_result = precompile
			.delegate(
				alice.address().to_account_id().0.into(),
				U256::ZERO,
				*usdc.address(),
				delegate_amount,
				vec![],
			)
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;

		assert!(delegate_result.status());
		anyhow::Ok(())
	});
}
