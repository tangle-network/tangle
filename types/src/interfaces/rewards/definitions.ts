// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import { Definitions } from "@polkadot/types/types";

export default {
	rpc: {
		queryUserRewards: {
			description: "Query the reward for a user for a specific asset",
			type: "RpcResult<Balance>",
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
	},
	runtime: {
		RewardsApi: [
			{
				version: 2,
				methods: {
					queryUserRewards: {
						description:
							"Query the reward for a user for a specific asset",
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
						type: "RpcResult<Balance>",
					},
				},
			},
		],
	},
} as const satisfies Definitions;
