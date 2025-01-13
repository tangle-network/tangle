// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import { Definitions } from '@polkadot/types/types'

export default {
  rpc: {
    queryServicesWithBlueprintsByOperator: {
      description:
        'Query all the services that this operator is providing along with their blueprints.',
      type: 'Vec<RpcServicesWithBlueprint>',
      params: [
        {
          name: 'operator',
          type: 'AccountId',
          isHistoric: true,
          isOptional: false,
        },
      ],
    },
  },
  types: {
    RpcServicesWithBlueprint: {
      blueprintId: 'u64',
      blueprint: 'ServiceBlueprint',
      services: 'Vec<Service>',
    },
    //
    ServiceBlueprint: {
      metadata: 'ServiceMetadata',
      jobs: 'Vec<JobDefinition>',
      registrationHook: 'ServiceRegistrationHook',
      registrationParams: 'Vec<FieldFieldType>',
      requestHook: 'ServiceRequestHook',
      requestParams: 'Vec<FieldFieldType>',
      gadget: 'Gadget',
    },
    ServiceMetadata: {
      name: 'Bytes',
      description: 'Option<Bytes>',
      author: 'Option<Bytes>',
      category: 'Option<Bytes>',
      codeRepository: 'Option<Bytes>',
      logo: 'Option<Bytes>',
      website: 'Option<Bytes>',
      license: 'Option<Bytes>',
    },
    JobDefinition: {
      metadata: 'JobMetadata',
      params: 'Vec<FieldFieldType>',
      result: 'Vec<FieldFieldType>',
      verifier: 'JobResultVerifier',
    },
    JobMetadata: {
      name: 'Bytes',
      description: 'Option<Bytes>',
    },
    FieldFieldType: {
      _enum: {
        Void: null,
        Bool: null,
        Uint8: null,
        Int8: null,
        Uint16: null,
        Int16: null,
        Uint32: null,
        Int32: null,
        Uint64: null,
        Int64: null,
        String: null,
        Bytes: null,
        Optional: 'FieldFieldType',
        Array: '(u64,FieldFieldType)',
        List: 'FieldFieldType',
        Struct: '(FieldFieldType,Vec<(FieldFieldType,FieldFieldType)>)',
        AccountId: null,
      },
    },
    JobResultVerifier: {
      _enum: {
        None: 'Null',
        Evm: 'H160',
      },
    },
    ServiceRegistrationHook: {
      _enum: {
        None: 'Null',
        Evm: 'H160',
      },
    },
    ServiceRequestHook: {
      _enum: {
        None: 'Null',
        Evm: 'H160',
      },
    },
    Gadget: {
      _enum: {
        Wasm: 'WasmGadget',
        Native: 'NativeGadget',
        Container: 'ContainerGadget',
      },
    },
    ContainerGadget: {
      sources: 'Vec<GadgetSource>',
    },
    NativeGadget: {
      sources: 'Vec<GadgetSource>',
    },
    WasmGadget: {
      runtime: 'WasmRuntime',
      sources: 'Vec<GadgetSource>',
    },
    WasmRuntime: {
      _enum: ['Wasmtime', 'Wasmer'],
    },
    GadgetSource: {
      fetcher: 'GadgetSourceFetcher',
    },
    GadgetSourceFetcher: {
      _enum: {
        IPFS: 'Bytes',
        Github: 'GithubFetcher',
        ContainerImage: 'ImageRegistryFetcher',
        Testing: 'TestFetcher',
      },
    },
    GithubFetcher: {
      owner: 'Bytes',
      repo: 'Bytes',
      tag: 'Bytes',
      binaries: 'Vec<GadgetBinary>',
    },
    GadgetBinary: {
      arch: 'Architecture',
      os: 'OperatingSystem',
      name: 'Bytes',
      sha256: '[u8;32]',
    },
    Architecture: {
      _enum: [
        'Wasm',
        'Wasm64',
        'Wasi',
        'Wasi64',
        'Amd',
        'Amd64',
        'Arm',
        'Arm64',
        'RiscV',
        'RiscV64',
      ],
    },
    OperatingSystem: {
      _enum: ['Unknown', 'Linux', 'Windows', 'MacOS', 'BSD'],
    },
    ImageRegistryFetcher: {
      _alias: {
        registry_: 'registry',
      },
      registry_: 'Bytes',
      image: 'Bytes',
      tag: 'Bytes',
    },
    TestFetcher: {
      cargoPackage: 'Bytes',
      cargoBin: 'Bytes',
      basePath: 'Bytes',
    },
    Service: {
      id: 'u64',
      blueprint: 'u64',
      owner: 'AccountId32',
      permittedCallers: 'Vec<AccountId32>',
      operators: 'Vec<AccountId32>',
      ttl: 'u64',
    },
  },
} satisfies Definitions
