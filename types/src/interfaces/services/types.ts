// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Enum, Option, Struct, U8aFixed, Vec, u128, u32, u64 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, H160, Percent } from '@polkadot/types/interfaces/runtime';

/** @name Architecture */
export interface Architecture extends Enum {
  readonly isWasm: boolean;
  readonly isWasm64: boolean;
  readonly isWasi: boolean;
  readonly isWasi64: boolean;
  readonly isAmd: boolean;
  readonly isAmd64: boolean;
  readonly isArm: boolean;
  readonly isArm64: boolean;
  readonly isRiscV: boolean;
  readonly isRiscV64: boolean;
  readonly type: 'Wasm' | 'Wasm64' | 'Wasi' | 'Wasi64' | 'Amd' | 'Amd64' | 'Arm' | 'Arm64' | 'RiscV' | 'RiscV64';
}

/** @name Asset */
export interface Asset extends Enum {
  readonly isCustom: boolean;
  readonly asCustom: u128;
  readonly isErc20: boolean;
  readonly asErc20: H160;
  readonly type: 'Custom' | 'Erc20';
}

/** @name AssetSecurityCommitment */
export interface AssetSecurityCommitment extends Struct {
  readonly asset: Asset;
  readonly exposurePercent: Percent;
}

/** @name AssetSecurityRequirement */
export interface AssetSecurityRequirement extends Struct {
  readonly asset: Asset;
  readonly minExposurePercent: Percent;
  readonly maxExposurePercent: Percent;
}

/** @name ContainerGadget */
export interface ContainerGadget extends Struct {
  readonly sources: Vec<GadgetSource>;
}

/** @name FieldFieldType */
export interface FieldFieldType extends Enum {
  readonly isVoid: boolean;
  readonly isBool: boolean;
  readonly isUint8: boolean;
  readonly isInt8: boolean;
  readonly isUint16: boolean;
  readonly isInt16: boolean;
  readonly isUint32: boolean;
  readonly isInt32: boolean;
  readonly isUint64: boolean;
  readonly isInt64: boolean;
  readonly isText: boolean;
  readonly isOptional: boolean;
  readonly asOptional: FieldFieldType;
  readonly isArray: boolean;
  readonly asArray: ITuple<[u64, FieldFieldType]>;
  readonly isList: boolean;
  readonly asList: FieldFieldType;
  readonly isStruct: boolean;
  readonly asStruct: Vec<FieldFieldType>;
  readonly isAccountId: boolean;
  readonly type: 'Void' | 'Bool' | 'Uint8' | 'Int8' | 'Uint16' | 'Int16' | 'Uint32' | 'Int32' | 'Uint64' | 'Int64' | 'Text' | 'Optional' | 'Array' | 'List' | 'Struct' | 'AccountId';
}

/** @name Gadget */
export interface Gadget extends Enum {
  readonly isWasm: boolean;
  readonly asWasm: WasmGadget;
  readonly isNative: boolean;
  readonly asNative: NativeGadget;
  readonly isContainer: boolean;
  readonly asContainer: ContainerGadget;
  readonly type: 'Wasm' | 'Native' | 'Container';
}

/** @name GadgetBinary */
export interface GadgetBinary extends Struct {
  readonly arch: Architecture;
  readonly os: OperatingSystem;
  readonly name: Bytes;
  readonly sha256: U8aFixed;
}

/** @name GadgetSource */
export interface GadgetSource extends Struct {
  readonly fetcher: GadgetSourceFetcher;
}

/** @name GadgetSourceFetcher */
export interface GadgetSourceFetcher extends Enum {
  readonly isIpfs: boolean;
  readonly asIpfs: Bytes;
  readonly isGithub: boolean;
  readonly asGithub: GithubFetcher;
  readonly isContainerImage: boolean;
  readonly asContainerImage: ImageRegistryFetcher;
  readonly isTesting: boolean;
  readonly asTesting: TestFetcher;
  readonly type: 'Ipfs' | 'Github' | 'ContainerImage' | 'Testing';
}

/** @name GithubFetcher */
export interface GithubFetcher extends Struct {
  readonly owner: Bytes;
  readonly repo: Bytes;
  readonly tag: Bytes;
  readonly binaries: Vec<GadgetBinary>;
}

/** @name ImageRegistryFetcher */
export interface ImageRegistryFetcher extends Struct {
  readonly registry_: Bytes;
  readonly image: Bytes;
  readonly tag: Bytes;
}

/** @name JobDefinition */
export interface JobDefinition extends Struct {
  readonly metadata: JobMetadata;
  readonly params: Vec<FieldFieldType>;
  readonly result: Vec<FieldFieldType>;
}

/** @name JobMetadata */
export interface JobMetadata extends Struct {
  readonly name: Bytes;
  readonly description: Option<Bytes>;
}

/** @name MasterBlueprintServiceManagerRevision */
export interface MasterBlueprintServiceManagerRevision extends Enum {
  readonly isLatest: boolean;
  readonly isSpecific: boolean;
  readonly asSpecific: u32;
  readonly type: 'Latest' | 'Specific';
}

/** @name MembershipModel */
export interface MembershipModel extends Enum {
  readonly isFixed: boolean;
  readonly asFixed: {
    readonly minOperators: u32;
  } & Struct;
  readonly isDynamic: boolean;
  readonly asDynamic: {
    readonly minOperators: u32;
    readonly maxOperators: Option<u32>;
  } & Struct;
  readonly type: 'Fixed' | 'Dynamic';
}

/** @name MembershipModelType */
export interface MembershipModelType extends Enum {
  readonly isFixed: boolean;
  readonly isDynamic: boolean;
  readonly type: 'Fixed' | 'Dynamic';
}

/** @name NativeGadget */
export interface NativeGadget extends Struct {
  readonly sources: Vec<GadgetSource>;
}

/** @name OperatingSystem */
export interface OperatingSystem extends Enum {
  readonly isUnknown: boolean;
  readonly isLinux: boolean;
  readonly isWindows: boolean;
  readonly isMacOS: boolean;
  readonly isBsd: boolean;
  readonly type: 'Unknown' | 'Linux' | 'Windows' | 'MacOS' | 'Bsd';
}

/** @name RpcServicesWithBlueprint */
export interface RpcServicesWithBlueprint extends Struct {
  readonly blueprintId: u64;
  readonly blueprint: ServiceBlueprint;
  readonly services: Vec<Service>;
}

/** @name Service */
export interface Service extends Struct {
  readonly id: u64;
  readonly blueprint: u64;
  readonly owner: AccountId32;
  readonly operatorSecurityCommitments: Vec<ITuple<[AccountId32, Vec<AssetSecurityCommitment>]>>;
  readonly securityRequirements: Vec<AssetSecurityRequirement>;
  readonly permittedCallers: Vec<AccountId32>;
  readonly ttl: u64;
  readonly membershipModel: MembershipModel;
}

/** @name ServiceBlueprint */
export interface ServiceBlueprint extends Struct {
  readonly metadata: ServiceMetadata;
  readonly jobs: Vec<JobDefinition>;
  readonly registrationParams: Vec<FieldFieldType>;
  readonly requestParams: Vec<FieldFieldType>;
  readonly manager: ServiceBlueprintServiceManager;
  readonly masterManagerRevision: MasterBlueprintServiceManagerRevision;
  readonly gadget: Gadget;
  readonly supportedMembershipModels: Vec<MembershipModelType>;
}

/** @name ServiceBlueprintServiceManager */
export interface ServiceBlueprintServiceManager extends Enum {
  readonly isEvm: boolean;
  readonly asEvm: H160;
  readonly type: 'Evm';
}

/** @name ServiceMetadata */
export interface ServiceMetadata extends Struct {
  readonly name: Bytes;
  readonly description: Option<Bytes>;
  readonly author: Option<Bytes>;
  readonly category: Option<Bytes>;
  readonly codeRepository: Option<Bytes>;
  readonly logo: Option<Bytes>;
  readonly website: Option<Bytes>;
  readonly license: Option<Bytes>;
}

/** @name ServiceRegistrationHook */
export interface ServiceRegistrationHook extends Enum {
  readonly isNone: boolean;
  readonly isEvm: boolean;
  readonly asEvm: H160;
  readonly type: 'None' | 'Evm';
}

/** @name ServiceRequestHook */
export interface ServiceRequestHook extends Enum {
  readonly isNone: boolean;
  readonly isEvm: boolean;
  readonly asEvm: H160;
  readonly type: 'None' | 'Evm';
}

/** @name TestFetcher */
export interface TestFetcher extends Struct {
  readonly cargoPackage: Bytes;
  readonly cargoBin: Bytes;
  readonly basePath: Bytes;
}

/** @name WasmGadget */
export interface WasmGadget extends Struct {
  readonly runtime: WasmRuntime;
  readonly sources: Vec<GadgetSource>;
}

/** @name WasmRuntime */
export interface WasmRuntime extends Enum {
  readonly isWasmtime: boolean;
  readonly isWasmer: boolean;
  readonly type: 'Wasmtime' | 'Wasmer';
}

export type PHANTOM_SERVICES = 'services';
