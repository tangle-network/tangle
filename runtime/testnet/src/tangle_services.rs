use super::*;

parameter_types! {
	pub const ServicesPalletId: PalletId = PalletId(*b"py/srvcs");
}

pub struct PalletEvmRunner;

impl pallet_services::EvmRunner<Runtime> for PalletEvmRunner {
	type Error = pallet_evm::Error<Runtime>;

	fn call(
		source: sp_core::H160,
		target: sp_core::H160,
		input: Vec<u8>,
		value: sp_core::U256,
		gas_limit: u64,
		is_transactional: bool,
		validate: bool,
	) -> Result<fp_evm::CallInfo, pallet_services::RunnerError<Self::Error>> {
		let max_fee_per_gas = DefaultBaseFeePerGas::get();
		let max_priority_fee_per_gas =
			max_fee_per_gas.saturating_mul(U256::from(3) / U256::from(2));
		let nonce = None;
		let access_list = Default::default();
		let weight_limit = None;
		let proof_size_base_cost = None;
		<<Runtime as pallet_evm::Config>::Runner as pallet_evm::Runner<Runtime>>::call(
			source,
			target,
			input,
			value,
			gas_limit,
			Some(max_fee_per_gas),
			Some(max_priority_fee_per_gas),
			nonce,
			access_list,
			is_transactional,
			validate,
			weight_limit,
			proof_size_base_cost,
			<Runtime as pallet_evm::Config>::config(),
		)
		.map_err(|o| pallet_services::RunnerError { error: o.error, weight: o.weight })
	}
}

pub struct PalletEVMGasWeightMapping;

impl pallet_services::EvmGasWeightMapping for PalletEVMGasWeightMapping {
	fn gas_to_weight(gas: u64, without_base_weight: bool) -> Weight {
		pallet_evm::FixedGasWeightMapping::<Runtime>::gas_to_weight(gas, without_base_weight)
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		pallet_evm::FixedGasWeightMapping::<Runtime>::weight_to_gas(weight)
	}
}

parameter_types! {
	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxFields: u32 = 256;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxFieldsSize: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxMetadataLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxJobsPerService: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxOperatorsPerService: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxPermittedCallers: u32 = 256;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxServicesPerOperator: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxBlueprintsPerOperator: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxServicesPerUser: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxBinariesPerGadget: u32 = 64;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxSourcesPerGadget: u32 = 64;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxGitOwnerLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxGitRepoLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxGitTagLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxBinaryNameLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxIpfsHashLength: u32 = 46;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxContainerRegistryLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxContainerImageNameLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxContainerImageTagLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo, Serialize, Deserialize)]
	pub const MaxAssetsPerService: u32 = 64;
}

pub type PalletServicesConstraints = pallet_services::types::ConstraintsOf<Runtime>;

impl pallet_services::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type Currency = Balances;
	type PalletId = ServicesPalletId;
	type EvmRunner = PalletEvmRunner;
	type EvmGasWeightMapping = PalletEVMGasWeightMapping;
	type AssetId = AssetId;
	type MaxFields = MaxFields;
	type MaxFieldsSize = MaxFieldsSize;
	type MaxMetadataLength = MaxMetadataLength;
	type MaxJobsPerService = MaxJobsPerService;
	type MaxOperatorsPerService = MaxOperatorsPerService;
	type MaxPermittedCallers = MaxPermittedCallers;
	type MaxServicesPerOperator = MaxServicesPerOperator;
	type MaxBlueprintsPerOperator = MaxBlueprintsPerOperator;
	type MaxServicesPerUser = MaxServicesPerUser;
	type MaxBinariesPerGadget = MaxBinariesPerGadget;
	type MaxSourcesPerGadget = MaxSourcesPerGadget;
	type MaxGitOwnerLength = MaxGitOwnerLength;
	type MaxGitRepoLength = MaxGitRepoLength;
	type MaxGitTagLength = MaxGitTagLength;
	type MaxBinaryNameLength = MaxBinaryNameLength;
	type MaxIpfsHashLength = MaxIpfsHashLength;
	type MaxContainerRegistryLength = MaxContainerRegistryLength;
	type MaxContainerImageNameLength = MaxContainerImageNameLength;
	type MaxContainerImageTagLength = MaxContainerImageTagLength;
	type MaxAssetsPerService = MaxAssetsPerService;
	type Constraints = PalletServicesConstraints;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type OperatorDelegationManager = MultiAssetDelegation;
	#[cfg(feature = "runtime-benchmarks")]
	type OperatorDelegationManager =
		pallet_services::BenchmarkingOperatorDelegationManager<Runtime, Balance, AssetId>;
	type WeightInfo = ();
}
