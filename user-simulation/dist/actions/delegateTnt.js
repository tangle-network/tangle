"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DelegateTnt = void 0;
class DelegateTnt {
    async execute(api, keyring, user, assetId, amount, validatorAddress) {
        try {
            console.log(`Delegating ${amount} of asset ${assetId} from user ${user.address} to validator ${validatorAddress}...`);
            // First bond if not already bonded
            const bondTx = api.tx.multiAssetDelegation.bond({ id: assetId }, // Asset ID
            user.address, // Account to bond
            amount, // Amount to bond
            'Staked' // Delegation type
            );
            const bondHash = await bondTx.signAndSend(user.getKeyPair());
            console.log(`Bond submitted with hash: ${bondHash.toHex()}`);
            // Wait for bond to be processed
            await new Promise(resolve => setTimeout(resolve, 12000));
            // Then delegate to the validator
            const delegateTx = api.tx.multiAssetDelegation.delegate({ id: assetId }, // Asset ID
            validatorAddress, // Validator to delegate to
            amount, // Amount to delegate
            'Staked' // Delegation type
            );
            const delegateHash = await delegateTx.signAndSend(user.getKeyPair());
            console.log(`Delegation submitted with hash: ${delegateHash.toHex()}`);
            // Wait for delegation to be processed
            await new Promise(resolve => setTimeout(resolve, 12000));
            // Query the delegation info
            const delegationInfo = await api.query.multiAssetDelegation.ledger(assetId, user.address);
            console.log(`Delegation info: ${delegationInfo.toString()}`);
        }
        catch (error) {
            console.error('Error during asset delegation:', error);
            throw error;
        }
    }
}
exports.DelegateTnt = DelegateTnt;
