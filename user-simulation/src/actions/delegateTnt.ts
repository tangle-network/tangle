import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class DelegateTnt implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User, amount: bigint, validatorAddress: string): Promise<void> {
        try {
            console.log(`Delegating ${amount} TNT from user ${user.address} to validator ${validatorAddress}...`);
            const hash = await user.delegateTnt(api, validatorAddress, amount);
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
