//! Tests for vault metadata functionality.

use crate::{mock::*, Error, Event, VaultMetadataStore};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

#[test]
fn set_vault_metadata_works() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name = b"Test Vault".to_vec();
		let logo = b"http://example.com/logo.png".to_vec();
		let origin = RuntimeOrigin::signed(mock_pub_key(1)); // Assuming mock_pub_key(1) is authorized

		// Set metadata
		assert_ok!(RewardsPallet::set_vault_metadata(
			origin.clone(),
			vault_id,
			name.clone(),
			logo.clone()
		));

		// Check storage
		let stored_metadata = VaultMetadataStore::<Runtime>::get(vault_id).unwrap();
		assert_eq!(stored_metadata.name.to_vec(), name);
		assert_eq!(stored_metadata.logo.to_vec(), logo);

		// Check event
		System::assert_last_event(RuntimeEvent::RewardsPallet(Event::VaultMetadataSet {
			vault_id,
			name: name.try_into().unwrap(),
			logo: logo.try_into().unwrap(),
		}));
	});
}

#[test]
fn set_vault_metadata_updates_existing() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name1 = b"Test Vault".to_vec();
		let logo1 = b"http://example.com/logo.png".to_vec();
		let name2 = b"Updated Vault".to_vec();
		let logo2 = b"http://example.com/new_logo.png".to_vec();
		let origin = RuntimeOrigin::signed(mock_pub_key(1));

		// Set initial metadata
		assert_ok!(RewardsPallet::set_vault_metadata(
			origin.clone(),
			vault_id,
			name1.clone(),
			logo1.clone()
		));

		// Update metadata
		assert_ok!(RewardsPallet::set_vault_metadata(
			origin.clone(),
			vault_id,
			name2.clone(),
			logo2.clone()
		));

		// Check storage for updated values
		let stored_metadata = VaultMetadataStore::<Runtime>::get(vault_id).unwrap();
		assert_eq!(stored_metadata.name.to_vec(), name2);
		assert_eq!(stored_metadata.logo.to_vec(), logo2);

		// Check event for update
		System::assert_last_event(RuntimeEvent::RewardsPallet(Event::VaultMetadataSet {
			vault_id,
			name: name2.try_into().unwrap(),
			logo: logo2.try_into().unwrap(),
		}));
	});
}

#[test]
fn set_vault_metadata_fails_name_too_long() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let long_name = vec![0u8; 100]; // Exceeds MaxVaultNameLength (64)
		let logo = b"logo.png".to_vec();
		let origin = RuntimeOrigin::signed(mock_pub_key(1));

		assert_noop!(
			RewardsPallet::set_vault_metadata(origin, vault_id, long_name, logo),
			Error::<Runtime>::NameTooLong
		);
	});
}

#[test]
fn set_vault_metadata_fails_logo_too_long() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name = b"Test".to_vec();
		let long_logo = vec![0u8; 300]; // Exceeds MaxVaultLogoLength (256)
		let origin = RuntimeOrigin::signed(mock_pub_key(1));

		assert_noop!(
			RewardsPallet::set_vault_metadata(origin, vault_id, name, long_logo),
			Error::<Runtime>::LogoTooLong
		);
	});
}

#[test]
fn set_vault_metadata_fails_bad_origin() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name = b"Test".to_vec();
		let logo = b"logo.png".to_vec();
		let bad_origin = RuntimeOrigin::none(); // Unauthorized origin

		assert_noop!(
			RewardsPallet::set_vault_metadata(bad_origin, vault_id, name, logo),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn remove_vault_metadata_works() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name = b"Test Vault".to_vec();
		let logo = b"logo.png".to_vec();
		let origin = RuntimeOrigin::signed(mock_pub_key(1));

		// Set metadata first
		assert_ok!(RewardsPallet::set_vault_metadata(origin.clone(), vault_id, name, logo));
		assert!(VaultMetadataStore::<Runtime>::contains_key(vault_id));

		// Remove metadata
		assert_ok!(RewardsPallet::remove_vault_metadata(origin.clone(), vault_id));

		// Check storage is empty
		assert!(!VaultMetadataStore::<Runtime>::contains_key(vault_id));

		// Check event
		System::assert_last_event(RuntimeEvent::RewardsPallet(Event::VaultMetadataRemoved {
			vault_id,
		}));
	});
}

#[test]
fn remove_vault_metadata_fails_not_found() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let origin = RuntimeOrigin::signed(mock_pub_key(1));

		// Ensure metadata doesn't exist
		assert!(!VaultMetadataStore::<Runtime>::contains_key(vault_id));

		assert_noop!(
			RewardsPallet::remove_vault_metadata(origin, vault_id),
			Error::<Runtime>::VaultMetadataNotFound
		);
	});
}

#[test]
fn remove_vault_metadata_fails_bad_origin() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let name = b"Test".to_vec();
		let logo = b"logo.png".to_vec();
		let origin = RuntimeOrigin::signed(mock_pub_key(1));
		let bad_origin = RuntimeOrigin::none();

		// Set metadata first
		assert_ok!(RewardsPallet::set_vault_metadata(origin, vault_id, name, logo));

		// Attempt remove with bad origin
		assert_noop!(
			RewardsPallet::remove_vault_metadata(bad_origin, vault_id),
			DispatchError::BadOrigin
		);

		// Ensure metadata still exists
		assert!(VaultMetadataStore::<Runtime>::contains_key(vault_id));
	});
}
