// Copyright (C) Polytope Labs Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod impls;
pub mod types;

mod benchmarking;
mod weights;
use crate::impls::{convert_to_balance, convert_to_erc20};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use frame_support::{
	ensure,
	traits::{
		fungibles::{self, Mutate},
		tokens::{fungible::Mutate as FungibleMutate, Preservation},
		Currency, ExistenceRequirement,
	},
};
use parity_scale_codec::{Decode, Encode};
pub use weights::WeightInfo;

use ismp::{
	events::Meta,
	router::{PostRequest, Request, Response, Timeout},
};

use sp_core::{Get, H160, U256};
use sp_runtime::{traits::Dispatchable, MultiSignature};
use token_gateway_primitives::{PALLET_TOKEN_GATEWAY_ID, TOKEN_GOVERNOR_ID};
use types::{AssetId, Body, BodyWithCall, EvmToSubstrate, RequestBody, SubstrateCalldata};

use alloc::{string::ToString, vec, vec::Vec};
use frame_system::RawOrigin;
use ismp::module::IsmpModule;
use primitive_types::H256;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use alloc::collections::BTreeMap;
	use pallet_hyperbridge::PALLET_HYPERBRIDGE;
	use sp_runtime::traits::AccountIdConversion;
	use types::{AssetRegistration, PrecisionUpdate, TeleportParams};

	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::{Fortitude, Precision, Preservation},
			Currency, ExistenceRequirement, WithdrawReasons,
		},
	};
	use frame_system::pallet_prelude::*;
	use ismp::{
		dispatcher::{DispatchPost, DispatchRequest, FeeMetadata, IsmpDispatcher},
		host::StateMachine,
	};
	use pallet_hyperbridge::{SubstrateHostParams, VersionedHostParams};
	use sp_runtime::traits::Zero;
	use token_gateway_primitives::{GatewayAssetUpdate, RemoteERC6160AssetRegistration};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_ismp::Config + pallet_hyperbridge::Config
	{
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The [`IsmpDispatcher`] for dispatching cross-chain requests
		type Dispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = Self::Balance>;

		/// A currency implementation for interacting with the native asset
		type NativeCurrency: Currency<Self::AccountId>;

		/// A funded account that would be set as asset admin and also make payments for asset
		/// creation
		type AssetAdmin: Get<Self::AccountId>;

		/// Account that is authorized to create and update assets.
		type CreateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Fungible asset implementation
		type Assets: fungibles::Mutate<Self::AccountId>
			+ fungibles::metadata::Mutate<Self::AccountId>;

		/// The native asset ID
		type NativeAssetId: Get<AssetId<Self>>;

		/// The decimals of the native currency
		#[pallet::constant]
		type Decimals: Get<u8>;

		/// A trait that converts an evm address to a substrate account
		/// Used for authenticating incoming cross-chain runtime calls.
		type EvmToSubstrate: EvmToSubstrate<Self>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	/// Assets supported by this instance of token gateway
	/// A map of the local asset id to the token gateway asset id
	#[pallet::storage]
	pub type SupportedAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId<T>, H256, OptionQuery>;

	/// Assets that originate from this chain
	#[pallet::storage]
	pub type NativeAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId<T>, bool, ValueQuery>;

	/// Assets supported by this instance of token gateway
	/// A map of the token gateway asset id to the local asset id
	#[pallet::storage]
	pub type LocalAssets<T: Config> = StorageMap<_, Identity, H256, AssetId<T>, OptionQuery>;

	/// The decimals used by the EVM counterpart of this asset
	#[pallet::storage]
	pub type Precisions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetId<T>,
		Blake2_128Concat,
		StateMachine,
		u8,
		OptionQuery,
	>;

	/// The token gateway adresses on different chains
	#[pallet::storage]
	pub type TokenGatewayAddresses<T: Config> =
		StorageMap<_, Blake2_128Concat, StateMachine, Vec<u8>, OptionQuery>;

	/// Pallet events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An asset has been teleported
		AssetTeleported {
			/// Source account
			from: T::AccountId,
			/// beneficiary account on destination
			to: H256,
			/// Amount transferred
			amount: <T::NativeCurrency as Currency<T::AccountId>>::Balance,
			/// Destination chain
			dest: StateMachine,
			/// Request commitment
			commitment: H256,
		},

		/// An asset has been received and transferred to the beneficiary's account
		AssetReceived {
			/// beneficiary account on relaychain
			beneficiary: T::AccountId,
			/// Amount transferred
			amount: <<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance,
			/// Destination chain
			source: StateMachine,
		},

		/// An asset has been refunded and transferred to the beneficiary's account
		AssetRefunded {
			/// beneficiary account on relaychain
			beneficiary: T::AccountId,
			/// Amount transferred
			amount: <<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance,
			/// Destination chain
			source: StateMachine,
		},

		/// ERC6160 asset creation request dispatched to hyperbridge
		ERC6160AssetRegistrationDispatched {
			/// Request commitment
			commitment: H256,
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// A asset that has not been registered
		UnregisteredAsset,
		/// Error while teleporting asset
		AssetTeleportError,
		/// Coprocessor was not configured in the runtime
		CoprocessorNotConfigured,
		/// Asset or update Dispatch Error
		DispatchError,
		/// Asset Id creation failed
		AssetCreationError,
		/// Asset decimals not found
		AssetDecimalsNotFound,
		/// Protocol Params have not been initialized
		NotInitialized,
		/// Unknown Asset
		UnknownAsset,
		/// Only root or asset owner can update asset
		NotAssetOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::AccountId: From<[u8; 32]>,
		u128: From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
		<T as pallet_ismp::Config>::Balance:
			From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
		<<T as Config>::Assets as fungibles::Inspect<T::AccountId>>::Balance:
			From<<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance>,
		<<T as Config>::Assets as fungibles::Inspect<T::AccountId>>::Balance: From<u128>,
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
	{
		/// Teleports a registered asset
		/// locks the asset and dispatches a request to token gateway on the destination
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::teleport())]
		pub fn teleport(
			origin: OriginFor<T>,
			params: TeleportParams<
				AssetId<T>,
				<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance,
			>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let dispatcher = <T as Config>::Dispatcher::default();
			let asset_id = SupportedAssets::<T>::get(params.asset_id.clone())
				.ok_or_else(|| Error::<T>::UnregisteredAsset)?;
			let decimals = if params.asset_id == T::NativeAssetId::get() {
				// Custody funds in pallet
				let is_native = NativeAssets::<T>::get(T::NativeAssetId::get());
				if is_native {
					<T as Config>::NativeCurrency::transfer(
						&who,
						&Self::pallet_account(),
						params.amount,
						ExistenceRequirement::AllowDeath,
					)?;
				} else {
					// Reduce total supply
					let imbalance = <T as Config>::NativeCurrency::burn(params.amount);
					// Burn amount from account
					<T as Config>::NativeCurrency::settle(
						&who,
						imbalance,
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::AllowDeath,
					)
					.map_err(|_| Error::<T>::AssetTeleportError)?;
				}

				T::Decimals::get()
			} else {
				let is_native = NativeAssets::<T>::get(params.asset_id.clone());
				if is_native {
					<T as Config>::Assets::transfer(
						params.asset_id.clone(),
						&who,
						&Self::pallet_account(),
						params.amount.into(),
						Preservation::Expendable,
					)?;
				} else {
					// Assets that do not originate from this chain are burned
					<T as Config>::Assets::burn_from(
						params.asset_id.clone(),
						&who,
						params.amount.into(),
						Preservation::Expendable,
						Precision::Exact,
						Fortitude::Polite,
					)?;
				}

				<T::Assets as fungibles::metadata::Inspect<T::AccountId>>::decimals(
					params.asset_id.clone(),
				)
			};

			let to = params.recepient.0;
			let from: [u8; 32] = who.clone().into();
			let erc_decimals = Precisions::<T>::get(params.asset_id, params.destination)
				.ok_or_else(|| Error::<T>::AssetDecimalsNotFound)?;

			let body = match params.call_data {
				Some(data) => {
					let body = BodyWithCall {
						amount: {
							let amount: u128 = params.amount.into();
							let mut bytes = [0u8; 32];
							convert_to_erc20(amount, erc_decimals, decimals)
								.to_big_endian(&mut bytes);
							alloy_primitives::U256::from_be_bytes(bytes)
						},
						asset_id: asset_id.0.into(),
						redeem: params.redeem,
						from: from.into(),
						to: to.into(),
						data: data.into(),
					};

					// Prefix with the handleIncomingAsset enum variant
					let mut encoded = vec![0];
					encoded.extend_from_slice(&BodyWithCall::abi_encode(&body));
					encoded
				},

				None => {
					let body = Body {
						amount: {
							let amount: u128 = params.amount.into();
							let mut bytes = [0u8; 32];
							convert_to_erc20(amount, erc_decimals, decimals)
								.to_big_endian(&mut bytes);
							alloy_primitives::U256::from_be_bytes(bytes)
						},
						asset_id: asset_id.0.into(),
						redeem: params.redeem,
						from: from.into(),
						to: to.into(),
					};

					// Prefix with the handleIncomingAsset enum variant
					let mut encoded = vec![0];
					encoded.extend_from_slice(&Body::abi_encode(&body));
					encoded
				},
			};

			let dispatch_post = DispatchPost {
				dest: params.destination,
				from: PALLET_TOKEN_GATEWAY_ID.to_vec(),
				to: params.token_gateway,
				timeout: params.timeout,
				body,
			};

			let metadata = FeeMetadata { payer: who.clone(), fee: params.relayer_fee.into() };
			let commitment = dispatcher
				.dispatch_request(DispatchRequest::Post(dispatch_post), metadata)
				.map_err(|_| Error::<T>::AssetTeleportError)?;

			Self::deposit_event(Event::<T>::AssetTeleported {
				from: who,
				to: params.recepient,
				dest: params.destination,
				amount: params.amount,
				commitment,
			});
			Ok(())
		}

		/// Set the token gateway address for specified chains
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::set_token_gateway_addresses(addresses.len() as u32))]
		pub fn set_token_gateway_addresses(
			origin: OriginFor<T>,
			addresses: BTreeMap<StateMachine, Vec<u8>>,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;
			for (chain, address) in addresses {
				TokenGatewayAddresses::<T>::insert(chain, address.clone());
			}
			Ok(())
		}

		/// Registers a multi-chain ERC6160 asset. The asset should not already exist.
		///
		/// This works by dispatching a request to the TokenGateway module on each requested chain
		/// to create the asset.
		/// `native` should be true if this asset originates from this chain
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_erc6160_asset(asset.precision.len() as u32))]
		pub fn create_erc6160_asset(
			origin: OriginFor<T>,
			asset: AssetRegistration<AssetId<T>>,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;
			let who = T::AssetAdmin::get();
			// charge hyperbridge fees
			let VersionedHostParams::V1(SubstrateHostParams { asset_registration_fee, .. }) =
				pallet_hyperbridge::Pallet::<T>::host_params();

			if asset_registration_fee != Zero::zero() {
				T::Currency::transfer(
					&who,
					&PALLET_HYPERBRIDGE.into_account_truncating(),
					asset_registration_fee.into(),
					Preservation::Expendable,
				)?;
			}

			let asset_id: H256 = sp_io::hashing::keccak_256(asset.reg.symbol.as_ref()).into();
			// If the local asset id already exists we do not change it's metadata we only store
			// the mapping to its token gateway asset id

			SupportedAssets::<T>::insert(asset.local_id.clone(), asset_id.clone());
			NativeAssets::<T>::insert(asset.local_id.clone(), asset.native);
			LocalAssets::<T>::insert(asset_id, asset.local_id.clone());
			for (state_machine, precision) in asset.precision {
				Precisions::<T>::insert(asset.local_id.clone(), state_machine, precision);
			}

			let dispatcher = <T as Config>::Dispatcher::default();
			let dispatch_post = DispatchPost {
				dest: T::Coprocessor::get().ok_or_else(|| Error::<T>::CoprocessorNotConfigured)?,
				from: PALLET_TOKEN_GATEWAY_ID.to_vec(),
				to: TOKEN_GOVERNOR_ID.to_vec(),
				timeout: 0,
				body: { RemoteERC6160AssetRegistration::CreateAsset(asset.reg).encode() },
			};

			let metadata = FeeMetadata { payer: who, fee: Default::default() };

			let commitment = dispatcher
				.dispatch_request(DispatchRequest::Post(dispatch_post), metadata)
				.map_err(|_| Error::<T>::DispatchError)?;
			Self::deposit_event(Event::<T>::ERC6160AssetRegistrationDispatched { commitment });

			Ok(())
		}

		/// Registers a multi-chain ERC6160 asset. The asset should not already exist.
		///
		/// This works by dispatching a request to the TokenGateway module on each requested chain
		/// to create the asset.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_erc6160_asset())]
		pub fn update_erc6160_asset(
			origin: OriginFor<T>,
			asset: GatewayAssetUpdate,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;
			let who = T::AssetAdmin::get();

			// charge hyperbridge fees
			let VersionedHostParams::V1(SubstrateHostParams { asset_registration_fee, .. }) =
				pallet_hyperbridge::Pallet::<T>::host_params();

			if asset_registration_fee != Zero::zero() {
				T::Currency::transfer(
					&who,
					&PALLET_HYPERBRIDGE.into_account_truncating(),
					asset_registration_fee.into(),
					Preservation::Expendable,
				)?;
			}

			let dispatcher = <T as Config>::Dispatcher::default();
			let dispatch_post = DispatchPost {
				dest: T::Coprocessor::get().ok_or_else(|| Error::<T>::CoprocessorNotConfigured)?,
				from: PALLET_TOKEN_GATEWAY_ID.to_vec(),
				to: TOKEN_GOVERNOR_ID.to_vec(),
				timeout: 0,
				body: { RemoteERC6160AssetRegistration::UpdateAsset(asset).encode() },
			};

			let metadata = FeeMetadata { payer: who, fee: Default::default() };

			let commitment = dispatcher
				.dispatch_request(DispatchRequest::Post(dispatch_post), metadata)
				.map_err(|_| Error::<T>::DispatchError)?;
			Self::deposit_event(Event::<T>::ERC6160AssetRegistrationDispatched { commitment });

			Ok(())
		}

		/// Update the precision for an existing asset
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::update_asset_precision(update.precisions.len() as u32))]
		pub fn update_asset_precision(
			origin: OriginFor<T>,
			update: PrecisionUpdate<AssetId<T>>,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;
			for (chain, precision) in update.precisions {
				Precisions::<T>::insert(update.asset_id.clone(), chain, precision);
			}
			Ok(())
		}
	}

	// Hack for implementing the [`Default`] bound needed for
	// [`IsmpDispatcher`](ismp::dispatcher::IsmpDispatcher) and
	// [`IsmpModule`](ismp::module::IsmpModule)
	impl<T> Default for Pallet<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}
}

impl<T: Config> IsmpModule for Pallet<T>
where
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
	<<T as Config>::NativeCurrency as Currency<T::AccountId>>::Balance: From<u128>,
	<<T as Config>::Assets as fungibles::Inspect<T::AccountId>>::Balance: From<u128>,
{
	fn on_accept(
		&self,
		PostRequest { body, from, source, dest, nonce, .. }: PostRequest,
	) -> Result<(), anyhow::Error> {
		let expected = TokenGatewayAddresses::<T>::get(source)
			.ok_or_else(|| anyhow!("Not configured to receive assets from {source:?}"))?;
		ensure!(
			from == expected,
			ismp::error::Error::ModuleDispatchError {
				msg: "Token Gateway: Unknown source contract address".to_string(),
				meta: Meta { source, dest, nonce },
			}
		);

		let body: RequestBody = if let Ok(body) = Body::abi_decode(&mut &body[1..], true) {
			body.into()
		} else if let Ok(body) = BodyWithCall::abi_decode(&mut &body[1..], true) {
			body.into()
		} else {
			Err(anyhow!("Token Gateway: Failed to decode request body"))?
		};

		let local_asset_id =
			LocalAssets::<T>::get(H256::from(body.asset_id.0)).ok_or_else(|| {
				ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Unknown asset".to_string(),
					meta: Meta { source, dest, nonce },
				}
			})?;

		let decimals = if local_asset_id == T::NativeAssetId::get() {
			T::Decimals::get()
		} else {
			<T::Assets as fungibles::metadata::Inspect<T::AccountId>>::decimals(
				local_asset_id.clone(),
			)
		};
		let erc_decimals = Precisions::<T>::get(local_asset_id.clone(), source)
			.ok_or_else(|| anyhow!("Asset decimals not configured"))?;
		let amount = convert_to_balance(
			U256::from_big_endian(&body.amount.to_be_bytes::<32>()),
			erc_decimals,
			decimals,
		)
		.map_err(|_| ismp::error::Error::ModuleDispatchError {
			msg: "Token Gateway: Trying to withdraw Invalid amount".to_string(),
			meta: Meta { source, dest, nonce },
		})?;
		let beneficiary: T::AccountId = body.to.0.into();
		if local_asset_id == T::NativeAssetId::get() {
			let is_native = NativeAssets::<T>::get(T::NativeAssetId::get());
			if is_native {
				<T as Config>::NativeCurrency::transfer(
					&Pallet::<T>::pallet_account(),
					&beneficiary,
					amount.into(),
					ExistenceRequirement::AllowDeath,
				)
				.map_err(|_| ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Failed to complete asset transfer".to_string(),
					meta: Meta { source, dest, nonce },
				})?;
			} else {
				// Increase total supply
				let imbalance = <T as Config>::NativeCurrency::issue(amount.into());
				// Mint into the beneficiary account
				<T as Config>::NativeCurrency::resolve_creating(&beneficiary, imbalance);
			}
		} else {
			// Assets that do not originate from this chain are minted
			let is_native = NativeAssets::<T>::get(local_asset_id.clone());
			if is_native {
				<T as Config>::Assets::transfer(
					local_asset_id,
					&Pallet::<T>::pallet_account(),
					&beneficiary,
					amount.into(),
					Preservation::Expendable,
				)
				.map_err(|_| ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Failed to complete asset transfer".to_string(),
					meta: Meta { source, dest, nonce },
				})?;
			} else {
				<T as Config>::Assets::mint_into(local_asset_id, &beneficiary, amount.into())
					.map_err(|_| ismp::error::Error::ModuleDispatchError {
						msg: "Token Gateway: Failed to complete asset transfer".to_string(),
						meta: Meta { source, dest, nonce },
					})?;
			}
		}

		if let Some(call_data) = body.data {
			let substrate_data = SubstrateCalldata::decode(&mut &call_data.0[..])
				.map_err(|err| anyhow!("Calldata decode error: {err:?}"))?;
			// Verify signature against encoded runtime call
			let nonce = frame_system::Pallet::<T>::account_nonce(beneficiary.clone());
			let payload = (nonce, substrate_data.runtime_call.clone()).encode();
			let message = sp_io::hashing::keccak_256(&payload);

			let multi_signature = MultiSignature::decode(&mut &*substrate_data.signature)
				.map_err(|err| anyhow!("Signature decode error: {err:?}"))?;

			match multi_signature {
				MultiSignature::Ed25519(sig) => {
					let pub_key = body.to.0.as_slice().try_into().map_err(|_| {
						anyhow!("Failed to decode beneficiary as Ed25519 public key")
					})?;
					if !sp_io::crypto::ed25519_verify(&sig, message.as_ref(), &pub_key) {
						Err(anyhow!(
							"Failed to verify ed25519 signature before dispatching token gateway call"
						))?
					}
				},
				MultiSignature::Sr25519(sig) => {
					let pub_key = body.to.0.as_slice().try_into().map_err(|_| {
						anyhow!("Failed to decode beneficiary as Sr25519 public key")
					})?;
					if !sp_io::crypto::sr25519_verify(&sig, message.as_ref(), &pub_key) {
						Err(anyhow!(
							"Failed to verify sr25519 signature before dispatching token gateway call"
						))?
					}
				},
				MultiSignature::Ecdsa(sig) => {
					let pub_key = sp_io::crypto::secp256k1_ecdsa_recover(&sig.0, &message)
						.map_err(|_| {
							anyhow!("Failed to recover ecdsa public key from signature")
						})?;
					let eth_address =
						H160::from_slice(&sp_io::hashing::keccak_256(&pub_key[..])[12..]);
					let substrate_account = T::EvmToSubstrate::convert(eth_address);
					if substrate_account != beneficiary {
						Err(anyhow!(
							"Failed to verify signature before dispatching token gateway call"
						))?
					}
				},
			}

			let runtime_call = T::RuntimeCall::decode(&mut &*substrate_data.runtime_call)
				.map_err(|err| anyhow!("RuntimeCall decode error: {err:?}"))?;
			runtime_call
				.dispatch(RawOrigin::Signed(beneficiary.clone()).into())
				.map_err(|e| anyhow!("Call dispatch executed with error {:?}", e.error))?;
			// Increase account nonce to ensure the call cannot be replayed
			frame_system::Pallet::<T>::inc_account_nonce(beneficiary.clone());
		}

		Self::deposit_event(Event::<T>::AssetReceived {
			beneficiary,
			amount: amount.into(),
			source,
		});
		Ok(())
	}

	fn on_response(&self, _response: Response) -> Result<(), anyhow::Error> {
		Err(anyhow!("Module does not accept responses".to_string()))
	}

	fn on_timeout(&self, request: Timeout) -> Result<(), anyhow::Error> {
		match request {
			Timeout::Request(Request::Post(PostRequest { body, source, dest, nonce, .. })) => {
				let body: RequestBody = if let Ok(body) = Body::abi_decode(&mut &body[1..], true) {
					body.into()
				} else if let Ok(body) = BodyWithCall::abi_decode(&mut &body[1..], true) {
					body.into()
				} else {
					Err(anyhow!("Token Gateway: Failed to decode request body"))?
				};
				let beneficiary = body.from.0.into();
				let local_asset_id = LocalAssets::<T>::get(H256::from(body.asset_id.0))
					.ok_or_else(|| ismp::error::Error::ModuleDispatchError {
						msg: "Token Gateway: Unknown asset".to_string(),
						meta: Meta { source, dest, nonce },
					})?;
				let decimals = if local_asset_id == T::NativeAssetId::get() {
					T::Decimals::get()
				} else {
					<T::Assets as fungibles::metadata::Inspect<T::AccountId>>::decimals(
						local_asset_id.clone(),
					)
				};
				let erc_decimals = Precisions::<T>::get(local_asset_id.clone(), dest)
					.ok_or_else(|| anyhow!("Asset decimals not configured"))?;
				let amount = convert_to_balance(
					U256::from_big_endian(&body.amount.to_be_bytes::<32>()),
					erc_decimals,
					decimals,
				)
				.map_err(|_| ismp::error::Error::ModuleDispatchError {
					msg: "Token Gateway: Trying to withdraw Invalid amount".to_string(),
					meta: Meta { source, dest, nonce },
				})?;

				if local_asset_id == T::NativeAssetId::get() {
					let is_native = NativeAssets::<T>::get(T::NativeAssetId::get());
					if is_native {
						<T as Config>::NativeCurrency::transfer(
							&Pallet::<T>::pallet_account(),
							&beneficiary,
							amount.into(),
							ExistenceRequirement::AllowDeath,
						)
						.map_err(|_| ismp::error::Error::ModuleDispatchError {
							msg: "Token Gateway: Failed to complete asset transfer".to_string(),
							meta: Meta { source, dest, nonce },
						})?;
					} else {
						let imbalance = <T as Config>::NativeCurrency::issue(amount.into());
						<T as Config>::NativeCurrency::resolve_creating(&beneficiary, imbalance);
					}
				} else {
					// Assets that do not originate from this chain are minted
					let is_native = NativeAssets::<T>::get(local_asset_id.clone());
					if is_native {
						<T as Config>::Assets::transfer(
							local_asset_id,
							&Pallet::<T>::pallet_account(),
							&beneficiary,
							amount.into(),
							Preservation::Expendable,
						)
						.map_err(|_| ismp::error::Error::ModuleDispatchError {
							msg: "Token Gateway: Failed to complete asset transfer".to_string(),
							meta: Meta { source, dest, nonce },
						})?;
					} else {
						<T as Config>::Assets::mint_into(
							local_asset_id,
							&beneficiary,
							amount.into(),
						)
						.map_err(|_| ismp::error::Error::ModuleDispatchError {
							msg: "Token Gateway: Failed to complete asset transfer".to_string(),
							meta: Meta { source, dest, nonce },
						})?;
					}
				}

				Pallet::<T>::deposit_event(Event::<T>::AssetRefunded {
					beneficiary,
					amount: amount.into(),
					source: dest,
				});
			},
			Timeout::Request(Request::Get(get)) => Err(ismp::error::Error::ModuleDispatchError {
				msg: "Tried to timeout unsupported request type".to_string(),
				meta: Meta { source: get.source, dest: get.dest, nonce: get.nonce },
			})?,

			Timeout::Response(response) => Err(ismp::error::Error::ModuleDispatchError {
				msg: "Tried to timeout unsupported request type".to_string(),
				meta: Meta {
					source: response.source_chain(),
					dest: response.dest_chain(),
					nonce: response.nonce(),
				},
			})?,
		}
		Ok(())
	}
}
