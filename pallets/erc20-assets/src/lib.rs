#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_core::{H160, U256};
use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_assets::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new asset was created for an ERC20 token
        /// [erc20_address, asset_id]
        AssetCreated(H160, T::AssetId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Failed to create the asset
        AssetCreationFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates a new native asset for an ERC20 token
        /// The asset ID is deterministically derived from the ERC20 address
        ///
        /// Parameters:
        /// - `origin`: Must be signed by an account
        /// - `erc20_address`: The address of the ERC20 token
        /// - `admin`: The account that will administer the asset
        ///
        /// Emits `AssetCreated` on success.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_asset())]
        pub fn create_asset(
            origin: OriginFor<T>,
            erc20_address: H160,
            admin: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let _ = ensure_signed(origin.clone())?;
            
            // Convert H160 to AssetId deterministically
            let asset_id = Self::address_to_asset_id(erc20_address)
                .ok_or(Error::<T>::AssetCreationFailed)?;

            // Create the asset
            pallet_assets::Pallet::<T>::create(
                origin,
                asset_id,
                admin,
                1u128.into(), // Minimum balance of 1
            )
            .map_err(|_| Error::<T>::AssetCreationFailed)?;

            // Emit event
            Self::deposit_event(Event::AssetCreated(erc20_address, asset_id));

            Ok(())
        }
    }
}

/// Weight functions needed for pallet_erc20_assets.
pub trait WeightInfo {
    fn create_asset() -> Weight;
}

impl WeightInfo for () {
    fn create_asset() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}

// Implement AddressToAssetId trait
impl<T: Config> pallet_evm_precompileset_assets_erc20::AddressToAssetId<T::AssetId> for Pallet<T> 
where
    T::AssetId: From<U256> + Into<U256>,
{
    fn address_to_asset_id(address: H160) -> Option<T::AssetId> {
        // Convert the H160 address to U256
        let mut bytes = [0u8; 32];
        bytes[12..32].copy_from_slice(address.as_bytes());
        let asset_id = U256::from_big_endian(&bytes);
        
        // Convert U256 to T::AssetId
        Some(asset_id.into())
    }

    fn asset_id_to_address(asset_id: T::AssetId) -> H160 {
        // Convert T::AssetId to U256
        let asset_id: U256 = asset_id.into();
        
        // Take the last 20 bytes of the U256 to create an H160
        let mut bytes = [0u8; 32];
        asset_id.to_big_endian(&mut bytes);
        H160::from_slice(&bytes[12..32])
    }
}
