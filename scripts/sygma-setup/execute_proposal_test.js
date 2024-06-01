require('dotenv').config();

const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {cryptoWaitReady} = require('@polkadot/util-crypto');
const {
    executeProposal,
    queryAssetBalance,
    queryBalance,
    queryMPCAddress,
    subEvents,
} = require("./util");

// these are the dummy proposal that used to verify if proposal execution works on pallet
// bridge amount from relayer is 100000000000000
const proposal_native = {
    origin_domain_id: 1,
    deposit_nonce: 3,
    resource_id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 90, 243, 16, 122, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 1, 1, 0, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]
}

// signer is the mpc address 0x1c5541A79AcC662ab2D2647F3B141a3B7Cdb2Ae4
const signature_native = [57, 218, 225, 125, 128, 217, 23, 82, 49, 217, 8, 197, 110, 174, 42, 157, 129, 43, 22, 63, 215, 213, 100, 179, 17, 170, 23, 95, 72, 80, 78, 181, 108, 176, 60, 138, 137, 29, 157, 138, 244, 0, 5, 180, 128, 243, 48, 99, 175, 53, 140, 245, 162, 111, 36, 65, 89, 208, 41, 69, 209, 149, 247, 149, 28];
const mpcAddress = "0x1c5541a79acc662ab2d2647f3b141a3b7cdb2ae4";
const aliceAddress = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

async function main() {
    const sygmaPalletProvider = new WsProvider(process.env.PALLETWSENDPOINT || 'ws://127.0.0.1:9944');
    const api = await ApiPromise.create({
        provider: sygmaPalletProvider,
    });

    await cryptoWaitReady();
    const keyring = new Keyring({type: 'sr25519'});
    const sudo = keyring.addFromUri('//Alice');

    // make sure mpc address matches
    const registeredMpcAddr = await queryMPCAddress(api);
    if (registeredMpcAddr !== mpcAddress) {
        throw Error("mpc address not match")
    }

    console.log(`sudo address ${sudo.address}`)

    const events = [];
    await subEvents(api, events);

    // Native asset
    const nativeBalanceBefore = await queryBalance(api, aliceAddress);
    console.log('native asset balance before: ', BigInt(nativeBalanceBefore.data.free.replaceAll(',', '')));
    await executeProposal(api, [proposal_native], signature_native, true, sudo);
    const nativeBalanceAfter = await queryBalance(api, aliceAddress);
    console.log('native asset balance after: ', BigInt(nativeBalanceAfter.data.free.replaceAll(',', '')));

    // checking if any sygma events emitted
    for (const e of events) {
        console.log('sygma pallets event emitted: \x1b[32m%s\x1b[0m\n', e);
    }

    if (events.includes("sygmaBridge:ProposalExecution")) {
        console.log('proposal execution test pass✅');
        return
    }

    const errMsg = "proposal execution test failed"
    throw Error(`\x1b[31m${errMsg}\x1b[0m`);
}

main().catch(console.error).finally(() => process.exit());
