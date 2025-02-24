"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.User = void 0;
class User {
    constructor(keyPair) {
        this.keyPair = keyPair;
        this.address = keyPair.address;
        this.balance = BigInt(0);
    }
    getPrivateKey() {
        // For sr25519 keypairs, we can access the seed/private key through the meta
        if (this.keyPair.meta && typeof this.keyPair.meta.suri === 'string') {
            return this.keyPair.meta.suri;
        }
        // Fallback to using the address if no private key is available
        return `${this.keyPair.type}-${this.keyPair.address}`;
    }
    getKeyPair() {
        return this.keyPair;
    }
    async updateBalance(api) {
        const accountData = await api.query.system.account(this.address);
        const free = accountData.data.free;
        this.balance = BigInt(free.toString());
    }
    async sendTransaction(api, recipient, amount) {
        try {
            const transfer = api.tx.balances.transfer(recipient, amount);
            const hash = await transfer.signAndSend(this.keyPair);
            return hash.toString();
        }
        catch (error) {
            console.error(`Transaction failed for user ${this.address}:`, error);
            throw error;
        }
    }
}
exports.User = User;
