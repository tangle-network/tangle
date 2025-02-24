"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransferAssets = void 0;
const fs_1 = require("fs");
class TransferAssets {
    async execute(api, keyring, assetId, seedPhrase, amount) {
        try {
            // Create the source account from the seed
            const sourceAccount = keyring.addFromUri(seedPhrase);
            console.log(`Source account: ${sourceAccount.address}`);
            // Read the generated users file to get child user addresses
            const usersContent = (0, fs_1.readFileSync)('generated_users.txt', 'utf-8');
            const childUsers = usersContent
                .split('\n')
                .filter(line => line.trim())
                .map(line => {
                const match = line.match(/Address: ([\w]+)/);
                return match ? match[1] : null;
            })
                .filter(address => address !== null);
            console.log(`Found ${childUsers.length} child users to transfer assets to`);
            // Transfer assets to each child user
            for (const userAddress of childUsers) {
                console.log(`Transferring ${amount} of asset ${assetId} to ${userAddress}`);
                try {
                    // Create and send the transfer transaction
                    const transfer = api.tx.assets.transfer(assetId, userAddress, amount);
                    const hash = await transfer.signAndSend(sourceAccount);
                    console.log(`Transfer submitted with hash: ${hash.toHex()}`);
                    // Wait for 2 blocks to ensure the transaction is processed
                    await new Promise(resolve => setTimeout(resolve, 12000));
                    // Query the balance to verify the transfer
                    const balance = await api.query.assets.account(assetId, userAddress);
                    console.log(`New balance for ${userAddress}: ${balance.toString()}`);
                }
                catch (error) {
                    console.error(`Failed to transfer to ${userAddress}:`, error);
                    throw error;
                }
                // Wait a bit between transfers to avoid nonce issues
                await new Promise(resolve => setTimeout(resolve, 2000));
            }
            console.log('All transfers completed successfully');
        }
        catch (error) {
            console.error('Error in transferring assets:', error);
            throw error;
        }
    }
}
exports.TransferAssets = TransferAssets;
