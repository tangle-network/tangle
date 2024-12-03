use crate::{AccountId, Babe, Grandpa, KeyTypeId, OpaqueKeys, Session, SessionKeys};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{BoundToRuntimeAppPublic, RuntimeAppPublic, RuntimeDebug};

/// Old session keys structure.
///
/// This struct represents the session keys used in the previous version of the runtime.
/// It includes keys for Grandpa, Babe, ImOnline, and Role.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, MaxEncodedLen)]
pub struct OldSessionKeys {
	/// Grandpa key.
	pub grandpa: <Grandpa as BoundToRuntimeAppPublic>::Public,
	/// Babe key.
	pub babe: <Babe as BoundToRuntimeAppPublic>::Public,
	/// ImOnline key.
	pub im_online: pallet_im_online::sr25519::AuthorityId,
	/// Role key.
	pub role: pallet_im_online::sr25519::AuthorityId,
}

impl OpaqueKeys for OldSessionKeys {
	type KeyTypeIdProviders = ();

	/// Return the key IDs of the old session keys.
	fn key_ids() -> &'static [KeyTypeId] {
		&[
			<<Grandpa as BoundToRuntimeAppPublic>::Public>::ID,
			<<Babe as BoundToRuntimeAppPublic>::Public>::ID,
			sp_core::crypto::key_types::IM_ONLINE,
			tangle_crypto_primitives::ROLE_KEY_TYPE,
		]
	}

	/// Get the raw byte representation of a key based on its KeyTypeId.
	fn get_raw(&self, i: KeyTypeId) -> &[u8] {
		match i {
			<<Grandpa as BoundToRuntimeAppPublic>::Public>::ID => self.grandpa.as_ref(),
			<<Babe as BoundToRuntimeAppPublic>::Public>::ID => self.babe.as_ref(),
			sp_core::crypto::key_types::IM_ONLINE => self.im_online.as_ref(),
			tangle_crypto_primitives::ROLE_KEY_TYPE => self.role.as_ref(),
			_ => &[],
		}
	}
}

/// Transform function to convert old session keys to the new session keys structure.
///
/// This function is used during the runtime upgrade to transform the old session keys into
/// the new session keys structure.
fn transform_session_keys(_val: AccountId, old: OldSessionKeys) -> SessionKeys {
	SessionKeys { grandpa: old.grandpa, babe: old.babe, im_online: old.im_online }
}

/// Runtime upgrade for migrating session keys.
///
/// This struct implements the `OnRuntimeUpgrade` trait and performs the migration of session keys
/// from the old structure (`OldSessionKeys`) to the new structure (`SessionKeys`).
pub struct MigrateSessionKeys<T>(sp_std::marker::PhantomData<T>);

impl<T: pallet_session::Config> OnRuntimeUpgrade for MigrateSessionKeys<T> {
	/// Perform the runtime upgrade.
	///
	/// This function upgrades the session keys by transforming them from the old structure to the
	/// new structure using the `transform_session_keys` function. It reads and writes to the
	/// database as needed.
	fn on_runtime_upgrade() -> Weight {
		Session::upgrade_keys::<OldSessionKeys, _>(transform_session_keys);
		T::DbWeight::get().reads_writes(10, 10)
	}
}
