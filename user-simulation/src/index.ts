import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import * as dotenv from 'dotenv';

dotenv.config();

class UserSimulation {
    private api: ApiPromise | null = null;
    private users: any[] = [];
    private keyring: Keyring | null = null;

    async initialize() {
        // Wait for the crypto to be ready
        await cryptoWaitReady();
        
        // Initialize the keyring
        this.keyring = new Keyring({ type: 'sr25519' });

        // Connect to the Tangle network
        const wsProvider = new WsProvider('ws://127.0.0.1:9944');
        this.api = await ApiPromise.create({ provider: wsProvider });

        console.log('Connected to Tangle Network');
    }

    async createUsers(count: number) {
        if (!this.keyring) throw new Error('Keyring not initialized');

        for (let i = 0; i < count; i++) {
            const user = this.keyring.addFromUri(`//User${i}`);
            this.users.push(user);
            console.log(`Created user ${i} with address: ${user.address}`);
        }
    }

    // Add more simulation methods here
}

async function main() {
    const simulation = new UserSimulation();
    try {
        await simulation.initialize();
        await simulation.createUsers(5); // Start with 5 users
        
        // Add your simulation logic here
        
    } catch (error) {
        console.error('Error in simulation:', error);
    }
}

main().catch(console.error);
