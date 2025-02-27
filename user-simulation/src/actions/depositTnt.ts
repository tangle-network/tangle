import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class DepositTnt implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User, assetId: number, amount: bigint): Promise<void> {
        try {
            console.log(`Depositing ${amount} of asset ${assetId} for user ${user.address}...`);
            
            // Create the deposit transaction
            const deposit = api.tx.multiAssetDelegation.bond(
                { id: assetId },  // Asset ID
                user.address,      // Account to bond
                amount,           // Amount to bond
                'Staked'          // Delegation type
            );

            // Sign and submit the transaction
            const hash = await deposit.signAndSend(user.getKeyPair());
            console.log(`Deposit submitted with hash: ${hash.toHex()}`);

            // Wait for 2 blocks to ensure the transaction is processed
            await new Promise(resolve => setTimeout(resolve, 12000));

            // Query the delegation info
            const delegationInfo = await api.query.multiAssetDelegation.ledger(assetId, user.address);
            console.log(`Delegation info: ${delegationInfo.toString()}`);
        } catch (error) {
            console.error('Error during asset deposit:', error);
            throw error;
        }
    }
}
