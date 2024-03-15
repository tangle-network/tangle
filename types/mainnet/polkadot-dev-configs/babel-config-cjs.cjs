// Copyright 2017-2023 @polkadot/dev authors & contributors
// SPDX-License-Identifier: Apache-2.0

const general = require('./babel-general.cjs');
const plugins = require('./babel-plugins.cjs');
const presets = require('./babel-presets.cjs');

module.exports = {
  ...general,
  plugins: plugins(false),
  presets: presets(false)
};