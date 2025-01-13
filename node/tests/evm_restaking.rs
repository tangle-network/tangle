//! Multi-Asset Delegation E2E Tests
//!
//! This module contains end-to-end tests for the Multi-Asset Delegation functionality,
//! testing both ERC20 and Asset ID based delegations. The tests verify operator joining,
//! asset delegation, and the correct state updates in the system.

use core::future::Future;
use core::ops::Div;
use core::time::Duration;

use alloy::primitives::*;
use alloy::providers::Provider;
use alloy::sol;
use anyhow::bail;
use sp_runtime::traits::AccountIdConversion;
use sp_tracing::{error, info};
use tangle_runtime::PalletId;
use tangle_subxt::subxt;
use tangle_subxt::subxt::tx::TxStatus;
use tangle_subxt::tangle_testnet_runtime::api;

mod common;

use common::*;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::pallet_multi_asset_delegation::types::operator::DelegatorBond;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services::Asset;

sol! {
	#[allow(clippy::too_many_arguments)]
	#[sol(rpc, all_derives)]
	MockERC20,
	"tests/fixtures/MockERC20.json",
}

sol! {
	#[sol(rpc, all_derives)]
	"../precompiles/multi-asset-delegation/MultiAssetDelegation.sol",
}

const MULTI_ASSET_DELEGATION: Address = address!("0000000000000000000000000000000000000822");

/// Waits for a specific block number to be reached
pub async fn wait_for_block(provider: &impl Provider, block_number: u64) {
	let mut current_block = provider.get_block_number().await.unwrap();
	while current_block < block_number {
		current_block = provider.get_block_number().await.unwrap();
		info!(%current_block, "Waiting for block #{}...", block_number);
		tokio::time::sleep(Duration::from_secs(1)).await;
	}
}

/// Waits for a specified number of additional blocks
pub async fn wait_for_more_blocks(provider: &impl Provider, blocks: u64) {
	let current_block = provider.get_block_number().await.unwrap();
	wait_for_block(provider, current_block + blocks).await;
}

/// Waits for the next session to start and returns the session index
pub async fn wait_for_next_session(
	client: &subxt::OnlineClient<subxt::PolkadotConfig>,
) -> anyhow::Result<u32> {
	let mut new_blocks = client.blocks().subscribe_best().await?;
	loop {
		if let Some(Ok(block)) = new_blocks.next().await {
			let evs = block.events().await?;
			if let Some(new_session) = evs.find_first::<api::session::events::NewSession>()? {
				return Ok(new_session.session_index);
			} else {
				info!("No new session event found in block #{}", block.number());
			}
		} else {
			bail!("Error while waiting for new blocks");
		}
	}
}

/// Deploys and initializes an ERC20 token contract
async fn deploy_erc20(
	provider: AlloyProviderWithWallet,
	name: &str,
	symbol: &str,
	decimals: u8,
) -> anyhow::Result<Address> {
	let token = MockERC20::deploy(provider.clone()).await?;
	token
		.initialize(name.to_string(), symbol.to_string(), decimals)
		.send()
		.await?
		.get_receipt()
		.await?;
	info!("Deployed {} token contract at address: {}", symbol, token.address());
	Ok(*token.address())
}

/// Creates a new asset in the runtime and returns the asset ID
async fn create_asset(
	subxt: &subxt::OnlineClient<subxt::PolkadotConfig>,
	signer: &TestAccount,
	asset_id: u128,
	name: &str,
	symbol: &str,
	decimals: u8,
) -> anyhow::Result<()> {
	let asset_create_call = api::tx().assets().create(asset_id, signer.account_id().into(), 1);
	let asset_metadata_call =
		api::tx().assets().set_metadata(asset_id, name.into(), symbol.into(), decimals);
	let mut result = subxt
		.tx()
		.sign_and_submit_then_watch_default(&asset_create_call, &signer.substrate_signer())
		.await?;

	while let Some(Ok(s)) = result.next().await {
		if let TxStatus::InBestBlock(b) = s {
			let evs = match b.wait_for_success().await {
				Ok(evs) => evs,
				Err(e) => {
					error!("Error: {:?}", e);
					break;
				},
			};
			let created = evs
				.find_first::<api::assets::events::Created>()?
				.expect("Created event to be emitted");
			assert_eq!(created.asset_id, asset_id, "Asset ID mismatch");
			break;
		}
	}

	result = subxt
		.tx()
		.sign_and_submit_then_watch_default(&asset_metadata_call, &signer.substrate_signer())
		.await?;

	while let Some(Ok(s)) = result.next().await {
		if let TxStatus::InBestBlock(b) = s {
			let evs = match b.wait_for_success().await {
				Ok(evs) => evs,
				Err(e) => {
					error!("Error: {:?}", e);
					break;
				},
			};
			let metadata_set = evs
				.find_first::<api::assets::events::MetadataSet>()?
				.expect("MetadataSet event to be emitted");
			assert_eq!(metadata_set.asset_id, asset_id, "Asset ID mismatch");
			break;
		}
	}

	Ok(())
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
		let alice_provider = alloy_provider_with_wallet(&provider, wallet.clone());

		// Deploy ERC20 tokens
		let usdc_addr = deploy_erc20(alice_provider.clone(), "USD Coin", "USDC", 6).await?;
		let weth_addr = deploy_erc20(alice_provider.clone(), "Wrapped Ether", "WETH", 18).await?;
		let wbtc_addr = deploy_erc20(alice_provider.clone(), "Wrapped Bitcoin", "WBTC", 8).await?;

		// Create runtime assets
		create_asset(&subxt, &alice, 0, "USD Coin", "USDC", 6).await?;
		create_asset(&subxt, &alice, 1, "Wrapped Ether", "WETH", 18).await?;
		create_asset(&subxt, &alice, 2, "Wrapped Bitcoin", "WBTC", 8).await?;

		let pallet_account_addr = api::constants().multi_asset_delegation().pallet_id();
		let pallet_account_id = subxt.constants().at(&pallet_account_addr).unwrap();
		let pallet_account_id =
			AccountIdConversion::<subxt::utils::AccountId32>::into_account_truncating(&PalletId(
				pallet_account_id.0,
			));

		// Send some balance to the MAD pallet
		let transfer_keep_alive_call = api::tx()
			.balances()
			.transfer_keep_alive(pallet_account_id.clone().into(), 100_000_000_000);

		let mut result = subxt
			.tx()
			.sign_and_submit_then_watch_default(
				&transfer_keep_alive_call,
				&alice.substrate_signer(),
			)
			.await?;

		while let Some(Ok(s)) = result.next().await {
			if let TxStatus::InBestBlock(b) = s {
				let evs = match b.wait_for_success().await {
					Ok(evs) => evs,
					Err(e) => {
						error!("Error: {:?}", e);
						break;
					},
				};
				evs.find_first::<api::balances::events::Transfer>()?
					.expect("Transfer event to be emitted");
				break;
			}
		}

		// Create a new vault and these assets to it.
		let vault_id = 0;
		let update_vault_reward_config = api::tx().sudo().sudo(
			api::runtime_types::tangle_testnet_runtime::RuntimeCall::Rewards(
				api::runtime_types::pallet_rewards::pallet::Call::update_vault_reward_config {
					vault_id,
					new_config:
						api::runtime_types::pallet_rewards::types::RewardConfigForAssetVault {
							apy: api::runtime_types::sp_arithmetic::per_things::Percent(1),
							incentive_cap: 100_000_000_000_000,
							deposit_cap: 100_000_000_000_000,
							boost_multiplier: None,
						},
				},
			),
		);

		let mut result = subxt
			.tx()
			.sign_and_submit_then_watch_default(
				&update_vault_reward_config,
				&alice.substrate_signer(),
			)
			.await?;

		while let Some(Ok(s)) = result.next().await {
			if let TxStatus::InBestBlock(b) = s {
				let evs = match b.wait_for_success().await {
					Ok(evs) => evs,
					Err(e) => {
						error!("Error: {:?}", e);
						break;
					},
				};
				evs.find_first::<api::rewards::events::VaultRewardConfigUpdated>()?
					.expect("VaultRewardConfigUpdated event to be emitted");
				break;
			}
		}

		let add_asset_to_vault = |x| {
			api::tx()
				.sudo()
				.sudo(api::runtime_types::tangle_testnet_runtime::RuntimeCall::Rewards(
					api::runtime_types::pallet_rewards::pallet::Call::manage_asset_reward_vault {
						vault_id,
						asset_id: x,
						action: api::runtime_types::pallet_rewards::types::AssetAction::Add,
					},
				))
		};
		let assets = [
			Asset::Erc20((<[u8; 20]>::from(usdc_addr)).into()),
			Asset::Erc20((<[u8; 20]>::from(weth_addr)).into()),
			Asset::Erc20((<[u8; 20]>::from(wbtc_addr)).into()),
			Asset::Custom(0),
			Asset::Custom(1),
			Asset::Custom(2),
		];
		for asset_id in assets {
			let mut result = subxt
				.tx()
				.sign_and_submit_then_watch_default(
					&add_asset_to_vault(asset_id),
					&alice.substrate_signer(),
				)
				.await?;

			while let Some(Ok(s)) = result.next().await {
				if let TxStatus::InBestBlock(b) = s {
					let evs = match b.wait_for_success().await {
						Ok(evs) => evs,
						Err(e) => {
							error!("Error: {:?}", e);
							break;
						},
					};
					evs.find_first::<api::rewards::events::AssetUpdatedInVault>()?
						.expect("AssetRewardVault event to be emitted");
					break;
				}
			}
		}

		let test_inputs = TestInputs {
			provider,
			subxt,
			pallet_account_id,
			usdc: usdc_addr,
			weth: weth_addr,
			wbtc: wbtc_addr,
			usdc_asset_id: 0,
			weth_asset_id: 1,
			wbtc_asset_id: 2,
		};
		let result = f(test_inputs).await;
		assert!(result.is_ok(), "Test failed: {result:?}");
		if result.is_ok() {
			info!("***************** Test passed **********");
		} else {
			error!("***************** Test failed **********");
		}
		result
	});
}

/// Test inputs for the E2E test.
#[allow(dead_code)]
pub struct TestInputs {
	/// The Alloy provider.
	provider: AlloyProvider,
	/// The Subxt client.
	subxt: subxt::OnlineClient<subxt::PolkadotConfig>,
	/// The MAD pallet account ID.
	pallet_account_id: subxt::utils::AccountId32,
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

/// Helper function for joining as an operator
async fn join_as_operator(provider: &AlloyProviderWithWallet, stake: U256) -> anyhow::Result<bool> {
	let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, provider);
	let result = precompile
		.joinOperators(stake)
		.send()
		.await?
		.with_timeout(Some(Duration::from_secs(5)))
		.get_receipt()
		.await?;
	Ok(result.status())
}

#[test]
fn operator_join_delegator_delegate_erc20() {
	run_mad_test(|t| async move {
		let alice = TestAccount::Alice;
		let alice_provider = alloy_provider_with_wallet(&t.provider, alice.evm_wallet());
		// Join operators
		let tnt = U256::from(100_000u128);
		assert!(join_as_operator(&alice_provider, tnt).await?);

		let operator_key = api::storage()
			.multi_asset_delegation()
			.operators(alice.address().to_account_id());
		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.map(|p| p.stake), Some(tnt.to::<u128>()));

		// Setup Bob as delegator
		let bob = TestAccount::Bob;
		let bob_provider = alloy_provider_with_wallet(&t.provider, bob.evm_wallet());
		let usdc = MockERC20::new(t.usdc, &bob_provider);

		// Mint USDC for Bob
		let mint_amount = U256::from(100_000_000u128);
		usdc.mint(bob.address(), mint_amount).send().await?.get_receipt().await?;

		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		assert_eq!(bob_balance._0, mint_amount);

		// Delegate assets
		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(U256::from(2));

		// Deposit and delegate
		let deposit_result = precompile
			.deposit(U256::ZERO, *usdc.address(), delegate_amount, 0)
			.from(bob.address())
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());

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

		// Verify state
		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.as_ref().map(|p| p.delegation_count), Some(1));
		assert_eq!(
			maybe_operator.map(|p| p.delegations.0[0].clone()),
			Some(DelegatorBond {
				delegator: bob.address().to_account_id(),
				amount: delegate_amount.to::<u128>(),
				asset_id: Asset::Erc20((<[u8; 20]>::from(*usdc.address())).into()),
				__ignore: std::marker::PhantomData
			})
		);

		anyhow::Ok(())
	});
}

#[test]
fn operator_join_delegator_delegate_asset_id() {
	run_mad_test(|t| async move {
		let alice = TestAccount::Alice;
		let alice_provider = alloy_provider_with_wallet(&t.provider, alice.evm_wallet());

		// Join operators
		let tnt = U256::from(100_000u128);
		assert!(join_as_operator(&alice_provider, tnt).await?);

		let operator_key = api::storage()
			.multi_asset_delegation()
			.operators(alice.address().to_account_id());
		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.map(|p| p.stake), Some(tnt.to::<u128>()));

		// Setup Bob as delegator
		let bob = TestAccount::Bob;
		let bob_provider = alloy_provider_with_wallet(&t.provider, bob.evm_wallet());

		// Mint USDC for Bob using asset ID
		let mint_amount = 100_000_000u128;
		let mint_call = |who| api::tx().assets().mint(t.usdc_asset_id, who, mint_amount);

		info!("Minting {mint_amount} USDC for Bob");

		let mut result = t
			.subxt
			.tx()
			.sign_and_submit_then_watch_default(
				&mint_call(bob.address().to_account_id().into()),
				&alice.substrate_signer(),
			)
			.await?;
		while let Some(Ok(s)) = result.next().await {
			if let TxStatus::InBestBlock(b) = s {
				let evs = match b.wait_for_success().await {
					Ok(evs) => evs,
					Err(e) => {
						error!("Error: {:?}", e);
						break;
					},
				};
				evs.find_first::<api::assets::events::Issued>()?
					.expect("Issued event to be emitted");
				info!("Minted {mint_amount} USDC for Bob");
				break;
			}
		}

		// Delegate assets
		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(2);

		// Deposit and delegate using asset ID
		let deposit_result = precompile
			.deposit(U256::from(t.usdc_asset_id), Address::ZERO, U256::from(delegate_amount), 0)
			.from(bob.address())
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());

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

		// Verify state
		let maybe_operator = t.subxt.storage().at_latest().await?.fetch(&operator_key).await?;
		assert!(maybe_operator.is_some());
		assert_eq!(maybe_operator.as_ref().map(|p| p.delegation_count), Some(1));
		assert_eq!(
			maybe_operator.map(|p| p.delegations.0[0].clone()),
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

#[test]
fn deposits_withdraw_erc20() {
	run_mad_test(|t| async move {
		// Setup Bob as delegator
		let bob = TestAccount::Bob;
		let bob_provider = alloy_provider_with_wallet(&t.provider, bob.evm_wallet());
		let usdc = MockERC20::new(t.usdc, &bob_provider);

		// Mint USDC for Bob
		let mint_amount = U256::from(100_000_000u128);
		usdc.mint(bob.address(), mint_amount).send().await?.get_receipt().await?;

		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		assert_eq!(bob_balance._0, mint_amount);

		// Delegate assets
		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(U256::from(2));

		let multiplier = 0;
		// Deposit and delegate
		let deposit_result = precompile
			.deposit(U256::ZERO, *usdc.address(), delegate_amount, multiplier)
			.from(bob.address())
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());

		let withdraw_amount = delegate_amount.div(U256::from(2));
		// Schedule a withdrawal
		let sch_withdraw_result = precompile
			.scheduleWithdraw(U256::ZERO, *usdc.address(), withdraw_amount)
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(sch_withdraw_result.status());

		// Wait for two new sessions to happen
		let session_index = wait_for_next_session(&t.subxt).await?;
		info!("New session started: {}", session_index);

		// Execute the withdrawal
		let exec_withdraw_result = precompile
			.executeWithdraw()
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;

		assert!(exec_withdraw_result.status());

		// Bob deposited `delegate_amount` and withdrew `withdraw_amount`
		// `delegate_amount` is 1/2 of the minted amount
		// `withdraw_amount` is 1/2 of the deposited amount
		// So, Bob should have `mint_amount - delegate_amount + withdraw_amount` USDC
		let expected_balance = mint_amount - delegate_amount + withdraw_amount;
		let bob_balance = usdc.balanceOf(bob.address()).call().await?;
		assert_eq!(bob_balance._0, expected_balance);

		anyhow::Ok(())
	})
}

#[test]
fn deposits_withdraw_asset_id() {
	run_mad_test(|t| async move {
		let alice = TestAccount::Alice;
		// Setup Bob as delegator
		let bob = TestAccount::Bob;
		let bob_provider = alloy_provider_with_wallet(&t.provider, bob.evm_wallet());

		// Mint USDC for Bob
		let mint_amount = U256::from(100_000_000u128);
		let mint_call = api::tx().assets().mint(
			t.usdc_asset_id,
			bob.address().to_account_id().into(),
			mint_amount.to::<u128>(),
		);

		let mut result = t
			.subxt
			.tx()
			.sign_and_submit_then_watch_default(&mint_call, &alice.substrate_signer())
			.await?;

		while let Some(Ok(s)) = result.next().await {
			if let TxStatus::InBestBlock(b) = s {
				let evs = match b.wait_for_success().await {
					Ok(evs) => evs,
					Err(e) => {
						error!("Error: {:?}", e);
						break;
					},
				};
				evs.find_first::<api::assets::events::Issued>()?
					.expect("Issued event to be emitted");
				break;
			}
		}

		// Delegate assets
		let precompile = MultiAssetDelegation::new(MULTI_ASSET_DELEGATION, &bob_provider);
		let delegate_amount = mint_amount.div(U256::from(2));

		let multiplier = 0;
		// Deposit and delegate
		let deposit_result = precompile
			.deposit(U256::from(t.usdc_asset_id), Address::ZERO, delegate_amount, multiplier)
			.from(bob.address())
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(deposit_result.status());

		let withdraw_amount = delegate_amount.div(U256::from(2));
		// Schedule a withdrawal
		let sch_withdraw_result = precompile
			.scheduleWithdraw(U256::from(t.usdc_asset_id), Address::ZERO, withdraw_amount)
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;
		assert!(sch_withdraw_result.status());

		// Wait for two new sessions to happen
		let session_index = wait_for_next_session(&t.subxt).await?;
		info!("New session started: {}", session_index);

		// Execute the withdrawal
		let exec_withdraw_result = precompile
			.executeWithdraw()
			.send()
			.await?
			.with_timeout(Some(Duration::from_secs(5)))
			.get_receipt()
			.await?;

		assert!(exec_withdraw_result.status());

		// Bob deposited `delegate_amount` and withdrew `withdraw_amount`
		// `delegate_amount` is 1/2 of the minted amount
		// `withdraw_amount` is 1/2 of the deposited amount
		// So, Bob should have `mint_amount - delegate_amount + withdraw_amount` USDC
		let expected_balance = mint_amount - delegate_amount + withdraw_amount;
		let balance_call =
			api::storage().assets().account(t.usdc_asset_id, bob.address().to_account_id());
		let bob_balance = t.subxt.storage().at_latest().await?.fetch(&balance_call).await?;
		assert_eq!(bob_balance.map(|b| b.balance), Some(expected_balance.to::<u128>()));

		anyhow::Ok(())
	})
}
