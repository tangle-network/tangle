import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import * as dotenv from 'dotenv';
import { User } from './types/user';

dotenv.config();

class UserSimulation {
    private api: ApiPromise | null = null;
    private users: User[] = [];
    private keyring: Keyring | null = null;

    async initialize(wsEndpoint: string = process.env.NODE_URL || 'ws://127.0.0.1:9944') {
        try {
            // Wait for the crypto to be ready
            await cryptoWaitReady();
            
            // Initialize the keyring
            this.keyring = new Keyring({ type: 'sr25519' });

            // Connect to the Tangle network
            const wsProvider = new WsProvider(wsEndpoint);
            this.api = await ApiPromise.create({ provider: wsProvider });

            console.log('Connected to Tangle Network');
            return true;
        } catch (error) {
            console.error('Failed to initialize:', error);
            return false;
        }
    }

    async createUsers(count: number): Promise<User[]> {
        if (!this.keyring) throw new Error('Keyring not initialized');
        if (!this.api) throw new Error('API not initialized');

        const newUsers: User[] = [];

        for (let i = 0; i < count; i++) {
            const keyPair = this.keyring.addFromUri(`//User${i}`);
            const user = new User(keyPair);
            await user.updateBalance(this.api);
            
            newUsers.push(user);
            this.users.push(user);
            
            console.log(`Created user ${i}:
                Address: ${user.address}
                Private Key: ${user.getPrivateKey()}
                Initial Balance: ${user.balance}`);
        }

        return newUsers;
    }

    async simulateTransactions(numTransactions: number) {
        if (!this.api) throw new Error('API not initialized');
        if (this.users.length < 2) throw new Error('Need at least 2 users for transactions');

        console.log(`Starting transaction simulation with ${numTransactions} transactions...`);

        for (let i = 0; i < numTransactions; i++) {
            const sender = this.users[Math.floor(Math.random() * this.users.length)];
            let recipient;
            do {
                recipient = this.users[Math.floor(Math.random() * this.users.length)];
            } while (recipient.address === sender.address);

            try {
                const amount = BigInt(1000000000); // 1 TOKEN
                const hash = await sender.sendTransaction(this.api, recipient.address, amount);
                console.log(`Transaction ${i + 1}/${numTransactions}:
                    From: ${sender.address}
                    To: ${recipient.address}
                    Amount: ${amount}
                    Hash: ${hash}`);

                // Update balances
                await Promise.all([
                    sender.updateBalance(this.api),
                    recipient.updateBalance(this.api)
                ]);
            } catch (error) {
                console.error(`Transaction ${i + 1} failed:`, error);
            }
        }
    }

    getUsers(): User[] {
        return this.users;
    }
}

async function main() {
    const simulation = new UserSimulation();
    try {
        const initialized = await simulation.initialize();
        if (!initialized) {
            throw new Error('Failed to initialize simulation');
        }

        // Create 5 users
        await simulation.createUsers(5);
        
        // Simulate 10 transactions between users
        await simulation.simulateTransactions(10);
        
    } catch (error) {
        console.error('Error in simulation:', error);
    }
}

if (require.main === module) {
    main().catch(console.error);
}

export { UserSimulation };
