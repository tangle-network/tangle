# Pallet Token Gateway

This allows polkadot-sdk chains make cross-chain asset transfers to and from each other as well EVM chains using Hyperbridge.

## Overview

The Pallet allows the [`CreateOrigin`](https://docs.rs/pallet-token-gateway/latest/pallet_token_gateway/pallet/trait.Config.html#associatedtype.CreateOrigin) to dispatch calls for setting token gateway addresses, creating and updating assets.
This enables receiving assets from those configured chains. Assets can also be received with a runtime call to be dispatched. This call must be signed by the beneficiary of the incoming assets. Assets can also be sent with some calldata, this calldata is an opaque blob of bytes
whose interpretation is left up to the recipient token gateway implementation. For polkadot-sdk chains, it must be a scale-encoded runtime call, for EVM chains it must be an abi-encoded contract call.

## Adding to Runtime

The first step is to implement the pallet config for the runtime.

```rust,ignore
use frame_support::parameter_types;
use ismp::module::IsmpModule;
use ismp::router::IsmpRouter;
use pallet_token_gateway::types::NativeAssetLocation;

parameter_types! {
    // Set the correct precision for the native currency
    pub const Decimals: u8 = 12;
}


/// A constant value that represents the native asset
const NativeAssetId: u32 = 0;


/// Should provide an account that is funded and can be used to pay for asset creation
pub struct AssetAdmin;

impl Get<AccountId> for AssetAdmin {
	fn get() -> AccountId {
		Treasury::account_id()
	}
}

impl pallet_token_gateway::Config for Runtime {
    // configure the runtime event
    type RuntimeEvent = RuntimeEvent;
    // pallet_ismp or pallet_hyperbridge
    type Dispatcher = pallet_hyperbridge::Pallet<Runtime>;
    // Pallet Assets
    type Assets = Assets;
    // Pallet balances
    type Currency = Balances;
    // AssetAdmin account
    type AssetAdmin = AssetAdmin;
    // The Native asset Id
    type NativeAssetId = NativeAssetId;
    // The precision of the native asset
    type Decimals = Decimals;
}

// Add the pallet to your ISMP router to receive assets from external chains
#[derive(Default)]
struct Router;
impl IsmpRouter for Router {
    fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
        let module = match id.as_slice() {
            id if TokenGateway::is_token_gateway(&id) => Box::new(TokenGateway::default()),
            _ => Err(Error::ModuleNotFound(id))?
        };
        Ok(module)
    }
}
```

## Setting up

The pallet requires some setting up before the teleport function is available for use in the runtime.

1.  Register your native assets directly on `Hyperbridge` by dispatching `create_erc6160_asset`.
2.  Set token gateway addresses for the EVM chains of interest by dispatching the `set_token_gateway_addresses` extrinsic. These addresses are used to validate incoming messages.

## Dispatchable Functions

-   `teleport` - This function is used to bridge assets through Hyperbridge.
-   `set_token_gateway_addresses` - This call allows the `AdminOrigin` origin to set the token gateway address for supported chains.
-   `create_erc6160_asset` - This call dispatches a request to Hyperbridge to create multi chain native assets on token gateway deployments
-   `update_erc6160_asset` - This priviledged call dispatches a request to Hyperbridge to update multi chain native assets on token gateway deployments
-   `update_asset_precision` - This priviledged call is used to set or update the precision for an asset deployed on a remote chain

## License

This library is licensed under the Apache 2.0 License, Copyright (c) 2025 Polytope Labs.
