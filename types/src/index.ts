// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import './interfaces/augment-api'
import './interfaces/augment-types'
import './interfaces/types-lookup'

import type {
  OverrideBundleDefinition,
  OverrideBundleType,
} from '@polkadot/types/types'

import * as tangleDefs from './interfaces/definitions'
import { jsonrpcFromDefs, typesAliasFromDefs, typesFromDefs } from './utils'

export * as tangleLookupTypes from './interfaces/lookup'

export * from './interfaces'

export const tangleTypes = typesFromDefs(tangleDefs)
export const tangleRpc = jsonrpcFromDefs(tangleDefs, {})
export const tangleTypesAlias = typesAliasFromDefs(tangleDefs, {})

const sharedBundle: OverrideBundleDefinition = {
  rpc: tangleRpc,
  types: [
    {
      minmax: [],
      types: tangleTypes,
    },
  ],
  alias: tangleTypesAlias,
}

export const tangleTypesBundle: OverrideBundleType = {
  chain: {
    tangle: sharedBundle,
  },
  spec: {
    tangle: sharedBundle,
  },
}

export const typesBundleForPolkadot = tangleTypesBundle
