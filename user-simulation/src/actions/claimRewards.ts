import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';

export class ClaimRewards implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User): Promise<void> {
        try {
            console.log(`Claiming rewards for user ${user.address}...`);
            
            const claimTx = api.tx.rewards.claimRewards();
            
            const hash = await new Promise<string>((resolve, reject) => {
                claimTx.signAndSend(user.getKeyPair(), ({ status, dispatchError }) => {
                    if (status.isFinalized) {
                        if (dispatchError) {
                            if (dispatchError.isModule) {
                                const decoded = api.registry.findMetaError(dispatchError.asModule);
                                reject(new Error(`${decoded.section}.${decoded.name}`));
                            } else {
                                reject(new Error(dispatchError.toString()));
                            }
                        } else {
                            resolve(status.asFinalized.toHex());
                        }
                    }
                }).catch(reject);
            });
            
            console.log(`Rewards claimed successfully! Transaction hash: ${hash}`);
            await user.updateBalance(api);
            console.log(`New balance after claiming rewards: ${user.balance}`);
        } catch (error) {
            console.error('Error during rewards claim:', error);
            throw error;
        }
    }
}
