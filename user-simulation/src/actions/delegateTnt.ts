import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class DelegateTnt implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User, amount: bigint, validatorAddress: string): Promise<void> {
        try {
            console.log(`Delegating ${amount} TNT from user ${user.address} to validator ${validatorAddress}...`);
            // First bond if not already bonded
            const bondTx = api.tx.multiAssetDelegation.bond(user.address, amount, 'Staked');
            await bondTx.signAndSend(user.getKeyPair());

            // Then nominate a validator
            const nominateTx = api.tx.staking.nominate([validatorAddress]);
            const hash = await nominateTx.signAndSend(user.getKeyPair());
            console.log(`Delegation successful! Transaction hash: ${hash}`);

            // Update user balance after delegation
            await user.updateBalance(api);
            console.log(`New balance: ${user.balance}`);
        } catch (error) {
            console.error('Error during TNT delegation:', error);
            throw error;
        }
    }
}
