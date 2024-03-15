const polkadotBabelConfig = require('./polkadot-dev-configs/babel-config-cjs.cjs');

module.exports = {
  plugins: [
    ...polkadotBabelConfig.plugins,
  ],
  presets: [
    ...polkadotBabelConfig.presets
  ]
};
