//! Ciphersuite-generic benchmark functions.
#![allow(clippy::unwrap_used)]

use core::iter;

use alloc::{collections::BTreeMap, format, vec::Vec};
use rand_core::{CryptoRng, RngCore};

use criterion::{BenchmarkId, Criterion, Throughput};

use crate as frost;
use crate::{batch, Ciphersuite, Signature, SigningKey, VerifyingKey};

struct Item<C: Ciphersuite> {
	vk: VerifyingKey<C>,
	sig: Signature<C>,
}

fn sigs_with_distinct_keys<C: Ciphersuite, R: RngCore + CryptoRng + Clone>(
	rng: &mut R,
) -> impl Iterator<Item = Item<C>> {
	let mut rng = rng.clone();
	iter::repeat_with(move || {
		let msg = b"Bench";
		let sk = SigningKey::new(&mut rng);
		let vk = VerifyingKey::from(&sk);
		let sig = sk.sign(&mut rng, &msg[..]);
		Item { vk, sig }
	})
}

/// Benchmark batched signature verification with the specified ciphersuite.
pub fn bench_batch_verify<C: Ciphersuite, R: RngCore + CryptoRng + Clone>(
	c: &mut Criterion,
	name: &str,
	rng: &mut R,
) {
	let mut group = c.benchmark_group(format!("Batch Verification {name}"));
	for &n in [8usize, 16, 24, 32, 40, 48, 56, 64].iter() {
		group.throughput(Throughput::Elements(n as u64));

		let sigs = sigs_with_distinct_keys::<C, R>(rng).take(n).collect::<Vec<_>>();

		group.bench_with_input(BenchmarkId::new("Unbatched verification", n), &sigs, |b, sigs| {
			b.iter(|| {
				for item in sigs.iter() {
					let msg = b"Bench";

					let Item { vk, sig } = item;
					let _ = vk.verify(msg, sig);
				}
			})
		});

		group.bench_with_input(BenchmarkId::new("Batched verification", n), &sigs, |b, sigs| {
			let mut rng = rng.clone();
			b.iter(|| {
				let mut batch = batch::Verifier::new();
				for item in sigs.iter() {
					let msg = b"Bench";

					let Item { vk, sig } = item;
					let item = batch::Item::<C>::new(*vk, *sig, msg).unwrap();
					batch.queue(item);
				}
				batch.verify(&mut rng)
			})
		});
	}
	group.finish();
}

/// Benchmark FROST signing with the specified ciphersuite.
pub fn bench_sign<C: Ciphersuite, R: RngCore + CryptoRng + Clone>(
	c: &mut Criterion,
	name: &str,
	rng: &mut R,
) {
	let mut group = c.benchmark_group(format!("FROST Signing {name}"));
	for &n in [3u16, 10, 100, 1000].iter() {
		let max_signers = n;
		let min_signers = (n * 2).div_ceil(3);

		group.bench_with_input(
			BenchmarkId::new("Key Generation with Dealer", max_signers),
			&(max_signers, min_signers),
			|b, (max_signers, min_signers)| {
				let mut rng = rng.clone();
				b.iter(|| {
					frost::keys::generate_with_dealer::<C, R>(
						*max_signers,
						*min_signers,
						frost::keys::IdentifierList::Default,
						&mut rng,
					)
					.unwrap();
				})
			},
		);

		let (shares, pubkeys) = frost::keys::generate_with_dealer::<C, R>(
			max_signers,
			min_signers,
			frost::keys::IdentifierList::Default,
			rng,
		)
		.unwrap();

		// Verifies the secret shares from the dealer
		let mut key_packages: BTreeMap<frost::Identifier<C>, frost::keys::KeyPackage<C>> =
			BTreeMap::new();

		for (k, v) in shares {
			key_packages.insert(k, frost::keys::KeyPackage::try_from(v).unwrap());
		}

		group.bench_with_input(
			BenchmarkId::new("Round 1", min_signers),
			&key_packages,
			|b, key_packages| {
				b.iter(|| {
					let participant_identifier = 1u16.try_into().expect("should be nonzero");
					frost::round1::commit(
						key_packages.get(&participant_identifier).unwrap().signing_share(),
						rng,
					);
				})
			},
		);

		let mut nonces: BTreeMap<_, _> = BTreeMap::new();
		let mut commitments: BTreeMap<_, _> = BTreeMap::new();

		for participant_index in 1..=min_signers {
			let participant_identifier = participant_index.try_into().expect("should be nonzero");
			let (nonce, commitment) = frost::round1::commit(
				key_packages.get(&participant_identifier).unwrap().signing_share(),
				rng,
			);
			nonces.insert(participant_identifier, nonce);
			commitments.insert(participant_identifier, commitment);
		}

		let message = "message to sign".as_bytes();
		let signing_package = frost::SigningPackage::new(commitments, message);

		group.bench_with_input(
			BenchmarkId::new("Round 2", min_signers),
			&(key_packages.clone(), nonces.clone(), signing_package.clone()),
			|b, (key_packages, nonces, signing_package)| {
				b.iter(|| {
					let participant_identifier = 1u16.try_into().expect("should be nonzero");
					let key_package = key_packages.get(&participant_identifier).unwrap();
					let nonces_to_use = &nonces.get(&participant_identifier).unwrap();
					frost::round2::sign(signing_package, nonces_to_use, key_package).unwrap();
				})
			},
		);

		let mut signature_shares = BTreeMap::new();
		for participant_identifier in nonces.keys() {
			let key_package = key_packages.get(participant_identifier).unwrap();
			let nonces_to_use = &nonces.get(participant_identifier).unwrap();
			let signature_share =
				frost::round2::sign(&signing_package, nonces_to_use, key_package).unwrap();
			signature_shares.insert(*key_package.identifier(), signature_share);
		}

		group.bench_with_input(
			BenchmarkId::new("Aggregate", min_signers),
			&(signing_package.clone(), signature_shares.clone(), pubkeys),
			|b, (signing_package, signature_shares, pubkeys)| {
				b.iter(|| {
					frost::aggregate(signing_package, signature_shares, pubkeys).unwrap();
				})
			},
		);
	}
	group.finish();
}
