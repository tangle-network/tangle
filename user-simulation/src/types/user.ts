import { KeyringPair } from '@polkadot/keyring/types';
import { ApiPromise } from '@polkadot/api';

export class User {
    private keyPair: KeyringPair;
    public address: string;
    public balance: bigint;

    constructor(keyPair: KeyringPair) {
        this.keyPair = keyPair;
        this.address = keyPair.address;
        this.balance = BigInt(0);
    }

    public getPrivateKey(): string {
        // For sr25519 keypairs, we can access the seed/private key through the meta
        if (this.keyPair.meta && typeof this.keyPair.meta.suri === 'string') {
            return this.keyPair.meta.suri;
        }
        // Fallback to using the address if no private key is available
        return `${this.keyPair.type}-${this.keyPair.address}`;
    }

    public getKeyPair(): KeyringPair {
        return this.keyPair;
    }

    public async updateBalance(api: ApiPromise): Promise<void> {
        const accountData = await api.query.system.account(this.address);
        const free = (accountData as any).data.free;
        this.balance = BigInt(free.toString());
    }

    public async sendTransaction(api: ApiPromise, recipient: string, amount: bigint): Promise<string> {
        try {
            const transfer = api.tx.balances.transfer(recipient, amount);
            const hash = await transfer.signAndSend(this.keyPair);
            return hash.toString();
        } catch (error) {
            console.error(`Transaction failed for user ${this.address}:`, error);
            throw error;
        }
    }

    public async depositTnt(api: ApiPromise, amount: bigint): Promise<string> {
        try {
            // Call the deposit function in the staking pallet
            const deposit = api.tx.multiAssetDelegation.bond(this.address, amount, 'Staked');
            const hash = await deposit.signAndSend(this.keyPair);
            return hash.toString();
        } catch (error) {
            console.error(`Deposit failed for user ${this.address}:`, error);
            throw error;
        }
    }

    public async delegateTnt(api: ApiPromise, validatorAddress: string, amount: bigint): Promise<string> {
        try {
            // First bond if not already bonded
            const bondTx = api.tx.multiAssetDelegation.bond(this.address, amount, 'Staked');
            await bondTx.signAndSend(this.keyPair);

            // Then nominate a validator
            const nominateTx = api.tx.staking.nominate([validatorAddress]);
            const hash = await nominateTx.signAndSend(this.keyPair);
            return hash.toString();
        } catch (error) {
            console.error(`Delegation failed for user ${this.address}:`, error);
            throw error;
        }
    }

    public async claimRewards(api: ApiPromise): Promise<string> {
        try {
            // Claim rewards from the rewards pallet
            const claimTx = api.tx.rewards.claim();
            const hash = await claimTx.signAndSend(this.keyPair);
            return hash.toString();
        } catch (error) {
            console.error(`Claiming rewards failed for user ${this.address}:`, error);
            throw error;
        }
    }
}
