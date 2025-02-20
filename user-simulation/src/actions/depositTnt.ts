import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class DepositTnt implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User, amount: bigint): Promise<void> {
        try {
            console.log(`Depositing ${amount} TNT for user ${user.address}...`);
            // Call the deposit function in the staking pallet
            const deposit = api.tx.multiAssetDelegation.bond(user.address, amount, 'Staked');
            const hash = await deposit.signAndSend(user.getKeyPair());
            console.log(`Deposit successful! Transaction hash: ${hash}`);

            // Update user balance after deposit
            await user.updateBalance(api);
            console.log(`New balance: ${user.balance}`);
        } catch (error) {
            console.error('Error during TNT deposit:', error);
            throw error;
        }
    }
}
