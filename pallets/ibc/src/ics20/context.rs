use super::super::*;
use crate::routing::Context;
use frame_support::traits::{
	fungibles::Mutate,
	tokens::{Fortitude, Precision},
	Currency, Get,
};
use ibc::{
	applications::transfer::{
		context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as Ics20Error,
		PORT_ID_STR,
	},
	core::ics24_host::identifier::{ChannelId, PortId},
};
use ibc_primitives::get_channel_escrow_address;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::IdentifyAccount;

impl<T: Config + Send + Sync> Ics20Reader for Context<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	type AccountId = T::AccountIdConversion;

	fn get_port(&self) -> Result<PortId, Ics20Error> {
		PortId::from_str(PORT_ID_STR)
			.map_err(|e| Ics20Error::invalid_port_id(PORT_ID_STR.to_string(), e))
	}

	fn get_channel_escrow_address(
		&self,
		port_id: &PortId,
		channel_id: ChannelId,
	) -> Result<<Self as Ics20Reader>::AccountId, Ics20Error> {
		get_channel_escrow_address(port_id, channel_id)?.try_into().map_err(|_| {
			log::trace!(target: "pallet_ibc", "Failed to get channel escrow address");
			Ics20Error::parse_account_failure()
		})
	}

	fn is_send_enabled(&self) -> bool {
		T::IsSendEnabled::get()
	}

	fn is_receive_enabled(&self) -> bool {
		T::IsReceiveEnabled::get()
	}
}

impl<T: Config + Send + Sync> Ics20Keeper for Context<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	type AccountId = T::AccountIdConversion;
}

impl<T: Config + Send + Sync> Ics20Context for Context<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	type AccountId = T::AccountIdConversion;
}

impl<T> BankKeeper for Context<T>
where
	T: Config + Send + Sync,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	type AccountId = T::AccountIdConversion;

	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &ibc::applications::transfer::PrefixedCoin,
	) -> Result<(), Ics20Error> {
		let amount: T::Balance = amt.amount.as_u256().as_u128().into();
		let denom = amt.denom.to_string();
		// Token should be registered already if sending an ibc asset
		let asset_id = T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(&denom)
			.map_err(|_| Ics20Error::invalid_token())?;

		if asset_id == T::NativeAssetId::get() {
			<T::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::transfer(
				&from.clone().into_account(),
				&to.clone().into_account(),
				amount,
				frame_support::traits::ExistenceRequirement::AllowDeath,
			)
			.map_err(|e| {
				log::debug!(target: "pallet_ibc", "Failed to transfer ibc asset: {asset_id:?}, denom: {denom}, error: {e:?}");
				Ics20Error::invalid_token()
			})?;
		} else {
			<<T as Config>::Fungibles as Mutate<<T as frame_system::Config>::AccountId>>::transfer(
				asset_id.clone(),
				&from.clone().into_account(),
				&to.clone().into_account(),
				amount,
				frame_support::traits::tokens::Preservation::Expendable,
			)
			.map_err(|e| {
				log::debug!(target: "pallet_ibc", "Failed to transfer ibc asset: {asset_id:?}, denom: {denom}, error: {e:?}");
				Ics20Error::token_balance_change()
			})?;
		}

		Ok(())
	}

	fn mint_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &ibc::applications::transfer::PrefixedCoin,
	) -> Result<(), Ics20Error> {
		let amount: T::Balance = amt.amount.as_u256().as_u128().into();
		let denom = amt.denom.to_string();
		// Find existing asset or create a new one
		let asset_id = T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(&denom)
			.map_err(|_err| Ics20Error::invalid_token())?;

		<<T as Config>::Fungibles as Mutate<<T as frame_system::Config>::AccountId>>::mint_into(
			asset_id,
			&account.clone().into_account(),
			amount,
		)
		.map_err(|e| {
			log::debug!(target: "pallet_ibc", "Failed to mint tokens: {:?}", e);
			Ics20Error::token_balance_change()
		})?;
		Ok(())
	}

	fn burn_coins(
		&mut self,
		account: &Self::AccountId,
		amt: &ibc::applications::transfer::PrefixedCoin,
	) -> Result<(), Ics20Error> {
		let amount: T::Balance = amt.amount.as_u256().as_u128().into();
		let denom = amt.denom.to_string();
		// Token should be registered already if burning a voucher
		let asset_id = T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(&denom)
			.map_err(|_| Ics20Error::invalid_token())?;
		<<T as Config>::Fungibles as Mutate<<T as frame_system::Config>::AccountId>>::burn_from(
			asset_id,
			&account.clone().into_account(),
			amount,
			Precision::Exact,
			Fortitude::Force,
		)
		.map_err(|e| {
			log::debug!(target: "pallet_ibc", "Failed to burn tokens: {:?}", e);
			Ics20Error::implementation_specific("Failed to burn tokens".to_string())
		})?;
		Ok(())
	}
}
