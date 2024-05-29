### Sygma Pallets Setup & Test

## Setup

- Setup script should be only used for Testnet setup or local testing environment setup.  

The setup script includes the following extrinsics:
1. Register Domain
2. Register the native asset with Fee handler pallet with supported domains
3. Register the bridging fee for the native asset with supported domains
4. Create the customized assets including asset metadata registration, mint.
5. Register the customized assets with Fee handler pallet with supported domains 
6. Register the bridging fee for the customized assets with supported domains
7. Setup MPC Address

Install the dependencies:
```bash
    cd ./tangle/scripts/sygam-setup && npm i
```

Launch the substrate node and make sure the RPC port is `9944` or using the Env var `PALLETWSENDPOINT`, and set the MPC address in the Env var `MPCADDR` if you want, then run the setup script:
```bash
    node setup.js  
```

What the setup script does?

The setup script will register Domain 1 with ChainID 1 for testing purposes. It will also register the native asset TNT with percentage fee type 
with fee rate as 5%, lower bound is 0 and upper bound is 1000 TNT. A customized asset named SygUSD will be created and mint to Alice with AssetID `2000`,
it will also be registered with percentage fee type with the same fee rate as TNT. The MPC address will be set by Env var `MPCADDR` so make sure you set the desired
ECDSA address before the execution of the script.

MPC address is optional when setting up Sygma pallets. Normally it will be the last thing to set up because once set up, all registered Domains will be unpaused and the bridge will start accepting the bridging requests.

Alice will be the super admin in the testnet runtime which means Alice has permission to call any admin level extrinsics in `SygmaAccessSegregator` pallet.

## Test

Sygma pallets provides two main functionalists: `Deposit` and `ProposalExecution`.

`Deposit` means sending bridging request from the current chain to other chain  
`ProposalExecution` means accepting bridging request from other chain

Here are two deposit encode data which send TNT bridging request and SygUSD bridging request respectively:

TNT deposit:
`0x30050000000f0000c16ff28623000306057379676d6100000000000000000000000000000000000000000000000000000005040614ffffffff0000000000000000000000000000066f000000000000000000000000`

SygUSD deposit:
`0x3005010300a10f06057379676d6100000000000000000000000000000000000000000000000000000006067379677573640000000000000000000000000000000000000000000000000000000b0040e59c3012000306057379676d6100000000000000000000000000000000000000000000000000000005040614ffffffff0000000000000000000000000000066f000000000000000000000000`

To accepting the bridging request, there is a script to test this functionality because it requires the EIP712 typed data and MPC address to verify the dummy signature, make sure you set up MPC address with `0x1c5541a79acc662ab2d2647f3b141a3b7cdb2ae4` otherwise the verification will fail.

Launch the substrate node and then run the proposal execution test script:
```bash
    node execute_proposal_test.js
```

For all other Sygma functionalities, you can construct the extrinsic on PJA.  

You should see the sygma related substrate events in the explorer when sending the extrinsic.


