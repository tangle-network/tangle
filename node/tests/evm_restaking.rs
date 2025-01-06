use core::future::Future;
use core::ops::{Div, Mul};
use core::time::Duration;

use alloy::network::Ethereum;
use alloy::primitives::{FixedBytes, *};
use alloy::providers::fillers::{FillProvider, RecommendedFillers, WalletFiller};
use alloy::providers::{Provider, WalletProvider};
use alloy::sol;
use alloy::transports::BoxTransport;
use anyhow::bail;
use sp_tracing::info;
use tangle_subxt::subxt::{self, OnlineClient};
use tangle_subxt::tangle_testnet_runtime::api;

mod common;

use common::*;
use MockERC20::MockERC20Instance;

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
pub fn run_mad_test<TFn, F>(f: TFn)
where
	TFn: FnOnce(TestInputs) -> F + Send + 'static,
	F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
	run_e2e_test(async move {
		let provider = alloy_provider().await;
		let subxt = subxt_client().await;
		wait_for_block(&provider, 1).await;
		let alice = TestAccount::Alice;
		let wallet = alice.evm_wallet();
		let alice_provider = alloy_provider_with_wallet(&provider, wallet);
		let usdc = MockERC20::deploy(alice_provider.clone()).await?;
		usdc.initialize(String::from("USD Coin"), String::from("USDC"), 6u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (USDC) contract at address: {}", usdc.address());

		let weth = MockERC20::deploy(alice_provider.clone()).await?;
		weth.initialize(String::from("Wrapped Ether"), String::from("WETH"), 18u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (WETH) contract at address: {}", weth.address());

		let wbtc = MockERC20::deploy(alice_provider.clone()).await?;
		wbtc.initialize(String::from("Wrapped Bitcoin"), String::from("WBTC"), 8u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (WBTC) contract at address: {}", wbtc.address());
		let test_inputs = TestInputs { provider, usdc, weth, wbtc, subxt };
		f(test_inputs).await
	});
}

struct TestInputs {
	provider: AlloyProvider,
	usdc: MockERC20Instance<BoxTransport, AlloyProviderWithWallet>,
	weth: MockERC20Instance<BoxTransport, AlloyProviderWithWallet>,
	wbtc: MockERC20Instance<BoxTransport, AlloyProviderWithWallet>,
	subxt: subxt::OnlineClient<subxt::PolkadotConfig>,
}

#[test]
fn operator_join_delegator_delegate() {
	run_mad_test(|t| async move {
		let alice = TestAccount::Alice;
		let wallet = alice.evm_wallet();
		let alice_provider = alloy_provider_with_wallet(&t.provider, wallet);
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
		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.map(|p| p.stake), Some(tnt.to::<u128>()));

		// Delegate assets to the operator.
		let bob = TestAccount::Bob;
		let bob_provider = alloy_provider_with_wallet(&t.provider, bob.evm_wallet());

		let usdc = MockERC20::new(*t.usdc.address(), &bob_provider);
		// Mint some USDC for Bob.
		let mint_amount = U256::from(100_000_000u128);
		usdc.mint(bob.address(), mint_amount).send().await?.get_receipt().await?;

		let bob_tnt_balance = bob_provider.get_balance(bob.address()).await?;
		assert!(bob_tnt_balance > U256::ZERO);

		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		info!("Bob ({:?}) balance: {:?} USDC (in Uints)", bob.address(), bob_balance._0);
		assert_eq!(bob_balance._0, mint_amount);

		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(U256::from(2));
		assert!(delegate_amount < mint_amount);
		// Deposit USDC to the MAD pallet.
		let deposit_result = precompile
			.deposit(U256::ZERO, *usdc.address(), delegate_amount)
			.from(bob.address())
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
				alice.address().into_word(),
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

		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.as_ref().map(|p| p.delegation_count), Some(1));
		assert_eq!(
			maybe_operator.map(|p| { p.delegations.0[0].clone() }),
			Some(
				api::runtime_types::pallet_multi_asset_delegation::types::operator::DelegatorBond {
					delegator: bob.address().to_account_id(),
					amount: delegate_amount.to::<u128>(),
					asset_id: api::runtime_types::tangle_primitives::services::Asset::Erc20(
						(<[u8; 20]>::from(*usdc.address())).into(),
					),
					__ignore: std::marker::PhantomData
				}
			)
		);
		anyhow::Ok(())
	});
}
