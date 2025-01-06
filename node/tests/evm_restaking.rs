use core::future::Future;
use core::ops::Div;
use core::time::Duration;

use alloy::primitives::*;
use alloy::providers::Provider;
use alloy::sol;
use sp_tracing::info;
use tangle_subxt::subxt;
use tangle_subxt::subxt::tx::TxStatus;
use tangle_subxt::tangle_testnet_runtime::api;

mod common;

use common::*;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::pallet_assets;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::pallet_multi_asset_delegation::types::operator::DelegatorBond;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services::Asset;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_testnet_runtime::RuntimeCall;

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

/// Setup the E2E test environment.
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
		let signer = alice.substrate_signer();
		let alice_provider = alloy_provider_with_wallet(&provider, wallet);
		let usdc = MockERC20::deploy(alice_provider.clone()).await?;
		usdc.initialize(String::from("USD Coin"), String::from("USDC"), 6u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (USDC) contract at address: {}", usdc.address());
		let next_asset_id_addr = api::storage().assets().next_asset_id();
		let usdc_asset_id = subxt.storage().at_latest().await?.fetch(&next_asset_id_addr).await?;
		info!("Next asset ID: {:?}", usdc_asset_id);

		let usdc_call = api::tx().utility().batch(vec![
			RuntimeCall::Assets(pallet_assets::pallet::Call::create {
				id: usdc_asset_id.unwrap_or_default(),
				admin: alice.account_id().into(),
				min_balance: 0,
			}),
			RuntimeCall::Assets(pallet_assets::pallet::Call::set_metadata {
				id: usdc_asset_id.unwrap_or_default(),
				name: "USD Coin".into(),
				symbol: "USDC".into(),
				decimals: 6,
			}),
		]);

		let mut result = subxt.tx().sign_and_submit_then_watch_default(&usdc_call, &signer).await?;
		while let Some(Ok(s)) = result.next().await {
			match s {
				TxStatus::InBestBlock(b) => {
					b.wait_for_success().await?;
					info!("Created USDC asset with ID: {}", usdc_asset_id.unwrap_or_default());
				},
				_ => continue,
			}
		}

		let weth = MockERC20::deploy(alice_provider.clone()).await?;
		weth.initialize(String::from("Wrapped Ether"), String::from("WETH"), 18u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (WETH) contract at address: {}", weth.address());

		// Create WETH asset.
		let next_asset_id_addr = api::storage().assets().next_asset_id();
		let weth_asset_id = subxt.storage().at_latest().await?.fetch(&next_asset_id_addr).await?;

		let weth_call = api::tx().utility().batch(vec![
			RuntimeCall::Assets(pallet_assets::pallet::Call::create {
				id: weth_asset_id.unwrap_or_default(),
				admin: alice.account_id().into(),
				min_balance: 0,
			}),
			RuntimeCall::Assets(pallet_assets::pallet::Call::set_metadata {
				id: weth_asset_id.unwrap_or_default(),
				name: "Wrapped Ether".into(),
				symbol: "WETH".into(),
				decimals: 18,
			}),
		]);

		let mut result = subxt.tx().sign_and_submit_then_watch_default(&weth_call, &signer).await?;
		while let Some(Ok(s)) = result.next().await {
			match s {
				TxStatus::InBestBlock(b) => {
					b.wait_for_success().await?;
					info!("Created WETH asset with ID: {}", weth_asset_id.unwrap_or_default());
				},
				_ => continue,
			}
		}

		let wbtc = MockERC20::deploy(alice_provider.clone()).await?;
		wbtc.initialize(String::from("Wrapped Bitcoin"), String::from("WBTC"), 8u8)
			.send()
			.await?
			.get_receipt()
			.await?;
		info!("Deployed MockERC20 (WBTC) contract at address: {}", wbtc.address());

		// Create WBTC asset.
		let next_asset_id_addr = api::storage().assets().next_asset_id();
		let wbtc_asset_id = subxt.storage().at_latest().await?.fetch(&next_asset_id_addr).await?;

		let wbtc_call = api::tx().utility().batch(vec![
			RuntimeCall::Assets(pallet_assets::pallet::Call::create {
				id: wbtc_asset_id.unwrap_or_default(),
				admin: alice.account_id().into(),
				min_balance: 0,
			}),
			RuntimeCall::Assets(pallet_assets::pallet::Call::set_metadata {
				id: wbtc_asset_id.unwrap_or_default(),
				name: "Wrapped Bitcoin".into(),
				symbol: "WBTC".into(),
				decimals: 8,
			}),
		]);

		let mut result = subxt.tx().sign_and_submit_then_watch_default(&wbtc_call, &signer).await?;
		while let Some(Ok(s)) = result.next().await {
			match s {
				TxStatus::InBestBlock(b) => {
					b.wait_for_success().await?;
					info!("Created WBTC asset with ID: {}", wbtc_asset_id.unwrap_or_default());
				},
				_ => continue,
			}
		}

		let test_inputs = TestInputs {
			provider,
			subxt,
			usdc: *usdc.address(),
			weth: *weth.address(),
			wbtc: *wbtc.address(),
			usdc_asset_id: usdc_asset_id.unwrap_or_default(),
			weth_asset_id: weth_asset_id.unwrap_or_default(),
			wbtc_asset_id: wbtc_asset_id.unwrap_or_default(),
		};
		f(test_inputs).await
	});
}

/// Test inputs for the E2E test.
pub struct TestInputs {
	/// The Alloy provider.
	provider: AlloyProvider,
	/// The Subxt client.
	subxt: subxt::OnlineClient<subxt::PolkadotConfig>,
	/// The USDC ERC20 contract address.
	usdc: Address,
	/// The WETH ERC20 contract address.
	weth: Address,
	/// The WBTC ERC20 contract address.
	wbtc: Address,
	/// The USDC asset ID.
	usdc_asset_id: u128,
	/// The WETH asset ID.
	weth_asset_id: u128,
	/// The WBTC asset ID.
	wbtc_asset_id: u128,
}

#[test]
fn operator_join_delegator_delegate_erc20() {
	// This test covers the following scenarios:
	// 1. Operator joins the MultiAssetDelegation pallet.
	// 2. Delegator deposits assets to the MultiAssetDelegation pallet.
	// 3. Delegator delegates assets to the operator.
	// 4. Operator's delegation count and delegations are updated.
	// 5. Delegator's balance is reduced by the delegate amount.
	// 6. Delegator's delegation is added to the operator's delegations.
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

		let usdc = MockERC20::new(t.usdc, &bob_provider);
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
			Some(DelegatorBond {
				delegator: bob.address().to_account_id(),
				amount: delegate_amount.to::<u128>(),
				asset_id: Asset::Erc20((<[u8; 20]>::from(*usdc.address())).into(),),
				__ignore: std::marker::PhantomData
			})
		);
		anyhow::Ok(())
	});
}

#[test]
fn operator_join_delegator_delegate_asset_id() {
	// This test covers the following scenarios:
	// 1. Operator joins the MultiAssetDelegation pallet.
	// 2. Delegator deposits assets to the MultiAssetDelegation pallet uisng asset ID.
	// 3. Delegator delegates assets to the operator using asset ID.
	// 4. Operator's delegation count and delegations are updated.
	// 5. Delegator's balance is reduced by the delegate amount.
	// 6. Delegator's delegation is added to the operator's delegations.
	run_mad_test(|t| async move {
		let alice = TestAccount::Alice;
		let wallet = alice.evm_wallet();
		let signer = alice.substrate_signer();
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

		// Mint some USDC for Bob.
		let mint_amount = 100_000_000u128;
		let mint_call = api::tx().assets().mint(
			t.usdc_asset_id,
			bob.address().to_account_id().into(),
			mint_amount,
		);
		let mut result =
			t.subxt.tx().sign_and_submit_then_watch_default(&mint_call, &signer).await?;
		while let Some(Ok(s)) = result.next().await {
			match s {
				TxStatus::InBestBlock(b) => {
					b.wait_for_success().await?;
					info!("Minted {:?} USDC for Bob", mint_amount);
				},
				_ => continue,
			}
		}

		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(2);
		assert!(delegate_amount < mint_amount);
		// Deposit USDC to the MAD pallet.
		let deposit_result = precompile
			.deposit(U256::from(t.usdc_asset_id), Address::ZERO, U256::from(delegate_amount))
			.from(bob.address())
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());
		// Bob balance should be reduced by the delegate amount.
		let bob_balance_addr =
			api::storage().assets().account(t.usdc_asset_id, bob.address().to_account_id());
		let bob_balance = t.subxt.storage().at_latest().await?.fetch(&bob_balance_addr).await?;

		assert_eq!(bob_balance.map(|a| a.balance), Some(mint_amount - delegate_amount));

		let delegate_result = precompile
			.delegate(
				alice.address().into_word(),
				U256::from(t.usdc_asset_id),
				Address::ZERO,
				U256::from(delegate_amount),
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
			Some(DelegatorBond {
				delegator: bob.address().to_account_id(),
				amount: delegate_amount,
				asset_id: Asset::Custom(t.usdc_asset_id),
				__ignore: std::marker::PhantomData
			})
		);

		anyhow::Ok(())
	});
}
