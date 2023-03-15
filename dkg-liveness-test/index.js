// Import the API
const { ApiPromise, WsProvider } = require("@polkadot/api");
var assert = require("assert");

async function main() {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider("ws://127.0.0.1:9944");

  // Create the API and wait until ready
  const api = await ApiPromise.create({ provider });

  // Make our basic chain state/storage queries, all in one go
  const [
    current_block,
    current_session,
    keygenThreshold,
    signatureThreshold,
    lastSessionRotationBlock,
    jailedKeygenAuthorities,
    jailedSigningAuthorities,
  ] = await Promise.all([
    api.query.system.number(),
    api.query.session.currentIndex(),
    api.query.dkg.keygenThreshold(),
    api.query.dkg.signatureThreshold(),
    api.query.dkg.lastSessionRotationBlock(),
    api.query.dkg.jailedKeygenAuthorities.entries(),
    api.query.dkg.jailedSigningAuthorities.entries(),
  ]);

  if (current_block > 0) {
    console.log("Network produces blocks : ", current_block);
    if (lastSessionRotationBlock > 0) {
      console.log("DKG has rotated : ", lastSessionRotationBlock);
      // ensure all params are normal
      assert(jailedKeygenAuthorities.entries().length == 0);
      assert(jailedSigningAuthorities.entries().length == 0);
      assert(current_session != 0);
      process.exit(0);
    }
  }

  process.exit(1);
}

main().catch(console.error);
