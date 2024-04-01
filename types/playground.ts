import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { u128 } from "@polkadot/types";
(async () => {
  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider });

  await api.isReady;
  const registry = api.registry;

  const sr25519Keyring = new Keyring({ type: 'sr25519' });

  const ALICE = sr25519Keyring.addFromUri('//Alice');
  const BOB = sr25519Keyring.addFromUri('//Bob');
  const CHARLIE = sr25519Keyring.addFromUri('//Charlie');
  const creatingProfileTx = api.tx.roles.createProfile({
    Shared: {
      records: [
        {
          role: {
            Tss: {
              DfnsCGGMP21Secp256k1: {},
            },
          },
        },
      ],
      amount: new u128(registry, 100_000_000_000_000),
    },
    //@ts-ignore
  }, null);

  for (const signer of [ALICE, BOB, CHARLIE]) {
    await new Promise(async (resolve) => {
      console.log('Creating profile for:', signer.address);
      const unsub = await creatingProfileTx.signAndSend(signer, async ({ events = [], status }) => {
        if (status.isInBlock) {
          console.log(
            '[creatingProfileTx] Included at block hash',
            status.asInBlock.toHex()
          );
          console.log('[creatingProfileTx] Events:');
          events.forEach(({ event: { data, method, section } }) => {
            console.log(`\t${section}.${method}:: ${data}`);
          });
          unsub();
          resolve(void 0);
        }
      });
    });
  }

  const submittingJobTx = api.tx.jobs.submitJob({
    expiry: 100_000_000,
    ttl: 100_000_000,
    jobType: {
      DkgtssPhaseOne: {
        participants: [ALICE.address, BOB.address, CHARLIE.address],
        threshold: 2,
        permittedCaller: null,
        roleType: {
          DfnsCGGMP21Secp256k1: {},
        },
      },
    },
  });

  // Sign and send the transaction and wait for it to be included.
  await new Promise(async (resolve) => {
    const unsub = await submittingJobTx.signAndSend(ALICE, async ({ events = [], status }) => {
      if (status.isFinalized) {
        console.log(
          '[submittingJobTx] Included at block hash',
          status.asFinalized.toHex()
        );
        console.log('[submittingJobTx] Events:');
        events.forEach(({ event: { data, method, section } }) => {
          console.log(`\t${section}.${method}:: ${data}`);
        });
        unsub();
        resolve(void 0);
      }
    });
  });

  process.exit(0);
})();
