import { ApiPromise } from '@polkadot/api';
import { User } from '../types/user';
import { Keyring } from '@polkadot/keyring';
import { Action } from '../types/action';
import * as fs from 'fs';
import * as dotenv from 'dotenv';

dotenv.config();

// GenerateChildUsers action
export class GenerateChildUsers implements Action {
    async execute(api: ApiPromise, keyring: Keyring, user: User, count: number): Promise<User[]> {
        const childUsers: User[] = [];
        const filePath = './generated_users.txt';

        // Use a base seed for generating child accounts
        const baseSeed = process.env.BASE_SEED || '//User';

        for (let i = 0; i < count; i++) {
            // Generate a unique seed for each child user
            const childSeed = `${baseSeed}/Child/${i}`;
            const childKeyPair = keyring.addFromUri(childSeed);
            const childUser = new User(childKeyPair);
            await childUser.updateBalance(api);
            childUsers.push(childUser);

            console.log(`Generated child user ${i}:
                Address: ${childUser.address}
                Seed: ${childSeed}
                Initial Balance: ${childUser.balance}`);

            // Append user data to the file
            const userData = `Child User ${i} - Address: ${childUser.address}, Seed: ${childSeed}, Balance: ${childUser.balance}\n`;
            fs.appendFileSync(filePath, userData);
        }

        console.log(`User data written to ${filePath}`);

        return childUsers;
    }
}
