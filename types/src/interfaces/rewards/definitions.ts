// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import { Definitions } from "@polkadot/types/types";

export default {
	rpc: {
		queryUserRewards: {
			description: "Query the reward for a user for a specific asset",
			type: "Result<Balance, ErrorObjectOwned>",
			params: [
				{
					name: "operator",
					type: "AccountId",
					isHistoric: false,
					isOptional: false,
				},
				{
					name: "asset",
					type: "ServicesTypesAsset",
					isHistoric: false,
					isOptional: false,
				},
			],
		},
	},
	types: {
		ServicesTypesAsset: {
			_enum: {
				Custom: "u128",
				Erc20: "H160",
			},
		},
		ErrorObjectOwned: {
			code: "ErrorCode",
			message: "String",
			data: "Option<RawValue>",
		},
		ErrorCode: {
			_enum: {
				ParseError: "Null",
				OversizedRequest: "Null",
				InvalidRequest: "Null",
				MethodNotFound: "Null",
				ServerIsBusy: "Null",
				InvalidParams: "Null",
				InternalError: "Null",
				ServerError: "u32",
			},
		},
		RawValue: {
			json: "String",
		},
	},
	runtime: {
		rewardsApi: [
			{
				version: 2,
				methods: {
					queryUserRewards: {
						description:
							"Query the reward for a user for a specific asset",
						type: "Result<u128, SpRuntimeDispatchError>",
						params: [
							{
								name: "operator",
								type: "AccountId",
							},
							{
								name: "asset",
								type: "ServicesTypesAsset",
							},
						],
					},
				},
			},
		],
	},
} as const satisfies Definitions;
