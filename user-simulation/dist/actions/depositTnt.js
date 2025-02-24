"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DepositTnt = void 0;
class DepositTnt {
    async execute(api, keyring, user, assetId, amount) {
        try {
            console.log(`Depositing ${amount} of asset ${assetId} for user ${user.address}...`);
            // Create the deposit transaction
            const deposit = api.tx.multiAssetDelegation.bond({ id: assetId }, // Asset ID
            user.address, // Account to bond
            amount, // Amount to bond
            'Staked' // Delegation type
            );
            // Sign and submit the transaction
            const hash = await deposit.signAndSend(user.getKeyPair());
            console.log(`Deposit submitted with hash: ${hash.toHex()}`);
            // Wait for 2 blocks to ensure the transaction is processed
            await new Promise(resolve => setTimeout(resolve, 12000));
            // Query the delegation info
            const delegationInfo = await api.query.multiAssetDelegation.ledger(assetId, user.address);
            console.log(`Delegation info: ${delegationInfo.toString()}`);
        }
        catch (error) {
            console.error('Error during asset deposit:', error);
            throw error;
        }
    }
}
exports.DepositTnt = DepositTnt;
