// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import { Definitions } from "@polkadot/types/types";

export default {
	rpc: {
		queryServicesWithBlueprintsByOperator: {
			description:
				"Query all the services that this operator is providing along with their blueprints.",
			type: "Vec<RpcServicesWithBlueprint>",
			params: [
				{
					name: "operator",
					type: "AccountId",
					isHistoric: false,
					isOptional: false,
				},
			],
		},
	},
	types: {
		RpcServicesWithBlueprint: {
			blueprintId: "u64",
			blueprint: "ServiceBlueprint",
			services: "Vec<Service>",
		},
		//
		ServiceBlueprint: {
			metadata: "ServiceMetadata",
			jobs: "Vec<JobDefinition>",
			registrationParams: "Vec<FieldFieldType>",
			requestParams: "Vec<FieldFieldType>",
			manager: "ServiceBlueprintServiceManager",
			masterManagerRevision: "MasterBlueprintServiceManagerRevision",
			gadget: "Gadget",
			supportedMembershipModels: "Vec<MembershipModelType>",
		},
		ServiceMetadata: {
			name: "Bytes",
			description: "Option<Bytes>",
			author: "Option<Bytes>",
			category: "Option<Bytes>",
			codeRepository: "Option<Bytes>",
			logo: "Option<Bytes>",
			website: "Option<Bytes>",
			license: "Option<Bytes>",
		},
		JobDefinition: {
			metadata: "JobMetadata",
			params: "Vec<FieldFieldType>",
			result: "Vec<FieldFieldType>",
		},
		JobMetadata: {
			name: "Bytes",
			description: "Option<Bytes>",
		},
		FieldFieldType: {
			_enum: {
				Void: "Null",
				Bool: "Null",
				Uint8: "Null",
				Int8: "Null",
				Uint16: "Null",
				Int16: "Null",
				Uint32: "Null",
				Int32: "Null",
				Uint64: "Null",
				Int64: "Null",
				String: "Null",
				Optional: "FieldFieldType",
				Array: "(u64,FieldFieldType)",
				List: "FieldFieldType",
				Struct: "Vec<FieldFieldType>",
				AccountId: "Null",
			},
		},
		ServiceRegistrationHook: {
			_enum: {
				None: "Null",
				Evm: "H160",
			},
		},
		ServiceRequestHook: {
			_enum: {
				None: "Null",
				Evm: "H160",
			},
		},
		Gadget: {
			_enum: {
				Wasm: "WasmGadget",
				Native: "NativeGadget",
				Container: "ContainerGadget",
			},
		},
		ContainerGadget: {
			sources: "Vec<GadgetSource>",
		},
		NativeGadget: {
			sources: "Vec<GadgetSource>",
		},
		WasmGadget: {
			runtime: "WasmRuntime",
			sources: "Vec<GadgetSource>",
		},
		WasmRuntime: {
			_enum: ["Wasmtime", "Wasmer"],
		},
		GadgetSource: {
			fetcher: "GadgetSourceFetcher",
		},
		GadgetSourceFetcher: {
			_enum: {
				IPFS: "Bytes",
				Github: "GithubFetcher",
				ContainerImage: "ImageRegistryFetcher",
				Testing: "TestFetcher",
			},
		},
		GithubFetcher: {
			owner: "Bytes",
			repo: "Bytes",
			tag: "Bytes",
			binaries: "Vec<GadgetBinary>",
		},
		GadgetBinary: {
			arch: "Architecture",
			os: "OperatingSystem",
			name: "Bytes",
			sha256: "[u8;32]",
		},
		Architecture: {
			_enum: [
				"Wasm",
				"Wasm64",
				"Wasi",
				"Wasi64",
				"Amd",
				"Amd64",
				"Arm",
				"Arm64",
				"RiscV",
				"RiscV64",
			],
		},
		OperatingSystem: {
			_enum: ["Unknown", "Linux", "Windows", "MacOS", "BSD"],
		},
		ImageRegistryFetcher: {
			registry: "Bytes",
			image: "Bytes",
			tag: "Bytes",
		},
		TestFetcher: {
			cargoPackage: "Bytes",
			cargoBin: "Bytes",
			basePath: "Bytes",
		},
		Service: {
			id: "u64",
			blueprint: "u64",
			owner: "AccountId32",
			operatorSecurityCommitments:
				"Vec<(AccountId32,Vec<AssetSecurityCommitment>)>",
			securityRequirements: "Vec<AssetSecurityRequirement>",
			permittedCallers: "Vec<AccountId32>",
			ttl: "u64",
			membershipModel: "MembershipModel",
		},
		ServiceBlueprintServiceManager: {
			_enum: {
				Evm: "H160",
			},
		},
		MasterBlueprintServiceManagerRevision: {
			_enum: {
				Latest: "Null",
				Specific: "u32",
			},
		},
		MembershipModelType: {
			_enum: ["Fixed", "Dynamic"],
		},
		AssetSecurityCommitment: {
			asset: "Asset",
			exposurePercent: "Percent",
		},
		Asset: {
			_enum: {
				Custom: "u128",
				Erc20: "H160",
			},
		},
		AssetSecurityRequirement: {
			asset: "Asset",
			minExposurePercent: "Percent",
			maxExposurePercent: "Percent",
		},
		MembershipModel: {
			_enum: {
				Fixed: "MembershipModelFixed",
				Dynamic: "MembershipModelDynamic",
			},
		},
		MembershipModelFixed: {
			minOperators: "u32",
		},
		MembershipModelDynamic: {
			minOperators: "u32",
			maxOperators: "Option<u32>",
		},
	},
	runtime: {
		ServiceApi: [
			{
				version: 2,
				methods: {
					queryServicesWithBlueprintsByOperator: {
						description:
							"Query all the services that this operator is providing along with their blueprints.",
						params: [
							{
								name: "operator",
								type: "AccountId",
							},
						],
						type: "Result<Vec<TanglePrimitivesServicesServiceRpcServicesWithBlueprint>, SpRuntimeDispatchError>",
					},
				},
			},
		],
	},
} as const satisfies Definitions;
