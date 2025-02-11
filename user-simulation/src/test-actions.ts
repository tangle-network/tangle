import { ApiPromise, WsProvider } from '@polkadot/api';

import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { DepositTnt } from './actions/depositTnt';
import { DelegateTnt } from './actions/delegateTnt';
import { ClaimRewards } from './actions/claimRewards';
import { User } from './types/user';
import * as dotenv from 'dotenv';

dotenv.config();

async function testActions() {
    try {
        // Initialize connection and crypto
        await cryptoWaitReady();
        console.log('Connecting to network...');
        
        // Connect to Tangle Network
        console.log('Connecting to Tangle Network...');
        const wsProvider = new WsProvider('wss://rpc.tangle.tools');
        
        // Create API instance with more detailed error handling
        const api = await ApiPromise.create({ 
            provider: wsProvider,
            throwOnConnect: true,
            noInitWarn: true
        }).catch(error => {
            console.error('Failed to create API:', error);
            throw error;
        });

        // Wait for API to be ready
        await api.isReady;
        const keyring = new Keyring({ type: 'sr25519' });

        console.log('Connected to Tangle Network');

        // Create a test user
        const testUser = new User(keyring.addFromUri('//TestUser1'));
        await testUser.updateBalance(api);
        console.log(`Test user created with address: ${testUser.address}`);
        console.log(`Initial balance: ${testUser.balance}`);

        // Test DepositTnt
        console.log('\n=== Testing DepositTnt ===');
        const depositAction = new DepositTnt();
        const depositAmount = BigInt(1_000_000_000); // 1 TNT
        await depositAction.execute(api, keyring, testUser, depositAmount);
        
        // Wait for the transaction to be included in a block
        await new Promise(resolve => setTimeout(resolve, 6000));
        
        // Test DelegateTnt
        console.log('\n=== Testing DelegateTnt ===');
        const delegateAction = new DelegateTnt();
        // You'll need to replace this with an actual validator address from your testnet
        const validatorAddress = process.env.VALIDATOR_ADDRESS || '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
        await delegateAction.execute(api, keyring, testUser, depositAmount, validatorAddress);
        
        // Wait for the transaction to be included in a block
        await new Promise(resolve => setTimeout(resolve, 6000));

        // Test ClaimRewards
        console.log('\n=== Testing ClaimRewards ===');
        const claimAction = new ClaimRewards();
        await claimAction.execute(api, keyring, testUser);
        
        // Wait for final balance update
        await new Promise(resolve => setTimeout(resolve, 6000));
        await testUser.updateBalance(api);
        console.log(`Final balance: ${testUser.balance}`);

        // Cleanup
        await api.disconnect();
        console.log('\nAll tests completed!');
    } catch (error) {
        console.error('Error during testing:', error);
        process.exit(1);
    }
}

// Run the tests
testActions().catch(console.error);
