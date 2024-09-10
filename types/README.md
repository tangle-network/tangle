# Tangle Substrate Types

This package is meant to be updated alongside changes to the tangle runtime.

The package builds the types against the tangle standalone runtime.

### Update Types

In order to update types after making changes to the Tangle APIs, do the following:

- Run a local instance of the appropriate runtime. The types in this package correspond to the tangle standalone runtime.
- Change your working directory into the `/types` folder (`cd types`).
- Install dependencies using `yarn`.
- Run the following yarn scripts:
```
yarn update:metadata
yarn build:interfaces
```

### Building the types package

After updating the types, run a build for the package with
```
yarn build
```

Note that you may run into some errors of missing imports while building. To resolve this, manually add the missing imports on the files with errors. If using VSCode, you can also use its `Add all missing imports` feature to speed up the process.

### Publishing and consuming types package

Once the types have been updated, open a new PR on this repository to submit your changes. Remember to update the types package's version. Once the PR is merged into the `master`, a GitHub Actions workflow will automatically publish them to NPM.

In case that you need to use or prototype the types before they are officially published to NPM, consider installing the package locally and using the local package until its updated version is published to NPM.