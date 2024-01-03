// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Pallet to process claims from Ethereum addresses.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod utils;
mod weights;
use weights::WeightInfo;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use crate::utils::{
	ethereum_address::{EcdsaSignature, EthereumAddress},
	MultiAddress, MultiAddressSignature,
};
use frame_support::{
	ensure,
	traits::{Currency, Get, VestingSchedule},
};
pub use pallet::*;
use pallet_evm::AddressMapping;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{self, Deserialize, Serialize};
use sp_core::{sr25519::Public, H160};
use sp_io::{
	crypto::{secp256k1_ecdsa_recover, sr25519_verify},
	hashing::keccak_256,
};
use sp_runtime::{
	traits::{CheckedSub, Zero},
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::{convert::TryInto, prelude::*, vec};
use utils::Sr25519Signature;
/// Custom validity errors used in Polkadot while validating transactions.
#[repr(u8)]
pub enum ValidityError {
	/// The Ethereum signature is invalid.
	InvalidEthereumSignature = 0,
	/// The signer has no claim.
	SignerHasNoClaim = 1,
	/// No permission to execute the call.
	NoPermission = 2,
	/// An invalid statement was made for a claim.
	InvalidStatement = 3,
}

impl From<ValidityError> for u8 {
	fn from(err: ValidityError) -> Self {
		err as u8
	}
}

type CurrencyOf<T> = <<T as Config>::VestingSchedule as VestingSchedule<
	<T as frame_system::Config>::AccountId,
>>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// The kind of statement an account needs to make for a claim to be valid.
#[derive(
	Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo, Serialize, Deserialize,
)]
pub enum StatementKind {
	/// Statement required to be made by non-SAFE holders.
	Regular,
	/// Statement required to be made by SAFE holders.
	Safe,
}

impl StatementKind {
	/// Convert this to the (English) statement it represents.
	fn to_text(self) -> &'static [u8] {
		match self {
			StatementKind::Regular =>
				&b"I hereby agree to the terms of the statement whose sha2256sum is \
				5627de05cfe235cd4ffa0d6375c8a5278b89cc9b9e75622fa2039f4d1b43dadf. (This may be found at the URL: \
				https://statement.tangle.tools/airdrop-statement.html)"[..],
			StatementKind::Safe =>
				&b"I hereby agree to the terms of the statement whose sha2256sum is \
				7eae145b00c1912c8b01674df5df4ad9abcf6d18ea3f33d27eb6897a762f4273. (This may be found at the URL: \
				https://https://statement.tangle.tools/safe-claim-statement)"[..],
		}
	}
}

impl Default for StatementKind {
	fn default() -> Self {
		StatementKind::Regular
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type VestingSchedule: VestingSchedule<Self::AccountId, Moment = BlockNumberFor<Self>>;
		#[pallet::constant]
		type Prefix: Get<&'static [u8]>;
		type MoveClaimOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// RuntimeOrigin permitted to call force_ extrinsics
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type MaxVestingSchedules: Get<u32>;
		type WeightInfo: weights::WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Someone claimed some native tokens.
		Claimed { recipient: T::AccountId, source: MultiAddress, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid Ethereum signature.
		InvalidEthereumSignature,
		/// Invalid Native (sr25519) signature
		InvalidNativeSignature,
		/// Invalid Native account decoding
		InvalidNativeAccount,
		/// Ethereum address has no claim.
		SignerHasNoClaim,
		/// Account ID sending transaction has no claim.
		SenderHasNoClaim,
		/// There's not enough in the pot to pay out some unvested amount. Generally implies a
		/// logic error.
		PotUnderflow,
		/// A needed statement was not included.
		InvalidStatement,
		/// The account already has a vested balance.
		VestedBalanceExists,
	}

	#[pallet::storage]
	#[pallet::getter(fn claims)]
	pub(super) type Claims<T: Config> = StorageMap<_, Identity, MultiAddress, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	pub(super) type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Expiry block and account to deposit expired funds
	#[pallet::storage]
	#[pallet::getter(fn expiry_time)]
	pub(super) type ExpiryConfig<T: Config> = StorageValue<_, (BlockNumberFor<T>, MultiAddress)>;

	/// Vesting schedule for a claim.
	/// First balance is the total amount that should be held for vesting.
	/// Second balance is how much should be unlocked per block.
	/// The block number is when the vesting should start.
	#[pallet::storage]
	#[pallet::getter(fn vesting)]
	pub(super) type Vesting<T: Config> = StorageMap<
		_,
		Identity,
		MultiAddress,
		BoundedVec<(BalanceOf<T>, BalanceOf<T>, BlockNumberFor<T>), T::MaxVestingSchedules>,
	>;

	/// The statement kind that must be signed, if any.
	#[pallet::storage]
	pub(super) type Signing<T: Config> = StorageMap<_, Identity, MultiAddress, StatementKind>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub claims: Vec<(MultiAddress, BalanceOf<T>, Option<StatementKind>)>,
		pub vesting: Vec<(
			MultiAddress,
			BoundedVec<(BalanceOf<T>, BalanceOf<T>, BlockNumberFor<T>), T::MaxVestingSchedules>,
		)>,
		pub expiry: Option<(BlockNumberFor<T>, MultiAddress)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { claims: Default::default(), vesting: Default::default(), expiry: None }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// build `Claims`
			self.claims.iter().map(|(a, b, _)| (a.clone(), b.clone())).for_each(|(a, b)| {
				Claims::<T>::insert(a, b);
			});
			// build `Total`
			Total::<T>::put(
				self.claims.iter().fold(Zero::zero(), |acc: BalanceOf<T>, &(_, b, _)| acc + b),
			);
			// build `Vesting`
			self.vesting.iter().for_each(|(k, v)| {
				Vesting::<T>::insert(k, v);
			});
			// build `Signing`
			self.claims
				.iter()
				.filter_map(|(a, _, s)| Some((a.clone(), s.clone()?)))
				.for_each(|(a, s)| {
					Signing::<T>::insert(a, s);
				});
			// build expiryConfig
			ExpiryConfig::<T>::set(self.expiry.clone())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(current_block: BlockNumberFor<T>) {
			// check if we have an expiry time set and have we crossed the limit
			let expiry_config = ExpiryConfig::<T>::get();
			if let Some(expiry_config) = expiry_config {
				if current_block > expiry_config.0 {
					let unclaimed_amount = Total::<T>::take();
					log::info!("Claims : Expiry block passed, sweeping remaining amount of {:?} to destination", unclaimed_amount);
					let expiry_destination =
						match Self::convert_multi_address_to_account_id(expiry_config.1) {
							Ok(a) => a,
							Err(_) => return,
						};
					CurrencyOf::<T>::deposit_creating(&expiry_destination, unclaimed_amount);
					// clear the expiry detail
					ExpiryConfig::<T>::take();
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Make a claim to collect your tokens.
		///
		/// The dispatch origin for this call must be _None_.
		///
		/// Unsigned Validation:
		/// A call to claim is deemed valid if the signature provided matches
		/// the expected signed message of:
		///
		/// > Ethereum Signed Message:
		/// > (configured prefix string)(address)
		///
		/// and `address` matches the `dest` account.
		///
		/// Parameters:
		/// - `dest`: The destination account to payout the claim.
		/// - `ethereum_signature`: The signature of an ethereum signed message matching the format
		///   described above.
		///
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// Weight includes logic to validate unsigned `claim` call.
		///
		/// Total Complexity: O(1)
		/// </weight>
		#[pallet::weight(T::WeightInfo::claim())]
		#[pallet::call_index(0)]
		pub fn claim(
			origin: OriginFor<T>,
			dest: Option<MultiAddress>,
			signer: Option<MultiAddress>,
			signature: MultiAddressSignature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let data = dest.using_encoded(to_ascii_hex);
			let signer = Self::get_signer_multi_address(signer.clone(), signature, data, vec![])?;
			ensure!(Signing::<T>::get(&signer).is_none(), Error::<T>::InvalidStatement);
			Self::process_claim(signer, dest)?;
			Ok(())
		}

		/// Mint a new claim to collect native tokens.
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// Parameters:
		/// - `who`: The Ethereum address allowed to collect this claim.
		/// - `value`: The number of native tokens that will be claimed.
		/// - `vesting_schedule`: An optional vesting schedule for these native tokens.
		///
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// We assume worst case that both vesting and statement is being inserted.
		///
		/// Total Complexity: O(1)
		/// </weight>
		#[pallet::weight(T::WeightInfo::mint_claim())]
		#[pallet::call_index(1)]
		pub fn mint_claim(
			origin: OriginFor<T>,
			who: MultiAddress,
			value: BalanceOf<T>,
			vesting_schedule: Option<
				BoundedVec<(BalanceOf<T>, BalanceOf<T>, BlockNumberFor<T>), T::MaxVestingSchedules>,
			>,
			statement: Option<StatementKind>,
		) -> DispatchResult {
			ensure_root(origin)?;

			<Total<T>>::mutate(|t| *t += value);
			<Claims<T>>::insert(who.clone(), value);
			if let Some(vs) = vesting_schedule {
				<Vesting<T>>::insert(who.clone(), vs);
			}
			if let Some(s) = statement {
				Signing::<T>::insert(who, s);
			}
			Ok(())
		}

		/// Make a claim to collect your native tokens by signing a statement.
		///
		/// The dispatch origin for this call must be _None_.
		///
		/// Unsigned Validation:
		/// A call to `claim_attest` is deemed valid if the signature provided matches
		/// the expected signed message of:
		///
		/// > Ethereum Signed Message:
		/// > (configured prefix string)(address)(statement)
		///
		/// and `address` matches the `dest` account; the `statement` must match that which is
		/// expected according to your purchase arrangement.
		///
		/// Parameters:
		/// - `dest`: The destination account to payout the claim.
		/// - `ethereum_signature`: The signature of an ethereum signed message matching the format
		///   described above.
		/// - `statement`: The identity of the statement which is being attested to in the
		///   signature.
		///
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// Weight includes logic to validate unsigned `claim_attest` call.
		///
		/// Total Complexity: O(1)
		/// </weight>
		#[pallet::weight(T::WeightInfo::claim_attest())]
		#[pallet::call_index(2)]
		pub fn claim_attest(
			origin: OriginFor<T>,
			dest: Option<MultiAddress>,
			signer: Option<MultiAddress>,
			signature: MultiAddressSignature,
			statement: Vec<u8>,
		) -> DispatchResult {
			ensure_none(origin)?;

			let data = dest.using_encoded(to_ascii_hex);
			let signer =
				Self::get_signer_multi_address(signer.clone(), signature, data, statement.clone())?;
			if let Some(s) = Signing::<T>::get(signer.clone()) {
				ensure!(s.to_text() == &statement[..], Error::<T>::InvalidStatement);
			}
			Self::process_claim(signer, dest)?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::move_claim())]
		#[pallet::call_index(4)]
		pub fn move_claim(
			origin: OriginFor<T>,
			old: MultiAddress,
			new: MultiAddress,
		) -> DispatchResultWithPostInfo {
			T::MoveClaimOrigin::try_origin(origin).map(|_| ()).or_else(ensure_root)?;

			Claims::<T>::take(&old).map(|c| Claims::<T>::insert(&new, c));
			Vesting::<T>::take(&old).map(|c| Vesting::<T>::insert(&new, c));
			Signing::<T>::take(&old).map(|c| Signing::<T>::insert(&new, c));
			Ok(Pays::No.into())
		}

		/// Set the value for expiryconfig
		/// Can only be called by ForceOrigin
		#[pallet::weight(T::WeightInfo::force_set_expiry_config())]
		#[pallet::call_index(5)]
		pub fn force_set_expiry_config(
			origin: OriginFor<T>,
			expiry_block: BlockNumberFor<T>,
			dest: MultiAddress,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			ExpiryConfig::<T>::set(Some((expiry_block, dest)));
			Ok(())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const PRIORITY: u64 = 100;

			let (maybe_signer, maybe_statement) = match call {
				// <weight>
				// The weight of this logic is included in the `claim` dispatchable.
				// </weight>
				Call::claim { dest: account, signer, signature } => {
					let data = account.using_encoded(to_ascii_hex);
					match Self::get_signer_multi_address(
						signer.clone(),
						signature.clone(),
						data,
						vec![],
					) {
						Ok(signer) => (Some(signer), None),
						Err(_) => (None, None),
					}
				},
				// <weight>
				// The weight of this logic is included in the `claim_attest` dispatchable.
				// </weight>
				Call::claim_attest { dest: account, signer, signature, statement } => {
					let data = account.using_encoded(to_ascii_hex);
					match Self::get_signer_multi_address(
						signer.clone(),
						signature.clone(),
						data,
						statement.clone(),
					) {
						Ok(signer) => (Some(signer), Some(statement.as_slice())),
						Err(_) => (None, None),
					}
				},
				_ => return Err(InvalidTransaction::Call.into()),
			};

			let signer = maybe_signer.ok_or(InvalidTransaction::Custom(
				ValidityError::InvalidEthereumSignature.into(),
			))?;

			let e = InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into());
			ensure!(<Claims<T>>::contains_key(&signer), e);

			let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
			match Signing::<T>::get(signer.clone()) {
				None => ensure!(maybe_statement.is_none(), e),
				Some(s) => ensure!(Some(s.to_text()) == maybe_statement, e),
			}

			Ok(ValidTransaction {
				priority: PRIORITY,
				requires: vec![],
				provides: vec![("claims", signer).encode()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		}
	}
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
	let mut r = Vec::with_capacity(data.len() * 2);
	let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
	for &b in data.iter() {
		push_nibble(b / 16);
		push_nibble(b % 16);
	}
	r
}

impl<T: Config> Pallet<T> {
	// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
	fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
		let prefix = T::Prefix::get();
		let mut l = prefix.len() + what.len() + extra.len();
		let mut rev = Vec::new();
		while l > 0 {
			rev.push(b'0' + (l % 10) as u8);
			l /= 10;
		}
		let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
		v.extend(rev.into_iter().rev());
		v.extend_from_slice(&prefix[..]);
		v.extend_from_slice(what);
		v.extend_from_slice(extra);
		v
	}

	// Attempts to recover the Ethereum address from a message signature signed by using
	// the Ethereum RPC's `personal_sign` and `eth_sign`.
	fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<MultiAddress> {
		let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
		let mut res = EthereumAddress::default();
		res.0
			.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
		Some(MultiAddress::EVM(res))
	}

	// Constructs the message that PolkadotJS would sign.
	fn polkadotjs_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
		let prefix = T::Prefix::get();
		let mut v = prefix.to_vec();
		v.extend_from_slice(what);
		v.extend_from_slice(extra);
		v
	}

	// Attempts to recover the Substrate address from a message signature signed by using
	// the Substrate RPC's `sign`.
	fn sr25519_recover(
		addr: MultiAddress,
		s: &Sr25519Signature,
		what: &[u8],
		extra: &[u8],
	) -> Option<MultiAddress> {
		let msg = keccak_256(&Self::polkadotjs_signable_message(what, extra));
		let public: Public = match addr.clone() {
			MultiAddress::EVM(_) => return None,
			MultiAddress::Native(a) => {
				let mut bytes = [0u8; 32];
				bytes.copy_from_slice(&a.encode());
				Public(bytes)
			},
		};
		match sr25519_verify(&s.0, &msg, &public) {
			true => Some(addr),
			false => None,
		}
	}

	fn process_claim(
		signer: MultiAddress,
		dest: Option<MultiAddress>,
	) -> sp_runtime::DispatchResult {
		let balance_due = <Claims<T>>::get(&signer).ok_or(Error::<T>::SignerHasNoClaim)?;

		let new_total = Self::total().checked_sub(&balance_due).ok_or(Error::<T>::PotUnderflow)?;
		// If there is a destination, then we need to transfer the balance to it.
		let recipient = match dest {
			Some(d) => d,
			None => signer.clone(),
		};
		// Convert the destination recipient to an account ID.
		let recipient = Self::convert_multi_address_to_account_id(recipient)?;

		let vesting = Vesting::<T>::get(&signer);
		if vesting.is_some() && T::VestingSchedule::vesting_balance(&recipient).is_some() {
			return Err(Error::<T>::VestedBalanceExists.into())
		}

		// We first need to deposit the balance to ensure that the account exists.
		CurrencyOf::<T>::deposit_creating(&recipient, balance_due);

		// Check if this claim should have a vesting schedule.
		if let Some(vs) = vesting {
			for v in vs.iter() {
				T::VestingSchedule::add_vesting_schedule(&recipient, v.0, v.1, v.2)
					.expect("No other vesting schedule exists, as checked above; qed");
			}
		}

		<Total<T>>::put(new_total);
		<Claims<T>>::remove(&signer);
		<Vesting<T>>::remove(&signer);
		Signing::<T>::remove(&signer);

		// Let's deposit an event to let the outside world know this happened.
		Self::deposit_event(Event::<T>::Claimed { recipient, source: signer, amount: balance_due });

		Ok(())
	}

	fn get_signer_multi_address(
		signer: Option<MultiAddress>,
		signature: MultiAddressSignature,
		data: Vec<u8>,
		statement: Vec<u8>,
	) -> Result<MultiAddress, Error<T>> {
		let signer = match signature {
			MultiAddressSignature::EVM(ethereum_signature) =>
				Self::eth_recover(&ethereum_signature, &data, &statement[..])
					.ok_or(Error::<T>::InvalidEthereumSignature)?,
			MultiAddressSignature::Native(sr25519_signature) => {
				ensure!(!signer.is_none(), Error::<T>::InvalidNativeAccount);
				Self::sr25519_recover(signer.unwrap(), &sr25519_signature, &data, &statement[..])
					.ok_or(Error::<T>::InvalidNativeSignature)?
			},
		};

		Ok(signer)
	}

	/// Convert a MultiAddress to an AccountId
	fn convert_multi_address_to_account_id(dest: MultiAddress) -> Result<T::AccountId, Error<T>> {
		let account = match dest {
			MultiAddress::EVM(a) => T::AddressMapping::into_account_id(H160::from(a)),
			MultiAddress::Native(a) => match Decode::decode(&mut a.encode().as_slice()) {
				Ok(a) => a,
				Err(_) => return Err(Error::<T>::InvalidNativeAccount),
			},
		};

		Ok(account)
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod secp_utils {
	use super::*;

	pub fn public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
		libsecp256k1::PublicKey::from_secret_key(secret)
	}
	pub fn eth(secret: &libsecp256k1::SecretKey) -> MultiAddress {
		let mut res = EthereumAddress::default();
		res.0.copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
		MultiAddress::EVM(res)
	}
	pub fn sig<T: Config>(
		secret: &libsecp256k1::SecretKey,
		what: &[u8],
		extra: &[u8],
	) -> MultiAddressSignature {
		let msg = keccak_256(&<super::Pallet<T>>::ethereum_signable_message(
			&to_ascii_hex(what)[..],
			extra,
		));
		let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		MultiAddressSignature::EVM(EcdsaSignature(r))
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod sr25519_utils {
	use super::*;
	use frame_support::assert_ok;
	use schnorrkel::Signature;
	use sp_core::{sr25519, Pair};

	#[allow(dead_code)]
	pub fn public(pair: &sr25519::Pair) -> sr25519::Public {
		pair.public()
	}

	pub fn sub(pair: &sr25519::Pair) -> MultiAddress {
		MultiAddress::Native(pair.public().into())
	}

	pub fn sig<T: Config>(
		pair: &sr25519::Pair,
		what: &[u8],
		extra: &[u8],
	) -> MultiAddressSignature {
		let msg = keccak_256(&<super::Pallet<T>>::polkadotjs_signable_message(
			&to_ascii_hex(what)[..],
			extra,
		));
		let sig = pair.sign(&msg);
		let pk = schnorrkel::PublicKey::from_bytes(&pair.public().0).unwrap();
		let signature = Signature::from_bytes(&sig.0).unwrap();
		let res = pk.verify_simple(b"substrate", &msg, &signature);
		assert_ok!(res);
		MultiAddressSignature::Native(Sr25519Signature(sig))
	}
}
