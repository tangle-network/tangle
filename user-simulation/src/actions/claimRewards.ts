import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class ClaimRewards implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User): Promise<void> {
        try {
            console.log(`Claiming rewards for user ${user.address}...`);
            // Claim rewards from the rewards pallet
            const claimTx = api.tx.rewards.claim();
            const hash = await claimTx.signAndSend(user.getKeyPair());
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
