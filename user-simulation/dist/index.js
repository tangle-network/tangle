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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.UserSimulation = void 0;
const api_1 = require("@polkadot/api");
const keyring_1 = require("@polkadot/keyring");
const util_crypto_1 = require("@polkadot/util-crypto");
const dotenv = __importStar(require("dotenv"));
const user_1 = require("./types/user");
const yargs_1 = __importDefault(require("yargs"));
const helpers_1 = require("yargs/helpers");
const generateChildUsers_1 = require("./actions/generateChildUsers");
const depositTnt_1 = require("./actions/depositTnt");
const delegateTnt_1 = require("./actions/delegateTnt");
const claimRewards_1 = require("./actions/claimRewards");
const transferAssets_1 = require("./actions/transferAssets");
dotenv.config();
class UserSimulation {
    constructor() {
        this.api = null;
        this.users = [];
        this.keyring = null;
    }
    async initialize(wsEndpoint = process.env.NODE_URL || 'wss://testnet-rpc.tangle.tools') {
        try {
            // Wait for the crypto to be ready
            await (0, util_crypto_1.cryptoWaitReady)();
            // Initialize the keyring
            this.keyring = new keyring_1.Keyring({ type: 'sr25519' });
            // Connect to the Tangle network
            console.log('Connecting to Tangle Network at:', wsEndpoint);
            const wsProvider = new api_1.WsProvider(wsEndpoint);
            // Wait for the provider to be connected
            await new Promise((resolve, reject) => {
                wsProvider.on('connected', () => {
                    console.log('WebSocket connected successfully');
                    resolve();
                });
                wsProvider.on('error', (error) => {
                    console.error('WebSocket connection error:', error);
                    reject(error);
                });
            });
            this.api = await api_1.ApiPromise.create({
                provider: wsProvider,
                throwOnConnect: true
            });
            // Wait for the API to be ready
            await this.api.isReady;
            console.log('Connected to Tangle Network');
            return true;
        }
        catch (error) {
            console.error('Failed to initialize:', error);
            return false;
        }
    }
    async createUsers(count) {
        if (!this.keyring)
            throw new Error('Keyring not initialized');
        if (!this.api)
            throw new Error('API not initialized');
        const newUsers = [];
        for (let i = 0; i < count; i++) {
            const keyPair = this.keyring.addFromUri(`//User${i}`);
            const user = new user_1.User(keyPair);
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
    async simulateTransactions(numTransactions) {
        if (!this.api)
            throw new Error('API not initialized');
        if (this.users.length < 2)
            throw new Error('Need at least 2 users for transactions');
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
            }
            catch (error) {
                console.error(`Transaction ${i + 1} failed:`, error);
            }
        }
    }
    getUsers() {
        return this.users;
    }
    getApi() {
        if (!this.api)
            throw new Error('API not initialized');
        return this.api;
    }
    getKeyring() {
        if (!this.keyring)
            throw new Error('Keyring not initialized');
        return this.keyring;
    }
}
exports.UserSimulation = UserSimulation;
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
        const args = await (0, yargs_1.default)((0, helpers_1.hideBin)(process.argv))
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
                const count = args.count;
                if (typeof count === 'number') {
                    const action = new generateChildUsers_1.GenerateChildUsers();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, count);
                }
                break;
            }
            case 'depositTnt': {
                const assetId = args.assetId;
                const amount = args.amount;
                if (typeof assetId === 'number' && typeof amount === 'string') {
                    const action = new depositTnt_1.DepositTnt();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, assetId, BigInt(amount));
                }
                break;
            }
            case 'delegateTnt': {
                const assetId = args.assetId;
                const amount = args.amount;
                const validator = args.validator;
                if (typeof assetId === 'number' && typeof amount === 'string' && typeof validator === 'string') {
                    const action = new delegateTnt_1.DelegateTnt();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser, assetId, BigInt(amount), validator);
                }
                break;
            }
            case 'claimRewards': {
                const action = new claimRewards_1.ClaimRewards();
                await action.execute(simulation.getApi(), simulation.getKeyring(), baseUser);
                break;
            }
            case 'transferAssets': {
                const assetId = args.assetId;
                const seedPhrase = args.seedPhrase;
                const amount = args.amount;
                if (typeof assetId === 'number' && typeof seedPhrase === 'string' && typeof amount === 'string') {
                    const action = new transferAssets_1.TransferAssets();
                    await action.execute(simulation.getApi(), simulation.getKeyring(), assetId, seedPhrase, BigInt(amount));
                }
                break;
            }
            default: {
                console.log('Please specify a valid command and arguments');
                break;
            }
        }
    }
    catch (error) {
        console.error('Error in simulation:', error);
    }
}
if (require.main === module) {
    main().catch(console.error);
}
