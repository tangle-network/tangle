// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import { Definitions } from "@polkadot/types/types";

export default {
	rpc: {
		queryUserRewards: {
			description:
				"Query all the rewards of a specific asset for a specific user",
			type: "Result<Balance, ErrorObjectOwned>",
			params: [
				{
					name: "accountId",
					type: "AccountId",
					isHistoric: false,
					isOptional: false,
				},
				{
					name: "assetId",
					type: "RewardsAssetId",
					isHistoric: false,
					isOptional: false,
				},
			],
		},
	},
	types: {
		RewardsAssetId: {
			_enum: {
				Custom: "u128",
				Erc20: "H160",
			},
		},
		ErrorObjectOwned: {
			code: "ErrorCode",
			message: "Text",
			data: "Option<RawValue>",
		},
		ErrorCode: {
			_enum: {
				ParseError: null,
				OversizedRequest: null,
				InvalidRequest: null,
				MethodNotFound: null,
				ServerIsBusy: null,
				InvalidParams: null,
				InternalError: null,
				ServerError: "i32",
			},
		},
		RawValue: {
			json: "Text",
		},
	},
	runtime: {
		RewardsApi: [
			{
				version: 1,
				methods: {
					queryUserRewards: {
						description:
							"Query all the rewards of a specific asset for a specific user",
						params: [
							{
								name: "accountId",
								type: "AccountId",
							},
							{
								name: "assetId",
								type: "AssetId",
							},
						],
						type: "Result<Balance, SpRuntimeDispatchError>",
					},
				},
			},
		],
	},
} as const satisfies Definitions;
