#!/usr/bin/env -S deno run -A --unstable

/// This a script to run and setup the tangle network parachain.

import {
  ApiPromise,
  Keyring,
  WsProvider,
} from "https://esm.sh/@polkadot/api@9.2.3?bundle";
import { u8aToHex } from "https://esm.sh/@polkadot/util@10.1.3?bundle";
import { Input } from "https://deno.land/x/cliffy@v0.24.3/prompt/mod.ts";
import { assert } from "https://deno.land/std@0.152.0/testing/asserts.ts";
import {
  keypress,
  KeyPressEvent,
} from "https://deno.land/x/cliffy@v0.24.3/keypress/mod.ts";

type ScrtipState = {
  relayChainPath: string;
  relayChain: {
    alice?: Deno.Process;
    bob?: Deno.Process;
  };
  tmpDir: string;
  rootDir: string;
};

const defaultState: ScrtipState = {
  relayChainPath: Deno.realPathSync(
    "../../paritytech/polkadot/target/release/polkadot",
  ),
  tmpDir: Deno.makeTempDirSync({ prefix: "tangle-" }),
  rootDir: new TextDecoder().decode(
    await Deno.run({
      cmd: ["git", "rev-parse", "--show-toplevel"],
      stdout: "piped",
    }).output(),
  ).replace(/\n$/, ""),
  relayChain: {},
};

let state: ScrtipState = defaultState;

const parachainPath = await Deno.realPath(
  `${state.rootDir}/target/release/tangle-collator`,
);
const relayChainPath: string = await Input.prompt({
  message: "Relay chain path?",
  default: state.relayChainPath,
});

// Build Relay chain spec
const chainSpec = await Deno.run({
  cmd: [
    relayChainPath,
    "build-spec",
    "--chain",
    "rococo-local",
    "--disable-default-bootnode",
    "--raw",
  ],
  stdout: "piped",
  stderr: "piped",
}).output();

const relaychainSpecPath = `${state.tmpDir}/relay-chain.spec.json`;
const parachainSpecPath = `${state.tmpDir}/parachain.spec.json`;
await Deno.writeFile(relaychainSpecPath, chainSpec);

state = {
  ...state,
  relayChainPath,
  relayChain: {
    alice: Deno.run({
      cmd: [relayChainPath, "--chain", relaychainSpecPath, "--alice", "--tmp"],
      stdout: "null",
      stderr: "piped",
    }),
    bob: Deno.run({
      cmd: [
        relayChainPath,
        "--chain",
        relaychainSpecPath,
        "--bob",
        "--tmp",
        "--port",
        "30334",
      ],
      stdout: "null",
      stderr: "piped",
    }),
  },
};

// Generate raw chainspec and artifacts (we are not passing the argument `--chain` since the default is `local-testnet`).
const rawChainSpec = await Deno.run({
  cmd: [parachainPath, "build-spec", "--disable-default-bootnode", "--raw"],
  stdout: "piped",
  stderr: "piped",
}).output();
await Deno.writeFile(parachainSpecPath, rawChainSpec);

const genesisStatePath = `${state.tmpDir}/genesis.hex`;
await Deno.run({
  cmd: [parachainPath, "export-genesis-state", genesisStatePath],
  stdout: "null",
  stderr: "null",
}).status();
const genesisState = await Deno.readTextFile(genesisStatePath);

const genesisWASMPath = `${state.tmpDir}/wasm.hex`;
await Deno.run({
  cmd: [parachainPath, "export-genesis-wasm", genesisWASMPath],
  stdout: "null",
  stderr: "null",
}).status();
const genesisWASM = await Deno.readTextFile(genesisWASMPath);

async function addKeyToKeyStore(
  basePath: string,
  suri: string,
  scheme: string,
  keyType: string,
) {
  const cmd = [
    parachainPath,
    "key",
    "insert",
    "--base-path",
    basePath,
    "--chain",
    parachainSpecPath,
    "--scheme",
    scheme,
    "--suri",
    suri,
    "--key-type",
    keyType,
  ];

  const status = await Deno.run({ cmd }).status();
  assert(status.success);
}

const keyStoreConfigs = [
  {
    basePath: `${state.tmpDir}/para/collator3`,
    keys: [{
      suri:
        "merge wrist ski trophy bounce road margin grain demand dentist key index",
      scheme: "Sr25519",
      keyType: "aura",
    }, {
      suri:
        "merge wrist ski trophy bounce road margin grain demand dentist key index",
      scheme: "Ecdsa",
      keyType: "wdkg",
    }, {
      suri:
        "sock jungle mercy cricket car media peace explain turkey garbage gauge chief",
      scheme: "Sr25519",
      keyType: "acco",
    }],
  },
  {
    basePath: `${state.tmpDir}/para/collator2`,
    keys: [{
      suri:
        "apology pencil toward orient history tray decrease unaware bullet bus dream quit",
      scheme: "Sr25519",
      keyType: "aura",
    }, {
      suri:
        "apology pencil toward orient history tray decrease unaware bullet bus dream quit",
      scheme: "Ecdsa",
      keyType: "wdkg",
    }, {
      suri:
        "soup unknown belt science purity north leave amount picture make grief stumble",
      scheme: "Sr25519",
      keyType: "acco",
    }],
  },
  {
    basePath: `${state.tmpDir}/para/collator1`,
    keys: [{
      suri:
        "vintage wedding poem carbon sad layer age fragile connect aerobic oval swamp",
      scheme: "Sr25519",
      keyType: "aura",
    }, {
      suri:
        "vintage wedding poem carbon sad layer age fragile connect aerobic oval swamp",
      scheme: "Ecdsa",
      keyType: "wdkg",
    }, {
      suri:
        "this orchard square vendor opera attack say correct front supply below gun",
      scheme: "Sr25519",
      keyType: "acco",
    }],
  },
];

for (const { basePath, keys } of keyStoreConfigs) {
  for (const { suri, scheme, keyType } of keys) {
    await addKeyToKeyStore(basePath, suri, scheme, keyType);
  }
}

console.log("Almost done, now run the following commands (in terminal tabs):");
console.log("");

const cmds = [1, 2, 3].map((n) =>
  `${parachainPath} --collator \\
  --chain ${parachainSpecPath} \\
  --base-path ${state.tmpDir}/para/collator${n} \\
  --port ${30334 + n} \\
  --force-authoring \\
  --ws-port ${9945 + n} \\
  --ws-external --rpc-port ${9979 + n} \\
  --rpc-cors all --rpc-external \\
  --unsafe-ws-external --unsafe-rpc-external \\
  -ldkg=debug \\
  -ldkg_gadget=debug \\
  -lruntime::dkg_metadata=debug \\
  -lruntime::offchain=debug \\
  -lruntime::dkg_proposal_handler=debug \\
  -ldkg_proposals=debug \\
  -ldkg_gadget::async_protocol::keygen=debug \\
  -ldkg_gadget::async_protocol::keygen=debug \\
  -ldkg_gadget::gossip_engine::network=debug \\
  -ldkg_gadget::storage::public_keys=debug \\
  --log info \\
  --rpc-methods=unsafe -- --execution wasm \\
  --chain ${relaychainSpecPath}`
);

for (const cmd of cmds) {
  console.log(cmd);
  console.log("");
}

console.log("");
console.log("Once done, hit enter to continue.");

keypress().addEventListener("keydown", (event: KeyPressEvent) => {
  if (event.key === "return") {
    deployParachain();
    keypress().dispose();
  }
});

// Now we just wait for the whole thing to finish.
await Promise.any([
  state.relayChain.alice?.status(),
  state.relayChain.bob?.status(),
]);

async function deployParachain() {
  console.log("Deploying parachains...");
  const api = await ApiPromise.create({
    provider: new WsProvider("ws://localhost:9944"),
  });
  await api.isReady;
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const call = api.tx.parasSudoWrapper.sudoScheduleParaInitialize(2000, {
    genesisHead: genesisState,
    validationCode: genesisWASM,
    parachain: true,
  });
  await api.tx.sudo.sudo(call).signAndSend(alice, (result) => {
    console.log(result.toHuman());
  });
}
