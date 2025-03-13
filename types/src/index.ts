// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import "./interfaces/augment-api";
import "./interfaces/augment-types";
import "./interfaces/types-lookup";

import type {
	OverrideBundleDefinition,
	OverrideBundleType,
} from "@polkadot/types/types";

import * as tangleDefs from "./interfaces/definitions";
import { jsonrpcFromDefs, typesAliasFromDefs, typesFromDefs } from "./utils";

export * as tangleLookupTypes from "./interfaces/lookup";

export * from "./interfaces";

export const tangleTypes = typesFromDefs(tangleDefs);
export const tangleRpc = jsonrpcFromDefs(tangleDefs, {});

const sharedBundle = {
	rpc: tangleRpc,
	types: [
		{
			minmax: [],
			types: tangleTypes,
		},
	],
} as const satisfies OverrideBundleDefinition;

export const tangleTypesBundle = {
	chain: {
		tangle: sharedBundle,
	},
	spec: {
		tangle: sharedBundle,
	},
} as const satisfies OverrideBundleType;

export const typesBundleForPolkadot = tangleTypesBundle;
