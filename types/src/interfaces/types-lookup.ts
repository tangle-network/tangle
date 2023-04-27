// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/types/lookup';

import type { Data } from '@polkadot/types';
import type { BTreeMap, Bytes, Compact, Enum, Null, Option, Result, Set, Struct, Text, U256, U8aFixed, Vec, bool, i128, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { Vote } from '@polkadot/types/interfaces/elections';
import type { OpaqueMultiaddr, OpaquePeerId } from '@polkadot/types/interfaces/imOnline';
import type { AccountId32, Call, H160, H256, MultiAddress, PerU16, Perbill, Percent } from '@polkadot/types/interfaces/runtime';
import type { Event } from '@polkadot/types/interfaces/system';

declare module '@polkadot/types/lookup' {
  /** @name FrameSystemAccountInfo (3) */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /** @name PalletBalancesAccountData (5) */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly miscFrozen: u128;
    readonly feeFrozen: u128;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (7) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /** @name SpWeightsWeightV2Weight (8) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /** @name SpRuntimeDigest (13) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (15) */
  interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean;
    readonly asOther: Bytes;
    readonly isConsensus: boolean;
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>;
    readonly isSeal: boolean;
    readonly asSeal: ITuple<[U8aFixed, Bytes]>;
    readonly isPreRuntime: boolean;
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>;
    readonly isRuntimeEnvironmentUpdated: boolean;
    readonly type: 'Other' | 'Consensus' | 'Seal' | 'PreRuntime' | 'RuntimeEnvironmentUpdated';
  }

  /** @name FrameSystemEventRecord (18) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (20) */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
    } & Struct;
    readonly isCodeUpdated: boolean;
    readonly isNewAccount: boolean;
    readonly asNewAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isKilledAccount: boolean;
    readonly asKilledAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRemarked: boolean;
    readonly asRemarked: {
      readonly sender: AccountId32;
      readonly hash_: H256;
    } & Struct;
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked';
  }

  /** @name FrameSupportDispatchDispatchInfo (21) */
  interface FrameSupportDispatchDispatchInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name FrameSupportDispatchDispatchClass (22) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportDispatchPays (23) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (24) */
  interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean;
    readonly isCannotLookup: boolean;
    readonly isBadOrigin: boolean;
    readonly isModule: boolean;
    readonly asModule: SpRuntimeModuleError;
    readonly isConsumerRemaining: boolean;
    readonly isNoProviders: boolean;
    readonly isTooManyConsumers: boolean;
    readonly isToken: boolean;
    readonly asToken: SpRuntimeTokenError;
    readonly isArithmetic: boolean;
    readonly asArithmetic: SpArithmeticArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly isExhausted: boolean;
    readonly isCorruption: boolean;
    readonly isUnavailable: boolean;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'TooManyConsumers' | 'Token' | 'Arithmetic' | 'Transactional' | 'Exhausted' | 'Corruption' | 'Unavailable';
  }

  /** @name SpRuntimeModuleError (25) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (26) */
  interface SpRuntimeTokenError extends Enum {
    readonly isNoFunds: boolean;
    readonly isWouldDie: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly type: 'NoFunds' | 'WouldDie' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported';
  }

  /** @name SpArithmeticArithmeticError (27) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name SpRuntimeTransactionalError (28) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: 'LimitReached' | 'NoLayer';
  }

  /** @name PalletSudoEvent (29) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly oldSudoer: Option<AccountId32>;
    } & Struct;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'Sudid' | 'KeyChanged' | 'SudoAsDone';
  }

  /** @name PalletBalancesEvent (33) */
  interface PalletBalancesEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly account: AccountId32;
      readonly freeBalance: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly account: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly who: AccountId32;
      readonly free: u128;
      readonly reserved: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly destinationStatus: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'BalanceSet' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'Deposit' | 'Withdraw' | 'Slashed';
  }

  /** @name FrameSupportTokensMiscBalanceStatus (34) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PalletTransactionPaymentEvent (35) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId32;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: 'TransactionFeePaid';
  }

  /** @name PalletGrandpaEvent (36) */
  interface PalletGrandpaEvent extends Enum {
    readonly isNewAuthorities: boolean;
    readonly asNewAuthorities: {
      readonly authoritySet: Vec<ITuple<[SpFinalityGrandpaAppPublic, u64]>>;
    } & Struct;
    readonly isPaused: boolean;
    readonly isResumed: boolean;
    readonly type: 'NewAuthorities' | 'Paused' | 'Resumed';
  }

  /** @name SpFinalityGrandpaAppPublic (39) */
  interface SpFinalityGrandpaAppPublic extends SpCoreEd25519Public {}

  /** @name SpCoreEd25519Public (40) */
  interface SpCoreEd25519Public extends U8aFixed {}

  /** @name PalletDkgMetadataEvent (41) */
  interface PalletDkgMetadataEvent extends Enum {
    readonly isPublicKeySubmitted: boolean;
    readonly asPublicKeySubmitted: {
      readonly compressedPubKey: Bytes;
      readonly uncompressedPubKey: Bytes;
    } & Struct;
    readonly isNextPublicKeySubmitted: boolean;
    readonly asNextPublicKeySubmitted: {
      readonly compressedPubKey: Bytes;
      readonly uncompressedPubKey: Bytes;
    } & Struct;
    readonly isNextPublicKeySignatureSubmitted: boolean;
    readonly asNextPublicKeySignatureSubmitted: {
      readonly pubKeySig: Bytes;
      readonly compressedPubKey: Bytes;
      readonly uncompressedPubKey: Bytes;
      readonly nonce: u32;
    } & Struct;
    readonly isPublicKeyChanged: boolean;
    readonly asPublicKeyChanged: {
      readonly compressedPubKey: Bytes;
      readonly uncompressedPubKey: Bytes;
    } & Struct;
    readonly isPublicKeySignatureChanged: boolean;
    readonly asPublicKeySignatureChanged: {
      readonly pubKeySig: Bytes;
      readonly compressedPubKey: Bytes;
      readonly uncompressedPubKey: Bytes;
      readonly nonce: u32;
    } & Struct;
    readonly isMisbehaviourReportsSubmitted: boolean;
    readonly asMisbehaviourReportsSubmitted: {
      readonly misbehaviourType: DkgRuntimePrimitivesMisbehaviourType;
      readonly reporters: Vec<DkgRuntimePrimitivesCryptoPublic>;
      readonly offender: DkgRuntimePrimitivesCryptoPublic;
    } & Struct;
    readonly isRefreshKeysFinished: boolean;
    readonly asRefreshKeysFinished: {
      readonly nextAuthoritySetId: u64;
    } & Struct;
    readonly isNextKeygenThresholdUpdated: boolean;
    readonly asNextKeygenThresholdUpdated: {
      readonly nextKeygenThreshold: u16;
    } & Struct;
    readonly isNextSignatureThresholdUpdated: boolean;
    readonly asNextSignatureThresholdUpdated: {
      readonly nextSignatureThreshold: u16;
    } & Struct;
    readonly isPendingKeygenThresholdUpdated: boolean;
    readonly asPendingKeygenThresholdUpdated: {
      readonly pendingKeygenThreshold: u16;
    } & Struct;
    readonly isPendingSignatureThresholdUpdated: boolean;
    readonly asPendingSignatureThresholdUpdated: {
      readonly pendingSignatureThreshold: u16;
    } & Struct;
    readonly isEmergencyKeygenTriggered: boolean;
    readonly isAuthorityJailed: boolean;
    readonly asAuthorityJailed: {
      readonly misbehaviourType: DkgRuntimePrimitivesMisbehaviourType;
      readonly authority: DkgRuntimePrimitivesCryptoPublic;
    } & Struct;
    readonly isAuthorityUnJailed: boolean;
    readonly asAuthorityUnJailed: {
      readonly authority: DkgRuntimePrimitivesCryptoPublic;
    } & Struct;
    readonly type: 'PublicKeySubmitted' | 'NextPublicKeySubmitted' | 'NextPublicKeySignatureSubmitted' | 'PublicKeyChanged' | 'PublicKeySignatureChanged' | 'MisbehaviourReportsSubmitted' | 'RefreshKeysFinished' | 'NextKeygenThresholdUpdated' | 'NextSignatureThresholdUpdated' | 'PendingKeygenThresholdUpdated' | 'PendingSignatureThresholdUpdated' | 'EmergencyKeygenTriggered' | 'AuthorityJailed' | 'AuthorityUnJailed';
  }

  /** @name DkgRuntimePrimitivesMisbehaviourType (42) */
  interface DkgRuntimePrimitivesMisbehaviourType extends Enum {
    readonly isKeygen: boolean;
    readonly isSign: boolean;
    readonly type: 'Keygen' | 'Sign';
  }

  /** @name DkgRuntimePrimitivesCryptoPublic (44) */
  interface DkgRuntimePrimitivesCryptoPublic extends SpCoreEcdsaPublic {}

  /** @name SpCoreEcdsaPublic (45) */
  interface SpCoreEcdsaPublic extends U8aFixed {}

  /** @name PalletDkgProposalsEvent (48) */
  interface PalletDkgProposalsEvent extends Enum {
    readonly isProposerThresholdChanged: boolean;
    readonly asProposerThresholdChanged: {
      readonly newThreshold: u32;
    } & Struct;
    readonly isChainWhitelisted: boolean;
    readonly asChainWhitelisted: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
    } & Struct;
    readonly isProposerAdded: boolean;
    readonly asProposerAdded: {
      readonly proposerId: AccountId32;
    } & Struct;
    readonly isProposerRemoved: boolean;
    readonly asProposerRemoved: {
      readonly proposerId: AccountId32;
    } & Struct;
    readonly isVoteFor: boolean;
    readonly asVoteFor: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly isVoteAgainst: boolean;
    readonly asVoteAgainst: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly isProposalApproved: boolean;
    readonly asProposalApproved: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isProposalRejected: boolean;
    readonly asProposalRejected: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isProposalSucceeded: boolean;
    readonly asProposalSucceeded: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isProposalFailed: boolean;
    readonly asProposalFailed: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isAuthorityProposersReset: boolean;
    readonly asAuthorityProposersReset: {
      readonly proposers: Vec<AccountId32>;
    } & Struct;
    readonly type: 'ProposerThresholdChanged' | 'ChainWhitelisted' | 'ProposerAdded' | 'ProposerRemoved' | 'VoteFor' | 'VoteAgainst' | 'ProposalApproved' | 'ProposalRejected' | 'ProposalSucceeded' | 'ProposalFailed' | 'AuthorityProposersReset';
  }

  /** @name WebbProposalsHeaderTypedChainId (49) */
  interface WebbProposalsHeaderTypedChainId extends Enum {
    readonly isNone: boolean;
    readonly isEvm: boolean;
    readonly asEvm: u32;
    readonly isSubstrate: boolean;
    readonly asSubstrate: u32;
    readonly isPolkadotParachain: boolean;
    readonly asPolkadotParachain: u32;
    readonly isKusamaParachain: boolean;
    readonly asKusamaParachain: u32;
    readonly isRococoParachain: boolean;
    readonly asRococoParachain: u32;
    readonly isCosmos: boolean;
    readonly asCosmos: u32;
    readonly isSolana: boolean;
    readonly asSolana: u32;
    readonly isInk: boolean;
    readonly asInk: u32;
    readonly type: 'None' | 'Evm' | 'Substrate' | 'PolkadotParachain' | 'KusamaParachain' | 'RococoParachain' | 'Cosmos' | 'Solana' | 'Ink';
  }

  /** @name PalletDkgProposalHandlerEvent (52) */
  interface PalletDkgProposalHandlerEvent extends Enum {
    readonly isInvalidProposalSignature: boolean;
    readonly asInvalidProposalSignature: {
      readonly kind: WebbProposalsProposalProposalKind;
      readonly data: Bytes;
      readonly invalidSignature: Bytes;
      readonly expectedPublicKey: Option<Bytes>;
      readonly actualPublicKey: Option<Bytes>;
    } & Struct;
    readonly isProposalAdded: boolean;
    readonly asProposalAdded: {
      readonly key: DkgRuntimePrimitivesProposalDkgPayloadKey;
      readonly targetChain: WebbProposalsHeaderTypedChainId;
      readonly data: Bytes;
    } & Struct;
    readonly isProposalRemoved: boolean;
    readonly asProposalRemoved: {
      readonly key: DkgRuntimePrimitivesProposalDkgPayloadKey;
      readonly targetChain: WebbProposalsHeaderTypedChainId;
      readonly expired: bool;
    } & Struct;
    readonly isProposalSigned: boolean;
    readonly asProposalSigned: {
      readonly key: DkgRuntimePrimitivesProposalDkgPayloadKey;
      readonly targetChain: WebbProposalsHeaderTypedChainId;
      readonly data: Bytes;
      readonly signature: Bytes;
    } & Struct;
    readonly type: 'InvalidProposalSignature' | 'ProposalAdded' | 'ProposalRemoved' | 'ProposalSigned';
  }

  /** @name WebbProposalsProposalProposalKind (53) */
  interface WebbProposalsProposalProposalKind extends Enum {
    readonly isRefresh: boolean;
    readonly isProposerSetUpdate: boolean;
    readonly isEvm: boolean;
    readonly isAnchorCreate: boolean;
    readonly isAnchorUpdate: boolean;
    readonly isTokenAdd: boolean;
    readonly isTokenRemove: boolean;
    readonly isWrappingFeeUpdate: boolean;
    readonly isResourceIdUpdate: boolean;
    readonly isRescueTokens: boolean;
    readonly isMaxDepositLimitUpdate: boolean;
    readonly isMinWithdrawalLimitUpdate: boolean;
    readonly isSetVerifier: boolean;
    readonly isSetTreasuryHandler: boolean;
    readonly isFeeRecipientUpdate: boolean;
    readonly type: 'Refresh' | 'ProposerSetUpdate' | 'Evm' | 'AnchorCreate' | 'AnchorUpdate' | 'TokenAdd' | 'TokenRemove' | 'WrappingFeeUpdate' | 'ResourceIdUpdate' | 'RescueTokens' | 'MaxDepositLimitUpdate' | 'MinWithdrawalLimitUpdate' | 'SetVerifier' | 'SetTreasuryHandler' | 'FeeRecipientUpdate';
  }

  /** @name DkgRuntimePrimitivesProposalDkgPayloadKey (55) */
  interface DkgRuntimePrimitivesProposalDkgPayloadKey extends Enum {
    readonly isEvmProposal: boolean;
    readonly asEvmProposal: u32;
    readonly isRefreshVote: boolean;
    readonly asRefreshVote: u32;
    readonly isProposerSetUpdateProposal: boolean;
    readonly asProposerSetUpdateProposal: u32;
    readonly isAnchorCreateProposal: boolean;
    readonly asAnchorCreateProposal: u32;
    readonly isAnchorUpdateProposal: boolean;
    readonly asAnchorUpdateProposal: u32;
    readonly isTokenAddProposal: boolean;
    readonly asTokenAddProposal: u32;
    readonly isTokenRemoveProposal: boolean;
    readonly asTokenRemoveProposal: u32;
    readonly isWrappingFeeUpdateProposal: boolean;
    readonly asWrappingFeeUpdateProposal: u32;
    readonly isResourceIdUpdateProposal: boolean;
    readonly asResourceIdUpdateProposal: u32;
    readonly isRescueTokensProposal: boolean;
    readonly asRescueTokensProposal: u32;
    readonly isMaxDepositLimitUpdateProposal: boolean;
    readonly asMaxDepositLimitUpdateProposal: u32;
    readonly isMinWithdrawalLimitUpdateProposal: boolean;
    readonly asMinWithdrawalLimitUpdateProposal: u32;
    readonly isSetVerifierProposal: boolean;
    readonly asSetVerifierProposal: u32;
    readonly isSetTreasuryHandlerProposal: boolean;
    readonly asSetTreasuryHandlerProposal: u32;
    readonly isFeeRecipientUpdateProposal: boolean;
    readonly asFeeRecipientUpdateProposal: u32;
    readonly type: 'EvmProposal' | 'RefreshVote' | 'ProposerSetUpdateProposal' | 'AnchorCreateProposal' | 'AnchorUpdateProposal' | 'TokenAddProposal' | 'TokenRemoveProposal' | 'WrappingFeeUpdateProposal' | 'ResourceIdUpdateProposal' | 'RescueTokensProposal' | 'MaxDepositLimitUpdateProposal' | 'MinWithdrawalLimitUpdateProposal' | 'SetVerifierProposal' | 'SetTreasuryHandlerProposal' | 'FeeRecipientUpdateProposal';
  }

  /** @name PalletBridgeRegistryEvent (57) */
  type PalletBridgeRegistryEvent = Null;

  /** @name PalletIndicesEvent (58) */
  interface PalletIndicesEvent extends Enum {
    readonly isIndexAssigned: boolean;
    readonly asIndexAssigned: {
      readonly who: AccountId32;
      readonly index: u32;
    } & Struct;
    readonly isIndexFreed: boolean;
    readonly asIndexFreed: {
      readonly index: u32;
    } & Struct;
    readonly isIndexFrozen: boolean;
    readonly asIndexFrozen: {
      readonly index: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly type: 'IndexAssigned' | 'IndexFreed' | 'IndexFrozen';
  }

  /** @name PalletDemocracyEvent (59) */
  interface PalletDemocracyEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isTabled: boolean;
    readonly asTabled: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isExternalTabled: boolean;
    readonly isStarted: boolean;
    readonly asStarted: {
      readonly refIndex: u32;
      readonly threshold: PalletDemocracyVoteThreshold;
    } & Struct;
    readonly isPassed: boolean;
    readonly asPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isNotPassed: boolean;
    readonly asNotPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isCancelled: boolean;
    readonly asCancelled: {
      readonly refIndex: u32;
    } & Struct;
    readonly isDelegated: boolean;
    readonly asDelegated: {
      readonly who: AccountId32;
      readonly target: AccountId32;
    } & Struct;
    readonly isUndelegated: boolean;
    readonly asUndelegated: {
      readonly account: AccountId32;
    } & Struct;
    readonly isVetoed: boolean;
    readonly asVetoed: {
      readonly who: AccountId32;
      readonly proposalHash: H256;
      readonly until: u32;
    } & Struct;
    readonly isBlacklisted: boolean;
    readonly asBlacklisted: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly voter: AccountId32;
      readonly refIndex: u32;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isSeconded: boolean;
    readonly asSeconded: {
      readonly seconder: AccountId32;
      readonly propIndex: u32;
    } & Struct;
    readonly isProposalCanceled: boolean;
    readonly asProposalCanceled: {
      readonly propIndex: u32;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly isMetadataCleared: boolean;
    readonly asMetadataCleared: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly isMetadataTransferred: boolean;
    readonly asMetadataTransferred: {
      readonly prevOwner: PalletDemocracyMetadataOwner;
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly type: 'Proposed' | 'Tabled' | 'ExternalTabled' | 'Started' | 'Passed' | 'NotPassed' | 'Cancelled' | 'Delegated' | 'Undelegated' | 'Vetoed' | 'Blacklisted' | 'Voted' | 'Seconded' | 'ProposalCanceled' | 'MetadataSet' | 'MetadataCleared' | 'MetadataTransferred';
  }

  /** @name PalletDemocracyVoteThreshold (60) */
  interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean;
    readonly isSuperMajorityAgainst: boolean;
    readonly isSimpleMajority: boolean;
    readonly type: 'SuperMajorityApprove' | 'SuperMajorityAgainst' | 'SimpleMajority';
  }

  /** @name PalletDemocracyVoteAccountVote (61) */
  interface PalletDemocracyVoteAccountVote extends Enum {
    readonly isStandard: boolean;
    readonly asStandard: {
      readonly vote: Vote;
      readonly balance: u128;
    } & Struct;
    readonly isSplit: boolean;
    readonly asSplit: {
      readonly aye: u128;
      readonly nay: u128;
    } & Struct;
    readonly type: 'Standard' | 'Split';
  }

  /** @name PalletDemocracyMetadataOwner (63) */
  interface PalletDemocracyMetadataOwner extends Enum {
    readonly isExternal: boolean;
    readonly isProposal: boolean;
    readonly asProposal: u32;
    readonly isReferendum: boolean;
    readonly asReferendum: u32;
    readonly type: 'External' | 'Proposal' | 'Referendum';
  }

  /** @name PalletCollectiveEvent (64) */
  interface PalletCollectiveEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly account: AccountId32;
      readonly proposalIndex: u32;
      readonly proposalHash: H256;
      readonly threshold: u32;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly account: AccountId32;
      readonly proposalHash: H256;
      readonly voted: bool;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly isApproved: boolean;
    readonly asApproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isDisapproved: boolean;
    readonly asDisapproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMemberExecuted: boolean;
    readonly asMemberExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isClosed: boolean;
    readonly asClosed: {
      readonly proposalHash: H256;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Voted' | 'Approved' | 'Disapproved' | 'Executed' | 'MemberExecuted' | 'Closed';
  }

  /** @name PalletVestingEvent (65) */
  interface PalletVestingEvent extends Enum {
    readonly isVestingUpdated: boolean;
    readonly asVestingUpdated: {
      readonly account: AccountId32;
      readonly unvested: u128;
    } & Struct;
    readonly isVestingCompleted: boolean;
    readonly asVestingCompleted: {
      readonly account: AccountId32;
    } & Struct;
    readonly type: 'VestingUpdated' | 'VestingCompleted';
  }

  /** @name PalletEcdsaClaimsEvent (66) */
  interface PalletEcdsaClaimsEvent extends Enum {
    readonly isClaimed: boolean;
    readonly asClaimed: {
      readonly who: AccountId32;
      readonly ethereumAddress: PalletEcdsaClaimsEthereumAddress;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Claimed';
  }

  /** @name PalletEcdsaClaimsEthereumAddress (67) */
  interface PalletEcdsaClaimsEthereumAddress extends U8aFixed {}

  /** @name PalletElectionsPhragmenEvent (69) */
  interface PalletElectionsPhragmenEvent extends Enum {
    readonly isNewTerm: boolean;
    readonly asNewTerm: {
      readonly newMembers: Vec<ITuple<[AccountId32, u128]>>;
    } & Struct;
    readonly isEmptyTerm: boolean;
    readonly isElectionError: boolean;
    readonly isMemberKicked: boolean;
    readonly asMemberKicked: {
      readonly member: AccountId32;
    } & Struct;
    readonly isRenounced: boolean;
    readonly asRenounced: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCandidateSlashed: boolean;
    readonly asCandidateSlashed: {
      readonly candidate: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSeatHolderSlashed: boolean;
    readonly asSeatHolderSlashed: {
      readonly seatHolder: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'NewTerm' | 'EmptyTerm' | 'ElectionError' | 'MemberKicked' | 'Renounced' | 'CandidateSlashed' | 'SeatHolderSlashed';
  }

  /** @name PalletElectionProviderMultiPhaseEvent (72) */
  interface PalletElectionProviderMultiPhaseEvent extends Enum {
    readonly isSolutionStored: boolean;
    readonly asSolutionStored: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
      readonly origin: Option<AccountId32>;
      readonly prevEjected: bool;
    } & Struct;
    readonly isElectionFinalized: boolean;
    readonly asElectionFinalized: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
      readonly score: SpNposElectionsElectionScore;
    } & Struct;
    readonly isElectionFailed: boolean;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly account: AccountId32;
      readonly value: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly account: AccountId32;
      readonly value: u128;
    } & Struct;
    readonly isPhaseTransitioned: boolean;
    readonly asPhaseTransitioned: {
      readonly from: PalletElectionProviderMultiPhasePhase;
      readonly to: PalletElectionProviderMultiPhasePhase;
      readonly round: u32;
    } & Struct;
    readonly type: 'SolutionStored' | 'ElectionFinalized' | 'ElectionFailed' | 'Rewarded' | 'Slashed' | 'PhaseTransitioned';
  }

  /** @name PalletElectionProviderMultiPhaseElectionCompute (73) */
  interface PalletElectionProviderMultiPhaseElectionCompute extends Enum {
    readonly isOnChain: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly isFallback: boolean;
    readonly isEmergency: boolean;
    readonly type: 'OnChain' | 'Signed' | 'Unsigned' | 'Fallback' | 'Emergency';
  }

  /** @name SpNposElectionsElectionScore (74) */
  interface SpNposElectionsElectionScore extends Struct {
    readonly minimalStake: u128;
    readonly sumStake: u128;
    readonly sumStakeSquared: u128;
  }

  /** @name PalletElectionProviderMultiPhasePhase (75) */
  interface PalletElectionProviderMultiPhasePhase extends Enum {
    readonly isOff: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly asUnsigned: ITuple<[bool, u32]>;
    readonly isEmergency: boolean;
    readonly type: 'Off' | 'Signed' | 'Unsigned' | 'Emergency';
  }

  /** @name PalletStakingPalletEvent (77) */
  interface PalletStakingPalletEvent extends Enum {
    readonly isEraPaid: boolean;
    readonly asEraPaid: {
      readonly eraIndex: u32;
      readonly validatorPayout: u128;
      readonly remainder: u128;
    } & Struct;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly staker: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashReported: boolean;
    readonly asSlashReported: {
      readonly validator: AccountId32;
      readonly fraction: Perbill;
      readonly slashEra: u32;
    } & Struct;
    readonly isOldSlashingReportDiscarded: boolean;
    readonly asOldSlashingReportDiscarded: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly isStakersElected: boolean;
    readonly isBonded: boolean;
    readonly asBonded: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnbonded: boolean;
    readonly asUnbonded: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isKicked: boolean;
    readonly asKicked: {
      readonly nominator: AccountId32;
      readonly stash: AccountId32;
    } & Struct;
    readonly isStakingElectionFailed: boolean;
    readonly isChilled: boolean;
    readonly asChilled: {
      readonly stash: AccountId32;
    } & Struct;
    readonly isPayoutStarted: boolean;
    readonly asPayoutStarted: {
      readonly eraIndex: u32;
      readonly validatorStash: AccountId32;
    } & Struct;
    readonly isValidatorPrefsSet: boolean;
    readonly asValidatorPrefsSet: {
      readonly stash: AccountId32;
      readonly prefs: PalletStakingValidatorPrefs;
    } & Struct;
    readonly isForceEra: boolean;
    readonly asForceEra: {
      readonly mode: PalletStakingForcing;
    } & Struct;
    readonly type: 'EraPaid' | 'Rewarded' | 'Slashed' | 'SlashReported' | 'OldSlashingReportDiscarded' | 'StakersElected' | 'Bonded' | 'Unbonded' | 'Withdrawn' | 'Kicked' | 'StakingElectionFailed' | 'Chilled' | 'PayoutStarted' | 'ValidatorPrefsSet' | 'ForceEra';
  }

  /** @name PalletStakingValidatorPrefs (79) */
  interface PalletStakingValidatorPrefs extends Struct {
    readonly commission: Compact<Perbill>;
    readonly blocked: bool;
  }

  /** @name PalletStakingForcing (81) */
  interface PalletStakingForcing extends Enum {
    readonly isNotForcing: boolean;
    readonly isForceNew: boolean;
    readonly isForceNone: boolean;
    readonly isForceAlways: boolean;
    readonly type: 'NotForcing' | 'ForceNew' | 'ForceNone' | 'ForceAlways';
  }

  /** @name PalletSessionEvent (82) */
  interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean;
    readonly asNewSession: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly type: 'NewSession';
  }

  /** @name PalletTreasuryEvent (83) */
  interface PalletTreasuryEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
    } & Struct;
    readonly isSpending: boolean;
    readonly asSpending: {
      readonly budgetRemaining: u128;
    } & Struct;
    readonly isAwarded: boolean;
    readonly asAwarded: {
      readonly proposalIndex: u32;
      readonly award: u128;
      readonly account: AccountId32;
    } & Struct;
    readonly isRejected: boolean;
    readonly asRejected: {
      readonly proposalIndex: u32;
      readonly slashed: u128;
    } & Struct;
    readonly isBurnt: boolean;
    readonly asBurnt: {
      readonly burntFunds: u128;
    } & Struct;
    readonly isRollover: boolean;
    readonly asRollover: {
      readonly rolloverBalance: u128;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly value: u128;
    } & Struct;
    readonly isSpendApproved: boolean;
    readonly asSpendApproved: {
      readonly proposalIndex: u32;
      readonly amount: u128;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isUpdatedInactive: boolean;
    readonly asUpdatedInactive: {
      readonly reactivated: u128;
      readonly deactivated: u128;
    } & Struct;
    readonly type: 'Proposed' | 'Spending' | 'Awarded' | 'Rejected' | 'Burnt' | 'Rollover' | 'Deposit' | 'SpendApproved' | 'UpdatedInactive';
  }

  /** @name PalletBountiesEvent (84) */
  interface PalletBountiesEvent extends Enum {
    readonly isBountyProposed: boolean;
    readonly asBountyProposed: {
      readonly index: u32;
    } & Struct;
    readonly isBountyRejected: boolean;
    readonly asBountyRejected: {
      readonly index: u32;
      readonly bond: u128;
    } & Struct;
    readonly isBountyBecameActive: boolean;
    readonly asBountyBecameActive: {
      readonly index: u32;
    } & Struct;
    readonly isBountyAwarded: boolean;
    readonly asBountyAwarded: {
      readonly index: u32;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isBountyClaimed: boolean;
    readonly asBountyClaimed: {
      readonly index: u32;
      readonly payout: u128;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isBountyCanceled: boolean;
    readonly asBountyCanceled: {
      readonly index: u32;
    } & Struct;
    readonly isBountyExtended: boolean;
    readonly asBountyExtended: {
      readonly index: u32;
    } & Struct;
    readonly type: 'BountyProposed' | 'BountyRejected' | 'BountyBecameActive' | 'BountyAwarded' | 'BountyClaimed' | 'BountyCanceled' | 'BountyExtended';
  }

  /** @name PalletChildBountiesEvent (85) */
  interface PalletChildBountiesEvent extends Enum {
    readonly isAdded: boolean;
    readonly asAdded: {
      readonly index: u32;
      readonly childIndex: u32;
    } & Struct;
    readonly isAwarded: boolean;
    readonly asAwarded: {
      readonly index: u32;
      readonly childIndex: u32;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isClaimed: boolean;
    readonly asClaimed: {
      readonly index: u32;
      readonly childIndex: u32;
      readonly payout: u128;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly index: u32;
      readonly childIndex: u32;
    } & Struct;
    readonly type: 'Added' | 'Awarded' | 'Claimed' | 'Canceled';
  }

  /** @name PalletBagsListEvent (86) */
  interface PalletBagsListEvent extends Enum {
    readonly isRebagged: boolean;
    readonly asRebagged: {
      readonly who: AccountId32;
      readonly from: u64;
      readonly to: u64;
    } & Struct;
    readonly isScoreUpdated: boolean;
    readonly asScoreUpdated: {
      readonly who: AccountId32;
      readonly newScore: u64;
    } & Struct;
    readonly type: 'Rebagged' | 'ScoreUpdated';
  }

  /** @name PalletNominationPoolsEvent (87) */
  interface PalletNominationPoolsEvent extends Enum {
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly depositor: AccountId32;
      readonly poolId: u32;
    } & Struct;
    readonly isBonded: boolean;
    readonly asBonded: {
      readonly member: AccountId32;
      readonly poolId: u32;
      readonly bonded: u128;
      readonly joined: bool;
    } & Struct;
    readonly isPaidOut: boolean;
    readonly asPaidOut: {
      readonly member: AccountId32;
      readonly poolId: u32;
      readonly payout: u128;
    } & Struct;
    readonly isUnbonded: boolean;
    readonly asUnbonded: {
      readonly member: AccountId32;
      readonly poolId: u32;
      readonly balance: u128;
      readonly points: u128;
      readonly era: u32;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly member: AccountId32;
      readonly poolId: u32;
      readonly balance: u128;
      readonly points: u128;
    } & Struct;
    readonly isDestroyed: boolean;
    readonly asDestroyed: {
      readonly poolId: u32;
    } & Struct;
    readonly isStateChanged: boolean;
    readonly asStateChanged: {
      readonly poolId: u32;
      readonly newState: PalletNominationPoolsPoolState;
    } & Struct;
    readonly isMemberRemoved: boolean;
    readonly asMemberRemoved: {
      readonly poolId: u32;
      readonly member: AccountId32;
    } & Struct;
    readonly isRolesUpdated: boolean;
    readonly asRolesUpdated: {
      readonly root: Option<AccountId32>;
      readonly bouncer: Option<AccountId32>;
      readonly nominator: Option<AccountId32>;
    } & Struct;
    readonly isPoolSlashed: boolean;
    readonly asPoolSlashed: {
      readonly poolId: u32;
      readonly balance: u128;
    } & Struct;
    readonly isUnbondingPoolSlashed: boolean;
    readonly asUnbondingPoolSlashed: {
      readonly poolId: u32;
      readonly era: u32;
      readonly balance: u128;
    } & Struct;
    readonly type: 'Created' | 'Bonded' | 'PaidOut' | 'Unbonded' | 'Withdrawn' | 'Destroyed' | 'StateChanged' | 'MemberRemoved' | 'RolesUpdated' | 'PoolSlashed' | 'UnbondingPoolSlashed';
  }

  /** @name PalletNominationPoolsPoolState (88) */
  interface PalletNominationPoolsPoolState extends Enum {
    readonly isOpen: boolean;
    readonly isBlocked: boolean;
    readonly isDestroying: boolean;
    readonly type: 'Open' | 'Blocked' | 'Destroying';
  }

  /** @name PalletSchedulerEvent (89) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isDispatched: boolean;
    readonly asDispatched: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isCallUnavailable: boolean;
    readonly asCallUnavailable: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPeriodicFailed: boolean;
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched' | 'CallUnavailable' | 'PeriodicFailed' | 'PermanentlyOverweight';
  }

  /** @name PalletPreimageEvent (92) */
  interface PalletPreimageEvent extends Enum {
    readonly isNoted: boolean;
    readonly asNoted: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly hash_: H256;
    } & Struct;
    readonly isCleared: boolean;
    readonly asCleared: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'Noted' | 'Requested' | 'Cleared';
  }

  /** @name PalletOffencesEvent (93) */
  interface PalletOffencesEvent extends Enum {
    readonly isOffence: boolean;
    readonly asOffence: {
      readonly kind: U8aFixed;
      readonly timeslot: Bytes;
    } & Struct;
    readonly type: 'Offence';
  }

  /** @name PalletHasherEvent (95) */
  type PalletHasherEvent = Null;

  /** @name PalletAssetRegistryEvent (96) */
  interface PalletAssetRegistryEvent extends Enum {
    readonly isRegistered: boolean;
    readonly asRegistered: {
      readonly assetId: u32;
      readonly name: Bytes;
      readonly assetType: PalletAssetRegistryAssetType;
    } & Struct;
    readonly isUpdated: boolean;
    readonly asUpdated: {
      readonly assetId: u32;
      readonly name: Bytes;
      readonly assetType: PalletAssetRegistryAssetType;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly assetId: u32;
      readonly symbol: Bytes;
      readonly decimals: u8;
    } & Struct;
    readonly isLocationSet: boolean;
    readonly asLocationSet: {
      readonly assetId: u32;
      readonly location: Null;
    } & Struct;
    readonly type: 'Registered' | 'Updated' | 'MetadataSet' | 'LocationSet';
  }

  /** @name PalletAssetRegistryAssetType (98) */
  interface PalletAssetRegistryAssetType extends Enum {
    readonly isToken: boolean;
    readonly isPoolShare: boolean;
    readonly asPoolShare: Vec<u32>;
    readonly type: 'Token' | 'PoolShare';
  }

  /** @name TangleStandaloneRuntimeProtocolSubstrateConfigMaxAssetIdInPool (99) */
  type TangleStandaloneRuntimeProtocolSubstrateConfigMaxAssetIdInPool = Null;

  /** @name OrmlTokensModuleEvent (102) */
  interface OrmlTokensModuleEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly currencyId: u32;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly currencyId: u32;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly status: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly free: u128;
      readonly reserved: u128;
    } & Struct;
    readonly isTotalIssuanceSet: boolean;
    readonly asTotalIssuanceSet: {
      readonly currencyId: u32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly freeAmount: u128;
      readonly reservedAmount: u128;
    } & Struct;
    readonly isDeposited: boolean;
    readonly asDeposited: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isLockSet: boolean;
    readonly asLockSet: {
      readonly lockId: U8aFixed;
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isLockRemoved: boolean;
    readonly asLockRemoved: {
      readonly lockId: U8aFixed;
      readonly currencyId: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly currencyId: u32;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'BalanceSet' | 'TotalIssuanceSet' | 'Withdrawn' | 'Slashed' | 'Deposited' | 'LockSet' | 'LockRemoved' | 'Locked' | 'Unlocked';
  }

  /** @name PalletTokenWrapperEvent (104) */
  interface PalletTokenWrapperEvent extends Enum {
    readonly isWrappedToken: boolean;
    readonly asWrappedToken: {
      readonly poolShareAsset: u32;
      readonly assetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
    } & Struct;
    readonly isUnwrappedToken: boolean;
    readonly asUnwrappedToken: {
      readonly poolShareAsset: u32;
      readonly assetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
    } & Struct;
    readonly isUpdatedWrappingFeePercent: boolean;
    readonly asUpdatedWrappingFeePercent: {
      readonly intoPoolShareId: u32;
      readonly wrappingFeePercent: u128;
    } & Struct;
    readonly isUpdatedFeeRecipient: boolean;
    readonly asUpdatedFeeRecipient: {
      readonly feeRecipient: AccountId32;
      readonly poolShareId: u32;
    } & Struct;
    readonly isTokensRescued: boolean;
    readonly asTokensRescued: {
      readonly fromPoolShareId: u32;
      readonly assetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
    } & Struct;
    readonly type: 'WrappedToken' | 'UnwrappedToken' | 'UpdatedWrappingFeePercent' | 'UpdatedFeeRecipient' | 'TokensRescued';
  }

  /** @name PalletVerifierEvent (105) */
  type PalletVerifierEvent = Null;

  /** @name PalletMtEvent (106) */
  interface PalletMtEvent extends Enum {
    readonly isTreeCreation: boolean;
    readonly asTreeCreation: {
      readonly treeId: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly isLeafInsertion: boolean;
    readonly asLeafInsertion: {
      readonly treeId: u32;
      readonly leafIndex: u32;
      readonly leaf: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    } & Struct;
    readonly type: 'TreeCreation' | 'LeafInsertion';
  }

  /** @name TangleStandaloneRuntimeProtocolSubstrateConfigElement (107) */
  interface TangleStandaloneRuntimeProtocolSubstrateConfigElement extends U8aFixed {}

  /** @name PalletLinkableTreeEvent (108) */
  interface PalletLinkableTreeEvent extends Enum {
    readonly isLinkableTreeCreation: boolean;
    readonly asLinkableTreeCreation: {
      readonly treeId: u32;
    } & Struct;
    readonly type: 'LinkableTreeCreation';
  }

  /** @name PalletMixerEvent (109) */
  interface PalletMixerEvent extends Enum {
    readonly isMixerCreation: boolean;
    readonly asMixerCreation: {
      readonly treeId: u32;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly treeId: u32;
      readonly leaf: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly treeId: u32;
      readonly recipient: AccountId32;
    } & Struct;
    readonly type: 'MixerCreation' | 'Deposit' | 'Withdraw';
  }

  /** @name PalletSignatureBridgeEvent (110) */
  interface PalletSignatureBridgeEvent extends Enum {
    readonly isMaintainerSet: boolean;
    readonly asMaintainerSet: {
      readonly oldMaintainer: Bytes;
      readonly newMaintainer: Bytes;
    } & Struct;
    readonly isChainWhitelisted: boolean;
    readonly asChainWhitelisted: {
      readonly chainId: u64;
    } & Struct;
    readonly isProposalApproved: boolean;
    readonly asProposalApproved: {
      readonly chainId: u64;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isProposalSucceeded: boolean;
    readonly asProposalSucceeded: {
      readonly chainId: u64;
      readonly proposalNonce: u32;
    } & Struct;
    readonly isProposalFailed: boolean;
    readonly asProposalFailed: {
      readonly chainId: u64;
      readonly proposalNonce: u32;
    } & Struct;
    readonly type: 'MaintainerSet' | 'ChainWhitelisted' | 'ProposalApproved' | 'ProposalSucceeded' | 'ProposalFailed';
  }

  /** @name PalletVanchorEvent (112) */
  interface PalletVanchorEvent extends Enum {
    readonly isVAnchorCreation: boolean;
    readonly asVAnchorCreation: {
      readonly treeId: u32;
    } & Struct;
    readonly isTransaction: boolean;
    readonly asTransaction: {
      readonly transactor: AccountId32;
      readonly treeId: u32;
      readonly leafs: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
      readonly encryptedOutput1: Bytes;
      readonly encryptedOutput2: Bytes;
      readonly amount: i128;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly depositor: AccountId32;
      readonly treeId: u32;
      readonly leaf: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    } & Struct;
    readonly isMaxDepositAmountChanged: boolean;
    readonly asMaxDepositAmountChanged: {
      readonly maxDepositAmount: u128;
    } & Struct;
    readonly isMinWithdrawAmountChanged: boolean;
    readonly asMinWithdrawAmountChanged: {
      readonly minWithdrawAmount: u128;
    } & Struct;
    readonly type: 'VAnchorCreation' | 'Transaction' | 'Deposit' | 'MaxDepositAmountChanged' | 'MinWithdrawAmountChanged';
  }

  /** @name PalletVanchorHandlerEvent (115) */
  interface PalletVanchorHandlerEvent extends Enum {
    readonly isAnchorCreated: boolean;
    readonly isAnchorEdgeAdded: boolean;
    readonly isAnchorEdgeUpdated: boolean;
    readonly isResourceAnchored: boolean;
    readonly type: 'AnchorCreated' | 'AnchorEdgeAdded' | 'AnchorEdgeUpdated' | 'ResourceAnchored';
  }

  /** @name PalletTokenWrapperHandlerEvent (116) */
  interface PalletTokenWrapperHandlerEvent extends Enum {
    readonly isUpdatedWrappingFeePercent: boolean;
    readonly asUpdatedWrappingFeePercent: {
      readonly wrappingFeePercent: u128;
    } & Struct;
    readonly type: 'UpdatedWrappingFeePercent';
  }

  /** @name PalletKeyStorageEvent (117) */
  interface PalletKeyStorageEvent extends Enum {
    readonly isPublicKeyRegistration: boolean;
    readonly asPublicKeyRegistration: {
      readonly owner: AccountId32;
      readonly publicKey: Bytes;
    } & Struct;
    readonly type: 'PublicKeyRegistration';
  }

  /** @name PalletVanchorVerifierEvent (119) */
  type PalletVanchorVerifierEvent = Null;

  /** @name PalletTransactionPauseModuleEvent (120) */
  interface PalletTransactionPauseModuleEvent extends Enum {
    readonly isTransactionPaused: boolean;
    readonly asTransactionPaused: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Bytes;
    } & Struct;
    readonly isTransactionUnpaused: boolean;
    readonly asTransactionUnpaused: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Bytes;
    } & Struct;
    readonly type: 'TransactionPaused' | 'TransactionUnpaused';
  }

  /** @name PalletImOnlineEvent (121) */
  interface PalletImOnlineEvent extends Enum {
    readonly isHeartbeatReceived: boolean;
    readonly asHeartbeatReceived: {
      readonly authorityId: PalletImOnlineSr25519AppSr25519Public;
    } & Struct;
    readonly isAllGood: boolean;
    readonly isSomeOffline: boolean;
    readonly asSomeOffline: {
      readonly offline: Vec<ITuple<[AccountId32, PalletStakingExposure]>>;
    } & Struct;
    readonly type: 'HeartbeatReceived' | 'AllGood' | 'SomeOffline';
  }

  /** @name PalletImOnlineSr25519AppSr25519Public (122) */
  interface PalletImOnlineSr25519AppSr25519Public extends SpCoreSr25519Public {}

  /** @name SpCoreSr25519Public (123) */
  interface SpCoreSr25519Public extends U8aFixed {}

  /** @name PalletStakingExposure (126) */
  interface PalletStakingExposure extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly others: Vec<PalletStakingIndividualExposure>;
  }

  /** @name PalletStakingIndividualExposure (129) */
  interface PalletStakingIndividualExposure extends Struct {
    readonly who: AccountId32;
    readonly value: Compact<u128>;
  }

  /** @name PalletIdentityEvent (130) */
  interface PalletIdentityEvent extends Enum {
    readonly isIdentitySet: boolean;
    readonly asIdentitySet: {
      readonly who: AccountId32;
    } & Struct;
    readonly isIdentityCleared: boolean;
    readonly asIdentityCleared: {
      readonly who: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isIdentityKilled: boolean;
    readonly asIdentityKilled: {
      readonly who: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isJudgementRequested: boolean;
    readonly asJudgementRequested: {
      readonly who: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementUnrequested: boolean;
    readonly asJudgementUnrequested: {
      readonly who: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementGiven: boolean;
    readonly asJudgementGiven: {
      readonly target: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isRegistrarAdded: boolean;
    readonly asRegistrarAdded: {
      readonly registrarIndex: u32;
    } & Struct;
    readonly isSubIdentityAdded: boolean;
    readonly asSubIdentityAdded: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRemoved: boolean;
    readonly asSubIdentityRemoved: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRevoked: boolean;
    readonly asSubIdentityRevoked: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly type: 'IdentitySet' | 'IdentityCleared' | 'IdentityKilled' | 'JudgementRequested' | 'JudgementUnrequested' | 'JudgementGiven' | 'RegistrarAdded' | 'SubIdentityAdded' | 'SubIdentityRemoved' | 'SubIdentityRevoked';
  }

  /** @name PalletUtilityEvent (131) */
  interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: {
      readonly index: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isBatchCompleted: boolean;
    readonly isBatchCompletedWithErrors: boolean;
    readonly isItemCompleted: boolean;
    readonly isItemFailed: boolean;
    readonly asItemFailed: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isDispatchedAs: boolean;
    readonly asDispatchedAs: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'BatchInterrupted' | 'BatchCompleted' | 'BatchCompletedWithErrors' | 'ItemCompleted' | 'ItemFailed' | 'DispatchedAs';
  }

  /** @name PalletEth2LightClientEvent (132) */
  interface PalletEth2LightClientEvent extends Enum {
    readonly isInit: boolean;
    readonly asInit: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly headerInfo: EthTypesExecutionHeaderInfo;
    } & Struct;
    readonly isRegisterSubmitter: boolean;
    readonly asRegisterSubmitter: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly submitter: AccountId32;
    } & Struct;
    readonly isUnregisterSubmitter: boolean;
    readonly asUnregisterSubmitter: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly submitter: AccountId32;
    } & Struct;
    readonly isSubmitBeaconChainLightClientUpdate: boolean;
    readonly asSubmitBeaconChainLightClientUpdate: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly submitter: AccountId32;
      readonly beaconBlockHeader: EthTypesEth2BeaconBlockHeader;
    } & Struct;
    readonly isSubmitExecutionHeader: boolean;
    readonly asSubmitExecutionHeader: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly headerInfo: EthTypesExecutionHeaderInfo;
    } & Struct;
    readonly isUpdateTrustedSigner: boolean;
    readonly asUpdateTrustedSigner: {
      readonly trustedSigner: AccountId32;
    } & Struct;
    readonly type: 'Init' | 'RegisterSubmitter' | 'UnregisterSubmitter' | 'SubmitBeaconChainLightClientUpdate' | 'SubmitExecutionHeader' | 'UpdateTrustedSigner';
  }

  /** @name EthTypesExecutionHeaderInfo (133) */
  interface EthTypesExecutionHeaderInfo extends Struct {
    readonly parentHash: H256;
    readonly blockNumber: u64;
    readonly submitter: AccountId32;
  }

  /** @name EthTypesEth2BeaconBlockHeader (134) */
  interface EthTypesEth2BeaconBlockHeader extends Struct {
    readonly slot: u64;
    readonly proposerIndex: u64;
    readonly parentRoot: H256;
    readonly stateRoot: H256;
    readonly bodyRoot: H256;
  }

  /** @name FrameSystemPhase (136) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (139) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCall (142) */
  interface FrameSystemCall extends Enum {
    readonly isRemark: boolean;
    readonly asRemark: {
      readonly remark: Bytes;
    } & Struct;
    readonly isSetHeapPages: boolean;
    readonly asSetHeapPages: {
      readonly pages: u64;
    } & Struct;
    readonly isSetCode: boolean;
    readonly asSetCode: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetCodeWithoutChecks: boolean;
    readonly asSetCodeWithoutChecks: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetStorage: boolean;
    readonly asSetStorage: {
      readonly items: Vec<ITuple<[Bytes, Bytes]>>;
    } & Struct;
    readonly isKillStorage: boolean;
    readonly asKillStorage: {
      readonly keys_: Vec<Bytes>;
    } & Struct;
    readonly isKillPrefix: boolean;
    readonly asKillPrefix: {
      readonly prefix: Bytes;
      readonly subkeys: u32;
    } & Struct;
    readonly isRemarkWithEvent: boolean;
    readonly asRemarkWithEvent: {
      readonly remark: Bytes;
    } & Struct;
    readonly type: 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent';
  }

  /** @name FrameSystemLimitsBlockWeights (146) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (147) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (148) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /** @name FrameSystemLimitsBlockLength (150) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (151) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name SpWeightsRuntimeDbWeight (152) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (153) */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly stateVersion: u8;
  }

  /** @name FrameSystemError (157) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered';
  }

  /** @name PalletTimestampCall (158) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletSudoCall (159) */
  interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isSetKey: boolean;
    readonly asSetKey: {
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSudoAs: boolean;
    readonly asSudoAs: {
      readonly who: MultiAddress;
      readonly call: Call;
    } & Struct;
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs';
  }

  /** @name PalletBalancesCall (161) */
  interface PalletBalancesCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
      readonly newReserved: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: MultiAddress;
      readonly keepAlive: bool;
    } & Struct;
    readonly isForceUnreserve: boolean;
    readonly asForceUnreserve: {
      readonly who: MultiAddress;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Transfer' | 'SetBalance' | 'ForceTransfer' | 'TransferKeepAlive' | 'TransferAll' | 'ForceUnreserve';
  }

  /** @name PalletGrandpaCall (163) */
  interface PalletGrandpaCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpFinalityGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpFinalityGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isNoteStalled: boolean;
    readonly asNoteStalled: {
      readonly delay: u32;
      readonly bestFinalizedBlockNumber: u32;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'NoteStalled';
  }

  /** @name SpFinalityGrandpaEquivocationProof (164) */
  interface SpFinalityGrandpaEquivocationProof extends Struct {
    readonly setId: u64;
    readonly equivocation: SpFinalityGrandpaEquivocation;
  }

  /** @name SpFinalityGrandpaEquivocation (165) */
  interface SpFinalityGrandpaEquivocation extends Enum {
    readonly isPrevote: boolean;
    readonly asPrevote: FinalityGrandpaEquivocationPrevote;
    readonly isPrecommit: boolean;
    readonly asPrecommit: FinalityGrandpaEquivocationPrecommit;
    readonly type: 'Prevote' | 'Precommit';
  }

  /** @name FinalityGrandpaEquivocationPrevote (166) */
  interface FinalityGrandpaEquivocationPrevote extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpFinalityGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrevote, SpFinalityGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrevote, SpFinalityGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrevote (167) */
  interface FinalityGrandpaPrevote extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name SpFinalityGrandpaAppSignature (168) */
  interface SpFinalityGrandpaAppSignature extends SpCoreEd25519Signature {}

  /** @name SpCoreEd25519Signature (169) */
  interface SpCoreEd25519Signature extends U8aFixed {}

  /** @name FinalityGrandpaEquivocationPrecommit (172) */
  interface FinalityGrandpaEquivocationPrecommit extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpFinalityGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrecommit, SpFinalityGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrecommit, SpFinalityGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrecommit (173) */
  interface FinalityGrandpaPrecommit extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name SpSessionMembershipProof (175) */
  interface SpSessionMembershipProof extends Struct {
    readonly session: u32;
    readonly trieNodes: Vec<Bytes>;
    readonly validatorCount: u32;
  }

  /** @name PalletDkgMetadataCall (176) */
  interface PalletDkgMetadataCall extends Enum {
    readonly isSetSignatureThreshold: boolean;
    readonly asSetSignatureThreshold: {
      readonly newThreshold: u16;
    } & Struct;
    readonly isSetKeygenThreshold: boolean;
    readonly asSetKeygenThreshold: {
      readonly newThreshold: u16;
    } & Struct;
    readonly isSetRefreshDelay: boolean;
    readonly asSetRefreshDelay: {
      readonly newDelay: u8;
    } & Struct;
    readonly isSubmitPublicKey: boolean;
    readonly asSubmitPublicKey: {
      readonly keysAndSignatures: DkgRuntimePrimitivesAggregatedPublicKeys;
    } & Struct;
    readonly isSubmitNextPublicKey: boolean;
    readonly asSubmitNextPublicKey: {
      readonly keysAndSignatures: DkgRuntimePrimitivesAggregatedPublicKeys;
    } & Struct;
    readonly isSubmitPublicKeySignature: boolean;
    readonly asSubmitPublicKeySignature: {
      readonly signatureProposal: DkgRuntimePrimitivesProposalRefreshProposalSigned;
    } & Struct;
    readonly isSubmitMisbehaviourReports: boolean;
    readonly asSubmitMisbehaviourReports: {
      readonly reports: DkgRuntimePrimitivesAggregatedMisbehaviourReports;
    } & Struct;
    readonly isUnjail: boolean;
    readonly isForceUnjailKeygen: boolean;
    readonly asForceUnjailKeygen: {
      readonly authority: DkgRuntimePrimitivesCryptoPublic;
    } & Struct;
    readonly isForceUnjailSigning: boolean;
    readonly asForceUnjailSigning: {
      readonly authority: DkgRuntimePrimitivesCryptoPublic;
    } & Struct;
    readonly isForceChangeAuthorities: boolean;
    readonly isTriggerEmergencyKeygen: boolean;
    readonly type: 'SetSignatureThreshold' | 'SetKeygenThreshold' | 'SetRefreshDelay' | 'SubmitPublicKey' | 'SubmitNextPublicKey' | 'SubmitPublicKeySignature' | 'SubmitMisbehaviourReports' | 'Unjail' | 'ForceUnjailKeygen' | 'ForceUnjailSigning' | 'ForceChangeAuthorities' | 'TriggerEmergencyKeygen';
  }

  /** @name DkgRuntimePrimitivesAggregatedPublicKeys (177) */
  interface DkgRuntimePrimitivesAggregatedPublicKeys extends Struct {
    readonly keysAndSignatures: Vec<ITuple<[Bytes, Bytes]>>;
  }

  /** @name DkgRuntimePrimitivesProposalRefreshProposalSigned (178) */
  interface DkgRuntimePrimitivesProposalRefreshProposalSigned extends Struct {
    readonly nonce: u32;
    readonly signature: Bytes;
  }

  /** @name DkgRuntimePrimitivesAggregatedMisbehaviourReports (179) */
  interface DkgRuntimePrimitivesAggregatedMisbehaviourReports extends Struct {
    readonly misbehaviourType: DkgRuntimePrimitivesMisbehaviourType;
    readonly sessionId: u64;
    readonly offender: DkgRuntimePrimitivesCryptoPublic;
    readonly reporters: Vec<DkgRuntimePrimitivesCryptoPublic>;
    readonly signatures: Vec<Bytes>;
  }

  /** @name DkgRuntimePrimitivesCustomU32Getter (180) */
  type DkgRuntimePrimitivesCustomU32Getter = Null;

  /** @name PalletDkgProposalsCall (186) */
  interface PalletDkgProposalsCall extends Enum {
    readonly isSetThreshold: boolean;
    readonly asSetThreshold: {
      readonly threshold: u32;
    } & Struct;
    readonly isSetResource: boolean;
    readonly asSetResource: {
      readonly id: WebbProposalsHeaderResourceId;
      readonly method: Bytes;
    } & Struct;
    readonly isRemoveResource: boolean;
    readonly asRemoveResource: {
      readonly id: WebbProposalsHeaderResourceId;
    } & Struct;
    readonly isWhitelistChain: boolean;
    readonly asWhitelistChain: {
      readonly chainId: WebbProposalsHeaderTypedChainId;
    } & Struct;
    readonly isAddProposer: boolean;
    readonly asAddProposer: {
      readonly nativeAccount: AccountId32;
      readonly externalAccount: Bytes;
    } & Struct;
    readonly isRemoveProposer: boolean;
    readonly asRemoveProposer: {
      readonly v: AccountId32;
    } & Struct;
    readonly isAcknowledgeProposal: boolean;
    readonly asAcknowledgeProposal: {
      readonly nonce: u32;
      readonly srcChainId: WebbProposalsHeaderTypedChainId;
      readonly rId: WebbProposalsHeaderResourceId;
      readonly prop: Bytes;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly nonce: u32;
      readonly srcChainId: WebbProposalsHeaderTypedChainId;
      readonly rId: WebbProposalsHeaderResourceId;
      readonly prop: Bytes;
    } & Struct;
    readonly isEvalVoteState: boolean;
    readonly asEvalVoteState: {
      readonly nonce: u32;
      readonly srcChainId: WebbProposalsHeaderTypedChainId;
      readonly prop: Bytes;
    } & Struct;
    readonly type: 'SetThreshold' | 'SetResource' | 'RemoveResource' | 'WhitelistChain' | 'AddProposer' | 'RemoveProposer' | 'AcknowledgeProposal' | 'RejectProposal' | 'EvalVoteState';
  }

  /** @name WebbProposalsHeaderResourceId (187) */
  interface WebbProposalsHeaderResourceId extends U8aFixed {}

  /** @name PalletDkgProposalHandlerCall (189) */
  interface PalletDkgProposalHandlerCall extends Enum {
    readonly isSubmitSignedProposals: boolean;
    readonly asSubmitSignedProposals: {
      readonly props: Vec<WebbProposalsProposal>;
    } & Struct;
    readonly isForceSubmitUnsignedProposal: boolean;
    readonly asForceSubmitUnsignedProposal: {
      readonly prop: WebbProposalsProposal;
    } & Struct;
    readonly type: 'SubmitSignedProposals' | 'ForceSubmitUnsignedProposal';
  }

  /** @name WebbProposalsProposal (191) */
  interface WebbProposalsProposal extends Enum {
    readonly isSigned: boolean;
    readonly asSigned: {
      readonly kind: WebbProposalsProposalProposalKind;
      readonly data: Bytes;
      readonly signature: Bytes;
    } & Struct;
    readonly isUnsigned: boolean;
    readonly asUnsigned: {
      readonly kind: WebbProposalsProposalProposalKind;
      readonly data: Bytes;
    } & Struct;
    readonly type: 'Signed' | 'Unsigned';
  }

  /** @name PalletBridgeRegistryCall (193) */
  interface PalletBridgeRegistryCall extends Enum {
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly bridgeIndex: u32;
      readonly info: PalletBridgeRegistryBridgeInfo;
    } & Struct;
    readonly isForceResetIndices: boolean;
    readonly asForceResetIndices: {
      readonly resourceIds: Vec<WebbProposalsHeaderResourceId>;
      readonly bridgeIndex: u32;
    } & Struct;
    readonly type: 'SetMetadata' | 'ForceResetIndices';
  }

  /** @name PalletBridgeRegistryBridgeInfo (194) */
  interface PalletBridgeRegistryBridgeInfo extends Struct {
    readonly additional: Vec<ITuple<[Data, Data]>>;
    readonly display: Data;
  }

  /** @name PalletIndicesCall (229) */
  interface PalletIndicesCall extends Enum {
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly index: u32;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly new_: MultiAddress;
      readonly index: u32;
    } & Struct;
    readonly isFree: boolean;
    readonly asFree: {
      readonly index: u32;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly new_: MultiAddress;
      readonly index: u32;
      readonly freeze: bool;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly index: u32;
    } & Struct;
    readonly type: 'Claim' | 'Transfer' | 'Free' | 'ForceTransfer' | 'Freeze';
  }

  /** @name PalletDemocracyCall (230) */
  interface PalletDemocracyCall extends Enum {
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly proposal: FrameSupportPreimagesBounded;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSecond: boolean;
    readonly asSecond: {
      readonly proposal: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly refIndex: Compact<u32>;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isEmergencyCancel: boolean;
    readonly asEmergencyCancel: {
      readonly refIndex: u32;
    } & Struct;
    readonly isExternalPropose: boolean;
    readonly asExternalPropose: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeMajority: boolean;
    readonly asExternalProposeMajority: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeDefault: boolean;
    readonly asExternalProposeDefault: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isFastTrack: boolean;
    readonly asFastTrack: {
      readonly proposalHash: H256;
      readonly votingPeriod: u32;
      readonly delay: u32;
    } & Struct;
    readonly isVetoExternal: boolean;
    readonly asVetoExternal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isCancelReferendum: boolean;
    readonly asCancelReferendum: {
      readonly refIndex: Compact<u32>;
    } & Struct;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly to: MultiAddress;
      readonly conviction: PalletDemocracyConviction;
      readonly balance: u128;
    } & Struct;
    readonly isUndelegate: boolean;
    readonly isClearPublicProposals: boolean;
    readonly isUnlock: boolean;
    readonly asUnlock: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isRemoveVote: boolean;
    readonly asRemoveVote: {
      readonly index: u32;
    } & Struct;
    readonly isRemoveOtherVote: boolean;
    readonly asRemoveOtherVote: {
      readonly target: MultiAddress;
      readonly index: u32;
    } & Struct;
    readonly isBlacklist: boolean;
    readonly asBlacklist: {
      readonly proposalHash: H256;
      readonly maybeRefIndex: Option<u32>;
    } & Struct;
    readonly isCancelProposal: boolean;
    readonly asCancelProposal: {
      readonly propIndex: Compact<u32>;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly maybeHash: Option<H256>;
    } & Struct;
    readonly type: 'Propose' | 'Second' | 'Vote' | 'EmergencyCancel' | 'ExternalPropose' | 'ExternalProposeMajority' | 'ExternalProposeDefault' | 'FastTrack' | 'VetoExternal' | 'CancelReferendum' | 'Delegate' | 'Undelegate' | 'ClearPublicProposals' | 'Unlock' | 'RemoveVote' | 'RemoveOtherVote' | 'Blacklist' | 'CancelProposal' | 'SetMetadata';
  }

  /** @name FrameSupportPreimagesBounded (231) */
  interface FrameSupportPreimagesBounded extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: {
      readonly hash_: H256;
    } & Struct;
    readonly isInline: boolean;
    readonly asInline: Bytes;
    readonly isLookup: boolean;
    readonly asLookup: {
      readonly hash_: H256;
      readonly len: u32;
    } & Struct;
    readonly type: 'Legacy' | 'Inline' | 'Lookup';
  }

  /** @name PalletDemocracyConviction (233) */
  interface PalletDemocracyConviction extends Enum {
    readonly isNone: boolean;
    readonly isLocked1x: boolean;
    readonly isLocked2x: boolean;
    readonly isLocked3x: boolean;
    readonly isLocked4x: boolean;
    readonly isLocked5x: boolean;
    readonly isLocked6x: boolean;
    readonly type: 'None' | 'Locked1x' | 'Locked2x' | 'Locked3x' | 'Locked4x' | 'Locked5x' | 'Locked6x';
  }

  /** @name PalletCollectiveCall (236) */
  interface PalletCollectiveCall extends Enum {
    readonly isSetMembers: boolean;
    readonly asSetMembers: {
      readonly newMembers: Vec<AccountId32>;
      readonly prime: Option<AccountId32>;
      readonly oldCount: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly threshold: Compact<u32>;
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly proposal: H256;
      readonly index: Compact<u32>;
      readonly approve: bool;
    } & Struct;
    readonly isCloseOldWeight: boolean;
    readonly asCloseOldWeight: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: Compact<u64>;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isDisapproveProposal: boolean;
    readonly asDisapproveProposal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: SpWeightsWeightV2Weight;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly type: 'SetMembers' | 'Execute' | 'Propose' | 'Vote' | 'CloseOldWeight' | 'DisapproveProposal' | 'Close';
  }

  /** @name PalletVestingCall (239) */
  interface PalletVestingCall extends Enum {
    readonly isVest: boolean;
    readonly isVestOther: boolean;
    readonly asVestOther: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isVestedTransfer: boolean;
    readonly asVestedTransfer: {
      readonly target: MultiAddress;
      readonly schedule: PalletVestingVestingInfo;
    } & Struct;
    readonly isForceVestedTransfer: boolean;
    readonly asForceVestedTransfer: {
      readonly source: MultiAddress;
      readonly target: MultiAddress;
      readonly schedule: PalletVestingVestingInfo;
    } & Struct;
    readonly isMergeSchedules: boolean;
    readonly asMergeSchedules: {
      readonly schedule1Index: u32;
      readonly schedule2Index: u32;
    } & Struct;
    readonly type: 'Vest' | 'VestOther' | 'VestedTransfer' | 'ForceVestedTransfer' | 'MergeSchedules';
  }

  /** @name PalletVestingVestingInfo (240) */
  interface PalletVestingVestingInfo extends Struct {
    readonly locked: u128;
    readonly perBlock: u128;
    readonly startingBlock: u32;
  }

  /** @name PalletEcdsaClaimsCall (241) */
  interface PalletEcdsaClaimsCall extends Enum {
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly dest: AccountId32;
      readonly ethereumSignature: PalletEcdsaClaimsEcdsaSignature;
    } & Struct;
    readonly isMintClaim: boolean;
    readonly asMintClaim: {
      readonly who: PalletEcdsaClaimsEthereumAddress;
      readonly value: u128;
      readonly vestingSchedule: Option<ITuple<[u128, u128, u32]>>;
      readonly statement: Option<PalletEcdsaClaimsStatementKind>;
    } & Struct;
    readonly isClaimAttest: boolean;
    readonly asClaimAttest: {
      readonly dest: AccountId32;
      readonly ethereumSignature: PalletEcdsaClaimsEcdsaSignature;
      readonly statement: Bytes;
    } & Struct;
    readonly isAttest: boolean;
    readonly asAttest: {
      readonly statement: Bytes;
    } & Struct;
    readonly isMoveClaim: boolean;
    readonly asMoveClaim: {
      readonly old: PalletEcdsaClaimsEthereumAddress;
      readonly new_: PalletEcdsaClaimsEthereumAddress;
      readonly maybePreclaim: Option<AccountId32>;
    } & Struct;
    readonly isForceSetExpiryConfig: boolean;
    readonly asForceSetExpiryConfig: {
      readonly expiryBlock: u32;
      readonly dest: AccountId32;
    } & Struct;
    readonly type: 'Claim' | 'MintClaim' | 'ClaimAttest' | 'Attest' | 'MoveClaim' | 'ForceSetExpiryConfig';
  }

  /** @name PalletEcdsaClaimsEcdsaSignature (242) */
  interface PalletEcdsaClaimsEcdsaSignature extends U8aFixed {}

  /** @name PalletEcdsaClaimsStatementKind (247) */
  interface PalletEcdsaClaimsStatementKind extends Enum {
    readonly isRegular: boolean;
    readonly isSaft: boolean;
    readonly type: 'Regular' | 'Saft';
  }

  /** @name PalletElectionsPhragmenCall (248) */
  interface PalletElectionsPhragmenCall extends Enum {
    readonly isVote: boolean;
    readonly asVote: {
      readonly votes: Vec<AccountId32>;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isRemoveVoter: boolean;
    readonly isSubmitCandidacy: boolean;
    readonly asSubmitCandidacy: {
      readonly candidateCount: Compact<u32>;
    } & Struct;
    readonly isRenounceCandidacy: boolean;
    readonly asRenounceCandidacy: {
      readonly renouncing: PalletElectionsPhragmenRenouncing;
    } & Struct;
    readonly isRemoveMember: boolean;
    readonly asRemoveMember: {
      readonly who: MultiAddress;
      readonly slashBond: bool;
      readonly rerunElection: bool;
    } & Struct;
    readonly isCleanDefunctVoters: boolean;
    readonly asCleanDefunctVoters: {
      readonly numVoters: u32;
      readonly numDefunct: u32;
    } & Struct;
    readonly type: 'Vote' | 'RemoveVoter' | 'SubmitCandidacy' | 'RenounceCandidacy' | 'RemoveMember' | 'CleanDefunctVoters';
  }

  /** @name PalletElectionsPhragmenRenouncing (249) */
  interface PalletElectionsPhragmenRenouncing extends Enum {
    readonly isMember: boolean;
    readonly isRunnerUp: boolean;
    readonly isCandidate: boolean;
    readonly asCandidate: Compact<u32>;
    readonly type: 'Member' | 'RunnerUp' | 'Candidate';
  }

  /** @name PalletElectionProviderMultiPhaseCall (250) */
  interface PalletElectionProviderMultiPhaseCall extends Enum {
    readonly isSubmitUnsigned: boolean;
    readonly asSubmitUnsigned: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
      readonly witness: PalletElectionProviderMultiPhaseSolutionOrSnapshotSize;
    } & Struct;
    readonly isSetMinimumUntrustedScore: boolean;
    readonly asSetMinimumUntrustedScore: {
      readonly maybeNextScore: Option<SpNposElectionsElectionScore>;
    } & Struct;
    readonly isSetEmergencyElectionResult: boolean;
    readonly asSetEmergencyElectionResult: {
      readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>;
    } & Struct;
    readonly isSubmit: boolean;
    readonly asSubmit: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
    } & Struct;
    readonly isGovernanceFallback: boolean;
    readonly asGovernanceFallback: {
      readonly maybeMaxVoters: Option<u32>;
      readonly maybeMaxTargets: Option<u32>;
    } & Struct;
    readonly type: 'SubmitUnsigned' | 'SetMinimumUntrustedScore' | 'SetEmergencyElectionResult' | 'Submit' | 'GovernanceFallback';
  }

  /** @name PalletElectionProviderMultiPhaseRawSolution (251) */
  interface PalletElectionProviderMultiPhaseRawSolution extends Struct {
    readonly solution: TangleStandaloneRuntimeNposSolution16;
    readonly score: SpNposElectionsElectionScore;
    readonly round: u32;
  }

  /** @name TangleStandaloneRuntimeNposSolution16 (252) */
  interface TangleStandaloneRuntimeNposSolution16 extends Struct {
    readonly votes1: Vec<ITuple<[Compact<u32>, Compact<u16>]>>;
    readonly votes2: Vec<ITuple<[Compact<u32>, ITuple<[Compact<u16>, Compact<PerU16>]>, Compact<u16>]>>;
    readonly votes3: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes4: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes5: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes6: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes7: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes8: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes9: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes10: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes11: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes12: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes13: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes14: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes15: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes16: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
  }

  /** @name PalletElectionProviderMultiPhaseSolutionOrSnapshotSize (303) */
  interface PalletElectionProviderMultiPhaseSolutionOrSnapshotSize extends Struct {
    readonly voters: Compact<u32>;
    readonly targets: Compact<u32>;
  }

  /** @name SpNposElectionsSupport (307) */
  interface SpNposElectionsSupport extends Struct {
    readonly total: u128;
    readonly voters: Vec<ITuple<[AccountId32, u128]>>;
  }

  /** @name PalletStakingPalletCall (308) */
  interface PalletStakingPalletCall extends Enum {
    readonly isBond: boolean;
    readonly asBond: {
      readonly controller: MultiAddress;
      readonly value: Compact<u128>;
      readonly payee: PalletStakingRewardDestination;
    } & Struct;
    readonly isBondExtra: boolean;
    readonly asBondExtra: {
      readonly maxAdditional: Compact<u128>;
    } & Struct;
    readonly isUnbond: boolean;
    readonly asUnbond: {
      readonly value: Compact<u128>;
    } & Struct;
    readonly isWithdrawUnbonded: boolean;
    readonly asWithdrawUnbonded: {
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isValidate: boolean;
    readonly asValidate: {
      readonly prefs: PalletStakingValidatorPrefs;
    } & Struct;
    readonly isNominate: boolean;
    readonly asNominate: {
      readonly targets: Vec<MultiAddress>;
    } & Struct;
    readonly isChill: boolean;
    readonly isSetPayee: boolean;
    readonly asSetPayee: {
      readonly payee: PalletStakingRewardDestination;
    } & Struct;
    readonly isSetController: boolean;
    readonly asSetController: {
      readonly controller: MultiAddress;
    } & Struct;
    readonly isSetValidatorCount: boolean;
    readonly asSetValidatorCount: {
      readonly new_: Compact<u32>;
    } & Struct;
    readonly isIncreaseValidatorCount: boolean;
    readonly asIncreaseValidatorCount: {
      readonly additional: Compact<u32>;
    } & Struct;
    readonly isScaleValidatorCount: boolean;
    readonly asScaleValidatorCount: {
      readonly factor: Percent;
    } & Struct;
    readonly isForceNoEras: boolean;
    readonly isForceNewEra: boolean;
    readonly isSetInvulnerables: boolean;
    readonly asSetInvulnerables: {
      readonly invulnerables: Vec<AccountId32>;
    } & Struct;
    readonly isForceUnstake: boolean;
    readonly asForceUnstake: {
      readonly stash: AccountId32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isForceNewEraAlways: boolean;
    readonly isCancelDeferredSlash: boolean;
    readonly asCancelDeferredSlash: {
      readonly era: u32;
      readonly slashIndices: Vec<u32>;
    } & Struct;
    readonly isPayoutStakers: boolean;
    readonly asPayoutStakers: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
    } & Struct;
    readonly isRebond: boolean;
    readonly asRebond: {
      readonly value: Compact<u128>;
    } & Struct;
    readonly isReapStash: boolean;
    readonly asReapStash: {
      readonly stash: AccountId32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isKick: boolean;
    readonly asKick: {
      readonly who: Vec<MultiAddress>;
    } & Struct;
    readonly isSetStakingConfigs: boolean;
    readonly asSetStakingConfigs: {
      readonly minNominatorBond: PalletStakingPalletConfigOpU128;
      readonly minValidatorBond: PalletStakingPalletConfigOpU128;
      readonly maxNominatorCount: PalletStakingPalletConfigOpU32;
      readonly maxValidatorCount: PalletStakingPalletConfigOpU32;
      readonly chillThreshold: PalletStakingPalletConfigOpPercent;
      readonly minCommission: PalletStakingPalletConfigOpPerbill;
    } & Struct;
    readonly isChillOther: boolean;
    readonly asChillOther: {
      readonly controller: AccountId32;
    } & Struct;
    readonly isForceApplyMinCommission: boolean;
    readonly asForceApplyMinCommission: {
      readonly validatorStash: AccountId32;
    } & Struct;
    readonly isSetMinCommission: boolean;
    readonly asSetMinCommission: {
      readonly new_: Perbill;
    } & Struct;
    readonly type: 'Bond' | 'BondExtra' | 'Unbond' | 'WithdrawUnbonded' | 'Validate' | 'Nominate' | 'Chill' | 'SetPayee' | 'SetController' | 'SetValidatorCount' | 'IncreaseValidatorCount' | 'ScaleValidatorCount' | 'ForceNoEras' | 'ForceNewEra' | 'SetInvulnerables' | 'ForceUnstake' | 'ForceNewEraAlways' | 'CancelDeferredSlash' | 'PayoutStakers' | 'Rebond' | 'ReapStash' | 'Kick' | 'SetStakingConfigs' | 'ChillOther' | 'ForceApplyMinCommission' | 'SetMinCommission';
  }

  /** @name PalletStakingRewardDestination (309) */
  interface PalletStakingRewardDestination extends Enum {
    readonly isStaked: boolean;
    readonly isStash: boolean;
    readonly isController: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Staked' | 'Stash' | 'Controller' | 'Account' | 'None';
  }

  /** @name PalletStakingPalletConfigOpU128 (312) */
  interface PalletStakingPalletConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpU32 (313) */
  interface PalletStakingPalletConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPercent (314) */
  interface PalletStakingPalletConfigOpPercent extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Percent;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPerbill (315) */
  interface PalletStakingPalletConfigOpPerbill extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Perbill;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletSessionCall (316) */
  interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: TangleStandaloneRuntimeOpaqueSessionKeys;
      readonly proof: Bytes;
    } & Struct;
    readonly isPurgeKeys: boolean;
    readonly type: 'SetKeys' | 'PurgeKeys';
  }

  /** @name TangleStandaloneRuntimeOpaqueSessionKeys (317) */
  interface TangleStandaloneRuntimeOpaqueSessionKeys extends Struct {
    readonly aura: SpConsensusAuraSr25519AppSr25519Public;
    readonly grandpa: SpFinalityGrandpaAppPublic;
    readonly imOnline: PalletImOnlineSr25519AppSr25519Public;
    readonly dkg: DkgRuntimePrimitivesCryptoPublic;
  }

  /** @name SpConsensusAuraSr25519AppSr25519Public (318) */
  interface SpConsensusAuraSr25519AppSr25519Public extends SpCoreSr25519Public {}

  /** @name PalletTreasuryCall (319) */
  interface PalletTreasuryCall extends Enum {
    readonly isProposeSpend: boolean;
    readonly asProposeSpend: {
      readonly value: Compact<u128>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isApproveProposal: boolean;
    readonly asApproveProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isSpend: boolean;
    readonly asSpend: {
      readonly amount: Compact<u128>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isRemoveApproval: boolean;
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly type: 'ProposeSpend' | 'RejectProposal' | 'ApproveProposal' | 'Spend' | 'RemoveApproval';
  }

  /** @name PalletBountiesCall (320) */
  interface PalletBountiesCall extends Enum {
    readonly isProposeBounty: boolean;
    readonly asProposeBounty: {
      readonly value: Compact<u128>;
      readonly description: Bytes;
    } & Struct;
    readonly isApproveBounty: boolean;
    readonly asApproveBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isProposeCurator: boolean;
    readonly asProposeCurator: {
      readonly bountyId: Compact<u32>;
      readonly curator: MultiAddress;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isUnassignCurator: boolean;
    readonly asUnassignCurator: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isAcceptCurator: boolean;
    readonly asAcceptCurator: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isAwardBounty: boolean;
    readonly asAwardBounty: {
      readonly bountyId: Compact<u32>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isClaimBounty: boolean;
    readonly asClaimBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isCloseBounty: boolean;
    readonly asCloseBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isExtendBountyExpiry: boolean;
    readonly asExtendBountyExpiry: {
      readonly bountyId: Compact<u32>;
      readonly remark: Bytes;
    } & Struct;
    readonly type: 'ProposeBounty' | 'ApproveBounty' | 'ProposeCurator' | 'UnassignCurator' | 'AcceptCurator' | 'AwardBounty' | 'ClaimBounty' | 'CloseBounty' | 'ExtendBountyExpiry';
  }

  /** @name PalletChildBountiesCall (321) */
  interface PalletChildBountiesCall extends Enum {
    readonly isAddChildBounty: boolean;
    readonly asAddChildBounty: {
      readonly parentBountyId: Compact<u32>;
      readonly value: Compact<u128>;
      readonly description: Bytes;
    } & Struct;
    readonly isProposeCurator: boolean;
    readonly asProposeCurator: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
      readonly curator: MultiAddress;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isAcceptCurator: boolean;
    readonly asAcceptCurator: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
    } & Struct;
    readonly isUnassignCurator: boolean;
    readonly asUnassignCurator: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
    } & Struct;
    readonly isAwardChildBounty: boolean;
    readonly asAwardChildBounty: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isClaimChildBounty: boolean;
    readonly asClaimChildBounty: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
    } & Struct;
    readonly isCloseChildBounty: boolean;
    readonly asCloseChildBounty: {
      readonly parentBountyId: Compact<u32>;
      readonly childBountyId: Compact<u32>;
    } & Struct;
    readonly type: 'AddChildBounty' | 'ProposeCurator' | 'AcceptCurator' | 'UnassignCurator' | 'AwardChildBounty' | 'ClaimChildBounty' | 'CloseChildBounty';
  }

  /** @name PalletBagsListCall (322) */
  interface PalletBagsListCall extends Enum {
    readonly isRebag: boolean;
    readonly asRebag: {
      readonly dislocated: MultiAddress;
    } & Struct;
    readonly isPutInFrontOf: boolean;
    readonly asPutInFrontOf: {
      readonly lighter: MultiAddress;
    } & Struct;
    readonly type: 'Rebag' | 'PutInFrontOf';
  }

  /** @name PalletNominationPoolsCall (323) */
  interface PalletNominationPoolsCall extends Enum {
    readonly isJoin: boolean;
    readonly asJoin: {
      readonly amount: Compact<u128>;
      readonly poolId: u32;
    } & Struct;
    readonly isBondExtra: boolean;
    readonly asBondExtra: {
      readonly extra: PalletNominationPoolsBondExtra;
    } & Struct;
    readonly isClaimPayout: boolean;
    readonly isUnbond: boolean;
    readonly asUnbond: {
      readonly memberAccount: MultiAddress;
      readonly unbondingPoints: Compact<u128>;
    } & Struct;
    readonly isPoolWithdrawUnbonded: boolean;
    readonly asPoolWithdrawUnbonded: {
      readonly poolId: u32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isWithdrawUnbonded: boolean;
    readonly asWithdrawUnbonded: {
      readonly memberAccount: MultiAddress;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly amount: Compact<u128>;
      readonly root: MultiAddress;
      readonly nominator: MultiAddress;
      readonly bouncer: MultiAddress;
    } & Struct;
    readonly isCreateWithPoolId: boolean;
    readonly asCreateWithPoolId: {
      readonly amount: Compact<u128>;
      readonly root: MultiAddress;
      readonly nominator: MultiAddress;
      readonly bouncer: MultiAddress;
      readonly poolId: u32;
    } & Struct;
    readonly isNominate: boolean;
    readonly asNominate: {
      readonly poolId: u32;
      readonly validators: Vec<AccountId32>;
    } & Struct;
    readonly isSetState: boolean;
    readonly asSetState: {
      readonly poolId: u32;
      readonly state: PalletNominationPoolsPoolState;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly poolId: u32;
      readonly metadata: Bytes;
    } & Struct;
    readonly isSetConfigs: boolean;
    readonly asSetConfigs: {
      readonly minJoinBond: PalletNominationPoolsConfigOpU128;
      readonly minCreateBond: PalletNominationPoolsConfigOpU128;
      readonly maxPools: PalletNominationPoolsConfigOpU32;
      readonly maxMembers: PalletNominationPoolsConfigOpU32;
      readonly maxMembersPerPool: PalletNominationPoolsConfigOpU32;
    } & Struct;
    readonly isUpdateRoles: boolean;
    readonly asUpdateRoles: {
      readonly poolId: u32;
      readonly newRoot: PalletNominationPoolsConfigOpAccountId32;
      readonly newNominator: PalletNominationPoolsConfigOpAccountId32;
      readonly newBouncer: PalletNominationPoolsConfigOpAccountId32;
    } & Struct;
    readonly isChill: boolean;
    readonly asChill: {
      readonly poolId: u32;
    } & Struct;
    readonly isBondExtraOther: boolean;
    readonly asBondExtraOther: {
      readonly member: MultiAddress;
      readonly extra: PalletNominationPoolsBondExtra;
    } & Struct;
    readonly isSetClaimPermission: boolean;
    readonly asSetClaimPermission: {
      readonly permission: PalletNominationPoolsClaimPermission;
    } & Struct;
    readonly isClaimPayoutOther: boolean;
    readonly asClaimPayoutOther: {
      readonly other: AccountId32;
    } & Struct;
    readonly type: 'Join' | 'BondExtra' | 'ClaimPayout' | 'Unbond' | 'PoolWithdrawUnbonded' | 'WithdrawUnbonded' | 'Create' | 'CreateWithPoolId' | 'Nominate' | 'SetState' | 'SetMetadata' | 'SetConfigs' | 'UpdateRoles' | 'Chill' | 'BondExtraOther' | 'SetClaimPermission' | 'ClaimPayoutOther';
  }

  /** @name PalletNominationPoolsBondExtra (324) */
  interface PalletNominationPoolsBondExtra extends Enum {
    readonly isFreeBalance: boolean;
    readonly asFreeBalance: u128;
    readonly isRewards: boolean;
    readonly type: 'FreeBalance' | 'Rewards';
  }

  /** @name PalletNominationPoolsConfigOpU128 (325) */
  interface PalletNominationPoolsConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsConfigOpU32 (326) */
  interface PalletNominationPoolsConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsConfigOpAccountId32 (327) */
  interface PalletNominationPoolsConfigOpAccountId32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: AccountId32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsClaimPermission (328) */
  interface PalletNominationPoolsClaimPermission extends Enum {
    readonly isPermissioned: boolean;
    readonly isPermissionlessCompound: boolean;
    readonly isPermissionlessWithdraw: boolean;
    readonly isPermissionlessAll: boolean;
    readonly type: 'Permissioned' | 'PermissionlessCompound' | 'PermissionlessWithdraw' | 'PermissionlessAll';
  }

  /** @name PalletSchedulerCall (329) */
  interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: U8aFixed;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: U8aFixed;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter';
  }

  /** @name PalletPreimageCall (331) */
  interface PalletPreimageCall extends Enum {
    readonly isNotePreimage: boolean;
    readonly asNotePreimage: {
      readonly bytes: Bytes;
    } & Struct;
    readonly isUnnotePreimage: boolean;
    readonly asUnnotePreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequestPreimage: boolean;
    readonly asRequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isUnrequestPreimage: boolean;
    readonly asUnrequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'NotePreimage' | 'UnnotePreimage' | 'RequestPreimage' | 'UnrequestPreimage';
  }

  /** @name PalletHasherCall (332) */
  interface PalletHasherCall extends Enum {
    readonly isForceSetParameters: boolean;
    readonly asForceSetParameters: {
      readonly parameters: Bytes;
    } & Struct;
    readonly type: 'ForceSetParameters';
  }

  /** @name PalletAssetRegistryCall (334) */
  interface PalletAssetRegistryCall extends Enum {
    readonly isRegister: boolean;
    readonly asRegister: {
      readonly name: Bytes;
      readonly assetType: PalletAssetRegistryAssetType;
      readonly existentialDeposit: u128;
    } & Struct;
    readonly isUpdate: boolean;
    readonly asUpdate: {
      readonly assetId: u32;
      readonly boundedName: Bytes;
      readonly assetType: PalletAssetRegistryAssetType;
      readonly existentialDeposit: Option<u128>;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly assetId: u32;
      readonly symbol: Bytes;
      readonly decimals: u8;
    } & Struct;
    readonly isSetLocation: boolean;
    readonly asSetLocation: {
      readonly assetId: u32;
      readonly location: Null;
    } & Struct;
    readonly isAddAssetToPool: boolean;
    readonly asAddAssetToPool: {
      readonly pool: Bytes;
      readonly assetId: u32;
    } & Struct;
    readonly isDeleteAssetFromPool: boolean;
    readonly asDeleteAssetFromPool: {
      readonly pool: Bytes;
      readonly assetId: u32;
    } & Struct;
    readonly type: 'Register' | 'Update' | 'SetMetadata' | 'SetLocation' | 'AddAssetToPool' | 'DeleteAssetFromPool';
  }

  /** @name OrmlCurrenciesModuleCall (336) */
  interface OrmlCurrenciesModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly currencyId: u32;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransferNativeCurrency: boolean;
    readonly asTransferNativeCurrency: {
      readonly dest: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isUpdateBalance: boolean;
    readonly asUpdateBalance: {
      readonly who: MultiAddress;
      readonly currencyId: u32;
      readonly amount: i128;
    } & Struct;
    readonly type: 'Transfer' | 'TransferNativeCurrency' | 'UpdateBalance';
  }

  /** @name OrmlTokensModuleCall (337) */
  interface OrmlTokensModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly currencyId: u32;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: MultiAddress;
      readonly currencyId: u32;
      readonly keepAlive: bool;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress;
      readonly currencyId: u32;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly currencyId: u32;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: MultiAddress;
      readonly currencyId: u32;
      readonly newFree: Compact<u128>;
      readonly newReserved: Compact<u128>;
    } & Struct;
    readonly type: 'Transfer' | 'TransferAll' | 'TransferKeepAlive' | 'ForceTransfer' | 'SetBalance';
  }

  /** @name PalletTokenWrapperCall (338) */
  interface PalletTokenWrapperCall extends Enum {
    readonly isSetWrappingFee: boolean;
    readonly asSetWrappingFee: {
      readonly fee: u128;
      readonly intoPoolShareId: u32;
      readonly nonce: u32;
    } & Struct;
    readonly isSetFeeRecipient: boolean;
    readonly asSetFeeRecipient: {
      readonly poolShareId: u32;
      readonly feeRecipient: AccountId32;
      readonly nonce: u32;
    } & Struct;
    readonly isWrap: boolean;
    readonly asWrap: {
      readonly fromAssetId: u32;
      readonly intoPoolShareId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
    } & Struct;
    readonly isUnwrap: boolean;
    readonly asUnwrap: {
      readonly fromPoolShareId: u32;
      readonly intoAssetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
    } & Struct;
    readonly isRescueTokens: boolean;
    readonly asRescueTokens: {
      readonly fromPoolShareId: u32;
      readonly assetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
      readonly nonce: u32;
    } & Struct;
    readonly type: 'SetWrappingFee' | 'SetFeeRecipient' | 'Wrap' | 'Unwrap' | 'RescueTokens';
  }

  /** @name PalletVerifierCall (339) */
  interface PalletVerifierCall extends Enum {
    readonly isForceSetParameters: boolean;
    readonly asForceSetParameters: {
      readonly parameters: Bytes;
    } & Struct;
    readonly type: 'ForceSetParameters';
  }

  /** @name PalletMtCall (341) */
  interface PalletMtCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly depth: u8;
    } & Struct;
    readonly isInsert: boolean;
    readonly asInsert: {
      readonly treeId: u32;
      readonly leaf: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    } & Struct;
    readonly isForceSetDefaultHashes: boolean;
    readonly asForceSetDefaultHashes: {
      readonly defaultHashes: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
    } & Struct;
    readonly type: 'Create' | 'Insert' | 'ForceSetDefaultHashes';
  }

  /** @name PalletLinkableTreeCall (343) */
  interface PalletLinkableTreeCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly maxEdges: u32;
      readonly depth: u8;
    } & Struct;
    readonly type: 'Create';
  }

  /** @name PalletMixerCall (344) */
  interface PalletMixerCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly depositSize: u128;
      readonly depth: u8;
      readonly asset: u32;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly treeId: u32;
      readonly leaf: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly id: u32;
      readonly proofBytes: Bytes;
      readonly root: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
      readonly nullifierHash: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
      readonly recipient: AccountId32;
      readonly relayer: AccountId32;
      readonly fee: u128;
      readonly refund: u128;
    } & Struct;
    readonly type: 'Create' | 'Deposit' | 'Withdraw';
  }

  /** @name PalletSignatureBridgeCall (345) */
  interface PalletSignatureBridgeCall extends Enum {
    readonly isSetMaintainer: boolean;
    readonly asSetMaintainer: {
      readonly message: Bytes;
      readonly signature: Bytes;
    } & Struct;
    readonly isForceSetMaintainer: boolean;
    readonly asForceSetMaintainer: {
      readonly nonce: u32;
      readonly newMaintainer: Bytes;
    } & Struct;
    readonly isSetResource: boolean;
    readonly asSetResource: {
      readonly id: WebbProposalsHeaderResourceId;
    } & Struct;
    readonly isRemoveResource: boolean;
    readonly asRemoveResource: {
      readonly id: WebbProposalsHeaderResourceId;
    } & Struct;
    readonly isWhitelistChain: boolean;
    readonly asWhitelistChain: {
      readonly id: u64;
    } & Struct;
    readonly isSetResourceWithSignature: boolean;
    readonly asSetResourceWithSignature: {
      readonly srcId: u64;
      readonly proposalData: Bytes;
      readonly signature: Bytes;
    } & Struct;
    readonly isExecuteProposal: boolean;
    readonly asExecuteProposal: {
      readonly srcId: u64;
      readonly proposalData: Bytes;
      readonly signature: Bytes;
    } & Struct;
    readonly type: 'SetMaintainer' | 'ForceSetMaintainer' | 'SetResource' | 'RemoveResource' | 'WhitelistChain' | 'SetResourceWithSignature' | 'ExecuteProposal';
  }

  /** @name PalletVanchorCall (347) */
  interface PalletVanchorCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly maxEdges: u32;
      readonly depth: u8;
      readonly asset: u32;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly id: u32;
      readonly proofData: WebbPrimitivesVanchorProofData;
      readonly extData: WebbPrimitivesVanchorExtData;
    } & Struct;
    readonly isRegisterAndTransact: boolean;
    readonly asRegisterAndTransact: {
      readonly owner: AccountId32;
      readonly publicKey: Bytes;
      readonly id: u32;
      readonly proofData: WebbPrimitivesVanchorProofData;
      readonly extData: WebbPrimitivesVanchorExtData;
    } & Struct;
    readonly isSetMaxDepositAmount: boolean;
    readonly asSetMaxDepositAmount: {
      readonly maxDepositAmount: u128;
      readonly nonce: u32;
    } & Struct;
    readonly isSetMinWithdrawAmount: boolean;
    readonly asSetMinWithdrawAmount: {
      readonly minWithdrawAmount: u128;
      readonly nonce: u32;
    } & Struct;
    readonly type: 'Create' | 'Transact' | 'RegisterAndTransact' | 'SetMaxDepositAmount' | 'SetMinWithdrawAmount';
  }

  /** @name WebbPrimitivesVanchorProofData (348) */
  interface WebbPrimitivesVanchorProofData extends Struct {
    readonly proof: Bytes;
    readonly publicAmount: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    readonly roots: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
    readonly inputNullifiers: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
    readonly outputCommitments: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
    readonly extDataHash: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
  }

  /** @name WebbPrimitivesVanchorExtData (349) */
  interface WebbPrimitivesVanchorExtData extends Struct {
    readonly recipient: AccountId32;
    readonly relayer: AccountId32;
    readonly extAmount: i128;
    readonly fee: u128;
    readonly refund: u128;
    readonly token: u32;
    readonly encryptedOutput1: Bytes;
    readonly encryptedOutput2: Bytes;
  }

  /** @name PalletVanchorHandlerCall (350) */
  interface PalletVanchorHandlerCall extends Enum {
    readonly isExecuteVanchorCreateProposal: boolean;
    readonly asExecuteVanchorCreateProposal: {
      readonly srcChainId: u64;
      readonly rId: WebbProposalsHeaderResourceId;
      readonly maxEdges: u32;
      readonly treeDepth: u8;
      readonly asset: u32;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteVanchorUpdateProposal: boolean;
    readonly asExecuteVanchorUpdateProposal: {
      readonly rId: WebbProposalsHeaderResourceId;
      readonly merkleRoot: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
      readonly srcResourceId: WebbProposalsHeaderResourceId;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteSetResourceProposal: boolean;
    readonly asExecuteSetResourceProposal: {
      readonly rId: WebbProposalsHeaderResourceId;
    } & Struct;
    readonly isExecuteSetMaxDepositLimitProposal: boolean;
    readonly asExecuteSetMaxDepositLimitProposal: {
      readonly maxDepositLimit: u128;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteSetMinWithdrawalLimitProposal: boolean;
    readonly asExecuteSetMinWithdrawalLimitProposal: {
      readonly minWithdrawLimit: u128;
      readonly nonce: u32;
    } & Struct;
    readonly type: 'ExecuteVanchorCreateProposal' | 'ExecuteVanchorUpdateProposal' | 'ExecuteSetResourceProposal' | 'ExecuteSetMaxDepositLimitProposal' | 'ExecuteSetMinWithdrawalLimitProposal';
  }

  /** @name PalletTokenWrapperHandlerCall (351) */
  interface PalletTokenWrapperHandlerCall extends Enum {
    readonly isExecuteWrappingFeeProposal: boolean;
    readonly asExecuteWrappingFeeProposal: {
      readonly wrappingFeePercent: u128;
      readonly intoPoolShareId: u32;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteAddTokenToPoolShare: boolean;
    readonly asExecuteAddTokenToPoolShare: {
      readonly name: Bytes;
      readonly assetId: u32;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteRemoveTokenFromPoolShare: boolean;
    readonly asExecuteRemoveTokenFromPoolShare: {
      readonly name: Bytes;
      readonly assetId: u32;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteSetFeeRecipientProposal: boolean;
    readonly asExecuteSetFeeRecipientProposal: {
      readonly poolShareId: u32;
      readonly feeRecipient: AccountId32;
      readonly nonce: u32;
    } & Struct;
    readonly isExecuteRescueTokensProposal: boolean;
    readonly asExecuteRescueTokensProposal: {
      readonly poolShareId: u32;
      readonly assetId: u32;
      readonly amount: u128;
      readonly recipient: AccountId32;
      readonly nonce: u32;
    } & Struct;
    readonly type: 'ExecuteWrappingFeeProposal' | 'ExecuteAddTokenToPoolShare' | 'ExecuteRemoveTokenFromPoolShare' | 'ExecuteSetFeeRecipientProposal' | 'ExecuteRescueTokensProposal';
  }

  /** @name PalletKeyStorageCall (352) */
  interface PalletKeyStorageCall extends Enum {
    readonly isRegister: boolean;
    readonly asRegister: {
      readonly owner: AccountId32;
      readonly publicKey: Bytes;
    } & Struct;
    readonly type: 'Register';
  }

  /** @name PalletVanchorVerifierCall (353) */
  interface PalletVanchorVerifierCall extends Enum {
    readonly isForceSetParameters: boolean;
    readonly asForceSetParameters: {
      readonly configuration: ITuple<[u8, u8]>;
      readonly parameters: Bytes;
    } & Struct;
    readonly type: 'ForceSetParameters';
  }

  /** @name PalletTransactionPauseModuleCall (355) */
  interface PalletTransactionPauseModuleCall extends Enum {
    readonly isPauseTransaction: boolean;
    readonly asPauseTransaction: {
      readonly palletName: Bytes;
      readonly functionName: Bytes;
    } & Struct;
    readonly isUnpauseTransaction: boolean;
    readonly asUnpauseTransaction: {
      readonly palletName: Bytes;
      readonly functionName: Bytes;
    } & Struct;
    readonly type: 'PauseTransaction' | 'UnpauseTransaction';
  }

  /** @name PalletImOnlineCall (356) */
  interface PalletImOnlineCall extends Enum {
    readonly isHeartbeat: boolean;
    readonly asHeartbeat: {
      readonly heartbeat: PalletImOnlineHeartbeat;
      readonly signature: PalletImOnlineSr25519AppSr25519Signature;
    } & Struct;
    readonly type: 'Heartbeat';
  }

  /** @name PalletImOnlineHeartbeat (357) */
  interface PalletImOnlineHeartbeat extends Struct {
    readonly blockNumber: u32;
    readonly networkState: SpCoreOffchainOpaqueNetworkState;
    readonly sessionIndex: u32;
    readonly authorityIndex: u32;
    readonly validatorsLen: u32;
  }

  /** @name SpCoreOffchainOpaqueNetworkState (358) */
  interface SpCoreOffchainOpaqueNetworkState extends Struct {
    readonly peerId: OpaquePeerId;
    readonly externalAddresses: Vec<OpaqueMultiaddr>;
  }

  /** @name PalletImOnlineSr25519AppSr25519Signature (362) */
  interface PalletImOnlineSr25519AppSr25519Signature extends SpCoreSr25519Signature {}

  /** @name SpCoreSr25519Signature (363) */
  interface SpCoreSr25519Signature extends U8aFixed {}

  /** @name PalletIdentityCall (364) */
  interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean;
    readonly asAddRegistrar: {
      readonly account: MultiAddress;
    } & Struct;
    readonly isSetIdentity: boolean;
    readonly asSetIdentity: {
      readonly info: PalletIdentityIdentityInfo;
    } & Struct;
    readonly isSetSubs: boolean;
    readonly asSetSubs: {
      readonly subs: Vec<ITuple<[AccountId32, Data]>>;
    } & Struct;
    readonly isClearIdentity: boolean;
    readonly isRequestJudgement: boolean;
    readonly asRequestJudgement: {
      readonly regIndex: Compact<u32>;
      readonly maxFee: Compact<u128>;
    } & Struct;
    readonly isCancelRequest: boolean;
    readonly asCancelRequest: {
      readonly regIndex: u32;
    } & Struct;
    readonly isSetFee: boolean;
    readonly asSetFee: {
      readonly index: Compact<u32>;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isSetAccountId: boolean;
    readonly asSetAccountId: {
      readonly index: Compact<u32>;
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSetFields: boolean;
    readonly asSetFields: {
      readonly index: Compact<u32>;
      readonly fields: PalletIdentityBitFlags;
    } & Struct;
    readonly isProvideJudgement: boolean;
    readonly asProvideJudgement: {
      readonly regIndex: Compact<u32>;
      readonly target: MultiAddress;
      readonly judgement: PalletIdentityJudgement;
      readonly identity: H256;
    } & Struct;
    readonly isKillIdentity: boolean;
    readonly asKillIdentity: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isAddSub: boolean;
    readonly asAddSub: {
      readonly sub: MultiAddress;
      readonly data: Data;
    } & Struct;
    readonly isRenameSub: boolean;
    readonly asRenameSub: {
      readonly sub: MultiAddress;
      readonly data: Data;
    } & Struct;
    readonly isRemoveSub: boolean;
    readonly asRemoveSub: {
      readonly sub: MultiAddress;
    } & Struct;
    readonly isQuitSub: boolean;
    readonly type: 'AddRegistrar' | 'SetIdentity' | 'SetSubs' | 'ClearIdentity' | 'RequestJudgement' | 'CancelRequest' | 'SetFee' | 'SetAccountId' | 'SetFields' | 'ProvideJudgement' | 'KillIdentity' | 'AddSub' | 'RenameSub' | 'RemoveSub' | 'QuitSub';
  }

  /** @name PalletIdentityIdentityInfo (365) */
  interface PalletIdentityIdentityInfo extends Struct {
    readonly additional: Vec<ITuple<[Data, Data]>>;
    readonly display: Data;
    readonly legal: Data;
    readonly web: Data;
    readonly riot: Data;
    readonly email: Data;
    readonly pgpFingerprint: Option<U8aFixed>;
    readonly image: Data;
    readonly twitter: Data;
  }

  /** @name PalletIdentityBitFlags (372) */
  interface PalletIdentityBitFlags extends Set {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
  }

  /** @name PalletIdentityIdentityField (373) */
  interface PalletIdentityIdentityField extends Enum {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
    readonly type: 'Display' | 'Legal' | 'Web' | 'Riot' | 'Email' | 'PgpFingerprint' | 'Image' | 'Twitter';
  }

  /** @name PalletIdentityJudgement (374) */
  interface PalletIdentityJudgement extends Enum {
    readonly isUnknown: boolean;
    readonly isFeePaid: boolean;
    readonly asFeePaid: u128;
    readonly isReasonable: boolean;
    readonly isKnownGood: boolean;
    readonly isOutOfDate: boolean;
    readonly isLowQuality: boolean;
    readonly isErroneous: boolean;
    readonly type: 'Unknown' | 'FeePaid' | 'Reasonable' | 'KnownGood' | 'OutOfDate' | 'LowQuality' | 'Erroneous';
  }

  /** @name PalletUtilityCall (375) */
  interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean;
    readonly asBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isAsDerivative: boolean;
    readonly asAsDerivative: {
      readonly index: u16;
      readonly call: Call;
    } & Struct;
    readonly isBatchAll: boolean;
    readonly asBatchAll: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isDispatchAs: boolean;
    readonly asDispatchAs: {
      readonly asOrigin: TangleStandaloneRuntimeOriginCaller;
      readonly call: Call;
    } & Struct;
    readonly isForceBatch: boolean;
    readonly asForceBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isWithWeight: boolean;
    readonly asWithWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'Batch' | 'AsDerivative' | 'BatchAll' | 'DispatchAs' | 'ForceBatch' | 'WithWeight';
  }

  /** @name TangleStandaloneRuntimeOriginCaller (377) */
  interface TangleStandaloneRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isVoid: boolean;
    readonly isCouncil: boolean;
    readonly asCouncil: PalletCollectiveRawOrigin;
    readonly type: 'System' | 'Void' | 'Council';
  }

  /** @name FrameSupportDispatchRawOrigin (378) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Root' | 'Signed' | 'None';
  }

  /** @name PalletCollectiveRawOrigin (379) */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId32;
    readonly isPhantom: boolean;
    readonly type: 'Members' | 'Member' | 'Phantom';
  }

  /** @name SpCoreVoid (380) */
  type SpCoreVoid = Null;

  /** @name PalletEth2LightClientCall (381) */
  interface PalletEth2LightClientCall extends Enum {
    readonly isInit: boolean;
    readonly asInit: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly args: EthTypesInitInput;
    } & Struct;
    readonly isRegisterSubmitter: boolean;
    readonly asRegisterSubmitter: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
    } & Struct;
    readonly isUnregisterSubmitter: boolean;
    readonly asUnregisterSubmitter: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
    } & Struct;
    readonly isSubmitBeaconChainLightClientUpdate: boolean;
    readonly asSubmitBeaconChainLightClientUpdate: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly lightClientUpdate: EthTypesEth2LightClientUpdate;
    } & Struct;
    readonly isSubmitExecutionHeader: boolean;
    readonly asSubmitExecutionHeader: {
      readonly typedChainId: WebbProposalsHeaderTypedChainId;
      readonly blockHeader: EthTypesBlockHeader;
    } & Struct;
    readonly isUpdateTrustedSigner: boolean;
    readonly asUpdateTrustedSigner: {
      readonly trustedSigner: AccountId32;
    } & Struct;
    readonly type: 'Init' | 'RegisterSubmitter' | 'UnregisterSubmitter' | 'SubmitBeaconChainLightClientUpdate' | 'SubmitExecutionHeader' | 'UpdateTrustedSigner';
  }

  /** @name EthTypesInitInput (382) */
  interface EthTypesInitInput extends Struct {
    readonly finalizedExecutionHeader: EthTypesBlockHeader;
    readonly finalizedBeaconHeader: EthTypesEth2ExtendedBeaconBlockHeader;
    readonly currentSyncCommittee: EthTypesEth2SyncCommittee;
    readonly nextSyncCommittee: EthTypesEth2SyncCommittee;
    readonly validateUpdates: bool;
    readonly verifyBlsSignatures: bool;
    readonly hashesGcThreshold: u64;
    readonly maxSubmittedBlocksByAccount: u32;
    readonly trustedSigner: Option<AccountId32>;
  }

  /** @name EthTypesBlockHeader (383) */
  interface EthTypesBlockHeader extends Struct {
    readonly parentHash: H256;
    readonly unclesHash: H256;
    readonly author: H160;
    readonly stateRoot: H256;
    readonly transactionsRoot: H256;
    readonly receiptsRoot: H256;
    readonly logBloom: EthTypesBloom;
    readonly difficulty: U256;
    readonly number: u64;
    readonly gasLimit: U256;
    readonly gasUsed: U256;
    readonly timestamp: u64;
    readonly extraData: Bytes;
    readonly mixHash: H256;
    readonly nonce: EthTypesH64;
    readonly baseFeePerGas: u64;
    readonly hash_: Option<H256>;
    readonly partialHash: Option<H256>;
  }

  /** @name EthTypesBloom (386) */
  interface EthTypesBloom extends EthbloomBloom {}

  /** @name EthbloomBloom (387) */
  interface EthbloomBloom extends U8aFixed {}

  /** @name EthTypesH64 (392) */
  interface EthTypesH64 extends EthereumTypesHashH64 {}

  /** @name EthereumTypesHashH64 (393) */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name EthTypesEth2ExtendedBeaconBlockHeader (395) */
  interface EthTypesEth2ExtendedBeaconBlockHeader extends Struct {
    readonly header: EthTypesEth2BeaconBlockHeader;
    readonly beaconBlockRoot: H256;
    readonly executionBlockHash: H256;
  }

  /** @name EthTypesEth2SyncCommittee (396) */
  interface EthTypesEth2SyncCommittee extends Struct {
    readonly pubkeys: EthTypesEth2SyncCommitteePublicKeys;
    readonly aggregatePubkey: EthTypesEth2PublicKeyBytes;
  }

  /** @name EthTypesEth2SyncCommitteePublicKeys (397) */
  interface EthTypesEth2SyncCommitteePublicKeys extends Vec<EthTypesEth2PublicKeyBytes> {}

  /** @name EthTypesEth2PublicKeyBytes (399) */
  interface EthTypesEth2PublicKeyBytes extends U8aFixed {}

  /** @name EthTypesEth2LightClientUpdate (401) */
  interface EthTypesEth2LightClientUpdate extends Struct {
    readonly attestedBeaconHeader: EthTypesEth2BeaconBlockHeader;
    readonly syncAggregate: EthTypesEth2SyncAggregate;
    readonly signatureSlot: u64;
    readonly finalityUpdate: EthTypesEth2FinalizedHeaderUpdate;
    readonly syncCommitteeUpdate: Option<EthTypesEth2SyncCommitteeUpdate>;
  }

  /** @name EthTypesEth2SyncAggregate (402) */
  interface EthTypesEth2SyncAggregate extends Struct {
    readonly syncCommitteeBits: EthTypesEth2SyncCommitteeBits;
    readonly syncCommitteeSignature: EthTypesEth2SignatureBytes;
  }

  /** @name EthTypesEth2SyncCommitteeBits (403) */
  interface EthTypesEth2SyncCommitteeBits extends U8aFixed {}

  /** @name EthTypesEth2SignatureBytes (404) */
  interface EthTypesEth2SignatureBytes extends U8aFixed {}

  /** @name EthTypesEth2FinalizedHeaderUpdate (406) */
  interface EthTypesEth2FinalizedHeaderUpdate extends Struct {
    readonly headerUpdate: EthTypesEth2HeaderUpdate;
    readonly finalityBranch: Vec<H256>;
  }

  /** @name EthTypesEth2HeaderUpdate (407) */
  interface EthTypesEth2HeaderUpdate extends Struct {
    readonly beaconHeader: EthTypesEth2BeaconBlockHeader;
    readonly executionBlockHash: H256;
    readonly executionHashBranch: Vec<H256>;
  }

  /** @name EthTypesEth2SyncCommitteeUpdate (410) */
  interface EthTypesEth2SyncCommitteeUpdate extends Struct {
    readonly nextSyncCommittee: EthTypesEth2SyncCommittee;
    readonly nextSyncCommitteeBranch: Vec<H256>;
  }

  /** @name PalletSudoError (411) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name PalletBalancesBalanceLock (414) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (415) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesReserveData (418) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name PalletBalancesError (420) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isKeepAlive: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly type: 'VestingBalance' | 'LiquidityRestrictions' | 'InsufficientBalance' | 'ExistentialDeposit' | 'KeepAlive' | 'ExistingVestingSchedule' | 'DeadAccount' | 'TooManyReserves';
  }

  /** @name PalletTransactionPaymentReleases (422) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name PalletGrandpaStoredState (426) */
  interface PalletGrandpaStoredState extends Enum {
    readonly isLive: boolean;
    readonly isPendingPause: boolean;
    readonly asPendingPause: {
      readonly scheduledAt: u32;
      readonly delay: u32;
    } & Struct;
    readonly isPaused: boolean;
    readonly isPendingResume: boolean;
    readonly asPendingResume: {
      readonly scheduledAt: u32;
      readonly delay: u32;
    } & Struct;
    readonly type: 'Live' | 'PendingPause' | 'Paused' | 'PendingResume';
  }

  /** @name PalletGrandpaStoredPendingChange (427) */
  interface PalletGrandpaStoredPendingChange extends Struct {
    readonly scheduledAt: u32;
    readonly delay: u32;
    readonly nextAuthorities: Vec<ITuple<[SpFinalityGrandpaAppPublic, u64]>>;
    readonly forced: Option<u32>;
  }

  /** @name PalletGrandpaError (429) */
  interface PalletGrandpaError extends Enum {
    readonly isPauseFailed: boolean;
    readonly isResumeFailed: boolean;
    readonly isChangePending: boolean;
    readonly isTooSoon: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isInvalidEquivocationProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly type: 'PauseFailed' | 'ResumeFailed' | 'ChangePending' | 'TooSoon' | 'InvalidKeyOwnershipProof' | 'InvalidEquivocationProof' | 'DuplicateOffenceReport';
  }

  /** @name PalletDkgMetadataRoundMetadata (433) */
  interface PalletDkgMetadataRoundMetadata extends Struct {
    readonly currRoundPubKey: Bytes;
    readonly nextRoundPubKey: Bytes;
    readonly refreshSignature: Bytes;
  }

  /** @name PalletDkgMetadataError (439) */
  interface PalletDkgMetadataError extends Enum {
    readonly isNoMappedAccount: boolean;
    readonly isInvalidThreshold: boolean;
    readonly isMustBeAQueuedAuthority: boolean;
    readonly isMustBeAnActiveAuthority: boolean;
    readonly isInvalidRefreshDelay: boolean;
    readonly isInvalidPublicKeys: boolean;
    readonly isAlreadySubmittedPublicKey: boolean;
    readonly isAlreadySubmittedSignature: boolean;
    readonly isUsedSignature: boolean;
    readonly isInvalidSignature: boolean;
    readonly isInvalidNonce: boolean;
    readonly isInvalidMisbehaviourReports: boolean;
    readonly isRefreshInProgress: boolean;
    readonly isNoNextPublicKey: boolean;
    readonly isInvalidControllerAccount: boolean;
    readonly isOutOfBounds: boolean;
    readonly type: 'NoMappedAccount' | 'InvalidThreshold' | 'MustBeAQueuedAuthority' | 'MustBeAnActiveAuthority' | 'InvalidRefreshDelay' | 'InvalidPublicKeys' | 'AlreadySubmittedPublicKey' | 'AlreadySubmittedSignature' | 'UsedSignature' | 'InvalidSignature' | 'InvalidNonce' | 'InvalidMisbehaviourReports' | 'RefreshInProgress' | 'NoNextPublicKey' | 'InvalidControllerAccount' | 'OutOfBounds';
  }

  /** @name PalletDkgProposalsProposalVotes (446) */
  interface PalletDkgProposalsProposalVotes extends Struct {
    readonly votesFor: Vec<AccountId32>;
    readonly votesAgainst: Vec<AccountId32>;
    readonly status: PalletDkgProposalsProposalStatus;
    readonly expiry: u32;
  }

  /** @name TangleStandaloneRuntimeMaxVotes (447) */
  type TangleStandaloneRuntimeMaxVotes = Null;

  /** @name PalletDkgProposalsProposalStatus (449) */
  interface PalletDkgProposalsProposalStatus extends Enum {
    readonly isInitiated: boolean;
    readonly isApproved: boolean;
    readonly isRejected: boolean;
    readonly type: 'Initiated' | 'Approved' | 'Rejected';
  }

  /** @name PalletDkgProposalsError (451) */
  interface PalletDkgProposalsError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isThresholdNotSet: boolean;
    readonly isInvalidChainId: boolean;
    readonly isInvalidThreshold: boolean;
    readonly isChainNotWhitelisted: boolean;
    readonly isChainAlreadyWhitelisted: boolean;
    readonly isResourceDoesNotExist: boolean;
    readonly isProposerAlreadyExists: boolean;
    readonly isProposerInvalid: boolean;
    readonly isMustBeProposer: boolean;
    readonly isProposerAlreadyVoted: boolean;
    readonly isProposalAlreadyExists: boolean;
    readonly isProposalDoesNotExist: boolean;
    readonly isProposalNotComplete: boolean;
    readonly isProposalAlreadyComplete: boolean;
    readonly isProposalExpired: boolean;
    readonly isProposerCountIsZero: boolean;
    readonly isOutOfBounds: boolean;
    readonly type: 'InvalidPermissions' | 'ThresholdNotSet' | 'InvalidChainId' | 'InvalidThreshold' | 'ChainNotWhitelisted' | 'ChainAlreadyWhitelisted' | 'ResourceDoesNotExist' | 'ProposerAlreadyExists' | 'ProposerInvalid' | 'MustBeProposer' | 'ProposerAlreadyVoted' | 'ProposalAlreadyExists' | 'ProposalDoesNotExist' | 'ProposalNotComplete' | 'ProposalAlreadyComplete' | 'ProposalExpired' | 'ProposerCountIsZero' | 'OutOfBounds';
  }

  /** @name DkgRuntimePrimitivesProposalStoredUnsignedProposal (453) */
  interface DkgRuntimePrimitivesProposalStoredUnsignedProposal extends Struct {
    readonly proposal: WebbProposalsProposal;
    readonly timestamp: u32;
  }

  /** @name PalletDkgProposalHandlerError (454) */
  interface PalletDkgProposalHandlerError extends Enum {
    readonly isNoneValue: boolean;
    readonly isStorageOverflow: boolean;
    readonly isProposalFormatInvalid: boolean;
    readonly isProposalMustBeUnsigned: boolean;
    readonly isInvalidProposalBytesLength: boolean;
    readonly isProposalSignatureInvalid: boolean;
    readonly isProposalDoesNotExists: boolean;
    readonly isProposalAlreadyExists: boolean;
    readonly isChainIdInvalid: boolean;
    readonly isProposalsLengthOverflow: boolean;
    readonly isProposalOutOfBounds: boolean;
    readonly type: 'NoneValue' | 'StorageOverflow' | 'ProposalFormatInvalid' | 'ProposalMustBeUnsigned' | 'InvalidProposalBytesLength' | 'ProposalSignatureInvalid' | 'ProposalDoesNotExists' | 'ProposalAlreadyExists' | 'ChainIdInvalid' | 'ProposalsLengthOverflow' | 'ProposalOutOfBounds';
  }

  /** @name PalletBridgeRegistryBridgeMetadata (455) */
  interface PalletBridgeRegistryBridgeMetadata extends Struct {
    readonly resourceIds: Vec<WebbProposalsHeaderResourceId>;
    readonly info: PalletBridgeRegistryBridgeInfo;
  }

  /** @name PalletBridgeRegistryError (457) */
  interface PalletBridgeRegistryError extends Enum {
    readonly isParametersNotInitialized: boolean;
    readonly isVerifyError: boolean;
    readonly isProposalNotSigned: boolean;
    readonly isBridgeIndexError: boolean;
    readonly isTooManyFields: boolean;
    readonly isBridgeNotFound: boolean;
    readonly isTooManyResources: boolean;
    readonly type: 'ParametersNotInitialized' | 'VerifyError' | 'ProposalNotSigned' | 'BridgeIndexError' | 'TooManyFields' | 'BridgeNotFound' | 'TooManyResources';
  }

  /** @name PalletIndicesError (459) */
  interface PalletIndicesError extends Enum {
    readonly isNotAssigned: boolean;
    readonly isNotOwner: boolean;
    readonly isInUse: boolean;
    readonly isNotTransfer: boolean;
    readonly isPermanent: boolean;
    readonly type: 'NotAssigned' | 'NotOwner' | 'InUse' | 'NotTransfer' | 'Permanent';
  }

  /** @name PalletDemocracyReferendumInfo (465) */
  interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletDemocracyReferendumStatus;
    readonly isFinished: boolean;
    readonly asFinished: {
      readonly approved: bool;
      readonly end: u32;
    } & Struct;
    readonly type: 'Ongoing' | 'Finished';
  }

  /** @name PalletDemocracyReferendumStatus (466) */
  interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u32;
    readonly proposal: FrameSupportPreimagesBounded;
    readonly threshold: PalletDemocracyVoteThreshold;
    readonly delay: u32;
    readonly tally: PalletDemocracyTally;
  }

  /** @name PalletDemocracyTally (467) */
  interface PalletDemocracyTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly turnout: u128;
  }

  /** @name PalletDemocracyVoteVoting (468) */
  interface PalletDemocracyVoteVoting extends Enum {
    readonly isDirect: boolean;
    readonly asDirect: {
      readonly votes: Vec<ITuple<[u32, PalletDemocracyVoteAccountVote]>>;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly isDelegating: boolean;
    readonly asDelegating: {
      readonly balance: u128;
      readonly target: AccountId32;
      readonly conviction: PalletDemocracyConviction;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly type: 'Direct' | 'Delegating';
  }

  /** @name PalletDemocracyDelegations (472) */
  interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /** @name PalletDemocracyVotePriorLock (473) */
  interface PalletDemocracyVotePriorLock extends ITuple<[u32, u128]> {}

  /** @name PalletDemocracyError (476) */
  interface PalletDemocracyError extends Enum {
    readonly isValueLow: boolean;
    readonly isProposalMissing: boolean;
    readonly isAlreadyCanceled: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalBlacklisted: boolean;
    readonly isNotSimpleMajority: boolean;
    readonly isInvalidHash: boolean;
    readonly isNoProposal: boolean;
    readonly isAlreadyVetoed: boolean;
    readonly isReferendumInvalid: boolean;
    readonly isNoneWaiting: boolean;
    readonly isNotVoter: boolean;
    readonly isNoPermission: boolean;
    readonly isAlreadyDelegating: boolean;
    readonly isInsufficientFunds: boolean;
    readonly isNotDelegating: boolean;
    readonly isVotesExist: boolean;
    readonly isInstantNotAllowed: boolean;
    readonly isNonsense: boolean;
    readonly isWrongUpperBound: boolean;
    readonly isMaxVotesReached: boolean;
    readonly isTooMany: boolean;
    readonly isVotingPeriodLow: boolean;
    readonly isPreimageNotExist: boolean;
    readonly type: 'ValueLow' | 'ProposalMissing' | 'AlreadyCanceled' | 'DuplicateProposal' | 'ProposalBlacklisted' | 'NotSimpleMajority' | 'InvalidHash' | 'NoProposal' | 'AlreadyVetoed' | 'ReferendumInvalid' | 'NoneWaiting' | 'NotVoter' | 'NoPermission' | 'AlreadyDelegating' | 'InsufficientFunds' | 'NotDelegating' | 'VotesExist' | 'InstantNotAllowed' | 'Nonsense' | 'WrongUpperBound' | 'MaxVotesReached' | 'TooMany' | 'VotingPeriodLow' | 'PreimageNotExist';
  }

  /** @name PalletCollectiveVotes (478) */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId32>;
    readonly nays: Vec<AccountId32>;
    readonly end: u32;
  }

  /** @name PalletCollectiveError (479) */
  interface PalletCollectiveError extends Enum {
    readonly isNotMember: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalMissing: boolean;
    readonly isWrongIndex: boolean;
    readonly isDuplicateVote: boolean;
    readonly isAlreadyInitialized: boolean;
    readonly isTooEarly: boolean;
    readonly isTooManyProposals: boolean;
    readonly isWrongProposalWeight: boolean;
    readonly isWrongProposalLength: boolean;
    readonly type: 'NotMember' | 'DuplicateProposal' | 'ProposalMissing' | 'WrongIndex' | 'DuplicateVote' | 'AlreadyInitialized' | 'TooEarly' | 'TooManyProposals' | 'WrongProposalWeight' | 'WrongProposalLength';
  }

  /** @name PalletVestingReleases (482) */
  interface PalletVestingReleases extends Enum {
    readonly isV0: boolean;
    readonly isV1: boolean;
    readonly type: 'V0' | 'V1';
  }

  /** @name PalletVestingError (483) */
  interface PalletVestingError extends Enum {
    readonly isNotVesting: boolean;
    readonly isAtMaxVestingSchedules: boolean;
    readonly isAmountLow: boolean;
    readonly isScheduleIndexOutOfBounds: boolean;
    readonly isInvalidScheduleParams: boolean;
    readonly type: 'NotVesting' | 'AtMaxVestingSchedules' | 'AmountLow' | 'ScheduleIndexOutOfBounds' | 'InvalidScheduleParams';
  }

  /** @name PalletEcdsaClaimsError (485) */
  interface PalletEcdsaClaimsError extends Enum {
    readonly isInvalidEthereumSignature: boolean;
    readonly isSignerHasNoClaim: boolean;
    readonly isSenderHasNoClaim: boolean;
    readonly isPotUnderflow: boolean;
    readonly isInvalidStatement: boolean;
    readonly isVestedBalanceExists: boolean;
    readonly type: 'InvalidEthereumSignature' | 'SignerHasNoClaim' | 'SenderHasNoClaim' | 'PotUnderflow' | 'InvalidStatement' | 'VestedBalanceExists';
  }

  /** @name PalletElectionsPhragmenSeatHolder (487) */
  interface PalletElectionsPhragmenSeatHolder extends Struct {
    readonly who: AccountId32;
    readonly stake: u128;
    readonly deposit: u128;
  }

  /** @name PalletElectionsPhragmenVoter (488) */
  interface PalletElectionsPhragmenVoter extends Struct {
    readonly votes: Vec<AccountId32>;
    readonly stake: u128;
    readonly deposit: u128;
  }

  /** @name PalletElectionsPhragmenError (489) */
  interface PalletElectionsPhragmenError extends Enum {
    readonly isUnableToVote: boolean;
    readonly isNoVotes: boolean;
    readonly isTooManyVotes: boolean;
    readonly isMaximumVotesExceeded: boolean;
    readonly isLowBalance: boolean;
    readonly isUnableToPayBond: boolean;
    readonly isMustBeVoter: boolean;
    readonly isDuplicatedCandidate: boolean;
    readonly isTooManyCandidates: boolean;
    readonly isMemberSubmit: boolean;
    readonly isRunnerUpSubmit: boolean;
    readonly isInsufficientCandidateFunds: boolean;
    readonly isNotMember: boolean;
    readonly isInvalidWitnessData: boolean;
    readonly isInvalidVoteCount: boolean;
    readonly isInvalidRenouncing: boolean;
    readonly isInvalidReplacement: boolean;
    readonly type: 'UnableToVote' | 'NoVotes' | 'TooManyVotes' | 'MaximumVotesExceeded' | 'LowBalance' | 'UnableToPayBond' | 'MustBeVoter' | 'DuplicatedCandidate' | 'TooManyCandidates' | 'MemberSubmit' | 'RunnerUpSubmit' | 'InsufficientCandidateFunds' | 'NotMember' | 'InvalidWitnessData' | 'InvalidVoteCount' | 'InvalidRenouncing' | 'InvalidReplacement';
  }

  /** @name PalletElectionProviderMultiPhaseReadySolution (490) */
  interface PalletElectionProviderMultiPhaseReadySolution extends Struct {
    readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>;
    readonly score: SpNposElectionsElectionScore;
    readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
  }

  /** @name PalletElectionProviderMultiPhaseRoundSnapshot (492) */
  interface PalletElectionProviderMultiPhaseRoundSnapshot extends Struct {
    readonly voters: Vec<ITuple<[AccountId32, u64, Vec<AccountId32>]>>;
    readonly targets: Vec<AccountId32>;
  }

  /** @name PalletElectionProviderMultiPhaseSignedSignedSubmission (499) */
  interface PalletElectionProviderMultiPhaseSignedSignedSubmission extends Struct {
    readonly who: AccountId32;
    readonly deposit: u128;
    readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
    readonly callFee: u128;
  }

  /** @name PalletElectionProviderMultiPhaseError (500) */
  interface PalletElectionProviderMultiPhaseError extends Enum {
    readonly isPreDispatchEarlySubmission: boolean;
    readonly isPreDispatchWrongWinnerCount: boolean;
    readonly isPreDispatchWeakSubmission: boolean;
    readonly isSignedQueueFull: boolean;
    readonly isSignedCannotPayDeposit: boolean;
    readonly isSignedInvalidWitness: boolean;
    readonly isSignedTooMuchWeight: boolean;
    readonly isOcwCallWrongEra: boolean;
    readonly isMissingSnapshotMetadata: boolean;
    readonly isInvalidSubmissionIndex: boolean;
    readonly isCallNotAllowed: boolean;
    readonly isFallbackFailed: boolean;
    readonly isBoundNotMet: boolean;
    readonly isTooManyWinners: boolean;
    readonly type: 'PreDispatchEarlySubmission' | 'PreDispatchWrongWinnerCount' | 'PreDispatchWeakSubmission' | 'SignedQueueFull' | 'SignedCannotPayDeposit' | 'SignedInvalidWitness' | 'SignedTooMuchWeight' | 'OcwCallWrongEra' | 'MissingSnapshotMetadata' | 'InvalidSubmissionIndex' | 'CallNotAllowed' | 'FallbackFailed' | 'BoundNotMet' | 'TooManyWinners';
  }

  /** @name PalletStakingStakingLedger (501) */
  interface PalletStakingStakingLedger extends Struct {
    readonly stash: AccountId32;
    readonly total: Compact<u128>;
    readonly active: Compact<u128>;
    readonly unlocking: Vec<PalletStakingUnlockChunk>;
    readonly claimedRewards: Vec<u32>;
  }

  /** @name PalletStakingUnlockChunk (503) */
  interface PalletStakingUnlockChunk extends Struct {
    readonly value: Compact<u128>;
    readonly era: Compact<u32>;
  }

  /** @name PalletStakingNominations (506) */
  interface PalletStakingNominations extends Struct {
    readonly targets: Vec<AccountId32>;
    readonly submittedIn: u32;
    readonly suppressed: bool;
  }

  /** @name PalletStakingActiveEraInfo (507) */
  interface PalletStakingActiveEraInfo extends Struct {
    readonly index: u32;
    readonly start: Option<u64>;
  }

  /** @name PalletStakingEraRewardPoints (509) */
  interface PalletStakingEraRewardPoints extends Struct {
    readonly total: u32;
    readonly individual: BTreeMap<AccountId32, u32>;
  }

  /** @name PalletStakingUnappliedSlash (514) */
  interface PalletStakingUnappliedSlash extends Struct {
    readonly validator: AccountId32;
    readonly own: u128;
    readonly others: Vec<ITuple<[AccountId32, u128]>>;
    readonly reporters: Vec<AccountId32>;
    readonly payout: u128;
  }

  /** @name PalletStakingSlashingSlashingSpans (516) */
  interface PalletStakingSlashingSlashingSpans extends Struct {
    readonly spanIndex: u32;
    readonly lastStart: u32;
    readonly lastNonzeroSlash: u32;
    readonly prior: Vec<u32>;
  }

  /** @name PalletStakingSlashingSpanRecord (517) */
  interface PalletStakingSlashingSpanRecord extends Struct {
    readonly slashed: u128;
    readonly paidOut: u128;
  }

  /** @name PalletStakingPalletError (520) */
  interface PalletStakingPalletError extends Enum {
    readonly isNotController: boolean;
    readonly isNotStash: boolean;
    readonly isAlreadyBonded: boolean;
    readonly isAlreadyPaired: boolean;
    readonly isEmptyTargets: boolean;
    readonly isDuplicateIndex: boolean;
    readonly isInvalidSlashIndex: boolean;
    readonly isInsufficientBond: boolean;
    readonly isNoMoreChunks: boolean;
    readonly isNoUnlockChunk: boolean;
    readonly isFundedTarget: boolean;
    readonly isInvalidEraToReward: boolean;
    readonly isInvalidNumberOfNominations: boolean;
    readonly isNotSortedAndUnique: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isIncorrectHistoryDepth: boolean;
    readonly isIncorrectSlashingSpans: boolean;
    readonly isBadState: boolean;
    readonly isTooManyTargets: boolean;
    readonly isBadTarget: boolean;
    readonly isCannotChillOther: boolean;
    readonly isTooManyNominators: boolean;
    readonly isTooManyValidators: boolean;
    readonly isCommissionTooLow: boolean;
    readonly isBoundNotMet: boolean;
    readonly type: 'NotController' | 'NotStash' | 'AlreadyBonded' | 'AlreadyPaired' | 'EmptyTargets' | 'DuplicateIndex' | 'InvalidSlashIndex' | 'InsufficientBond' | 'NoMoreChunks' | 'NoUnlockChunk' | 'FundedTarget' | 'InvalidEraToReward' | 'InvalidNumberOfNominations' | 'NotSortedAndUnique' | 'AlreadyClaimed' | 'IncorrectHistoryDepth' | 'IncorrectSlashingSpans' | 'BadState' | 'TooManyTargets' | 'BadTarget' | 'CannotChillOther' | 'TooManyNominators' | 'TooManyValidators' | 'CommissionTooLow' | 'BoundNotMet';
  }

  /** @name SpCoreCryptoKeyTypeId (524) */
  interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (525) */
  interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean;
    readonly isNoAssociatedValidatorId: boolean;
    readonly isDuplicatedKey: boolean;
    readonly isNoKeys: boolean;
    readonly isNoAccount: boolean;
    readonly type: 'InvalidProof' | 'NoAssociatedValidatorId' | 'DuplicatedKey' | 'NoKeys' | 'NoAccount';
  }

  /** @name PalletTreasuryProposal (526) */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly beneficiary: AccountId32;
    readonly bond: u128;
  }

  /** @name FrameSupportPalletId (528) */
  interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (529) */
  interface PalletTreasuryError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly isInsufficientPermission: boolean;
    readonly isProposalNotApproved: boolean;
    readonly type: 'InsufficientProposersBalance' | 'InvalidIndex' | 'TooManyApprovals' | 'InsufficientPermission' | 'ProposalNotApproved';
  }

  /** @name PalletBountiesBounty (530) */
  interface PalletBountiesBounty extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly fee: u128;
    readonly curatorDeposit: u128;
    readonly bond: u128;
    readonly status: PalletBountiesBountyStatus;
  }

  /** @name PalletBountiesBountyStatus (531) */
  interface PalletBountiesBountyStatus extends Enum {
    readonly isProposed: boolean;
    readonly isApproved: boolean;
    readonly isFunded: boolean;
    readonly isCuratorProposed: boolean;
    readonly asCuratorProposed: {
      readonly curator: AccountId32;
    } & Struct;
    readonly isActive: boolean;
    readonly asActive: {
      readonly curator: AccountId32;
      readonly updateDue: u32;
    } & Struct;
    readonly isPendingPayout: boolean;
    readonly asPendingPayout: {
      readonly curator: AccountId32;
      readonly beneficiary: AccountId32;
      readonly unlockAt: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Approved' | 'Funded' | 'CuratorProposed' | 'Active' | 'PendingPayout';
  }

  /** @name PalletBountiesError (533) */
  interface PalletBountiesError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isReasonTooBig: boolean;
    readonly isUnexpectedStatus: boolean;
    readonly isRequireCurator: boolean;
    readonly isInvalidValue: boolean;
    readonly isInvalidFee: boolean;
    readonly isPendingPayout: boolean;
    readonly isPremature: boolean;
    readonly isHasActiveChildBounty: boolean;
    readonly isTooManyQueued: boolean;
    readonly type: 'InsufficientProposersBalance' | 'InvalidIndex' | 'ReasonTooBig' | 'UnexpectedStatus' | 'RequireCurator' | 'InvalidValue' | 'InvalidFee' | 'PendingPayout' | 'Premature' | 'HasActiveChildBounty' | 'TooManyQueued';
  }

  /** @name PalletChildBountiesChildBounty (534) */
  interface PalletChildBountiesChildBounty extends Struct {
    readonly parentBounty: u32;
    readonly value: u128;
    readonly fee: u128;
    readonly curatorDeposit: u128;
    readonly status: PalletChildBountiesChildBountyStatus;
  }

  /** @name PalletChildBountiesChildBountyStatus (535) */
  interface PalletChildBountiesChildBountyStatus extends Enum {
    readonly isAdded: boolean;
    readonly isCuratorProposed: boolean;
    readonly asCuratorProposed: {
      readonly curator: AccountId32;
    } & Struct;
    readonly isActive: boolean;
    readonly asActive: {
      readonly curator: AccountId32;
    } & Struct;
    readonly isPendingPayout: boolean;
    readonly asPendingPayout: {
      readonly curator: AccountId32;
      readonly beneficiary: AccountId32;
      readonly unlockAt: u32;
    } & Struct;
    readonly type: 'Added' | 'CuratorProposed' | 'Active' | 'PendingPayout';
  }

  /** @name PalletChildBountiesError (536) */
  interface PalletChildBountiesError extends Enum {
    readonly isParentBountyNotActive: boolean;
    readonly isInsufficientBountyBalance: boolean;
    readonly isTooManyChildBounties: boolean;
    readonly type: 'ParentBountyNotActive' | 'InsufficientBountyBalance' | 'TooManyChildBounties';
  }

  /** @name PalletBagsListListNode (537) */
  interface PalletBagsListListNode extends Struct {
    readonly id: AccountId32;
    readonly prev: Option<AccountId32>;
    readonly next: Option<AccountId32>;
    readonly bagUpper: u64;
    readonly score: u64;
  }

  /** @name PalletBagsListListBag (538) */
  interface PalletBagsListListBag extends Struct {
    readonly head: Option<AccountId32>;
    readonly tail: Option<AccountId32>;
  }

  /** @name PalletBagsListError (540) */
  interface PalletBagsListError extends Enum {
    readonly isList: boolean;
    readonly asList: PalletBagsListListListError;
    readonly type: 'List';
  }

  /** @name PalletBagsListListListError (541) */
  interface PalletBagsListListListError extends Enum {
    readonly isDuplicate: boolean;
    readonly isNotHeavier: boolean;
    readonly isNotInSameBag: boolean;
    readonly isNodeNotFound: boolean;
    readonly type: 'Duplicate' | 'NotHeavier' | 'NotInSameBag' | 'NodeNotFound';
  }

  /** @name PalletNominationPoolsPoolMember (542) */
  interface PalletNominationPoolsPoolMember extends Struct {
    readonly poolId: u32;
    readonly points: u128;
    readonly lastRecordedRewardCounter: u128;
    readonly unbondingEras: BTreeMap<u32, u128>;
  }

  /** @name PalletNominationPoolsBondedPoolInner (547) */
  interface PalletNominationPoolsBondedPoolInner extends Struct {
    readonly points: u128;
    readonly state: PalletNominationPoolsPoolState;
    readonly memberCounter: u32;
    readonly roles: PalletNominationPoolsPoolRoles;
  }

  /** @name PalletNominationPoolsPoolRoles (548) */
  interface PalletNominationPoolsPoolRoles extends Struct {
    readonly depositor: AccountId32;
    readonly root: Option<AccountId32>;
    readonly nominator: Option<AccountId32>;
    readonly bouncer: Option<AccountId32>;
  }

  /** @name PalletNominationPoolsRewardPool (549) */
  interface PalletNominationPoolsRewardPool extends Struct {
    readonly lastRecordedRewardCounter: u128;
    readonly lastRecordedTotalPayouts: u128;
    readonly totalRewardsClaimed: u128;
  }

  /** @name PalletNominationPoolsSubPools (550) */
  interface PalletNominationPoolsSubPools extends Struct {
    readonly noEra: PalletNominationPoolsUnbondPool;
    readonly withEra: BTreeMap<u32, PalletNominationPoolsUnbondPool>;
  }

  /** @name PalletNominationPoolsUnbondPool (551) */
  interface PalletNominationPoolsUnbondPool extends Struct {
    readonly points: u128;
    readonly balance: u128;
  }

  /** @name PalletNominationPoolsError (557) */
  interface PalletNominationPoolsError extends Enum {
    readonly isPoolNotFound: boolean;
    readonly isPoolMemberNotFound: boolean;
    readonly isRewardPoolNotFound: boolean;
    readonly isSubPoolsNotFound: boolean;
    readonly isAccountBelongsToOtherPool: boolean;
    readonly isFullyUnbonding: boolean;
    readonly isMaxUnbondingLimit: boolean;
    readonly isCannotWithdrawAny: boolean;
    readonly isMinimumBondNotMet: boolean;
    readonly isOverflowRisk: boolean;
    readonly isNotDestroying: boolean;
    readonly isNotNominator: boolean;
    readonly isNotKickerOrDestroying: boolean;
    readonly isNotOpen: boolean;
    readonly isMaxPools: boolean;
    readonly isMaxPoolMembers: boolean;
    readonly isCanNotChangeState: boolean;
    readonly isDoesNotHavePermission: boolean;
    readonly isMetadataExceedsMaxLen: boolean;
    readonly isDefensive: boolean;
    readonly asDefensive: PalletNominationPoolsDefensiveError;
    readonly isPartialUnbondNotAllowedPermissionlessly: boolean;
    readonly isPoolIdInUse: boolean;
    readonly isInvalidPoolId: boolean;
    readonly isBondExtraRestricted: boolean;
    readonly type: 'PoolNotFound' | 'PoolMemberNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'AccountBelongsToOtherPool' | 'FullyUnbonding' | 'MaxUnbondingLimit' | 'CannotWithdrawAny' | 'MinimumBondNotMet' | 'OverflowRisk' | 'NotDestroying' | 'NotNominator' | 'NotKickerOrDestroying' | 'NotOpen' | 'MaxPools' | 'MaxPoolMembers' | 'CanNotChangeState' | 'DoesNotHavePermission' | 'MetadataExceedsMaxLen' | 'Defensive' | 'PartialUnbondNotAllowedPermissionlessly' | 'PoolIdInUse' | 'InvalidPoolId' | 'BondExtraRestricted';
  }

  /** @name PalletNominationPoolsDefensiveError (558) */
  interface PalletNominationPoolsDefensiveError extends Enum {
    readonly isNotEnoughSpaceInUnbondPool: boolean;
    readonly isPoolNotFound: boolean;
    readonly isRewardPoolNotFound: boolean;
    readonly isSubPoolsNotFound: boolean;
    readonly isBondedStashKilledPrematurely: boolean;
    readonly type: 'NotEnoughSpaceInUnbondPool' | 'PoolNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'BondedStashKilledPrematurely';
  }

  /** @name PalletSchedulerScheduled (561) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: TangleStandaloneRuntimeOriginCaller;
  }

  /** @name PalletSchedulerError (563) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly isNamed: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange' | 'Named';
  }

  /** @name PalletPreimageRequestStatus (564) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly deposit: ITuple<[AccountId32, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly deposit: Option<ITuple<[AccountId32, u128]>>;
      readonly count: u32;
      readonly len: Option<u32>;
    } & Struct;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageError (568) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly type: 'TooBig' | 'AlreadyNoted' | 'NotAuthorized' | 'NotNoted' | 'Requested' | 'NotRequested';
  }

  /** @name SpStakingOffenceOffenceDetails (569) */
  interface SpStakingOffenceOffenceDetails extends Struct {
    readonly offender: ITuple<[AccountId32, PalletStakingExposure]>;
    readonly reporters: Vec<AccountId32>;
  }

  /** @name PalletHasherError (571) */
  interface PalletHasherError extends Enum {
    readonly isHashingParametersNotInitialized: boolean;
    readonly isHashError: boolean;
    readonly type: 'HashingParametersNotInitialized' | 'HashError';
  }

  /** @name PalletAssetRegistryAssetDetails (572) */
  interface PalletAssetRegistryAssetDetails extends Struct {
    readonly name: Bytes;
    readonly assetType: PalletAssetRegistryAssetType;
    readonly existentialDeposit: u128;
    readonly locked: bool;
  }

  /** @name PalletAssetRegistryAssetMetadata (573) */
  interface PalletAssetRegistryAssetMetadata extends Struct {
    readonly symbol: Bytes;
    readonly decimals: u8;
  }

  /** @name PalletAssetRegistryError (574) */
  interface PalletAssetRegistryError extends Enum {
    readonly isNoIdAvailable: boolean;
    readonly isAssetNotFound: boolean;
    readonly isTooLong: boolean;
    readonly isAssetNotRegistered: boolean;
    readonly isAssetAlreadyRegistered: boolean;
    readonly isInvalidSharedAssetLen: boolean;
    readonly isAssetExistsInPool: boolean;
    readonly isAssetNotFoundInPool: boolean;
    readonly isMaxAssetIdInPoolReached: boolean;
    readonly type: 'NoIdAvailable' | 'AssetNotFound' | 'TooLong' | 'AssetNotRegistered' | 'AssetAlreadyRegistered' | 'InvalidSharedAssetLen' | 'AssetExistsInPool' | 'AssetNotFoundInPool' | 'MaxAssetIdInPoolReached';
  }

  /** @name OrmlCurrenciesModuleError (575) */
  interface OrmlCurrenciesModuleError extends Enum {
    readonly isAmountIntoBalanceFailed: boolean;
    readonly isBalanceTooLow: boolean;
    readonly isDepositFailed: boolean;
    readonly type: 'AmountIntoBalanceFailed' | 'BalanceTooLow' | 'DepositFailed';
  }

  /** @name OrmlTokensBalanceLock (577) */
  interface OrmlTokensBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name OrmlTokensAccountData (579) */
  interface OrmlTokensAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly frozen: u128;
  }

  /** @name OrmlTokensReserveData (581) */
  interface OrmlTokensReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name OrmlTokensModuleError (583) */
  interface OrmlTokensModuleError extends Enum {
    readonly isBalanceTooLow: boolean;
    readonly isAmountIntoBalanceFailed: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isMaxLocksExceeded: boolean;
    readonly isKeepAlive: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly type: 'BalanceTooLow' | 'AmountIntoBalanceFailed' | 'LiquidityRestrictions' | 'MaxLocksExceeded' | 'KeepAlive' | 'ExistentialDeposit' | 'DeadAccount' | 'TooManyReserves';
  }

  /** @name PalletTokenWrapperError (584) */
  interface PalletTokenWrapperError extends Enum {
    readonly isInvalidAmount: boolean;
    readonly isUnregisteredAssetId: boolean;
    readonly isNotFoundInPool: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isNoWrappingFeePercentFound: boolean;
    readonly isInvalidNonce: boolean;
    readonly isNameExceedsMaximumLimit: boolean;
    readonly type: 'InvalidAmount' | 'UnregisteredAssetId' | 'NotFoundInPool' | 'InsufficientBalance' | 'NoWrappingFeePercentFound' | 'InvalidNonce' | 'NameExceedsMaximumLimit';
  }

  /** @name PalletVerifierError (585) */
  interface PalletVerifierError extends Enum {
    readonly isVerifyingParametersNotInitialized: boolean;
    readonly isVerifyError: boolean;
    readonly type: 'VerifyingParametersNotInitialized' | 'VerifyError';
  }

  /** @name WebbPrimitivesDepositDetails (586) */
  interface WebbPrimitivesDepositDetails extends Struct {
    readonly depositor: AccountId32;
    readonly deposit: u128;
  }

  /** @name PalletMtTreeMetadata (587) */
  interface PalletMtTreeMetadata extends Struct {
    readonly creator: Option<AccountId32>;
    readonly paused: bool;
    readonly leafCount: u32;
    readonly maxLeaves: u32;
    readonly depth: u8;
    readonly root: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    readonly edgeNodes: Vec<TangleStandaloneRuntimeProtocolSubstrateConfigElement>;
  }

  /** @name TangleStandaloneRuntimeProtocolSubstrateConfigMaxEdges (588) */
  type TangleStandaloneRuntimeProtocolSubstrateConfigMaxEdges = Null;

  /** @name PalletMtError (590) */
  interface PalletMtError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isInvalidTreeDepth: boolean;
    readonly isInvalidLeafIndex: boolean;
    readonly isExceedsMaxLeaves: boolean;
    readonly isTreeDoesntExist: boolean;
    readonly isExceedsMaxDefaultHashes: boolean;
    readonly isExceedsMaxEdges: boolean;
    readonly type: 'InvalidPermissions' | 'InvalidTreeDepth' | 'InvalidLeafIndex' | 'ExceedsMaxLeaves' | 'TreeDoesntExist' | 'ExceedsMaxDefaultHashes' | 'ExceedsMaxEdges';
  }

  /** @name PalletLinkableTreeEdgeMetadata (592) */
  interface PalletLinkableTreeEdgeMetadata extends Struct {
    readonly srcChainId: u64;
    readonly root: TangleStandaloneRuntimeProtocolSubstrateConfigElement;
    readonly latestLeafIndex: u32;
    readonly srcResourceId: WebbProposalsHeaderResourceId;
  }

  /** @name PalletLinkableTreeError (594) */
  interface PalletLinkableTreeError extends Enum {
    readonly isUnknownRoot: boolean;
    readonly isInvalidMerkleRoots: boolean;
    readonly isInvalidNeighborWithdrawRoot: boolean;
    readonly isTooManyEdges: boolean;
    readonly isEdgeAlreadyExists: boolean;
    readonly isEdgeDoesntExists: boolean;
    readonly isInvalidLatestLeafIndex: boolean;
    readonly type: 'UnknownRoot' | 'InvalidMerkleRoots' | 'InvalidNeighborWithdrawRoot' | 'TooManyEdges' | 'EdgeAlreadyExists' | 'EdgeDoesntExists' | 'InvalidLatestLeafIndex';
  }

  /** @name PalletMixerMixerMetadata (595) */
  interface PalletMixerMixerMetadata extends Struct {
    readonly depositSize: u128;
    readonly asset: u32;
  }

  /** @name PalletMixerError (597) */
  interface PalletMixerError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isInvalidWithdrawProof: boolean;
    readonly isAlreadyRevealedNullifier: boolean;
    readonly isInvalidArbitraryData: boolean;
    readonly isUnknownRoot: boolean;
    readonly isNoMixerFound: boolean;
    readonly type: 'InvalidPermissions' | 'InvalidWithdrawProof' | 'AlreadyRevealedNullifier' | 'InvalidArbitraryData' | 'UnknownRoot' | 'NoMixerFound';
  }

  /** @name PalletSignatureBridgeError (598) */
  interface PalletSignatureBridgeError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isInvalidChainId: boolean;
    readonly isChainNotWhitelisted: boolean;
    readonly isChainAlreadyWhitelisted: boolean;
    readonly isResourceDoesNotExist: boolean;
    readonly isResourceAlreadyExists: boolean;
    readonly isSignatureInvalid: boolean;
    readonly isMustBeMaintainer: boolean;
    readonly isProposalAlreadyExists: boolean;
    readonly isCallNotConsistentWithProposalData: boolean;
    readonly isCallDoesNotMatchResourceId: boolean;
    readonly isIncorrectExecutionChainIdType: boolean;
    readonly isInvalidNonce: boolean;
    readonly isInvalidProposalData: boolean;
    readonly isInvalidCall: boolean;
    readonly isStringLimitExceeded: boolean;
    readonly type: 'InvalidPermissions' | 'InvalidChainId' | 'ChainNotWhitelisted' | 'ChainAlreadyWhitelisted' | 'ResourceDoesNotExist' | 'ResourceAlreadyExists' | 'SignatureInvalid' | 'MustBeMaintainer' | 'ProposalAlreadyExists' | 'CallNotConsistentWithProposalData' | 'CallDoesNotMatchResourceId' | 'IncorrectExecutionChainIdType' | 'InvalidNonce' | 'InvalidProposalData' | 'InvalidCall' | 'StringLimitExceeded';
  }

  /** @name WebbPrimitivesVanchorVAnchorMetadata (599) */
  interface WebbPrimitivesVanchorVAnchorMetadata extends Struct {
    readonly creator: Option<AccountId32>;
    readonly asset: u32;
  }

  /** @name PalletVanchorError (600) */
  interface PalletVanchorError extends Enum {
    readonly isInvalidTransactionProof: boolean;
    readonly isNoVAnchorFound: boolean;
    readonly isAlreadyRevealedNullifier: boolean;
    readonly isInvalidExtAmount: boolean;
    readonly isInvalidDepositAmount: boolean;
    readonly isInvalidWithdrawAmount: boolean;
    readonly isInvalidExtData: boolean;
    readonly isInvalidInputNullifiers: boolean;
    readonly isInvalidFee: boolean;
    readonly isInvalidPublicAmount: boolean;
    readonly isInvalidNonce: boolean;
    readonly type: 'InvalidTransactionProof' | 'NoVAnchorFound' | 'AlreadyRevealedNullifier' | 'InvalidExtAmount' | 'InvalidDepositAmount' | 'InvalidWithdrawAmount' | 'InvalidExtData' | 'InvalidInputNullifiers' | 'InvalidFee' | 'InvalidPublicAmount' | 'InvalidNonce';
  }

  /** @name PalletVanchorHandlerError (601) */
  interface PalletVanchorHandlerError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isResourceIsAlreadyAnchored: boolean;
    readonly isAnchorHandlerNotFound: boolean;
    readonly isSourceChainIdNotFound: boolean;
    readonly isStorageOverflow: boolean;
    readonly isInvalidNonce: boolean;
    readonly isInvalidResourceId: boolean;
    readonly type: 'InvalidPermissions' | 'ResourceIsAlreadyAnchored' | 'AnchorHandlerNotFound' | 'SourceChainIdNotFound' | 'StorageOverflow' | 'InvalidNonce' | 'InvalidResourceId';
  }

  /** @name PalletTokenWrapperHandlerError (602) */
  interface PalletTokenWrapperHandlerError extends Enum {
    readonly isInvalidPermissions: boolean;
    readonly isResourceIsAlreadyAnchored: boolean;
    readonly isTokenWrapperHandlerNotFound: boolean;
    readonly isStorageOverflow: boolean;
    readonly type: 'InvalidPermissions' | 'ResourceIsAlreadyAnchored' | 'TokenWrapperHandlerNotFound' | 'StorageOverflow';
  }

  /** @name PalletKeyStorageError (606) */
  interface PalletKeyStorageError extends Enum {
    readonly isPubKeyOutOfBounds: boolean;
    readonly isMaxPubkeyOwnersExceeded: boolean;
    readonly type: 'PubKeyOutOfBounds' | 'MaxPubkeyOwnersExceeded';
  }

  /** @name PalletVanchorVerifierError (607) */
  interface PalletVanchorVerifierError extends Enum {
    readonly isVerifyingParametersNotInitialized: boolean;
    readonly isVerifyError: boolean;
    readonly type: 'VerifyingParametersNotInitialized' | 'VerifyError';
  }

  /** @name PalletTransactionPauseModuleError (608) */
  interface PalletTransactionPauseModuleError extends Enum {
    readonly isCannotPause: boolean;
    readonly isInvalidCharacter: boolean;
    readonly type: 'CannotPause' | 'InvalidCharacter';
  }

  /** @name PalletImOnlineBoundedOpaqueNetworkState (612) */
  interface PalletImOnlineBoundedOpaqueNetworkState extends Struct {
    readonly peerId: Bytes;
    readonly externalAddresses: Vec<Bytes>;
  }

  /** @name PalletImOnlineError (616) */
  interface PalletImOnlineError extends Enum {
    readonly isInvalidKey: boolean;
    readonly isDuplicatedHeartbeat: boolean;
    readonly type: 'InvalidKey' | 'DuplicatedHeartbeat';
  }

  /** @name PalletIdentityRegistration (617) */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityIdentityInfo;
  }

  /** @name PalletIdentityRegistrarInfo (625) */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId32;
    readonly fee: u128;
    readonly fields: PalletIdentityBitFlags;
  }

  /** @name PalletIdentityError (627) */
  interface PalletIdentityError extends Enum {
    readonly isTooManySubAccounts: boolean;
    readonly isNotFound: boolean;
    readonly isNotNamed: boolean;
    readonly isEmptyIndex: boolean;
    readonly isFeeChanged: boolean;
    readonly isNoIdentity: boolean;
    readonly isStickyJudgement: boolean;
    readonly isJudgementGiven: boolean;
    readonly isInvalidJudgement: boolean;
    readonly isInvalidIndex: boolean;
    readonly isInvalidTarget: boolean;
    readonly isTooManyFields: boolean;
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly isJudgementForDifferentIdentity: boolean;
    readonly isJudgementPaymentFailed: boolean;
    readonly type: 'TooManySubAccounts' | 'NotFound' | 'NotNamed' | 'EmptyIndex' | 'FeeChanged' | 'NoIdentity' | 'StickyJudgement' | 'JudgementGiven' | 'InvalidJudgement' | 'InvalidIndex' | 'InvalidTarget' | 'TooManyFields' | 'TooManyRegistrars' | 'AlreadyClaimed' | 'NotSub' | 'NotOwned' | 'JudgementForDifferentIdentity' | 'JudgementPaymentFailed';
  }

  /** @name PalletUtilityError (628) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: 'TooManyCalls';
  }

  /** @name PalletEth2LightClientError (632) */
  interface PalletEth2LightClientError extends Enum {
    readonly isAlreadyInitialized: boolean;
    readonly isSubmitterAlreadyRegistered: boolean;
    readonly isSubmitterNotRegistered: boolean;
    readonly isSubmitterHasUsedStorage: boolean;
    readonly isLightClientUpdateNotAllowed: boolean;
    readonly isBlockAlreadySubmitted: boolean;
    readonly isUnknownParentHeader: boolean;
    readonly isNotTrustedSigner: boolean;
    readonly isValidateUpdatesParameterError: boolean;
    readonly isTrustlessModeError: boolean;
    readonly isInvalidSyncCommitteeBitsSum: boolean;
    readonly isSyncCommitteeBitsSumLessThanThreshold: boolean;
    readonly isForkVersionNotFound: boolean;
    readonly isForkEpochNotFound: boolean;
    readonly isGenesisValidatorsRootNotFound: boolean;
    readonly isInvalidBlsSignature: boolean;
    readonly isInvalidExecutionBlock: boolean;
    readonly isActiveHeaderSlotNumberLessThanFinalizedSlot: boolean;
    readonly isInvalidUpdatePeriod: boolean;
    readonly isInvalidFinalityProof: boolean;
    readonly isInvalidExecutionBlockHashProof: boolean;
    readonly isNextSyncCommitteeNotPresent: boolean;
    readonly isInvalidNextSyncCommitteeProof: boolean;
    readonly isFinalizedExecutionHeaderNotPresent: boolean;
    readonly isFinalizedBeaconHeaderNotPresent: boolean;
    readonly isUnfinalizedHeaderNotPresent: boolean;
    readonly isSyncCommitteeUpdateNotPresent: boolean;
    readonly isSubmitterExhaustedLimit: boolean;
    readonly isHeaderHashDoesNotExist: boolean;
    readonly isBlockHashesDoNotMatch: boolean;
    readonly isInvalidSignaturePeriod: boolean;
    readonly isCurrentSyncCommitteeNotSet: boolean;
    readonly isNextSyncCommitteeNotSet: boolean;
    readonly type: 'AlreadyInitialized' | 'SubmitterAlreadyRegistered' | 'SubmitterNotRegistered' | 'SubmitterHasUsedStorage' | 'LightClientUpdateNotAllowed' | 'BlockAlreadySubmitted' | 'UnknownParentHeader' | 'NotTrustedSigner' | 'ValidateUpdatesParameterError' | 'TrustlessModeError' | 'InvalidSyncCommitteeBitsSum' | 'SyncCommitteeBitsSumLessThanThreshold' | 'ForkVersionNotFound' | 'ForkEpochNotFound' | 'GenesisValidatorsRootNotFound' | 'InvalidBlsSignature' | 'InvalidExecutionBlock' | 'ActiveHeaderSlotNumberLessThanFinalizedSlot' | 'InvalidUpdatePeriod' | 'InvalidFinalityProof' | 'InvalidExecutionBlockHashProof' | 'NextSyncCommitteeNotPresent' | 'InvalidNextSyncCommitteeProof' | 'FinalizedExecutionHeaderNotPresent' | 'FinalizedBeaconHeaderNotPresent' | 'UnfinalizedHeaderNotPresent' | 'SyncCommitteeUpdateNotPresent' | 'SubmitterExhaustedLimit' | 'HeaderHashDoesNotExist' | 'BlockHashesDoNotMatch' | 'InvalidSignaturePeriod' | 'CurrentSyncCommitteeNotSet' | 'NextSyncCommitteeNotSet';
  }

  /** @name SpRuntimeMultiSignature (634) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: SpCoreEd25519Signature;
    readonly isSr25519: boolean;
    readonly asSr25519: SpCoreSr25519Signature;
    readonly isEcdsa: boolean;
    readonly asEcdsa: SpCoreEcdsaSignature;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa';
  }

  /** @name SpCoreEcdsaSignature (635) */
  interface SpCoreEcdsaSignature extends U8aFixed {}

  /** @name FrameSystemExtensionsCheckNonZeroSender (637) */
  type FrameSystemExtensionsCheckNonZeroSender = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (638) */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (639) */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (640) */
  type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (643) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (644) */
  type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (645) */
  interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name TangleStandaloneRuntimeRuntime (646) */
  type TangleStandaloneRuntimeRuntime = Null;

} // declare module
