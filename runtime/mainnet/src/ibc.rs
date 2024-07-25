use ::ibc::core::{
	ics24_host::identifier::PortId,
	ics26_routing::context::{Module, ModuleId},
};
use core::{
	convert::Infallible,
	fmt::{Display, Formatter},
	str::FromStr,
};
use cumulus_primitives_core::ParaId;
use frame_support::traits::fungibles::metadata::Inspect;
use ibc_primitives::{runtime_interface::ss58_to_account_id_32, IbcAccount};
use light_client_common::StandaloneChain;
use pallet_ibc::{
	ics20::{MemoData, SubstrateMultihopXcmHandlerNone, ValidateMemo},
	ics20_fee::NonFlatFeeConverter,
	light_client_common::ChainType,
	routing::ModuleRouter,
	DenomToAssetId, IbcAssetIds, IbcAssets, IbcDenoms, LightClientProtocol,
};
use sp_core::keccak_256;
use sp_runtime::{
	traits::AccountIdConversion,
	DispatchError, Either,
	Either::{Left, Right},
};

use super::*;

impl pallet_ibc_ping::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IbcHandler = Ibc;
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Router {
	pallet_ibc_ping: pallet_ibc_ping::IbcModule<Runtime>,
}

impl ModuleRouter for Router {
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		match module_id.as_ref() {
			pallet_ibc_ping::MODULE_ID => Some(&mut self.pallet_ibc_ping),
			_ => None,
		}
	}

	fn has_route(module_id: &ModuleId) -> bool {
		matches!(module_id.as_ref(), pallet_ibc_ping::MODULE_ID)
	}

	fn lookup_module_by_port(port_id: &PortId) -> Option<ModuleId> {
		match port_id.as_str() {
			pallet_ibc_ping::PORT_ID => ModuleId::from_str(pallet_ibc_ping::MODULE_ID).ok(),
			_ => None,
		}
	}
}

pub struct IbcDenomToAssetIdConversion;

impl DenomToAssetId<Runtime> for IbcDenomToAssetIdConversion {
	type Error = DispatchError;

	fn from_denom_to_asset_id(denom: &str) -> Result<AssetId, Self::Error> {
		use frame_support::traits::fungibles::{metadata::Mutate, Create};

		let denom_bytes = denom.as_bytes().to_vec();
		if let Some(id) = IbcDenoms::<Runtime>::get(&denom_bytes) {
			return Ok(id);
		}

		let pallet_id: AccountId = PalletId(*b"pall-ibc").into_account_truncating();

		let symbol = denom
			.split('/')
			.last()
			.ok_or(DispatchError::Other("denom missing a name"))?
			.as_bytes()
			.to_vec();
		let asset_id_256 = U256::from_big_endian(&keccak_256(&symbol)[..]);
		let asset_id: u128 = asset_id_256.as_u128();

		IbcDenoms::<Runtime>::insert(denom_bytes.clone(), asset_id);
		IbcAssetIds::<Runtime>::insert(asset_id, denom_bytes.clone());

		<pallet_assets::Pallet<Runtime> as Create<AccountId>>::create(
			asset_id,
			pallet_id.clone(),
			true,
			1,
		)?;

		<pallet_assets::Pallet<Runtime> as Mutate<AccountId>>::set(
			asset_id,
			&pallet_id,
			denom_bytes,
			symbol,
			18,
		)?;

		Ok(asset_id)
	}

	fn from_asset_id_to_denom(id: AssetId) -> Option<String> {
		let name = <pallet_assets::Pallet<Runtime> as Inspect<AccountId>>::name(id);
		String::from_utf8(name).ok()
	}

	fn ibc_assets(start_key: Option<Either<AssetId, u32>>, limit: u64) -> IbcAssets<AssetId> {
		let mut iterator = match start_key {
			None => IbcAssetIds::<Runtime>::iter().skip(1),
			Some(Left(asset_id)) => {
				let raw_key = asset_id.encode();
				IbcAssetIds::<Runtime>::iter_from(raw_key).skip(1)
			},
			Some(Right(offset)) => IbcAssetIds::<Runtime>::iter().skip(offset as usize),
		};

		let denoms = iterator.by_ref().take(limit as usize).map(|(_, denom)| denom).collect();
		let maybe_currency_id = iterator.next().map(|(id, ..)| id);
		IbcAssets {
			denoms,
			total_count: IbcAssetIds::<Runtime>::count() as u64,
			next_id: maybe_currency_id,
		}
	}
}

#[derive(
	Debug,
	parity_scale_codec::Encode,
	Clone,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	scale_info::TypeInfo,
	Default,
)]
pub struct MemoMessage;

impl ToString for MemoMessage {
	fn to_string(&self) -> String {
		Default::default()
	}
}

impl core::str::FromStr for MemoMessage {
	type Err = ();

	fn from_str(_s: &str) -> Result<Self, Self::Err> {
		Ok(Default::default())
	}
}

parameter_types! {
	pub const GRANDPA: LightClientProtocol = LightClientProtocol::GrandpaStandalone;
	pub const IbcTriePrefix : &'static [u8] = b"ibc/";
	pub FeeAccount: <Runtime as pallet_ibc::Config>::AccountIdConversion = create_alice_key();
	pub const CleanUpPacketsPeriod: BlockNumber = 100;
	pub AssetIdUSDT: AssetId = 0;
	pub FlatFeeUSDTAmount: Balance = 0;
	pub IbcIcs20ServiceCharge: Perbill = Perbill::from_rational(0_u32, 1000_u32 );
}

fn create_alice_key() -> <Runtime as pallet_ibc::Config>::AccountIdConversion {
	let alice = "5yNZjX24n2eg7W6EVamaTXNQbWCwchhThEaSWB7V3GRjtHeL";
	let account_id_32 = ss58_to_account_id_32(alice).unwrap().into();
	IbcAccount(account_id_32)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MemoMiddlewareNamespaceChain {
	Forward { next: Option<Box<Self>> },
	Wasm { next: Option<Box<Self>> },
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Encode, Decode, TypeInfo)]
pub struct RawMemo(pub String);

impl FromStr for RawMemo {
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.to_string()))
	}
}

impl TryFrom<MemoData> for RawMemo {
	type Error = <String as TryFrom<MemoData>>::Error;

	fn try_from(value: MemoData) -> Result<Self, Self::Error> {
		Ok(Self(value.try_into()?))
	}
}

impl TryFrom<RawMemo> for MemoData {
	type Error = <MemoData as TryFrom<String>>::Error;

	fn try_from(value: RawMemo) -> Result<Self, Self::Error> {
		value.0.try_into()
	}
}

impl Display for RawMemo {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl ValidateMemo for RawMemo {
	fn validate(&self) -> Result<(), String> {
		// the MiddlewareNamespaceChain type contains all the supported middlewares
		serde_json::from_str::<MemoMiddlewareNamespaceChain>(&self.0)
			.map(|_| ())
			.map_err(|e| e.to_string())
	}
}

parameter_types! {
	pub const NativeAssetId: AssetId = 0;
	pub const ChainIdentifier: ParaId = ParaId::new(1337);
	pub const ExpectedBlockTime: u64 = 6000;
	pub const SpamProtectionDeposit: Balance = 10000;
	pub const MinimumConnectionDelay: u64 = 300; // 5 minutes
	pub const TangleChainType: ChainType = ChainType::StandaloneChain(StandaloneChain::Tangle);
}

impl pallet_ibc::Config for Runtime {
	type TimeProvider = Timestamp;
	type RuntimeEvent = RuntimeEvent;
	type NativeCurrency = Balances;
	type Balance = Balance;
	type AssetId = AssetId;
	type NativeAssetId = NativeAssetId;
	type IbcDenomToAssetIdConversion = IbcDenomToAssetIdConversion;
	type AccountIdConversion = ibc_primitives::IbcAccount<AccountId>;
	type Fungibles = Assets;
	type ExpectedBlockTime = ExpectedBlockTime;
	type Router = Router;
	type MinimumConnectionDelay = MinimumConnectionDelay;
	type WeightInfo = ();
	type AdminOrigin = EnsureRoot<AccountId>;
	type FreezeOrigin = EnsureRoot<AccountId>;
	type SpamProtectionDeposit = SpamProtectionDeposit;
	type TransferOrigin = EnsureSigned<Self::IbcAccountId>;
	type RelayerOrigin = EnsureSigned<Self::AccountId>;
	type MemoMessage = RawMemo;
	type IsReceiveEnabled = sp_core::ConstBool<true>;
	type IsSendEnabled = sp_core::ConstBool<true>;
	type HandleMemo = ();
	type PalletPrefix = IbcTriePrefix;
	type LightClientProtocol = GRANDPA;
	type IbcAccountId = Self::AccountId;
	type FeeAccount = FeeAccount;
	type CleanUpPacketsPeriod = CleanUpPacketsPeriod;
	type ServiceChargeOut = IbcIcs20ServiceCharge;
	type FlatFeeConverter = NonFlatFeeConverter<Runtime>;
	type FlatFeeAssetId = AssetIdUSDT;
	type FlatFeeAmount = FlatFeeUSDTAmount;
	type SubstrateMultihopXcmHandler = SubstrateMultihopXcmHandlerNone<Runtime>;

	type ChainId = ChainIdentifier;
	type ChainType = TangleChainType;
}