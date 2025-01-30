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
        return this.keyPair.uri || '';
    }

    public getKeyPair(): KeyringPair {
        return this.keyPair;
    }

    public async updateBalance(api: ApiPromise): Promise<void> {
        const { data: balance } = await api.query.system.account(this.address);
        this.balance = BigInt(balance.free.toString());
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
}
