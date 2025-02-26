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
}
