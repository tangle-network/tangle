"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const api_1 = require("@polkadot/api");
const keyring_1 = require("@polkadot/keyring");
const util_crypto_1 = require("@polkadot/util-crypto");
const depositTnt_1 = require("./actions/depositTnt");
const delegateTnt_1 = require("./actions/delegateTnt");
const claimRewards_1 = require("./actions/claimRewards");
const user_1 = require("./types/user");
const dotenv = __importStar(require("dotenv"));
dotenv.config();
async function testActions() {
    try {
        // Initialize connection and crypto
        await (0, util_crypto_1.cryptoWaitReady)();
        console.log('Connecting to network...');
        // Connect to Tangle Network
        console.log('Connecting to Tangle Network...');
        const wsProvider = new api_1.WsProvider('wss://rpc.tangle.tools');
        // Create API instance with more detailed error handling
        const api = await api_1.ApiPromise.create({
            provider: wsProvider,
            throwOnConnect: true,
            noInitWarn: true
        }).catch(error => {
            console.error('Failed to create API:', error);
            throw error;
        });
        // Wait for API to be ready
        await api.isReady;
        const keyring = new keyring_1.Keyring({ type: 'sr25519' });
        console.log('Connected to Tangle Network');
        // Create a test user
        const testUser = new user_1.User(keyring.addFromUri('//TestUser1'));
        await testUser.updateBalance(api);
        console.log(`Test user created with address: ${testUser.address}`);
        console.log(`Initial balance: ${testUser.balance}`);
        // Test DepositTnt
        console.log('\n=== Testing DepositTnt ===');
        const depositAction = new depositTnt_1.DepositTnt();
        const depositAmount = BigInt(1000000000); // 1 TNT
        await depositAction.execute(api, keyring, testUser, 1, depositAmount);
        // Wait for the transaction to be included in a block
        await new Promise(resolve => setTimeout(resolve, 6000));
        // Test DelegateTnt
        console.log('\n=== Testing DelegateTnt ===');
        const delegateAction = new delegateTnt_1.DelegateTnt();
        // You'll need to replace this with an actual validator address from your testnet
        const validatorAddress = process.env.VALIDATOR_ADDRESS || '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
        await delegateAction.execute(api, keyring, testUser, 1, depositAmount, validatorAddress);
        // Wait for the transaction to be included in a block
        await new Promise(resolve => setTimeout(resolve, 6000));
        // Test ClaimRewards
        console.log('\n=== Testing ClaimRewards ===');
        const claimAction = new claimRewards_1.ClaimRewards();
        await claimAction.execute(api, keyring, testUser);
        // Wait for final balance update
        await new Promise(resolve => setTimeout(resolve, 6000));
        await testUser.updateBalance(api);
        console.log(`Final balance: ${testUser.balance}`);
        // Cleanup
        await api.disconnect();
        console.log('\nAll tests completed!');
    }
    catch (error) {
        console.error('Error during testing:', error);
        process.exit(1);
    }
}
// Run the tests
testActions().catch(console.error);
