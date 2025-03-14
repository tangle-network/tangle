// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Enum, Option, Struct, Text, u128, u32 } from '@polkadot/types-codec';
import type { H160 } from '@polkadot/types/interfaces/runtime';

/** @name ErrorCode */
export interface ErrorCode extends Enum {
  readonly isParseError: boolean;
  readonly isOversizedRequest: boolean;
  readonly isInvalidRequest: boolean;
  readonly isMethodNotFound: boolean;
  readonly isServerIsBusy: boolean;
  readonly isInvalidParams: boolean;
  readonly isInternalError: boolean;
  readonly isServerError: boolean;
  readonly asServerError: u32;
  readonly type: 'ParseError' | 'OversizedRequest' | 'InvalidRequest' | 'MethodNotFound' | 'ServerIsBusy' | 'InvalidParams' | 'InternalError' | 'ServerError';
}

/** @name ErrorObjectOwned */
export interface ErrorObjectOwned extends Struct {
  readonly code: ErrorCode;
  readonly message: Text;
  readonly data: Option<RawValue>;
}

/** @name RawValue */
export interface RawValue extends Struct {
  readonly json: Text;
}

/** @name ServicesTypesAsset */
export interface ServicesTypesAsset extends Enum {
  readonly isCustom: boolean;
  readonly asCustom: u128;
  readonly isErc20: boolean;
  readonly asErc20: H160;
  readonly type: 'Custom' | 'Erc20';
}

export type PHANTOM_REWARDS = 'rewards';
