import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import * as dotenv from 'dotenv';
import { User } from './types/user';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { GenerateChildUsers } from './actions/generateChildUsers';
import { DepositTnt } from './actions/depositTnt';
import { DelegateTnt } from './actions/delegateTnt';
import { ClaimRewards } from './actions/claimRewards';
import { TransferAssets } from './actions/transferAssets';

dotenv.config();

class UserSimulation {
    private api: ApiPromise | null = null;
    private users: User[] = [];
    private keyring: Keyring | null = null;

    async initialize(wsEndpoint: string = process.env.NODE_URL || 'wss://testnet-rpc.tangle.tools') {
        try {
            // Wait for the crypto to be ready
            await cryptoWaitReady();
            
            // Initialize the keyring
            this.keyring = new Keyring({ type: 'sr25519' });

            // Connect to the Tangle network
            console.log('Connecting to Tangle Network at:', wsEndpoint);
            const wsProvider = new WsProvider(wsEndpoint);
            
            // Wait for the provider to be connected
            await new Promise<void>((resolve, reject) => {
                wsProvider.on('connected', () => {
                    console.log('WebSocket connected successfully');
                    resolve();
                });
                wsProvider.on('error', (error) => {
                    console.error('WebSocket connection error:', error);
                    reject(error);
                });
            });

            this.api = await ApiPromise.create({ 
                provider: wsProvider,
                throwOnConnect: true
            });

            // Wait for the API to be ready
            await this.api.isReady;
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

    getApi(): ApiPromise {
        if (!this.api) throw new Error('API not initialized');
        return this.api;
    }

    getKeyring(): Keyring {
        if (!this.keyring) throw new Error('Keyring not initialized');
        return this.keyring;
    }
}

async function main() {
    const simulation = new UserSimulation();
    try {
        const initialized = await simulation.initialize();
        if (!initialized) {
            throw new Error('Failed to initialize simulation');
        }

        // Create users if not already created
        if (simulation.getUsers().length === 0) {
            await simulation.createUsers(1); // Create at least one base user
        }

        const baseUser = simulation.getUsers()[0];
        if (!baseUser) {
            throw new Error('Failed to create base user');
        }

        // Parse command line arguments
        const args = await yargs(hideBin(process.argv))
            .command('generateChildUsers', 'Generate child users from a base user', {
                count: {
                    description: 'Number of child users to generate',
                    alias: 'c',
                    type: 'number',
                    demandOption: true
                }
            })
            .command('depositTnt', 'Deposit tokens for staking', {
                assetId: {
                    description: 'Asset ID to deposit',
                    type: 'number',
                    demandOption: true
                },
                amount: {
                    description: 'Amount to deposit',
                    type: 'string',
                    demandOption: true
                }
            })
            .command('delegateTnt', 'Delegate tokens to a validator', {
                assetId: {
                    description: 'Asset ID to delegate',
                    type: 'number',
                    demandOption: true
                },
                amount: {
                    description: 'Amount to delegate',
                    type: 'string',
                    demandOption: true
                },
                validator: {
                    description: 'Address of the validator to delegate to',
                    type: 'string',
                    demandOption: true
                }
            })
            .command('claimRewards', 'Claim staking rewards')
            .command('transferAssets', 'Transfer assets to all child users', {
                assetId: {
                    description: 'Asset ID to transfer',
                    type: 'number',
                    demandOption: true
                },
                seedPhrase: {
                    description: 'Seed phrase of the source account',
                    type: 'string',
                    demandOption: true
                },
                amount: {
                    description: 'Amount to transfer to each user',
                    type: 'string',
                    demandOption: true
                }
            })
            .help()
            .alias('help', 'h')
            .parseAsync();

        const command = args._[0];
        switch (command) {
            case 'generateChildUsers': {
                const count = args.count as number;
                if (typeof count === 'number') {
                    const action = new GenerateChildUsers();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, count);
                }
                break;
            }
            case 'depositTnt': {
                const assetId = args.assetId as number;
                const amount = args.amount as string;
                if (typeof assetId === 'number' && typeof amount === 'string') {
                    const action = new DepositTnt();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, assetId, BigInt(amount));
                }
                break;
            }
            case 'delegateTnt': {
                const assetId = args.assetId as number;
                const amount = args.amount as string;
                const validator = args.validator as string;
                if (typeof assetId === 'number' && typeof amount === 'string' && typeof validator === 'string') {
                    const action = new DelegateTnt();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, assetId, BigInt(amount), validator);
                }
                break;
            }
            case 'claimRewards': {
                const action = new ClaimRewards();
                await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser);
                break;
            }
            case 'transferAssets': {
                const assetId = args.assetId as number;
                const seedPhrase = args.seedPhrase as string;
                const amount = args.amount as string;
                if (typeof assetId === 'number' && typeof seedPhrase === 'string' && typeof amount === 'string') {
                    const action = new TransferAssets();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), assetId, seedPhrase, BigInt(amount));
                }
                break;
            }
            default: {
                console.log('Please specify a valid command and arguments');
                break;
            }
        }
        
    } catch (error) {
        console.error('Error in simulation:', error);
    }
}

if (require.main === module) {
    main().catch(console.error);
}

export { UserSimulation };
