import { ApiPromise } from '@polkadot/api';
import { User } from './user';
import { Keyring } from '@polkadot/keyring';

// Base Action type
export interface Action {
    execute(api: ApiPromise, keyring: Keyring, user: User, ...args: any[]): Promise<any>;
}
