import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class ClaimRewards implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User): Promise<void> {
        try {
            console.log(`Claiming rewards for user ${user.address}...`);
            const hash = await user.claimRewards(api);
            console.log(`Rewards claimed successfully! Transaction hash: ${hash}`);

            // Update user balance after claiming rewards
            await user.updateBalance(api);
            console.log(`New balance after claiming rewards: ${user.balance}`);
        } catch (error) {
            console.error('Error during rewards claim:', error);
            throw error;
        }
    }
}
