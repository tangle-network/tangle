import { ApiPromise, WsProvider } from "@polkadot/api";
import { SubmittableExtrinsic, AddressOrPair } from "@polkadot/api-base/types";
import { Keyring } from "@polkadot/keyring";
import { u8aToHex, hexToU8a } from "@polkadot/util";
import * as readline from "node:readline/promises";
import { stdin as input, stdout as output, exit } from "node:process";

/**
 * This script demonstrates how to work with pallet-services, it will demonstrate how to:
 * - Create a service blueprint
 * - Register on a service as a service provider (operator)
 * - Other users can request the service (consumer)
 * - Invoke functions on the service
 * - Service Lifecycle.
 */
(async () => {
  const provider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({ provider, noInitWarn: true });

  const rl = readline.createInterface({
    input,
    output,
    prompt: "> ",
    removeHistoryDuplicates: true,
  });

  await api.isReady;

  rl.on("close", async () => {
    await api.disconnect();
  });

  rl.on("SIGINT", async () => {
    console.log("Received SIGINT, closing the connection");
    await api.disconnect();
    exit(0);
  });

  console.log(`Connected to local node`);

  const sr25519Keyring = new Keyring({ type: "sr25519" });
  const ecdsaKeyring = new Keyring({ type: "ecdsa" });
  const [ALICE, BOB, CHARLIE, DAVE, EVE] = [
    "Alice",
    "Bob",
    "Charlie",
    "Dave",
    "Eve",
  ].map((name) => sr25519Keyring.addFromUri(`//${name}`));

  const [BOBEcdsa, CHARLIEEcdsa, DAVEEcdsa, DKG] = [
    "Bob",
    "Charlie",
    "Dave",
    "DKG",
  ].map((name) => ecdsaKeyring.addFromUri(`//${name}`));
  // Alice is a devloper and she wants to create a service blueprint
  // Bob, Charlie, Dave are service providers
  // Eve is a consumer

  // Alice will create the service blueprint, in our demo, it is a Simple TSS service
  // Similar to CGGMP21.

  const blueprintId = await api.query.services.nextBlueprintId();

  console.log("=> Creating a new service blueprint");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Creating a new service blueprint");
  const createBlueprintTx = api.tx.services.createBlueprint({
    metadata: {
      name: "CGGMP21",
      description: "A simple Threshold Signature Scheme as a service",
      author: "Alice",
    },
    jobs: [
      {
        metadata: {
          name: "keygen",
          description: "run a new t-of-n DKG",
        },
        params: ["Uint8"],
        result: ["Bytes"],
        verifier: "None",
      },
      {
        metadata: {
          name: "sign",
          description: "sign a message using a t-of-n DKG",
        },
        params: ["Bytes"],
        result: ["Bytes"],
        verifier: "None",
      },
    ],
    registrationHook: "None",
    requestHook: "None",
    requestParams: [],
    registrationParams: [],
    gadget: {
      Wasm: {
        Ipfs: [],
      },
    },
  });

  await signAndSend(ALICE, createBlueprintTx);
  console.log(`|-- Alice created a service blueprint with id: ${blueprintId}`);

  // Bob, Charlie, Dave are service providers, they will register on the service.
  const bobRegisterTx = api.tx.services.register(
    blueprintId,
    {
      key: BOBEcdsa.publicKey,
      approval: "None",
    },
    [],
  );

  console.log("=> Bob is registering on the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Bob is registering on the service");
  await signAndSend(BOB, bobRegisterTx);
  console.log("|-- Bob registered on the service");

  const charlieRegisterTx = api.tx.services.register(
    blueprintId,
    {
      key: CHARLIEEcdsa.publicKey,
      approval: "None",
    },
    [],
  );

  console.log("=> Charlie is registering on the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Charlie is registering on the service");
  await signAndSend(CHARLIE, charlieRegisterTx);
  console.log("|-- Charlie registered on the service");

  const daveRegisterTx = api.tx.services.register(
    blueprintId,
    {
      key: DAVEEcdsa.publicKey,
      approval: "None",
    },
    [],
  );

  console.log("=> Dave is registering on the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Dave is registering on the service");
  await signAndSend(DAVE, daveRegisterTx);
  console.log("|-- Dave registered on the service");

  // Eve is a consumer, she will request the service.
  const serviceInstanceId = await api.query.services.nextInstanceId();
  console.log("=> Eve is requesting the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Eve is requesting the service");
  const requestServiceTx = api.tx.services.request(
    blueprintId,
    [],
    [BOB.address, CHARLIE.address, DAVE.address],
    10_000,
    [],
  );
  await signAndSend(EVE, requestServiceTx);
  console.log(`|-- Eve requested the service: ${serviceInstanceId}`);

  // Eve will invoke the keygen job on the service
  console.log("=> Eve is invoking the keygen job on the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Eve is invoking the keygen job on the service");
  const jobCallId = await api.query.services.nextJobCallId();
  const keygenJobTx = api.tx.services.jobCall(serviceInstanceId, 0, [
    { Uint8: 2 },
  ]);
  await signAndSend(EVE, keygenJobTx);
  console.log(`|-- Eve invoked the keygen job with id: ${jobCallId}`);

  // Bob, Charlie, Dave will accept the job and submit the result
  console.log(
    "=> Bob, Charlie, Dave are accepting the job and submitting the result",
  );
  await rl.question("|- Press Enter to continue");
  console.log(
    "|-- Bob, Charlie, Dave are accepting the job and submitting the result",
  );
  const jobResultTx = api.tx.services.jobSubmit(serviceInstanceId, jobCallId, [
    {
      Bytes: u8aToHex(DKG.publicKey),
    },
  ]);
  await signAndSend(BOB, jobResultTx);
  console.log(
    "|-- Bob, charlie, Dave accepted the job and submitted the result",
  );

  // Eve will invoke the sign job on the service
  console.log("=> Eve is invoking the sign job on the service");
  await rl.question("|- Press Enter to continue");
  console.log("|-- Eve is invoking the sign job on the service");
  const signJobCallId = await api.query.services.nextJobCallId();
  const signJobTx = api.tx.services.jobCall(serviceInstanceId, 1, [
    { Bytes: "0xf00dc00ed" },
  ]);

  await signAndSend(EVE, signJobTx);
  console.log(`|-- Eve invoked the sign job with id: ${signJobCallId}`);

  // Bob, Charlie, Dave will accept the job and submit the result
  console.log(
    "=> Bob, Charlie, Dave are accepting the job and submitting the result",
  );
  await rl.question("|- Press Enter to continue");
  console.log(
    "|-- Bob, Charlie, Dave are accepting the job and submitting the result",
  );
  const signature = DKG.sign(hexToU8a("0xf00dc00ed"));
  // Bob, Charlie, Dave will accept the job and submit the result
  const signJobResultTx = api.tx.services.jobSubmit(
    serviceInstanceId,
    signJobCallId,
    [
      {
        Bytes: u8aToHex(signature),
      },
    ],
  );
  await signAndSend(BOB, signJobResultTx);
  console.log(
    "|-- Bob, charlie, Dave accepted the job and submitted the result",
  );

  // *** --- Helper Functions --- ***

  async function signAndSend(
    signer: AddressOrPair,
    tx: SubmittableExtrinsic<"promise">,
    waitTillFinallized: boolean = false,
  ): Promise<void> {
    // Sign and send the transaction and wait for it to be included.
    await new Promise(async (resolve, reject) => {
      const unsub = await tx.signAndSend(
        signer,
        async ({ events = [], status, dispatchError }) => {
          if (dispatchError) {
            if (dispatchError.isModule) {
              // for module errors, we have the section indexed, lookup
              const decoded = api.registry.findMetaError(
                dispatchError.asModule,
              );
              const { docs, name, section } = decoded;

              console.log(`|--- ${section}.${name}: ${docs.join(" ")}`);
              reject(`${section}.${name}`);
            }
          }
          if (status.isInBlock && !waitTillFinallized) {
            console.log("|--- Events:");
            events.forEach(({ event: { data, method, section } }) => {
              console.log(`|---- ${section}.${method}:: ${data}`);
            });
            unsub();
            resolve(void 0);
          }
          if (status.isFinalized && waitTillFinallized) {
            console.log("|--- Events:");
            events.forEach(({ event: { data, method, section } }) => {
              console.log(`|---- ${section}.${method}:: ${data}`);
            });
            unsub();
            resolve(void 0);
          }
        },
      );
    });
  }
})();
