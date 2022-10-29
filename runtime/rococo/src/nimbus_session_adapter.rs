// Copyright 2022 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
use crate::{AccountId, AuraId};
use frame_support::traits::OneSessionHandler;
use frame_system::Config;
use pallet_author_inherent::Pallet as AuthorInherent;
use sp_application_crypto::{sr25519, BoundToRuntimeAppPublic, KeyTypeId, UncheckedFrom};
use sp_runtime::ConsensusEngineId;

pub type NimbusId = nimbus_primitives::NimbusId;

/// This adapts pallet AuthorInherent to be compatible with pallet session
/// making it suitable as a SessionKey entry
pub struct AuthorInherentWithNoOpSession<T: Config>(pub AuthorInherent<T>);

impl<T: Config> BoundToRuntimeAppPublic for AuthorInherentWithNoOpSession<T> {
	type Public = <AuthorInherent<T> as BoundToRuntimeAppPublic>::Public;
}

impl<T: Config> OneSessionHandler<T::AccountId> for AuthorInherentWithNoOpSession<T> {
	type Key = <AuthorInherent<T> as BoundToRuntimeAppPublic>::Public;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
	}

	fn on_disabled(_: u32) {}

	fn on_before_session_ending() {}
}

/// This adapts VrfSessionKey to be compatible with pallet session
/// making it suitable as a SessionKey entry
pub struct VrfWithNoOpSession(pub VrfSessionKey);

impl BoundToRuntimeAppPublic for VrfWithNoOpSession {
	type Public = <VrfSessionKey as BoundToRuntimeAppPublic>::Public;
}

impl OneSessionHandler<AccountId> for VrfWithNoOpSession {
	type Key = <VrfSessionKey as BoundToRuntimeAppPublic>::Public;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
	{
	}

	fn on_disabled(_: u32) {}

	fn on_before_session_ending() {}
}

/// VRF Key type, which is sr25519
/// Struct to implement `BoundToRuntimeAppPublic` by assigning Public = VrfId
pub struct VrfSessionKey;

impl BoundToRuntimeAppPublic for VrfSessionKey {
	type Public = VrfId;
}

/// Reinterprets Aura public key as a VRFId.
/// NO CORRESPONDING PRIVATE KEY TO THAT KEY WILL EXIST
pub fn dummy_key_from(aura_id: AuraId) -> VrfId {
	let aura_as_sr25519: sr25519::Public = aura_id.into();
	let sr25519_as_bytes: [u8; 32] = aura_as_sr25519.into();
	sr25519::Public::unchecked_from(sr25519_as_bytes).into()
}

/// The ConsensusEngineId for VRF keys
pub const VRF_ENGINE_ID: ConsensusEngineId = *b"rand";

/// The KeyTypeId used for VRF keys
pub const VRF_KEY_ID: KeyTypeId = KeyTypeId(VRF_ENGINE_ID);

// The strongly-typed crypto wrappers to be used by VRF in the keystore
mod vrf_crypto {
	use super::*;
	use sp_application_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, VRF_KEY_ID);
}

/// A vrf public key.
pub type VrfId = vrf_crypto::Public;

/// A vrf signature.
pub type VrfSignature = vrf_crypto::Signature;

sp_application_crypto::with_pair! {
	/// A vrf key pair
	pub type VrfPair = vrf_crypto::Pair;
}

#[test]
fn creating_dummy_vrf_id_from_aura_id_is_sane() {
	for x in 0u8..10u8 {
		let aura_id: AuraId = sr25519::Public::unchecked_from([x; 32]).into();
		let expected_vrf_id: VrfId = sr25519::Public::unchecked_from([x; 32]).into();
		let aura_to_vrf_id: VrfId = dummy_key_from(aura_id);
		assert_eq!(expected_vrf_id, aura_to_vrf_id);
	}
}
