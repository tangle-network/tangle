use sc_service::{ChainType, Configuration};
use sp_core::{ed25519, sr25519, ByteArray, Pair, Public};
use sp_keystore::{Keystore, KeystorePtr};
use sp_runtime::{
	key_types::{ACCOUNT, AURA, GRANDPA, IM_ONLINE},
	KeyTypeId,
};

/// Helper function to generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{seed}"), None)
		.expect("static values are valid; qed")
		.public()
}

/// Inserts a key of type `ACCOUNT` into the keystore for development/testing.
pub fn insert_controller_account_keys_into_keystore(
	config: &Configuration,
	key_store: Option<KeystorePtr>,
) {
	insert_account_keys_into_keystore::<sr25519::Public>(
		config,
		ACCOUNT,
		key_store.clone(),
		"acco",
	);
	insert_account_keys_into_keystore::<ed25519::Public>(
		config,
		GRANDPA,
		key_store.clone(),
		"Grandpa",
	);
	insert_account_keys_into_keystore::<sr25519::Public>(config, AURA, key_store.clone(), "Aura");
	insert_account_keys_into_keystore::<sr25519::Public>(
		config,
		IM_ONLINE,
		key_store.clone(),
		"ImOnline",
	);
}

/// Inserts keys of specified type into the keystore.
fn insert_account_keys_into_keystore<TPublic: Public>(
	config: &Configuration,
	key_type: KeyTypeId,
	key_store: Option<KeystorePtr>,
	key_name: &str,
) {
	let seed = &config.network.node_name[..];

	let pub_key = get_from_seed::<TPublic>(seed).to_raw_vec();
	if let Some(keystore) = key_store {
		let _ = Keystore::insert(&*keystore, key_type, &format!("//{seed}"), &pub_key);
	}

	println!("++++++++++++++++++++++++++++++++++++++++++++++++  
                AUTO GENERATED KEYS                                                                        
                {:?} key inserted to keystore
                Seed : //{:?}
                Pubkey : {:?}
                STORE THE KEYS SAFELY, NOT TO BE SHARED WITH ANYONE ELSE.
    ++++++++++++++++++++++++++++++++++++++++++++++++   							
            \n", key_name, seed, pub_key);
}

/// Inserts a key of type `ACCOUNT` into the keystore for development/testing.
///
/// Currently, this only successfully inserts keys if the seed is development related.
/// i.e. for Alice, Bob, Charlie, etc.
pub fn insert_dev_controller_account_keys_into_keystore(
	config: &Configuration,
	key_store: Option<KeystorePtr>,
) {
	insert_dev_account_keys_into_keystore::<sr25519::Public>(config, ACCOUNT, key_store.clone());
}

/// Inserts keys of specified type into the keystore for predefined nodes in development mode.
pub fn insert_dev_account_keys_into_keystore<TPublic: Public>(
	config: &Configuration,
	key_type: KeyTypeId,
	key_store: Option<KeystorePtr>,
) {
	let chain_type = config.chain_spec.chain_type();
	let seed = &config.network.node_name[..];

	match seed {
		// When running the chain in dev or local test net, we insert the sr25519 account keys for
		// collator accounts or validator accounts into the keystore Only if the node running is one
		// of the predefined nodes Alice, Bob, Charlie, Dave, Eve or Ferdie
		"Alice" | "Bob" | "Charlie" | "Dave" | "Eve" | "Ferdie" => {
			if chain_type == ChainType::Development || chain_type == ChainType::Local {
				let pub_key = get_from_seed::<TPublic>(seed).to_raw_vec();
				if let Some(keystore) = key_store {
					let _ = Keystore::insert(&*keystore, key_type, &format!("//{seed}"), &pub_key);
				}
			}
		},
		_ => {},
	}
}

/// Ensures all keys exist in the keystore.
pub fn ensure_all_keys_exist_in_keystore(key_store: KeystorePtr) -> Result<(), String> {
	let key_types = [ACCOUNT, GRANDPA, AURA, IM_ONLINE];

	for key_type in key_types {
		// Ensure key is present
		if !ensure_keytype_exists_in_keystore(key_type, key_store.clone()) {
			println!("{key_type:?} key not found!");
			return Err("Key not found!".to_string())
		}
	}

	Ok(())
}

/// Checks if a key of a specific type exists in the keystore.
fn ensure_keytype_exists_in_keystore(key_type: KeyTypeId, key_store: KeystorePtr) -> bool {
	match Keystore::keys(&key_store, key_type) {
		Ok(keys) => !keys.is_empty(),
		Err(_) => false,
	}
}
