// Copyright 2017-2023 @polkadot/dev authors & contributors
// SPDX-License-Identifier: Apache-2.0

module.exports = function resolver (input) {
    return Array.isArray(input)
      ? input
        .filter((plugin) => !!plugin)
        .map((plugin) =>
          Array.isArray(plugin)
            ? [require.resolve(plugin[0]), plugin[1]]
            : require.resolve(plugin)
        )
      : require.resolve(input);
  };