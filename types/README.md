# Tangle Substrate Types

This package is meant to be updated alongside changes to the tangle runtime.

The package builds the types against the tangle standalone runtime.

### Updating Types

To update the types after modifying the Tangle APIs, follow these steps:

1. Build the `tangle` project with the testnet feature:

   ```bash
   cargo build --release --package tangle --features testnet
   ```

2. Ensure you have ![Node.js](https://nodejs.org/) version 18 or higher installed.

3. Generate the updated TypeScript types by running the `generate-ts-types.js` script:

   ```bash
   node types/scripts/generate-ts-types.js
   ```

This process will automatically update the TypeScript types to reflect the latest changes in the Tangle APIs.

### Publishing and consuming types package

Once the types have been updated, open a new PR on this repository to submit your changes. Remember to update the types package's version. Once the PR is merged into the `master`, a GitHub Actions workflow will automatically publish them to NPM.

In case that you need to use or prototype the types before they are officially published to NPM, consider installing the package locally and using the local package until its updated version is published to NPM.
