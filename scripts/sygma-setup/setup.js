require('dotenv').config();

const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {cryptoWaitReady} = require('@polkadot/util-crypto');
const {
    setBalance,
    setFeeHandler,
    setMpcAddress,
    registerDomain,
    setFee,
    setFeeRate,
    getNativeAssetId,
    getPHAAssetId,
    createAsset,
    setAssetMetadata,
    mintAsset,
    getSygUSDAssetId,
    queryBridgePauseStatus
} = require("./util");

const BN = require('bn.js');
const bn1e6 = new BN(10).pow(new BN(6));
const bn1e12 = new BN(10).pow(new BN(12));
const bn1e18 = new BN(10).pow(new BN(18));

const feeHandlerType = {
    BasicFeeHandler: "BasicFeeHandler",
    PercentageFeeHandler: "PercentageFeeHandler",
    DynamicFeeHandler: "DynamicFeeHandler"
}

const supportedDestDomains = [
    {
        domainID: 1,
        chainID: 1
    }
]

// those accounts are configured in the substrate-node runtime, and are only applicable for sygma pallet standalone node,
const FeeReserveAccountAddress = "5ELLU7ibt5ZrNEYRwohtaRBDBa3TzcWwwPELBPSWWd2mbgv3";
const NativeTokenTransferReserveAccount = "5EYCAe5jLbHcAAMKvLFSXgCTbPrLgBJusvPwfKcaKzuf5X5e";
const OtherTokenTransferReserveAccount = "5EYCAe5jLbHcAAMKvLFiGhk3htXY8jQncbLTDGJQnpnPMAVp";

async function main() {
    const sygmaPalletProvider = new WsProvider(process.env.PALLETWSENDPOINT || 'ws://127.0.0.1:9944');
    const api = await ApiPromise.create({
        provider: sygmaPalletProvider,
    });

    await cryptoWaitReady();
    const keyring = new Keyring({type: 'sr25519'});
    const sudo = keyring.addFromUri('//Alice');
    const basicFeeAmount = bn1e18.mul(new BN(1)); // 1 * 10 ** 18
    const percentageFeeRate = 500; // 5%
    const feeRateLowerBound = 0;
    const feeRateUpperBound = bn1e18.mul(new BN(1000)); // 1000 * 10 ** 18
    const mpcAddr = process.env.MPCADDR;

    // register dest domains
    for (const domain of supportedDestDomains) {
        await registerDomain(api, domain.domainID, domain.chainID, true, sudo);
    }

    // set fee rate for native asset for domains
    for (const domain of supportedDestDomains) {
        await setFeeHandler(api, domain.domainID, getNativeAssetId(api), feeHandlerType.PercentageFeeHandler, true, sudo)
        await setFeeRate(api, domain.domainID, getNativeAssetId(api), percentageFeeRate, feeRateLowerBound, feeRateUpperBound, true, sudo);
    }

    // create SygUSD test asset (non-reserved foreign asset)
    // SygUSDAssetId: AssetId defined in runtime.rs
    const sygUSDAssetID = 2000;
    const sygUSDAdmin = sudo.address;
    const sygUSDMinBalance = 1;
    const sygUSDName = "sygUSD";
    const sygUSDSymbol = "sygUSD";
    const sygUSDDecimal = 6;
    await createAsset(api, sygUSDAssetID, sygUSDAdmin, sygUSDMinBalance, true, sudo);
    await setAssetMetadata(api, sygUSDAssetID, sygUSDName, sygUSDSymbol, sygUSDDecimal, true, sudo);
    await mintAsset(api, sygUSDAssetID, sygUSDAdmin, bn1e6.mul(new BN(100)), true, sudo); // mint 100 sygUSD to Alice

    // create PHA test asset (reserved foreign asset)
    // PHAAssetId: AssetId defined in runtime.rs
    const PHAAssetID = 2001;
    const PHAAdmin = sudo.address;
    const PHAMinBalance = 1;
    const PHAName = "PHA";
    const PHASymbol = "PHA";
    const PHADecimal = 12;
    await createAsset(api, PHAAssetID, PHAAdmin, PHAMinBalance, true, sudo);
    await setAssetMetadata(api, PHAAssetID, PHAName, PHASymbol, PHADecimal, true, sudo);
    await mintAsset(api, PHAAssetID, PHAAdmin, bn1e12.mul(new BN(100)), true, sudo); // mint 100 PHA to Alice

    // set fee for tokens with domains
    for (const domain of supportedDestDomains) {
        await setFeeHandler(api, domain.domainID, getSygUSDAssetId(api), feeHandlerType.PercentageFeeHandler, true, sudo)
        await setFeeRate(api, domain.domainID, getSygUSDAssetId(api), percentageFeeRate, feeRateLowerBound, feeRateUpperBound,true, sudo);

        await setFeeHandler(api, domain.domainID, getPHAAssetId(api), feeHandlerType.PercentageFeeHandler, true, sudo)
        await setFeeRate(api, domain.domainID, getPHAAssetId(api), percentageFeeRate, feeRateLowerBound, feeRateUpperBound,true, sudo);
    }

    // transfer some native asset to FeeReserveAccount and TransferReserveAccount as Existential Deposit(aka ED)
    await setBalance(api, FeeReserveAccountAddress, bn1e18.mul(new BN(10000)), true, sudo); // set balance to 10000 native asset
    await setBalance(api, NativeTokenTransferReserveAccount, bn1e18.mul(new BN(10000)), true, sudo); // set balance to 10000 native asset
    await setBalance(api, OtherTokenTransferReserveAccount, bn1e18.mul(new BN(10000)), true, sudo); // set balance to 10000 native asset

    // set up MPC address(will also unpause all registered domains)
    if (mpcAddr) {
        console.log(`set up mpc address: ${mpcAddr}`);
        await setMpcAddress(api, mpcAddr, true, sudo);
        // bridge should be unpaused by the end of the setup
        for (const domain of supportedDestDomains) {
            if (!await queryBridgePauseStatus(api, domain.domainID)) console.log(`DestDomainID: ${domain.domainID} is ready✅`);
        }
    }

    console.log('🚀 Sygma substrate pallet setup is done! 🚀');

    // It is unnecessary to set up access segregator here since ALICE will be the sudo account and all methods with access control logic are already setup in this script.
    // so that on Relayer, E2E test only cares about public extrinsic such as deposit, executionProposal, retry .etc
}

main().catch(console.error).finally(() => process.exit());
