// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/types/lookup';

import type { Data } from '@polkadot/types';
import type { BTreeMap, BTreeSet, Bytes, Compact, Enum, Null, Option, Result, Struct, Text, U256, U8aFixed, Vec, bool, i16, i32, i64, i8, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { Vote } from '@polkadot/types/interfaces/elections';
import type { AccountId32, Call, H160, H256, MultiAddress, PerU16, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
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
    readonly frozen: u128;
    readonly flags: u128;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (9) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /** @name SpWeightsWeightV2Weight (10) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /** @name SpRuntimeDigest (15) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (17) */
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

  /** @name FrameSystemEventRecord (20) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (22) */
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
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256;
      readonly checkVersion: bool;
    } & Struct;
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked' | 'UpgradeAuthorized';
  }

  /** @name FrameSupportDispatchDispatchInfo (23) */
  interface FrameSupportDispatchDispatchInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name FrameSupportDispatchDispatchClass (24) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportDispatchPays (25) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (26) */
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
    readonly isRootNotAllowed: boolean;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'TooManyConsumers' | 'Token' | 'Arithmetic' | 'Transactional' | 'Exhausted' | 'Corruption' | 'Unavailable' | 'RootNotAllowed';
  }

  /** @name SpRuntimeModuleError (27) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (28) */
  interface SpRuntimeTokenError extends Enum {
    readonly isFundsUnavailable: boolean;
    readonly isOnlyProvider: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly isCannotCreateHold: boolean;
    readonly isNotExpendable: boolean;
    readonly isBlocked: boolean;
    readonly type: 'FundsUnavailable' | 'OnlyProvider' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported' | 'CannotCreateHold' | 'NotExpendable' | 'Blocked';
  }

  /** @name SpArithmeticArithmeticError (29) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name SpRuntimeTransactionalError (30) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: 'LimitReached' | 'NoLayer';
  }

  /** @name PalletSudoEvent (31) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly old: Option<AccountId32>;
      readonly new_: AccountId32;
    } & Struct;
    readonly isKeyRemoved: boolean;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'Sudid' | 'KeyChanged' | 'KeyRemoved' | 'SudoAsDone';
  }

  /** @name PalletAssetsEvent (35) */
  interface PalletAssetsEvent extends Enum {
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly assetId: u128;
      readonly creator: AccountId32;
      readonly owner: AccountId32;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly assetId: u128;
      readonly owner: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransferred: boolean;
    readonly asTransferred: {
      readonly assetId: u128;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly assetId: u128;
      readonly owner: AccountId32;
      readonly balance: u128;
    } & Struct;
    readonly isTeamChanged: boolean;
    readonly asTeamChanged: {
      readonly assetId: u128;
      readonly issuer: AccountId32;
      readonly admin: AccountId32;
      readonly freezer: AccountId32;
    } & Struct;
    readonly isOwnerChanged: boolean;
    readonly asOwnerChanged: {
      readonly assetId: u128;
      readonly owner: AccountId32;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly assetId: u128;
      readonly who: AccountId32;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly assetId: u128;
      readonly who: AccountId32;
    } & Struct;
    readonly isAssetFrozen: boolean;
    readonly asAssetFrozen: {
      readonly assetId: u128;
    } & Struct;
    readonly isAssetThawed: boolean;
    readonly asAssetThawed: {
      readonly assetId: u128;
    } & Struct;
    readonly isAccountsDestroyed: boolean;
    readonly asAccountsDestroyed: {
      readonly assetId: u128;
      readonly accountsDestroyed: u32;
      readonly accountsRemaining: u32;
    } & Struct;
    readonly isApprovalsDestroyed: boolean;
    readonly asApprovalsDestroyed: {
      readonly assetId: u128;
      readonly approvalsDestroyed: u32;
      readonly approvalsRemaining: u32;
    } & Struct;
    readonly isDestructionStarted: boolean;
    readonly asDestructionStarted: {
      readonly assetId: u128;
    } & Struct;
    readonly isDestroyed: boolean;
    readonly asDestroyed: {
      readonly assetId: u128;
    } & Struct;
    readonly isForceCreated: boolean;
    readonly asForceCreated: {
      readonly assetId: u128;
      readonly owner: AccountId32;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly assetId: u128;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
      readonly isFrozen: bool;
    } & Struct;
    readonly isMetadataCleared: boolean;
    readonly asMetadataCleared: {
      readonly assetId: u128;
    } & Struct;
    readonly isApprovedTransfer: boolean;
    readonly asApprovedTransfer: {
      readonly assetId: u128;
      readonly source: AccountId32;
      readonly delegate: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isApprovalCancelled: boolean;
    readonly asApprovalCancelled: {
      readonly assetId: u128;
      readonly owner: AccountId32;
      readonly delegate: AccountId32;
    } & Struct;
    readonly isTransferredApproved: boolean;
    readonly asTransferredApproved: {
      readonly assetId: u128;
      readonly owner: AccountId32;
      readonly delegate: AccountId32;
      readonly destination: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isAssetStatusChanged: boolean;
    readonly asAssetStatusChanged: {
      readonly assetId: u128;
    } & Struct;
    readonly isAssetMinBalanceChanged: boolean;
    readonly asAssetMinBalanceChanged: {
      readonly assetId: u128;
      readonly newMinBalance: u128;
    } & Struct;
    readonly isTouched: boolean;
    readonly asTouched: {
      readonly assetId: u128;
      readonly who: AccountId32;
      readonly depositor: AccountId32;
    } & Struct;
    readonly isBlocked: boolean;
    readonly asBlocked: {
      readonly assetId: u128;
      readonly who: AccountId32;
    } & Struct;
    readonly isDeposited: boolean;
    readonly asDeposited: {
      readonly assetId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly assetId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Created' | 'Issued' | 'Transferred' | 'Burned' | 'TeamChanged' | 'OwnerChanged' | 'Frozen' | 'Thawed' | 'AssetFrozen' | 'AssetThawed' | 'AccountsDestroyed' | 'ApprovalsDestroyed' | 'DestructionStarted' | 'Destroyed' | 'ForceCreated' | 'MetadataSet' | 'MetadataCleared' | 'ApprovedTransfer' | 'ApprovalCancelled' | 'TransferredApproved' | 'AssetStatusChanged' | 'AssetMinBalanceChanged' | 'Touched' | 'Blocked' | 'Deposited' | 'Withdrawn';
  }

  /** @name PalletBalancesEvent (36) */
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
    readonly isMinted: boolean;
    readonly asMinted: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSuspended: boolean;
    readonly asSuspended: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isRestored: boolean;
    readonly asRestored: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUpgraded: boolean;
    readonly asUpgraded: {
      readonly who: AccountId32;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly amount: u128;
    } & Struct;
    readonly isRescinded: boolean;
    readonly asRescinded: {
      readonly amount: u128;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTotalIssuanceForced: boolean;
    readonly asTotalIssuanceForced: {
      readonly old: u128;
      readonly new_: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'BalanceSet' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'Deposit' | 'Withdraw' | 'Slashed' | 'Minted' | 'Burned' | 'Suspended' | 'Restored' | 'Upgraded' | 'Issued' | 'Rescinded' | 'Locked' | 'Unlocked' | 'Frozen' | 'Thawed' | 'TotalIssuanceForced';
  }

  /** @name FrameSupportTokensMiscBalanceStatus (37) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PalletTransactionPaymentEvent (38) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId32;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: 'TransactionFeePaid';
  }

  /** @name PalletGrandpaEvent (39) */
  interface PalletGrandpaEvent extends Enum {
    readonly isNewAuthorities: boolean;
    readonly asNewAuthorities: {
      readonly authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>;
    } & Struct;
    readonly isPaused: boolean;
    readonly isResumed: boolean;
    readonly type: 'NewAuthorities' | 'Paused' | 'Resumed';
  }

  /** @name SpConsensusGrandpaAppPublic (42) */
  interface SpConsensusGrandpaAppPublic extends U8aFixed {}

  /** @name PalletIndicesEvent (43) */
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

  /** @name PalletDemocracyEvent (44) */
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
      readonly until: u64;
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

  /** @name PalletDemocracyVoteThreshold (45) */
  interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean;
    readonly isSuperMajorityAgainst: boolean;
    readonly isSimpleMajority: boolean;
    readonly type: 'SuperMajorityApprove' | 'SuperMajorityAgainst' | 'SimpleMajority';
  }

  /** @name PalletDemocracyVoteAccountVote (46) */
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

  /** @name PalletDemocracyMetadataOwner (48) */
  interface PalletDemocracyMetadataOwner extends Enum {
    readonly isExternal: boolean;
    readonly isProposal: boolean;
    readonly asProposal: u32;
    readonly isReferendum: boolean;
    readonly asReferendum: u32;
    readonly type: 'External' | 'Proposal' | 'Referendum';
  }

  /** @name PalletCollectiveEvent (49) */
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

  /** @name PalletVestingEvent (50) */
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

  /** @name PalletElectionsPhragmenEvent (51) */
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

  /** @name PalletElectionProviderMultiPhaseEvent (54) */
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

  /** @name PalletElectionProviderMultiPhaseElectionCompute (55) */
  interface PalletElectionProviderMultiPhaseElectionCompute extends Enum {
    readonly isOnChain: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly isFallback: boolean;
    readonly isEmergency: boolean;
    readonly type: 'OnChain' | 'Signed' | 'Unsigned' | 'Fallback' | 'Emergency';
  }

  /** @name SpNposElectionsElectionScore (56) */
  interface SpNposElectionsElectionScore extends Struct {
    readonly minimalStake: u128;
    readonly sumStake: u128;
    readonly sumStakeSquared: u128;
  }

  /** @name PalletElectionProviderMultiPhasePhase (57) */
  interface PalletElectionProviderMultiPhasePhase extends Enum {
    readonly isOff: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly asUnsigned: ITuple<[bool, u64]>;
    readonly isEmergency: boolean;
    readonly type: 'Off' | 'Signed' | 'Unsigned' | 'Emergency';
  }

  /** @name PalletStakingPalletEvent (59) */
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
      readonly dest: PalletStakingRewardDestination;
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
    readonly isSnapshotVotersSizeExceeded: boolean;
    readonly asSnapshotVotersSizeExceeded: {
      readonly size_: u32;
    } & Struct;
    readonly isSnapshotTargetsSizeExceeded: boolean;
    readonly asSnapshotTargetsSizeExceeded: {
      readonly size_: u32;
    } & Struct;
    readonly isForceEra: boolean;
    readonly asForceEra: {
      readonly mode: PalletStakingForcing;
    } & Struct;
    readonly isControllerBatchDeprecated: boolean;
    readonly asControllerBatchDeprecated: {
      readonly failures: u32;
    } & Struct;
    readonly type: 'EraPaid' | 'Rewarded' | 'Slashed' | 'SlashReported' | 'OldSlashingReportDiscarded' | 'StakersElected' | 'Bonded' | 'Unbonded' | 'Withdrawn' | 'Kicked' | 'StakingElectionFailed' | 'Chilled' | 'PayoutStarted' | 'ValidatorPrefsSet' | 'SnapshotVotersSizeExceeded' | 'SnapshotTargetsSizeExceeded' | 'ForceEra' | 'ControllerBatchDeprecated';
  }

  /** @name PalletStakingRewardDestination (60) */
  interface PalletStakingRewardDestination extends Enum {
    readonly isStaked: boolean;
    readonly isStash: boolean;
    readonly isController: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Staked' | 'Stash' | 'Controller' | 'Account' | 'None';
  }

  /** @name PalletStakingValidatorPrefs (62) */
  interface PalletStakingValidatorPrefs extends Struct {
    readonly commission: Compact<Perbill>;
    readonly blocked: bool;
  }

  /** @name PalletStakingForcing (64) */
  interface PalletStakingForcing extends Enum {
    readonly isNotForcing: boolean;
    readonly isForceNew: boolean;
    readonly isForceNone: boolean;
    readonly isForceAlways: boolean;
    readonly type: 'NotForcing' | 'ForceNew' | 'ForceNone' | 'ForceAlways';
  }

  /** @name PalletSessionEvent (65) */
  interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean;
    readonly asNewSession: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly type: 'NewSession';
  }

  /** @name PalletTreasuryEvent (66) */
  interface PalletTreasuryEvent extends Enum {
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
    readonly isAssetSpendApproved: boolean;
    readonly asAssetSpendApproved: {
      readonly index: u32;
      readonly assetKind: Null;
      readonly amount: u128;
      readonly beneficiary: AccountId32;
      readonly validFrom: u64;
      readonly expireAt: u64;
    } & Struct;
    readonly isAssetSpendVoided: boolean;
    readonly asAssetSpendVoided: {
      readonly index: u32;
    } & Struct;
    readonly isPaid: boolean;
    readonly asPaid: {
      readonly index: u32;
      readonly paymentId: Null;
    } & Struct;
    readonly isPaymentFailed: boolean;
    readonly asPaymentFailed: {
      readonly index: u32;
      readonly paymentId: Null;
    } & Struct;
    readonly isSpendProcessed: boolean;
    readonly asSpendProcessed: {
      readonly index: u32;
    } & Struct;
    readonly type: 'Spending' | 'Awarded' | 'Burnt' | 'Rollover' | 'Deposit' | 'SpendApproved' | 'UpdatedInactive' | 'AssetSpendApproved' | 'AssetSpendVoided' | 'Paid' | 'PaymentFailed' | 'SpendProcessed';
  }

  /** @name PalletBountiesEvent (67) */
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
    readonly isBountyApproved: boolean;
    readonly asBountyApproved: {
      readonly index: u32;
    } & Struct;
    readonly isCuratorProposed: boolean;
    readonly asCuratorProposed: {
      readonly bountyId: u32;
      readonly curator: AccountId32;
    } & Struct;
    readonly isCuratorUnassigned: boolean;
    readonly asCuratorUnassigned: {
      readonly bountyId: u32;
    } & Struct;
    readonly isCuratorAccepted: boolean;
    readonly asCuratorAccepted: {
      readonly bountyId: u32;
      readonly curator: AccountId32;
    } & Struct;
    readonly type: 'BountyProposed' | 'BountyRejected' | 'BountyBecameActive' | 'BountyAwarded' | 'BountyClaimed' | 'BountyCanceled' | 'BountyExtended' | 'BountyApproved' | 'CuratorProposed' | 'CuratorUnassigned' | 'CuratorAccepted';
  }

  /** @name PalletChildBountiesEvent (68) */
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

  /** @name PalletBagsListEvent (69) */
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

  /** @name PalletNominationPoolsEvent (70) */
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
    readonly isPoolCommissionUpdated: boolean;
    readonly asPoolCommissionUpdated: {
      readonly poolId: u32;
      readonly current: Option<ITuple<[Perbill, AccountId32]>>;
    } & Struct;
    readonly isPoolMaxCommissionUpdated: boolean;
    readonly asPoolMaxCommissionUpdated: {
      readonly poolId: u32;
      readonly maxCommission: Perbill;
    } & Struct;
    readonly isPoolCommissionChangeRateUpdated: boolean;
    readonly asPoolCommissionChangeRateUpdated: {
      readonly poolId: u32;
      readonly changeRate: PalletNominationPoolsCommissionChangeRate;
    } & Struct;
    readonly isPoolCommissionClaimPermissionUpdated: boolean;
    readonly asPoolCommissionClaimPermissionUpdated: {
      readonly poolId: u32;
      readonly permission: Option<PalletNominationPoolsCommissionClaimPermission>;
    } & Struct;
    readonly isPoolCommissionClaimed: boolean;
    readonly asPoolCommissionClaimed: {
      readonly poolId: u32;
      readonly commission: u128;
    } & Struct;
    readonly isMinBalanceDeficitAdjusted: boolean;
    readonly asMinBalanceDeficitAdjusted: {
      readonly poolId: u32;
      readonly amount: u128;
    } & Struct;
    readonly isMinBalanceExcessAdjusted: boolean;
    readonly asMinBalanceExcessAdjusted: {
      readonly poolId: u32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Created' | 'Bonded' | 'PaidOut' | 'Unbonded' | 'Withdrawn' | 'Destroyed' | 'StateChanged' | 'MemberRemoved' | 'RolesUpdated' | 'PoolSlashed' | 'UnbondingPoolSlashed' | 'PoolCommissionUpdated' | 'PoolMaxCommissionUpdated' | 'PoolCommissionChangeRateUpdated' | 'PoolCommissionClaimPermissionUpdated' | 'PoolCommissionClaimed' | 'MinBalanceDeficitAdjusted' | 'MinBalanceExcessAdjusted';
  }

  /** @name PalletNominationPoolsPoolState (71) */
  interface PalletNominationPoolsPoolState extends Enum {
    readonly isOpen: boolean;
    readonly isBlocked: boolean;
    readonly isDestroying: boolean;
    readonly type: 'Open' | 'Blocked' | 'Destroying';
  }

  /** @name PalletNominationPoolsCommissionChangeRate (74) */
  interface PalletNominationPoolsCommissionChangeRate extends Struct {
    readonly maxIncrease: Perbill;
    readonly minDelay: u64;
  }

  /** @name PalletNominationPoolsCommissionClaimPermission (76) */
  interface PalletNominationPoolsCommissionClaimPermission extends Enum {
    readonly isPermissionless: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Permissionless' | 'Account';
  }

  /** @name PalletSchedulerEvent (77) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: {
      readonly when: u64;
      readonly index: u32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly when: u64;
      readonly index: u32;
    } & Struct;
    readonly isDispatched: boolean;
    readonly asDispatched: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isRetrySet: boolean;
    readonly asRetrySet: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
      readonly period: u64;
      readonly retries: u8;
    } & Struct;
    readonly isRetryCancelled: boolean;
    readonly asRetryCancelled: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isCallUnavailable: boolean;
    readonly asCallUnavailable: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPeriodicFailed: boolean;
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isRetryFailed: boolean;
    readonly asRetryFailed: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u64, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched' | 'RetrySet' | 'RetryCancelled' | 'CallUnavailable' | 'PeriodicFailed' | 'RetryFailed' | 'PermanentlyOverweight';
  }

  /** @name PalletPreimageEvent (80) */
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

  /** @name PalletOffencesEvent (81) */
  interface PalletOffencesEvent extends Enum {
    readonly isOffence: boolean;
    readonly asOffence: {
      readonly kind: U8aFixed;
      readonly timeslot: Bytes;
    } & Struct;
    readonly type: 'Offence';
  }

  /** @name PalletTxPauseEvent (83) */
  interface PalletTxPauseEvent extends Enum {
    readonly isCallPaused: boolean;
    readonly asCallPaused: {
      readonly fullName: ITuple<[Bytes, Bytes]>;
    } & Struct;
    readonly isCallUnpaused: boolean;
    readonly asCallUnpaused: {
      readonly fullName: ITuple<[Bytes, Bytes]>;
    } & Struct;
    readonly type: 'CallPaused' | 'CallUnpaused';
  }

  /** @name PalletImOnlineEvent (86) */
  interface PalletImOnlineEvent extends Enum {
    readonly isHeartbeatReceived: boolean;
    readonly asHeartbeatReceived: {
      readonly authorityId: PalletImOnlineSr25519AppSr25519Public;
    } & Struct;
    readonly isAllGood: boolean;
    readonly isSomeOffline: boolean;
    readonly asSomeOffline: {
      readonly offline: Vec<ITuple<[AccountId32, SpStakingExposure]>>;
    } & Struct;
    readonly type: 'HeartbeatReceived' | 'AllGood' | 'SomeOffline';
  }

  /** @name PalletImOnlineSr25519AppSr25519Public (87) */
  interface PalletImOnlineSr25519AppSr25519Public extends U8aFixed {}

  /** @name SpStakingExposure (90) */
  interface SpStakingExposure extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly others: Vec<SpStakingIndividualExposure>;
  }

  /** @name SpStakingIndividualExposure (93) */
  interface SpStakingIndividualExposure extends Struct {
    readonly who: AccountId32;
    readonly value: Compact<u128>;
  }

  /** @name PalletIdentityEvent (94) */
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
    readonly isAuthorityAdded: boolean;
    readonly asAuthorityAdded: {
      readonly authority: AccountId32;
    } & Struct;
    readonly isAuthorityRemoved: boolean;
    readonly asAuthorityRemoved: {
      readonly authority: AccountId32;
    } & Struct;
    readonly isUsernameSet: boolean;
    readonly asUsernameSet: {
      readonly who: AccountId32;
      readonly username: Bytes;
    } & Struct;
    readonly isUsernameQueued: boolean;
    readonly asUsernameQueued: {
      readonly who: AccountId32;
      readonly username: Bytes;
      readonly expiration: u64;
    } & Struct;
    readonly isPreapprovalExpired: boolean;
    readonly asPreapprovalExpired: {
      readonly whose: AccountId32;
    } & Struct;
    readonly isPrimaryUsernameSet: boolean;
    readonly asPrimaryUsernameSet: {
      readonly who: AccountId32;
      readonly username: Bytes;
    } & Struct;
    readonly isDanglingUsernameRemoved: boolean;
    readonly asDanglingUsernameRemoved: {
      readonly who: AccountId32;
      readonly username: Bytes;
    } & Struct;
    readonly type: 'IdentitySet' | 'IdentityCleared' | 'IdentityKilled' | 'JudgementRequested' | 'JudgementUnrequested' | 'JudgementGiven' | 'RegistrarAdded' | 'SubIdentityAdded' | 'SubIdentityRemoved' | 'SubIdentityRevoked' | 'AuthorityAdded' | 'AuthorityRemoved' | 'UsernameSet' | 'UsernameQueued' | 'PreapprovalExpired' | 'PrimaryUsernameSet' | 'DanglingUsernameRemoved';
  }

  /** @name PalletUtilityEvent (96) */
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

  /** @name PalletMultisigEvent (97) */
  interface PalletMultisigEvent extends Enum {
    readonly isNewMultisig: boolean;
    readonly asNewMultisig: {
      readonly approving: AccountId32;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigApproval: boolean;
    readonly asMultisigApproval: {
      readonly approving: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigExecuted: boolean;
    readonly asMultisigExecuted: {
      readonly approving: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMultisigCancelled: boolean;
    readonly asMultisigCancelled: {
      readonly cancelling: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: 'NewMultisig' | 'MultisigApproval' | 'MultisigExecuted' | 'MultisigCancelled';
  }

  /** @name PalletMultisigTimepoint (98) */
  interface PalletMultisigTimepoint extends Struct {
    readonly height: u64;
    readonly index: u32;
  }

  /** @name PalletEthereumEvent (99) */
  interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly from: H160;
      readonly to: H160;
      readonly transactionHash: H256;
      readonly exitReason: EvmCoreErrorExitReason;
      readonly extraData: Bytes;
    } & Struct;
    readonly type: 'Executed';
  }

  /** @name EvmCoreErrorExitReason (102) */
  interface EvmCoreErrorExitReason extends Enum {
    readonly isSucceed: boolean;
    readonly asSucceed: EvmCoreErrorExitSucceed;
    readonly isError: boolean;
    readonly asError: EvmCoreErrorExitError;
    readonly isRevert: boolean;
    readonly asRevert: EvmCoreErrorExitRevert;
    readonly isFatal: boolean;
    readonly asFatal: EvmCoreErrorExitFatal;
    readonly type: 'Succeed' | 'Error' | 'Revert' | 'Fatal';
  }

  /** @name EvmCoreErrorExitSucceed (103) */
  interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean;
    readonly isReturned: boolean;
    readonly isSuicided: boolean;
    readonly type: 'Stopped' | 'Returned' | 'Suicided';
  }

  /** @name EvmCoreErrorExitError (104) */
  interface EvmCoreErrorExitError extends Enum {
    readonly isStackUnderflow: boolean;
    readonly isStackOverflow: boolean;
    readonly isInvalidJump: boolean;
    readonly isInvalidRange: boolean;
    readonly isDesignatedInvalid: boolean;
    readonly isCallTooDeep: boolean;
    readonly isCreateCollision: boolean;
    readonly isCreateContractLimit: boolean;
    readonly isOutOfOffset: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutOfFund: boolean;
    readonly isPcUnderflow: boolean;
    readonly isCreateEmpty: boolean;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly isMaxNonce: boolean;
    readonly isInvalidCode: boolean;
    readonly asInvalidCode: u8;
    readonly type: 'StackUnderflow' | 'StackOverflow' | 'InvalidJump' | 'InvalidRange' | 'DesignatedInvalid' | 'CallTooDeep' | 'CreateCollision' | 'CreateContractLimit' | 'OutOfOffset' | 'OutOfGas' | 'OutOfFund' | 'PcUnderflow' | 'CreateEmpty' | 'Other' | 'MaxNonce' | 'InvalidCode';
  }

  /** @name EvmCoreErrorExitRevert (108) */
  interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean;
    readonly type: 'Reverted';
  }

  /** @name EvmCoreErrorExitFatal (109) */
  interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean;
    readonly isUnhandledInterrupt: boolean;
    readonly isCallErrorAsFatal: boolean;
    readonly asCallErrorAsFatal: EvmCoreErrorExitError;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: 'NotSupported' | 'UnhandledInterrupt' | 'CallErrorAsFatal' | 'Other';
  }

  /** @name PalletEvmEvent (110) */
  interface PalletEvmEvent extends Enum {
    readonly isLog: boolean;
    readonly asLog: {
      readonly log: EthereumLog;
    } & Struct;
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly address: H160;
    } & Struct;
    readonly isCreatedFailed: boolean;
    readonly asCreatedFailed: {
      readonly address: H160;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly address: H160;
    } & Struct;
    readonly isExecutedFailed: boolean;
    readonly asExecutedFailed: {
      readonly address: H160;
    } & Struct;
    readonly type: 'Log' | 'Created' | 'CreatedFailed' | 'Executed' | 'ExecutedFailed';
  }

  /** @name EthereumLog (111) */
  interface EthereumLog extends Struct {
    readonly address: H160;
    readonly topics: Vec<H256>;
    readonly data: Bytes;
  }

  /** @name PalletBaseFeeEvent (113) */
  interface PalletBaseFeeEvent extends Enum {
    readonly isNewBaseFeePerGas: boolean;
    readonly asNewBaseFeePerGas: {
      readonly fee: U256;
    } & Struct;
    readonly isBaseFeeOverflow: boolean;
    readonly isNewElasticity: boolean;
    readonly asNewElasticity: {
      readonly elasticity: Permill;
    } & Struct;
    readonly type: 'NewBaseFeePerGas' | 'BaseFeeOverflow' | 'NewElasticity';
  }

  /** @name PalletAirdropClaimsEvent (117) */
  interface PalletAirdropClaimsEvent extends Enum {
    readonly isClaimed: boolean;
    readonly asClaimed: {
      readonly recipient: AccountId32;
      readonly source: PalletAirdropClaimsUtilsMultiAddress;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Claimed';
  }

  /** @name PalletAirdropClaimsUtilsMultiAddress (118) */
  interface PalletAirdropClaimsUtilsMultiAddress extends Enum {
    readonly isEvm: boolean;
    readonly asEvm: PalletAirdropClaimsUtilsEthereumAddress;
    readonly isNative: boolean;
    readonly asNative: AccountId32;
    readonly type: 'Evm' | 'Native';
  }

  /** @name PalletAirdropClaimsUtilsEthereumAddress (119) */
  interface PalletAirdropClaimsUtilsEthereumAddress extends U8aFixed {}

  /** @name PalletProxyEvent (120) */
  interface PalletProxyEvent extends Enum {
    readonly isProxyExecuted: boolean;
    readonly asProxyExecuted: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isPureCreated: boolean;
    readonly asPureCreated: {
      readonly pure: AccountId32;
      readonly who: AccountId32;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly disambiguationIndex: u16;
    } & Struct;
    readonly isAnnounced: boolean;
    readonly asAnnounced: {
      readonly real: AccountId32;
      readonly proxy: AccountId32;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAdded: boolean;
    readonly asProxyAdded: {
      readonly delegator: AccountId32;
      readonly delegatee: AccountId32;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly delay: u64;
    } & Struct;
    readonly isProxyRemoved: boolean;
    readonly asProxyRemoved: {
      readonly delegator: AccountId32;
      readonly delegatee: AccountId32;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly delay: u64;
    } & Struct;
    readonly type: 'ProxyExecuted' | 'PureCreated' | 'Announced' | 'ProxyAdded' | 'ProxyRemoved';
  }

  /** @name TangleTestnetRuntimeProxyType (121) */
  interface TangleTestnetRuntimeProxyType extends Enum {
    readonly isAny: boolean;
    readonly isNonTransfer: boolean;
    readonly isGovernance: boolean;
    readonly isStaking: boolean;
    readonly type: 'Any' | 'NonTransfer' | 'Governance' | 'Staking';
  }

  /** @name PalletMultiAssetDelegationEvent (123) */
  interface PalletMultiAssetDelegationEvent extends Enum {
    readonly isOperatorJoined: boolean;
    readonly asOperatorJoined: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorLeavingScheduled: boolean;
    readonly asOperatorLeavingScheduled: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorLeaveCancelled: boolean;
    readonly asOperatorLeaveCancelled: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorLeaveExecuted: boolean;
    readonly asOperatorLeaveExecuted: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorBondMore: boolean;
    readonly asOperatorBondMore: {
      readonly who: AccountId32;
      readonly additionalBond: u128;
    } & Struct;
    readonly isOperatorBondLessScheduled: boolean;
    readonly asOperatorBondLessScheduled: {
      readonly who: AccountId32;
      readonly unstakeAmount: u128;
    } & Struct;
    readonly isOperatorBondLessExecuted: boolean;
    readonly asOperatorBondLessExecuted: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorBondLessCancelled: boolean;
    readonly asOperatorBondLessCancelled: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorWentOffline: boolean;
    readonly asOperatorWentOffline: {
      readonly who: AccountId32;
    } & Struct;
    readonly isOperatorWentOnline: boolean;
    readonly asOperatorWentOnline: {
      readonly who: AccountId32;
    } & Struct;
    readonly isDeposited: boolean;
    readonly asDeposited: {
      readonly who: AccountId32;
      readonly amount: u128;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
    } & Struct;
    readonly isScheduledWithdraw: boolean;
    readonly asScheduledWithdraw: {
      readonly who: AccountId32;
      readonly amount: u128;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly when: u32;
    } & Struct;
    readonly isExecutedWithdraw: boolean;
    readonly asExecutedWithdraw: {
      readonly who: AccountId32;
    } & Struct;
    readonly isCancelledWithdraw: boolean;
    readonly asCancelledWithdraw: {
      readonly who: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isDelegated: boolean;
    readonly asDelegated: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
    } & Struct;
    readonly isDelegatorUnstakeScheduled: boolean;
    readonly asDelegatorUnstakeScheduled: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
      readonly when: u32;
    } & Struct;
    readonly isDelegatorUnstakeExecuted: boolean;
    readonly asDelegatorUnstakeExecuted: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isDelegatorUnstakeCancelled: boolean;
    readonly asDelegatorUnstakeCancelled: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isOperatorSlashed: boolean;
    readonly asOperatorSlashed: {
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly era: u32;
    } & Struct;
    readonly isDelegatorSlashed: boolean;
    readonly asDelegatorSlashed: {
      readonly delegator: AccountId32;
      readonly amount: u128;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly era: u32;
    } & Struct;
    readonly isNominatedSlash: boolean;
    readonly asNominatedSlash: {
      readonly delegator: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly era: u32;
    } & Struct;
    readonly isEvmReverted: boolean;
    readonly asEvmReverted: {
      readonly from: H160;
      readonly to: H160;
      readonly data: Bytes;
      readonly reason: Bytes;
    } & Struct;
    readonly isNominationDelegated: boolean;
    readonly asNominationDelegated: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isNominationUnstakeScheduled: boolean;
    readonly asNominationUnstakeScheduled: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly when: u32;
    } & Struct;
    readonly isNominationUnstakeExecuted: boolean;
    readonly asNominationUnstakeExecuted: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isNominationUnstakeCancelled: boolean;
    readonly asNominationUnstakeCancelled: {
      readonly who: AccountId32;
      readonly operator: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'OperatorJoined' | 'OperatorLeavingScheduled' | 'OperatorLeaveCancelled' | 'OperatorLeaveExecuted' | 'OperatorBondMore' | 'OperatorBondLessScheduled' | 'OperatorBondLessExecuted' | 'OperatorBondLessCancelled' | 'OperatorWentOffline' | 'OperatorWentOnline' | 'Deposited' | 'ScheduledWithdraw' | 'ExecutedWithdraw' | 'CancelledWithdraw' | 'Delegated' | 'DelegatorUnstakeScheduled' | 'DelegatorUnstakeExecuted' | 'DelegatorUnstakeCancelled' | 'OperatorSlashed' | 'DelegatorSlashed' | 'NominatedSlash' | 'EvmReverted' | 'NominationDelegated' | 'NominationUnstakeScheduled' | 'NominationUnstakeExecuted' | 'NominationUnstakeCancelled';
  }

  /** @name TanglePrimitivesServicesTypesAssetU128 (124) */
  interface TanglePrimitivesServicesTypesAssetU128 extends Enum {
    readonly isCustom: boolean;
    readonly asCustom: u128;
    readonly isErc20: boolean;
    readonly asErc20: H160;
    readonly type: 'Custom' | 'Erc20';
  }

  /** @name PalletServicesModuleEvent (125) */
  interface PalletServicesModuleEvent extends Enum {
    readonly isBlueprintCreated: boolean;
    readonly asBlueprintCreated: {
      readonly owner: AccountId32;
      readonly blueprintId: u64;
    } & Struct;
    readonly isPreRegistration: boolean;
    readonly asPreRegistration: {
      readonly operator: AccountId32;
      readonly blueprintId: u64;
    } & Struct;
    readonly isRegistered: boolean;
    readonly asRegistered: {
      readonly provider: AccountId32;
      readonly blueprintId: u64;
      readonly preferences: TanglePrimitivesServicesTypesOperatorPreferences;
      readonly registrationArgs: Vec<TanglePrimitivesServicesField>;
    } & Struct;
    readonly isUnregistered: boolean;
    readonly asUnregistered: {
      readonly operator: AccountId32;
      readonly blueprintId: u64;
    } & Struct;
    readonly isServiceRequested: boolean;
    readonly asServiceRequested: {
      readonly owner: AccountId32;
      readonly requestId: u64;
      readonly blueprintId: u64;
      readonly pendingApprovals: Vec<AccountId32>;
      readonly approved: Vec<AccountId32>;
      readonly securityRequirements: Vec<TanglePrimitivesServicesTypesAssetSecurityRequirement>;
    } & Struct;
    readonly isServiceRequestApproved: boolean;
    readonly asServiceRequestApproved: {
      readonly operator: AccountId32;
      readonly requestId: u64;
      readonly blueprintId: u64;
      readonly pendingApprovals: Vec<AccountId32>;
      readonly approved: Vec<AccountId32>;
    } & Struct;
    readonly isServiceRequestRejected: boolean;
    readonly asServiceRequestRejected: {
      readonly operator: AccountId32;
      readonly requestId: u64;
      readonly blueprintId: u64;
    } & Struct;
    readonly isServiceInitiated: boolean;
    readonly asServiceInitiated: {
      readonly owner: AccountId32;
      readonly requestId: u64;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly operatorSecurityCommitments: Vec<ITuple<[AccountId32, Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>]>>;
    } & Struct;
    readonly isServiceTerminated: boolean;
    readonly asServiceTerminated: {
      readonly owner: AccountId32;
      readonly serviceId: u64;
      readonly blueprintId: u64;
    } & Struct;
    readonly isJobCalled: boolean;
    readonly asJobCalled: {
      readonly caller: AccountId32;
      readonly serviceId: u64;
      readonly callId: u64;
      readonly job: u8;
      readonly args: Vec<TanglePrimitivesServicesField>;
    } & Struct;
    readonly isPayOncePaymentProcessed: boolean;
    readonly asPayOncePaymentProcessed: {
      readonly payer: AccountId32;
      readonly serviceId: u64;
      readonly callId: u64;
      readonly jobIndex: u8;
      readonly amount: u128;
    } & Struct;
    readonly isSubscriptionBillingProcessed: boolean;
    readonly asSubscriptionBillingProcessed: {
      readonly subscriber: AccountId32;
      readonly serviceId: u64;
      readonly jobIndex: u8;
      readonly amount: u128;
      readonly blockNumber: u64;
    } & Struct;
    readonly isRewardDistributed: boolean;
    readonly asRewardDistributed: {
      readonly operator: AccountId32;
      readonly serviceId: u64;
      readonly amount: u128;
      readonly pricingModel: TanglePrimitivesServicesTypesPricingModelU64;
    } & Struct;
    readonly isJobResultSubmitted: boolean;
    readonly asJobResultSubmitted: {
      readonly operator: AccountId32;
      readonly serviceId: u64;
      readonly callId: u64;
      readonly job: u8;
      readonly result: Vec<TanglePrimitivesServicesField>;
    } & Struct;
    readonly isEvmReverted: boolean;
    readonly asEvmReverted: {
      readonly from: H160;
      readonly to: H160;
      readonly data: Bytes;
      readonly reason: Bytes;
    } & Struct;
    readonly isUnappliedSlash: boolean;
    readonly asUnappliedSlash: {
      readonly index: u32;
      readonly operator: AccountId32;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly slashPercent: Percent;
      readonly era: u32;
    } & Struct;
    readonly isSlashDiscarded: boolean;
    readonly asSlashDiscarded: {
      readonly index: u32;
      readonly operator: AccountId32;
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly slashPercent: Percent;
      readonly era: u32;
    } & Struct;
    readonly isMasterBlueprintServiceManagerRevised: boolean;
    readonly asMasterBlueprintServiceManagerRevised: {
      readonly revision: u32;
      readonly address: H160;
    } & Struct;
    readonly isRequestForQuote: boolean;
    readonly asRequestForQuote: {
      readonly requester: AccountId32;
      readonly blueprintId: u64;
    } & Struct;
    readonly isRpcAddressUpdated: boolean;
    readonly asRpcAddressUpdated: {
      readonly operator: AccountId32;
      readonly blueprintId: u64;
      readonly rpcAddress: Bytes;
    } & Struct;
    readonly isHeartbeatReceived: boolean;
    readonly asHeartbeatReceived: {
      readonly serviceId: u64;
      readonly blueprintId: u64;
      readonly operator: AccountId32;
      readonly blockNumber: u64;
    } & Struct;
    readonly isDefaultHeartbeatThresholdUpdated: boolean;
    readonly asDefaultHeartbeatThresholdUpdated: {
      readonly threshold: u8;
    } & Struct;
    readonly isDefaultHeartbeatIntervalUpdated: boolean;
    readonly asDefaultHeartbeatIntervalUpdated: {
      readonly interval: u64;
    } & Struct;
    readonly isDefaultHeartbeatSlashingWindowUpdated: boolean;
    readonly asDefaultHeartbeatSlashingWindowUpdated: {
      readonly window: u64;
    } & Struct;
    readonly type: 'BlueprintCreated' | 'PreRegistration' | 'Registered' | 'Unregistered' | 'ServiceRequested' | 'ServiceRequestApproved' | 'ServiceRequestRejected' | 'ServiceInitiated' | 'ServiceTerminated' | 'JobCalled' | 'PayOncePaymentProcessed' | 'SubscriptionBillingProcessed' | 'RewardDistributed' | 'JobResultSubmitted' | 'EvmReverted' | 'UnappliedSlash' | 'SlashDiscarded' | 'MasterBlueprintServiceManagerRevised' | 'RequestForQuote' | 'RpcAddressUpdated' | 'HeartbeatReceived' | 'DefaultHeartbeatThresholdUpdated' | 'DefaultHeartbeatIntervalUpdated' | 'DefaultHeartbeatSlashingWindowUpdated';
  }

  /** @name TanglePrimitivesServicesTypesOperatorPreferences (126) */
  interface TanglePrimitivesServicesTypesOperatorPreferences extends Struct {
    readonly key: U8aFixed;
    readonly rpcAddress: Bytes;
  }

  /** @name TanglePrimitivesServicesField (131) */
  interface TanglePrimitivesServicesField extends Enum {
    readonly isOptional: boolean;
    readonly asOptional: ITuple<[TanglePrimitivesServicesFieldFieldType, Option<TanglePrimitivesServicesField>]>;
    readonly isBool: boolean;
    readonly asBool: bool;
    readonly isUint8: boolean;
    readonly asUint8: u8;
    readonly isInt8: boolean;
    readonly asInt8: i8;
    readonly isUint16: boolean;
    readonly asUint16: u16;
    readonly isInt16: boolean;
    readonly asInt16: i16;
    readonly isUint32: boolean;
    readonly asUint32: u32;
    readonly isInt32: boolean;
    readonly asInt32: i32;
    readonly isUint64: boolean;
    readonly asUint64: u64;
    readonly isInt64: boolean;
    readonly asInt64: i64;
    readonly isString: boolean;
    readonly asString: Bytes;
    readonly isArray: boolean;
    readonly asArray: ITuple<[TanglePrimitivesServicesFieldFieldType, Vec<TanglePrimitivesServicesField>]>;
    readonly isList: boolean;
    readonly asList: ITuple<[TanglePrimitivesServicesFieldFieldType, Vec<TanglePrimitivesServicesField>]>;
    readonly isStruct: boolean;
    readonly asStruct: ITuple<[Bytes, Vec<ITuple<[Bytes, TanglePrimitivesServicesField]>>]>;
    readonly isAccountId: boolean;
    readonly asAccountId: AccountId32;
    readonly type: 'Optional' | 'Bool' | 'Uint8' | 'Int8' | 'Uint16' | 'Int16' | 'Uint32' | 'Int32' | 'Uint64' | 'Int64' | 'String' | 'Array' | 'List' | 'Struct' | 'AccountId';
  }

  /** @name TanglePrimitivesServicesFieldFieldType (132) */
  interface TanglePrimitivesServicesFieldFieldType extends Enum {
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
    readonly isString: boolean;
    readonly isOptional: boolean;
    readonly asOptional: TanglePrimitivesServicesFieldFieldType;
    readonly isArray: boolean;
    readonly asArray: ITuple<[u64, TanglePrimitivesServicesFieldFieldType]>;
    readonly isList: boolean;
    readonly asList: TanglePrimitivesServicesFieldFieldType;
    readonly isStruct: boolean;
    readonly asStruct: Vec<TanglePrimitivesServicesFieldFieldType>;
    readonly isAccountId: boolean;
    readonly type: 'Void' | 'Bool' | 'Uint8' | 'Int8' | 'Uint16' | 'Int16' | 'Uint32' | 'Int32' | 'Uint64' | 'Int64' | 'String' | 'Optional' | 'Array' | 'List' | 'Struct' | 'AccountId';
  }

  /** @name TanglePrimitivesServicesTypesAssetSecurityRequirement (148) */
  interface TanglePrimitivesServicesTypesAssetSecurityRequirement extends Struct {
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly minExposurePercent: Percent;
    readonly maxExposurePercent: Percent;
  }

  /** @name TanglePrimitivesServicesTypesAssetSecurityCommitment (154) */
  interface TanglePrimitivesServicesTypesAssetSecurityCommitment extends Struct {
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly exposurePercent: Percent;
  }

  /** @name TanglePrimitivesServicesTypesPricingModelU64 (157) */
  interface TanglePrimitivesServicesTypesPricingModelU64 extends Enum {
    readonly isPayOnce: boolean;
    readonly asPayOnce: {
      readonly amount: u128;
    } & Struct;
    readonly isSubscription: boolean;
    readonly asSubscription: {
      readonly ratePerInterval: u128;
      readonly interval: u64;
      readonly maybeEnd: Option<u64>;
    } & Struct;
    readonly isEventDriven: boolean;
    readonly asEventDriven: {
      readonly rewardPerEvent: u128;
    } & Struct;
    readonly type: 'PayOnce' | 'Subscription' | 'EventDriven';
  }

  /** @name PalletTangleLstEvent (159) */
  interface PalletTangleLstEvent extends Enum {
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
      readonly newState: PalletTangleLstPoolsPoolState;
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
    readonly isPoolCommissionUpdated: boolean;
    readonly asPoolCommissionUpdated: {
      readonly poolId: u32;
      readonly current: Option<ITuple<[Perbill, AccountId32]>>;
    } & Struct;
    readonly isPoolMaxCommissionUpdated: boolean;
    readonly asPoolMaxCommissionUpdated: {
      readonly poolId: u32;
      readonly maxCommission: Perbill;
    } & Struct;
    readonly isPoolCommissionChangeRateUpdated: boolean;
    readonly asPoolCommissionChangeRateUpdated: {
      readonly poolId: u32;
      readonly changeRate: PalletTangleLstCommissionCommissionChangeRate;
    } & Struct;
    readonly isPoolCommissionClaimPermissionUpdated: boolean;
    readonly asPoolCommissionClaimPermissionUpdated: {
      readonly poolId: u32;
      readonly permission: Option<PalletTangleLstCommissionCommissionClaimPermission>;
    } & Struct;
    readonly isPoolCommissionClaimed: boolean;
    readonly asPoolCommissionClaimed: {
      readonly poolId: u32;
      readonly commission: u128;
    } & Struct;
    readonly isMinBalanceDeficitAdjusted: boolean;
    readonly asMinBalanceDeficitAdjusted: {
      readonly poolId: u32;
      readonly amount: u128;
    } & Struct;
    readonly isMinBalanceExcessAdjusted: boolean;
    readonly asMinBalanceExcessAdjusted: {
      readonly poolId: u32;
      readonly amount: u128;
    } & Struct;
    readonly isLastPoolIdUpdated: boolean;
    readonly asLastPoolIdUpdated: {
      readonly poolId: u32;
    } & Struct;
    readonly type: 'Created' | 'Bonded' | 'PaidOut' | 'Unbonded' | 'Withdrawn' | 'Destroyed' | 'StateChanged' | 'MemberRemoved' | 'RolesUpdated' | 'PoolSlashed' | 'UnbondingPoolSlashed' | 'PoolCommissionUpdated' | 'PoolMaxCommissionUpdated' | 'PoolCommissionChangeRateUpdated' | 'PoolCommissionClaimPermissionUpdated' | 'PoolCommissionClaimed' | 'MinBalanceDeficitAdjusted' | 'MinBalanceExcessAdjusted' | 'LastPoolIdUpdated';
  }

  /** @name PalletTangleLstPoolsPoolState (160) */
  interface PalletTangleLstPoolsPoolState extends Enum {
    readonly isOpen: boolean;
    readonly isBlocked: boolean;
    readonly isDestroying: boolean;
    readonly type: 'Open' | 'Blocked' | 'Destroying';
  }

  /** @name PalletTangleLstCommissionCommissionChangeRate (161) */
  interface PalletTangleLstCommissionCommissionChangeRate extends Struct {
    readonly maxIncrease: Perbill;
    readonly minDelay: u64;
  }

  /** @name PalletTangleLstCommissionCommissionClaimPermission (163) */
  interface PalletTangleLstCommissionCommissionClaimPermission extends Enum {
    readonly isPermissionless: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Permissionless' | 'Account';
  }

  /** @name PalletRewardsEvent (164) */
  interface PalletRewardsEvent extends Enum {
    readonly isRewardsClaimed: boolean;
    readonly asRewardsClaimed: {
      readonly account: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isIncentiveAPYAndCapSet: boolean;
    readonly asIncentiveAPYAndCapSet: {
      readonly vaultId: u32;
      readonly apy: Perbill;
      readonly cap: u128;
    } & Struct;
    readonly isBlueprintWhitelisted: boolean;
    readonly asBlueprintWhitelisted: {
      readonly blueprintId: u64;
    } & Struct;
    readonly isAssetUpdatedInVault: boolean;
    readonly asAssetUpdatedInVault: {
      readonly vaultId: u32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly action: PalletRewardsAssetAction;
    } & Struct;
    readonly isVaultRewardConfigUpdated: boolean;
    readonly asVaultRewardConfigUpdated: {
      readonly vaultId: u32;
      readonly newConfig: PalletRewardsRewardConfigForAssetVault;
    } & Struct;
    readonly isRewardVaultCreated: boolean;
    readonly asRewardVaultCreated: {
      readonly vaultId: u32;
      readonly newConfig: PalletRewardsRewardConfigForAssetVault;
      readonly potAccount: AccountId32;
    } & Struct;
    readonly isTotalScoreUpdated: boolean;
    readonly asTotalScoreUpdated: {
      readonly vaultId: u32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly totalScore: u128;
      readonly lockMultiplier: Option<TanglePrimitivesRewardsLockMultiplier>;
    } & Struct;
    readonly isTotalDepositUpdated: boolean;
    readonly asTotalDepositUpdated: {
      readonly vaultId: u32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly totalDeposit: u128;
    } & Struct;
    readonly isDecayConfigUpdated: boolean;
    readonly asDecayConfigUpdated: {
      readonly startPeriod: u64;
      readonly rate: Perbill;
    } & Struct;
    readonly isApyBlocksUpdated: boolean;
    readonly asApyBlocksUpdated: {
      readonly blocks: u64;
    } & Struct;
    readonly isVaultMetadataSet: boolean;
    readonly asVaultMetadataSet: {
      readonly vaultId: u32;
      readonly name: Bytes;
      readonly logo: Bytes;
    } & Struct;
    readonly isVaultMetadataRemoved: boolean;
    readonly asVaultMetadataRemoved: {
      readonly vaultId: u32;
    } & Struct;
    readonly isRewardRecorded: boolean;
    readonly asRewardRecorded: {
      readonly operator: AccountId32;
      readonly serviceId: u64;
      readonly amount: u128;
    } & Struct;
    readonly isOperatorRewardsClaimed: boolean;
    readonly asOperatorRewardsClaimed: {
      readonly operator: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'RewardsClaimed' | 'IncentiveAPYAndCapSet' | 'BlueprintWhitelisted' | 'AssetUpdatedInVault' | 'VaultRewardConfigUpdated' | 'RewardVaultCreated' | 'TotalScoreUpdated' | 'TotalDepositUpdated' | 'DecayConfigUpdated' | 'ApyBlocksUpdated' | 'VaultMetadataSet' | 'VaultMetadataRemoved' | 'RewardRecorded' | 'OperatorRewardsClaimed';
  }

  /** @name PalletRewardsAssetAction (165) */
  interface PalletRewardsAssetAction extends Enum {
    readonly isAdd: boolean;
    readonly isRemove: boolean;
    readonly type: 'Add' | 'Remove';
  }

  /** @name PalletRewardsRewardConfigForAssetVault (166) */
  interface PalletRewardsRewardConfigForAssetVault extends Struct {
    readonly apy: Perbill;
    readonly incentiveCap: u128;
    readonly depositCap: u128;
    readonly boostMultiplier: Option<u32>;
  }

  /** @name TanglePrimitivesRewardsLockMultiplier (169) */
  interface TanglePrimitivesRewardsLockMultiplier extends Enum {
    readonly isOneMonth: boolean;
    readonly isTwoMonths: boolean;
    readonly isThreeMonths: boolean;
    readonly isSixMonths: boolean;
    readonly type: 'OneMonth' | 'TwoMonths' | 'ThreeMonths' | 'SixMonths';
  }

  /** @name PalletIsmpEvent (172) */
  interface PalletIsmpEvent extends Enum {
    readonly isStateMachineUpdated: boolean;
    readonly asStateMachineUpdated: {
      readonly stateMachineId: IsmpConsensusStateMachineId;
      readonly latestHeight: u64;
    } & Struct;
    readonly isStateCommitmentVetoed: boolean;
    readonly asStateCommitmentVetoed: {
      readonly height: IsmpConsensusStateMachineHeight;
      readonly fisherman: Bytes;
    } & Struct;
    readonly isConsensusClientCreated: boolean;
    readonly asConsensusClientCreated: {
      readonly consensusClientId: U8aFixed;
    } & Struct;
    readonly isConsensusClientFrozen: boolean;
    readonly asConsensusClientFrozen: {
      readonly consensusClientId: U8aFixed;
    } & Struct;
    readonly isResponse: boolean;
    readonly asResponse: {
      readonly destChain: IsmpHostStateMachine;
      readonly sourceChain: IsmpHostStateMachine;
      readonly requestNonce: u64;
      readonly commitment: H256;
      readonly reqCommitment: H256;
    } & Struct;
    readonly isRequest: boolean;
    readonly asRequest: {
      readonly destChain: IsmpHostStateMachine;
      readonly sourceChain: IsmpHostStateMachine;
      readonly requestNonce: u64;
      readonly commitment: H256;
    } & Struct;
    readonly isErrors: boolean;
    readonly asErrors: {
      readonly errors: Vec<PalletIsmpErrorsHandlingError>;
    } & Struct;
    readonly isPostRequestHandled: boolean;
    readonly asPostRequestHandled: IsmpEventsRequestResponseHandled;
    readonly isPostResponseHandled: boolean;
    readonly asPostResponseHandled: IsmpEventsRequestResponseHandled;
    readonly isGetRequestHandled: boolean;
    readonly asGetRequestHandled: IsmpEventsRequestResponseHandled;
    readonly isPostRequestTimeoutHandled: boolean;
    readonly asPostRequestTimeoutHandled: IsmpEventsTimeoutHandled;
    readonly isPostResponseTimeoutHandled: boolean;
    readonly asPostResponseTimeoutHandled: IsmpEventsTimeoutHandled;
    readonly isGetRequestTimeoutHandled: boolean;
    readonly asGetRequestTimeoutHandled: IsmpEventsTimeoutHandled;
    readonly type: 'StateMachineUpdated' | 'StateCommitmentVetoed' | 'ConsensusClientCreated' | 'ConsensusClientFrozen' | 'Response' | 'Request' | 'Errors' | 'PostRequestHandled' | 'PostResponseHandled' | 'GetRequestHandled' | 'PostRequestTimeoutHandled' | 'PostResponseTimeoutHandled' | 'GetRequestTimeoutHandled';
  }

  /** @name IsmpConsensusStateMachineId (173) */
  interface IsmpConsensusStateMachineId extends Struct {
    readonly stateId: IsmpHostStateMachine;
    readonly consensusStateId: U8aFixed;
  }

  /** @name IsmpHostStateMachine (174) */
  interface IsmpHostStateMachine extends Enum {
    readonly isEvm: boolean;
    readonly asEvm: u32;
    readonly isPolkadot: boolean;
    readonly asPolkadot: u32;
    readonly isKusama: boolean;
    readonly asKusama: u32;
    readonly isSubstrate: boolean;
    readonly asSubstrate: U8aFixed;
    readonly isTendermint: boolean;
    readonly asTendermint: U8aFixed;
    readonly type: 'Evm' | 'Polkadot' | 'Kusama' | 'Substrate' | 'Tendermint';
  }

  /** @name IsmpConsensusStateMachineHeight (175) */
  interface IsmpConsensusStateMachineHeight extends Struct {
    readonly id: IsmpConsensusStateMachineId;
    readonly height: u64;
  }

  /** @name PalletIsmpErrorsHandlingError (177) */
  interface PalletIsmpErrorsHandlingError extends Struct {
    readonly message: Bytes;
  }

  /** @name IsmpEventsRequestResponseHandled (179) */
  interface IsmpEventsRequestResponseHandled extends Struct {
    readonly commitment: H256;
    readonly relayer: Bytes;
  }

  /** @name IsmpEventsTimeoutHandled (180) */
  interface IsmpEventsTimeoutHandled extends Struct {
    readonly commitment: H256;
    readonly source: IsmpHostStateMachine;
    readonly dest: IsmpHostStateMachine;
  }

  /** @name IsmpGrandpaEvent (181) */
  interface IsmpGrandpaEvent extends Enum {
    readonly isStateMachineAdded: boolean;
    readonly asStateMachineAdded: {
      readonly stateMachines: Vec<IsmpHostStateMachine>;
    } & Struct;
    readonly isStateMachineRemoved: boolean;
    readonly asStateMachineRemoved: {
      readonly stateMachines: Vec<IsmpHostStateMachine>;
    } & Struct;
    readonly type: 'StateMachineAdded' | 'StateMachineRemoved';
  }

  /** @name PalletHyperbridgeEvent (183) */
  interface PalletHyperbridgeEvent extends Enum {
    readonly isHostParamsUpdated: boolean;
    readonly asHostParamsUpdated: {
      readonly old: PalletHyperbridgeVersionedHostParams;
      readonly new_: PalletHyperbridgeVersionedHostParams;
    } & Struct;
    readonly isRelayerFeeWithdrawn: boolean;
    readonly asRelayerFeeWithdrawn: {
      readonly amount: u128;
      readonly account: AccountId32;
    } & Struct;
    readonly isProtocolRevenueWithdrawn: boolean;
    readonly asProtocolRevenueWithdrawn: {
      readonly amount: u128;
      readonly account: AccountId32;
    } & Struct;
    readonly type: 'HostParamsUpdated' | 'RelayerFeeWithdrawn' | 'ProtocolRevenueWithdrawn';
  }

  /** @name PalletHyperbridgeVersionedHostParams (184) */
  interface PalletHyperbridgeVersionedHostParams extends Enum {
    readonly isV1: boolean;
    readonly asV1: PalletHyperbridgeSubstrateHostParams;
    readonly type: 'V1';
  }

  /** @name PalletHyperbridgeSubstrateHostParams (185) */
  interface PalletHyperbridgeSubstrateHostParams extends Struct {
    readonly defaultPerByteFee: u128;
    readonly perByteFees: BTreeMap<IsmpHostStateMachine, u128>;
    readonly assetRegistrationFee: u128;
  }

  /** @name PalletTokenGatewayEvent (189) */
  interface PalletTokenGatewayEvent extends Enum {
    readonly isAssetTeleported: boolean;
    readonly asAssetTeleported: {
      readonly from: AccountId32;
      readonly to: H256;
      readonly amount: u128;
      readonly dest: IsmpHostStateMachine;
      readonly commitment: H256;
    } & Struct;
    readonly isAssetReceived: boolean;
    readonly asAssetReceived: {
      readonly beneficiary: AccountId32;
      readonly amount: u128;
      readonly source: IsmpHostStateMachine;
    } & Struct;
    readonly isAssetRefunded: boolean;
    readonly asAssetRefunded: {
      readonly beneficiary: AccountId32;
      readonly amount: u128;
      readonly source: IsmpHostStateMachine;
    } & Struct;
    readonly isErc6160AssetRegistrationDispatched: boolean;
    readonly asErc6160AssetRegistrationDispatched: {
      readonly commitment: H256;
    } & Struct;
    readonly type: 'AssetTeleported' | 'AssetReceived' | 'AssetRefunded' | 'Erc6160AssetRegistrationDispatched';
  }

  /** @name PalletCreditsEvent (190) */
  interface PalletCreditsEvent extends Enum {
    readonly isCreditsGrantedFromBurn: boolean;
    readonly asCreditsGrantedFromBurn: {
      readonly who: AccountId32;
      readonly tntBurned: u128;
      readonly creditsGranted: u128;
    } & Struct;
    readonly isCreditsClaimed: boolean;
    readonly asCreditsClaimed: {
      readonly who: AccountId32;
      readonly amountClaimed: u128;
      readonly offchainAccountId: Bytes;
    } & Struct;
    readonly isStakeTiersUpdated: boolean;
    readonly isAssetStakeTiersUpdated: boolean;
    readonly asAssetStakeTiersUpdated: {
      readonly assetId: u128;
    } & Struct;
    readonly type: 'CreditsGrantedFromBurn' | 'CreditsClaimed' | 'StakeTiersUpdated' | 'AssetStakeTiersUpdated';
  }

  /** @name FrameSystemPhase (192) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (194) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCodeUpgradeAuthorization (196) */
  interface FrameSystemCodeUpgradeAuthorization extends Struct {
    readonly codeHash: H256;
    readonly checkVersion: bool;
  }

  /** @name FrameSystemCall (197) */
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
    readonly isAuthorizeUpgrade: boolean;
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256;
    } & Struct;
    readonly isAuthorizeUpgradeWithoutChecks: boolean;
    readonly asAuthorizeUpgradeWithoutChecks: {
      readonly codeHash: H256;
    } & Struct;
    readonly isApplyAuthorizedUpgrade: boolean;
    readonly asApplyAuthorizedUpgrade: {
      readonly code: Bytes;
    } & Struct;
    readonly type: 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent' | 'AuthorizeUpgrade' | 'AuthorizeUpgradeWithoutChecks' | 'ApplyAuthorizedUpgrade';
  }

  /** @name FrameSystemLimitsBlockWeights (201) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (202) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (203) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /** @name FrameSystemLimitsBlockLength (205) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (206) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name SpWeightsRuntimeDbWeight (207) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (208) */
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

  /** @name FrameSystemError (213) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly isMultiBlockMigrationsOngoing: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered' | 'MultiBlockMigrationsOngoing' | 'NothingAuthorized' | 'Unauthorized';
  }

  /** @name PalletTimestampCall (214) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletSudoCall (215) */
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
    readonly isRemoveKey: boolean;
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs' | 'RemoveKey';
  }

  /** @name PalletAssetsCall (217) */
  interface PalletAssetsCall extends Enum {
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly id: Compact<u128>;
      readonly admin: MultiAddress;
      readonly minBalance: u128;
    } & Struct;
    readonly isForceCreate: boolean;
    readonly asForceCreate: {
      readonly id: Compact<u128>;
      readonly owner: MultiAddress;
      readonly isSufficient: bool;
      readonly minBalance: Compact<u128>;
    } & Struct;
    readonly isStartDestroy: boolean;
    readonly asStartDestroy: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isDestroyAccounts: boolean;
    readonly asDestroyAccounts: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isDestroyApprovals: boolean;
    readonly asDestroyApprovals: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isFinishDestroy: boolean;
    readonly asFinishDestroy: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isMint: boolean;
    readonly asMint: {
      readonly id: Compact<u128>;
      readonly beneficiary: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly id: Compact<u128>;
      readonly target: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly id: Compact<u128>;
      readonly target: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly id: Compact<u128>;
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
    } & Struct;
    readonly isThaw: boolean;
    readonly asThaw: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
    } & Struct;
    readonly isFreezeAsset: boolean;
    readonly asFreezeAsset: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isThawAsset: boolean;
    readonly asThawAsset: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isTransferOwnership: boolean;
    readonly asTransferOwnership: {
      readonly id: Compact<u128>;
      readonly owner: MultiAddress;
    } & Struct;
    readonly isSetTeam: boolean;
    readonly asSetTeam: {
      readonly id: Compact<u128>;
      readonly issuer: MultiAddress;
      readonly admin: MultiAddress;
      readonly freezer: MultiAddress;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly id: Compact<u128>;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
    } & Struct;
    readonly isClearMetadata: boolean;
    readonly asClearMetadata: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isForceSetMetadata: boolean;
    readonly asForceSetMetadata: {
      readonly id: Compact<u128>;
      readonly name: Bytes;
      readonly symbol: Bytes;
      readonly decimals: u8;
      readonly isFrozen: bool;
    } & Struct;
    readonly isForceClearMetadata: boolean;
    readonly asForceClearMetadata: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isForceAssetStatus: boolean;
    readonly asForceAssetStatus: {
      readonly id: Compact<u128>;
      readonly owner: MultiAddress;
      readonly issuer: MultiAddress;
      readonly admin: MultiAddress;
      readonly freezer: MultiAddress;
      readonly minBalance: Compact<u128>;
      readonly isSufficient: bool;
      readonly isFrozen: bool;
    } & Struct;
    readonly isApproveTransfer: boolean;
    readonly asApproveTransfer: {
      readonly id: Compact<u128>;
      readonly delegate: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isCancelApproval: boolean;
    readonly asCancelApproval: {
      readonly id: Compact<u128>;
      readonly delegate: MultiAddress;
    } & Struct;
    readonly isForceCancelApproval: boolean;
    readonly asForceCancelApproval: {
      readonly id: Compact<u128>;
      readonly owner: MultiAddress;
      readonly delegate: MultiAddress;
    } & Struct;
    readonly isTransferApproved: boolean;
    readonly asTransferApproved: {
      readonly id: Compact<u128>;
      readonly owner: MultiAddress;
      readonly destination: MultiAddress;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTouch: boolean;
    readonly asTouch: {
      readonly id: Compact<u128>;
    } & Struct;
    readonly isRefund: boolean;
    readonly asRefund: {
      readonly id: Compact<u128>;
      readonly allowBurn: bool;
    } & Struct;
    readonly isSetMinBalance: boolean;
    readonly asSetMinBalance: {
      readonly id: Compact<u128>;
      readonly minBalance: u128;
    } & Struct;
    readonly isTouchOther: boolean;
    readonly asTouchOther: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
    } & Struct;
    readonly isRefundOther: boolean;
    readonly asRefundOther: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
    } & Struct;
    readonly isBlock: boolean;
    readonly asBlock: {
      readonly id: Compact<u128>;
      readonly who: MultiAddress;
    } & Struct;
    readonly type: 'Create' | 'ForceCreate' | 'StartDestroy' | 'DestroyAccounts' | 'DestroyApprovals' | 'FinishDestroy' | 'Mint' | 'Burn' | 'Transfer' | 'TransferKeepAlive' | 'ForceTransfer' | 'Freeze' | 'Thaw' | 'FreezeAsset' | 'ThawAsset' | 'TransferOwnership' | 'SetTeam' | 'SetMetadata' | 'ClearMetadata' | 'ForceSetMetadata' | 'ForceClearMetadata' | 'ForceAssetStatus' | 'ApproveTransfer' | 'CancelApproval' | 'ForceCancelApproval' | 'TransferApproved' | 'Touch' | 'Refund' | 'SetMinBalance' | 'TouchOther' | 'RefundOther' | 'Block';
  }

  /** @name PalletBalancesCall (219) */
  interface PalletBalancesCall extends Enum {
    readonly isTransferAllowDeath: boolean;
    readonly asTransferAllowDeath: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
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
    readonly isUpgradeAccounts: boolean;
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId32>;
    } & Struct;
    readonly isForceSetBalance: boolean;
    readonly asForceSetBalance: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
    } & Struct;
    readonly isForceAdjustTotalIssuance: boolean;
    readonly asForceAdjustTotalIssuance: {
      readonly direction: PalletBalancesAdjustmentDirection;
      readonly delta: Compact<u128>;
    } & Struct;
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly value: Compact<u128>;
      readonly keepAlive: bool;
    } & Struct;
    readonly type: 'TransferAllowDeath' | 'ForceTransfer' | 'TransferKeepAlive' | 'TransferAll' | 'ForceUnreserve' | 'UpgradeAccounts' | 'ForceSetBalance' | 'ForceAdjustTotalIssuance' | 'Burn';
  }

  /** @name PalletBalancesAdjustmentDirection (220) */
  interface PalletBalancesAdjustmentDirection extends Enum {
    readonly isIncrease: boolean;
    readonly isDecrease: boolean;
    readonly type: 'Increase' | 'Decrease';
  }

  /** @name PalletBabeCall (221) */
  interface PalletBabeCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isPlanConfigChange: boolean;
    readonly asPlanConfigChange: {
      readonly config: SpConsensusBabeDigestsNextConfigDescriptor;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'PlanConfigChange';
  }

  /** @name SpConsensusSlotsEquivocationProof (222) */
  interface SpConsensusSlotsEquivocationProof extends Struct {
    readonly offender: SpConsensusBabeAppPublic;
    readonly slot: u64;
    readonly firstHeader: SpRuntimeHeader;
    readonly secondHeader: SpRuntimeHeader;
  }

  /** @name SpRuntimeHeader (223) */
  interface SpRuntimeHeader extends Struct {
    readonly parentHash: H256;
    readonly number: Compact<u64>;
    readonly stateRoot: H256;
    readonly extrinsicsRoot: H256;
    readonly digest: SpRuntimeDigest;
  }

  /** @name SpConsensusBabeAppPublic (224) */
  interface SpConsensusBabeAppPublic extends U8aFixed {}

  /** @name SpSessionMembershipProof (226) */
  interface SpSessionMembershipProof extends Struct {
    readonly session: u32;
    readonly trieNodes: Vec<Bytes>;
    readonly validatorCount: u32;
  }

  /** @name SpConsensusBabeDigestsNextConfigDescriptor (227) */
  interface SpConsensusBabeDigestsNextConfigDescriptor extends Enum {
    readonly isV1: boolean;
    readonly asV1: {
      readonly c: ITuple<[u64, u64]>;
      readonly allowedSlots: SpConsensusBabeAllowedSlots;
    } & Struct;
    readonly type: 'V1';
  }

  /** @name SpConsensusBabeAllowedSlots (229) */
  interface SpConsensusBabeAllowedSlots extends Enum {
    readonly isPrimarySlots: boolean;
    readonly isPrimaryAndSecondaryPlainSlots: boolean;
    readonly isPrimaryAndSecondaryVRFSlots: boolean;
    readonly type: 'PrimarySlots' | 'PrimaryAndSecondaryPlainSlots' | 'PrimaryAndSecondaryVRFSlots';
  }

  /** @name PalletGrandpaCall (230) */
  interface PalletGrandpaCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof;
      readonly keyOwnerProof: SpCoreVoid;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof;
      readonly keyOwnerProof: SpCoreVoid;
    } & Struct;
    readonly isNoteStalled: boolean;
    readonly asNoteStalled: {
      readonly delay: u64;
      readonly bestFinalizedBlockNumber: u64;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'NoteStalled';
  }

  /** @name SpConsensusGrandpaEquivocationProof (231) */
  interface SpConsensusGrandpaEquivocationProof extends Struct {
    readonly setId: u64;
    readonly equivocation: SpConsensusGrandpaEquivocation;
  }

  /** @name SpConsensusGrandpaEquivocation (232) */
  interface SpConsensusGrandpaEquivocation extends Enum {
    readonly isPrevote: boolean;
    readonly asPrevote: FinalityGrandpaEquivocationPrevote;
    readonly isPrecommit: boolean;
    readonly asPrecommit: FinalityGrandpaEquivocationPrecommit;
    readonly type: 'Prevote' | 'Precommit';
  }

  /** @name FinalityGrandpaEquivocationPrevote (233) */
  interface FinalityGrandpaEquivocationPrevote extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpConsensusGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrevote (234) */
  interface FinalityGrandpaPrevote extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u64;
  }

  /** @name SpConsensusGrandpaAppSignature (235) */
  interface SpConsensusGrandpaAppSignature extends U8aFixed {}

  /** @name FinalityGrandpaEquivocationPrecommit (238) */
  interface FinalityGrandpaEquivocationPrecommit extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpConsensusGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrecommit (239) */
  interface FinalityGrandpaPrecommit extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u64;
  }

  /** @name SpCoreVoid (241) */
  type SpCoreVoid = Null;

  /** @name PalletIndicesCall (242) */
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

  /** @name PalletDemocracyCall (243) */
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
      readonly votingPeriod: u64;
      readonly delay: u64;
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

  /** @name FrameSupportPreimagesBounded (244) */
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

  /** @name SpRuntimeBlakeTwo256 (245) */
  type SpRuntimeBlakeTwo256 = Null;

  /** @name PalletDemocracyConviction (247) */
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

  /** @name PalletCollectiveCall (249) */
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
    readonly type: 'SetMembers' | 'Execute' | 'Propose' | 'Vote' | 'DisapproveProposal' | 'Close';
  }

  /** @name PalletVestingCall (250) */
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
    readonly isForceRemoveVestingSchedule: boolean;
    readonly asForceRemoveVestingSchedule: {
      readonly target: MultiAddress;
      readonly scheduleIndex: u32;
    } & Struct;
    readonly type: 'Vest' | 'VestOther' | 'VestedTransfer' | 'ForceVestedTransfer' | 'MergeSchedules' | 'ForceRemoveVestingSchedule';
  }

  /** @name PalletVestingVestingInfo (251) */
  interface PalletVestingVestingInfo extends Struct {
    readonly locked: u128;
    readonly perBlock: u128;
    readonly startingBlock: u64;
  }

  /** @name PalletElectionsPhragmenCall (252) */
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

  /** @name PalletElectionsPhragmenRenouncing (253) */
  interface PalletElectionsPhragmenRenouncing extends Enum {
    readonly isMember: boolean;
    readonly isRunnerUp: boolean;
    readonly isCandidate: boolean;
    readonly asCandidate: Compact<u32>;
    readonly type: 'Member' | 'RunnerUp' | 'Candidate';
  }

  /** @name PalletElectionProviderMultiPhaseCall (254) */
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

  /** @name PalletElectionProviderMultiPhaseRawSolution (255) */
  interface PalletElectionProviderMultiPhaseRawSolution extends Struct {
    readonly solution: TangleTestnetRuntimeNposSolution16;
    readonly score: SpNposElectionsElectionScore;
    readonly round: u32;
  }

  /** @name TangleTestnetRuntimeNposSolution16 (256) */
  interface TangleTestnetRuntimeNposSolution16 extends Struct {
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

  /** @name PalletElectionProviderMultiPhaseSolutionOrSnapshotSize (307) */
  interface PalletElectionProviderMultiPhaseSolutionOrSnapshotSize extends Struct {
    readonly voters: Compact<u32>;
    readonly targets: Compact<u32>;
  }

  /** @name SpNposElectionsSupport (311) */
  interface SpNposElectionsSupport extends Struct {
    readonly total: u128;
    readonly voters: Vec<ITuple<[AccountId32, u128]>>;
  }

  /** @name PalletStakingPalletCall (312) */
  interface PalletStakingPalletCall extends Enum {
    readonly isBond: boolean;
    readonly asBond: {
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
      readonly maxStakedRewards: PalletStakingPalletConfigOpPercent;
    } & Struct;
    readonly isChillOther: boolean;
    readonly asChillOther: {
      readonly stash: AccountId32;
    } & Struct;
    readonly isForceApplyMinCommission: boolean;
    readonly asForceApplyMinCommission: {
      readonly validatorStash: AccountId32;
    } & Struct;
    readonly isSetMinCommission: boolean;
    readonly asSetMinCommission: {
      readonly new_: Perbill;
    } & Struct;
    readonly isPayoutStakersByPage: boolean;
    readonly asPayoutStakersByPage: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
      readonly page: u32;
    } & Struct;
    readonly isUpdatePayee: boolean;
    readonly asUpdatePayee: {
      readonly controller: AccountId32;
    } & Struct;
    readonly isDeprecateControllerBatch: boolean;
    readonly asDeprecateControllerBatch: {
      readonly controllers: Vec<AccountId32>;
    } & Struct;
    readonly isRestoreLedger: boolean;
    readonly asRestoreLedger: {
      readonly stash: AccountId32;
      readonly maybeController: Option<AccountId32>;
      readonly maybeTotal: Option<u128>;
      readonly maybeUnlocking: Option<Vec<PalletStakingUnlockChunk>>;
    } & Struct;
    readonly type: 'Bond' | 'BondExtra' | 'Unbond' | 'WithdrawUnbonded' | 'Validate' | 'Nominate' | 'Chill' | 'SetPayee' | 'SetController' | 'SetValidatorCount' | 'IncreaseValidatorCount' | 'ScaleValidatorCount' | 'ForceNoEras' | 'ForceNewEra' | 'SetInvulnerables' | 'ForceUnstake' | 'ForceNewEraAlways' | 'CancelDeferredSlash' | 'PayoutStakers' | 'Rebond' | 'ReapStash' | 'Kick' | 'SetStakingConfigs' | 'ChillOther' | 'ForceApplyMinCommission' | 'SetMinCommission' | 'PayoutStakersByPage' | 'UpdatePayee' | 'DeprecateControllerBatch' | 'RestoreLedger';
  }

  /** @name PalletStakingPalletConfigOpU128 (315) */
  interface PalletStakingPalletConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpU32 (316) */
  interface PalletStakingPalletConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPercent (317) */
  interface PalletStakingPalletConfigOpPercent extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Percent;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPerbill (318) */
  interface PalletStakingPalletConfigOpPerbill extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Perbill;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingUnlockChunk (323) */
  interface PalletStakingUnlockChunk extends Struct {
    readonly value: Compact<u128>;
    readonly era: Compact<u32>;
  }

  /** @name PalletSessionCall (325) */
  interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: TangleTestnetRuntimeOpaqueSessionKeys;
      readonly proof: Bytes;
    } & Struct;
    readonly isPurgeKeys: boolean;
    readonly type: 'SetKeys' | 'PurgeKeys';
  }

  /** @name TangleTestnetRuntimeOpaqueSessionKeys (326) */
  interface TangleTestnetRuntimeOpaqueSessionKeys extends Struct {
    readonly babe: SpConsensusBabeAppPublic;
    readonly grandpa: SpConsensusGrandpaAppPublic;
    readonly imOnline: PalletImOnlineSr25519AppSr25519Public;
  }

  /** @name PalletTreasuryCall (327) */
  interface PalletTreasuryCall extends Enum {
    readonly isSpendLocal: boolean;
    readonly asSpendLocal: {
      readonly amount: Compact<u128>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isRemoveApproval: boolean;
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isSpend: boolean;
    readonly asSpend: {
      readonly assetKind: Null;
      readonly amount: Compact<u128>;
      readonly beneficiary: AccountId32;
      readonly validFrom: Option<u64>;
    } & Struct;
    readonly isPayout: boolean;
    readonly asPayout: {
      readonly index: u32;
    } & Struct;
    readonly isCheckStatus: boolean;
    readonly asCheckStatus: {
      readonly index: u32;
    } & Struct;
    readonly isVoidSpend: boolean;
    readonly asVoidSpend: {
      readonly index: u32;
    } & Struct;
    readonly type: 'SpendLocal' | 'RemoveApproval' | 'Spend' | 'Payout' | 'CheckStatus' | 'VoidSpend';
  }

  /** @name PalletBountiesCall (328) */
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

  /** @name PalletChildBountiesCall (329) */
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

  /** @name PalletBagsListCall (330) */
  interface PalletBagsListCall extends Enum {
    readonly isRebag: boolean;
    readonly asRebag: {
      readonly dislocated: MultiAddress;
    } & Struct;
    readonly isPutInFrontOf: boolean;
    readonly asPutInFrontOf: {
      readonly lighter: MultiAddress;
    } & Struct;
    readonly isPutInFrontOfOther: boolean;
    readonly asPutInFrontOfOther: {
      readonly heavier: MultiAddress;
      readonly lighter: MultiAddress;
    } & Struct;
    readonly type: 'Rebag' | 'PutInFrontOf' | 'PutInFrontOfOther';
  }

  /** @name PalletNominationPoolsCall (331) */
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
      readonly globalMaxCommission: PalletNominationPoolsConfigOpPerbill;
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
    readonly isSetCommission: boolean;
    readonly asSetCommission: {
      readonly poolId: u32;
      readonly newCommission: Option<ITuple<[Perbill, AccountId32]>>;
    } & Struct;
    readonly isSetCommissionMax: boolean;
    readonly asSetCommissionMax: {
      readonly poolId: u32;
      readonly maxCommission: Perbill;
    } & Struct;
    readonly isSetCommissionChangeRate: boolean;
    readonly asSetCommissionChangeRate: {
      readonly poolId: u32;
      readonly changeRate: PalletNominationPoolsCommissionChangeRate;
    } & Struct;
    readonly isClaimCommission: boolean;
    readonly asClaimCommission: {
      readonly poolId: u32;
    } & Struct;
    readonly isAdjustPoolDeposit: boolean;
    readonly asAdjustPoolDeposit: {
      readonly poolId: u32;
    } & Struct;
    readonly isSetCommissionClaimPermission: boolean;
    readonly asSetCommissionClaimPermission: {
      readonly poolId: u32;
      readonly permission: Option<PalletNominationPoolsCommissionClaimPermission>;
    } & Struct;
    readonly isApplySlash: boolean;
    readonly asApplySlash: {
      readonly memberAccount: MultiAddress;
    } & Struct;
    readonly isMigrateDelegation: boolean;
    readonly asMigrateDelegation: {
      readonly memberAccount: MultiAddress;
    } & Struct;
    readonly isMigratePoolToDelegateStake: boolean;
    readonly asMigratePoolToDelegateStake: {
      readonly poolId: u32;
    } & Struct;
    readonly type: 'Join' | 'BondExtra' | 'ClaimPayout' | 'Unbond' | 'PoolWithdrawUnbonded' | 'WithdrawUnbonded' | 'Create' | 'CreateWithPoolId' | 'Nominate' | 'SetState' | 'SetMetadata' | 'SetConfigs' | 'UpdateRoles' | 'Chill' | 'BondExtraOther' | 'SetClaimPermission' | 'ClaimPayoutOther' | 'SetCommission' | 'SetCommissionMax' | 'SetCommissionChangeRate' | 'ClaimCommission' | 'AdjustPoolDeposit' | 'SetCommissionClaimPermission' | 'ApplySlash' | 'MigrateDelegation' | 'MigratePoolToDelegateStake';
  }

  /** @name PalletNominationPoolsBondExtra (332) */
  interface PalletNominationPoolsBondExtra extends Enum {
    readonly isFreeBalance: boolean;
    readonly asFreeBalance: u128;
    readonly isRewards: boolean;
    readonly type: 'FreeBalance' | 'Rewards';
  }

  /** @name PalletNominationPoolsConfigOpU128 (333) */
  interface PalletNominationPoolsConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsConfigOpU32 (334) */
  interface PalletNominationPoolsConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsConfigOpPerbill (335) */
  interface PalletNominationPoolsConfigOpPerbill extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Perbill;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsConfigOpAccountId32 (336) */
  interface PalletNominationPoolsConfigOpAccountId32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: AccountId32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletNominationPoolsClaimPermission (337) */
  interface PalletNominationPoolsClaimPermission extends Enum {
    readonly isPermissioned: boolean;
    readonly isPermissionlessCompound: boolean;
    readonly isPermissionlessWithdraw: boolean;
    readonly isPermissionlessAll: boolean;
    readonly type: 'Permissioned' | 'PermissionlessCompound' | 'PermissionlessWithdraw' | 'PermissionlessAll';
  }

  /** @name PalletSchedulerCall (338) */
  interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u64;
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u64;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: U8aFixed;
      readonly when: u64;
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u64;
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: U8aFixed;
      readonly after: u64;
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isSetRetry: boolean;
    readonly asSetRetry: {
      readonly task: ITuple<[u64, u32]>;
      readonly retries: u8;
      readonly period: u64;
    } & Struct;
    readonly isSetRetryNamed: boolean;
    readonly asSetRetryNamed: {
      readonly id: U8aFixed;
      readonly retries: u8;
      readonly period: u64;
    } & Struct;
    readonly isCancelRetry: boolean;
    readonly asCancelRetry: {
      readonly task: ITuple<[u64, u32]>;
    } & Struct;
    readonly isCancelRetryNamed: boolean;
    readonly asCancelRetryNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter' | 'SetRetry' | 'SetRetryNamed' | 'CancelRetry' | 'CancelRetryNamed';
  }

  /** @name PalletPreimageCall (340) */
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
    readonly isEnsureUpdated: boolean;
    readonly asEnsureUpdated: {
      readonly hashes: Vec<H256>;
    } & Struct;
    readonly type: 'NotePreimage' | 'UnnotePreimage' | 'RequestPreimage' | 'UnrequestPreimage' | 'EnsureUpdated';
  }

  /** @name PalletTxPauseCall (341) */
  interface PalletTxPauseCall extends Enum {
    readonly isPause: boolean;
    readonly asPause: {
      readonly fullName: ITuple<[Bytes, Bytes]>;
    } & Struct;
    readonly isUnpause: boolean;
    readonly asUnpause: {
      readonly ident: ITuple<[Bytes, Bytes]>;
    } & Struct;
    readonly type: 'Pause' | 'Unpause';
  }

  /** @name PalletImOnlineCall (342) */
  interface PalletImOnlineCall extends Enum {
    readonly isHeartbeat: boolean;
    readonly asHeartbeat: {
      readonly heartbeat: PalletImOnlineHeartbeat;
      readonly signature: PalletImOnlineSr25519AppSr25519Signature;
    } & Struct;
    readonly type: 'Heartbeat';
  }

  /** @name PalletImOnlineHeartbeat (343) */
  interface PalletImOnlineHeartbeat extends Struct {
    readonly blockNumber: u64;
    readonly sessionIndex: u32;
    readonly authorityIndex: u32;
    readonly validatorsLen: u32;
  }

  /** @name PalletImOnlineSr25519AppSr25519Signature (344) */
  interface PalletImOnlineSr25519AppSr25519Signature extends U8aFixed {}

  /** @name PalletIdentityCall (345) */
  interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean;
    readonly asAddRegistrar: {
      readonly account: MultiAddress;
    } & Struct;
    readonly isSetIdentity: boolean;
    readonly asSetIdentity: {
      readonly info: PalletIdentityLegacyIdentityInfo;
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
      readonly fields: u64;
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
    readonly isAddUsernameAuthority: boolean;
    readonly asAddUsernameAuthority: {
      readonly authority: MultiAddress;
      readonly suffix: Bytes;
      readonly allocation: u32;
    } & Struct;
    readonly isRemoveUsernameAuthority: boolean;
    readonly asRemoveUsernameAuthority: {
      readonly authority: MultiAddress;
    } & Struct;
    readonly isSetUsernameFor: boolean;
    readonly asSetUsernameFor: {
      readonly who: MultiAddress;
      readonly username: Bytes;
      readonly signature: Option<SpRuntimeMultiSignature>;
    } & Struct;
    readonly isAcceptUsername: boolean;
    readonly asAcceptUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isRemoveExpiredApproval: boolean;
    readonly asRemoveExpiredApproval: {
      readonly username: Bytes;
    } & Struct;
    readonly isSetPrimaryUsername: boolean;
    readonly asSetPrimaryUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly isRemoveDanglingUsername: boolean;
    readonly asRemoveDanglingUsername: {
      readonly username: Bytes;
    } & Struct;
    readonly type: 'AddRegistrar' | 'SetIdentity' | 'SetSubs' | 'ClearIdentity' | 'RequestJudgement' | 'CancelRequest' | 'SetFee' | 'SetAccountId' | 'SetFields' | 'ProvideJudgement' | 'KillIdentity' | 'AddSub' | 'RenameSub' | 'RemoveSub' | 'QuitSub' | 'AddUsernameAuthority' | 'RemoveUsernameAuthority' | 'SetUsernameFor' | 'AcceptUsername' | 'RemoveExpiredApproval' | 'SetPrimaryUsername' | 'RemoveDanglingUsername';
  }

  /** @name PalletIdentityLegacyIdentityInfo (346) */
  interface PalletIdentityLegacyIdentityInfo extends Struct {
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

  /** @name PalletIdentityJudgement (382) */
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

  /** @name SpRuntimeMultiSignature (384) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: U8aFixed;
    readonly isSr25519: boolean;
    readonly asSr25519: U8aFixed;
    readonly isEcdsa: boolean;
    readonly asEcdsa: U8aFixed;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa';
  }

  /** @name PalletUtilityCall (385) */
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
      readonly asOrigin: TangleTestnetRuntimeOriginCaller;
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

  /** @name TangleTestnetRuntimeOriginCaller (387) */
  interface TangleTestnetRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isCouncil: boolean;
    readonly asCouncil: PalletCollectiveRawOrigin;
    readonly isEthereum: boolean;
    readonly asEthereum: PalletEthereumRawOrigin;
    readonly type: 'System' | 'Council' | 'Ethereum';
  }

  /** @name FrameSupportDispatchRawOrigin (388) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Root' | 'Signed' | 'None';
  }

  /** @name PalletCollectiveRawOrigin (389) */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId32;
    readonly isPhantom: boolean;
    readonly type: 'Members' | 'Member' | 'Phantom';
  }

  /** @name PalletEthereumRawOrigin (390) */
  interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean;
    readonly asEthereumTransaction: H160;
    readonly type: 'EthereumTransaction';
  }

  /** @name PalletMultisigCall (391) */
  interface PalletMultisigCall extends Enum {
    readonly isAsMultiThreshold1: boolean;
    readonly asAsMultiThreshold1: {
      readonly otherSignatories: Vec<AccountId32>;
      readonly call: Call;
    } & Struct;
    readonly isAsMulti: boolean;
    readonly asAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly call: Call;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isApproveAsMulti: boolean;
    readonly asApproveAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly callHash: U8aFixed;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isCancelAsMulti: boolean;
    readonly asCancelAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly timepoint: PalletMultisigTimepoint;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: 'AsMultiThreshold1' | 'AsMulti' | 'ApproveAsMulti' | 'CancelAsMulti';
  }

  /** @name PalletEthereumCall (393) */
  interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2;
    } & Struct;
    readonly type: 'Transact';
  }

  /** @name EthereumTransactionTransactionV2 (394) */
  interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumTransactionLegacyTransaction;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumTransactionEip2930Transaction;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumTransactionEip1559Transaction;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumTransactionLegacyTransaction (395) */
  interface EthereumTransactionLegacyTransaction extends Struct {
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly signature: EthereumTransactionTransactionSignature;
  }

  /** @name EthereumTransactionTransactionAction (396) */
  interface EthereumTransactionTransactionAction extends Enum {
    readonly isCall: boolean;
    readonly asCall: H160;
    readonly isCreate: boolean;
    readonly type: 'Call' | 'Create';
  }

  /** @name EthereumTransactionTransactionSignature (397) */
  interface EthereumTransactionTransactionSignature extends Struct {
    readonly v: u64;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionEip2930Transaction (399) */
  interface EthereumTransactionEip2930Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionAccessListItem (401) */
  interface EthereumTransactionAccessListItem extends Struct {
    readonly address: H160;
    readonly storageKeys: Vec<H256>;
  }

  /** @name EthereumTransactionEip1559Transaction (402) */
  interface EthereumTransactionEip1559Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly maxPriorityFeePerGas: U256;
    readonly maxFeePerGas: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name PalletEvmCall (403) */
  interface PalletEvmCall extends Enum {
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly address: H160;
      readonly value: u128;
    } & Struct;
    readonly isCall: boolean;
    readonly asCall: {
      readonly source: H160;
      readonly target: H160;
      readonly input: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly source: H160;
      readonly init: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate2: boolean;
    readonly asCreate2: {
      readonly source: H160;
      readonly init: Bytes;
      readonly salt: H256;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly type: 'Withdraw' | 'Call' | 'Create' | 'Create2';
  }

  /** @name PalletDynamicFeeCall (407) */
  interface PalletDynamicFeeCall extends Enum {
    readonly isNoteMinGasPriceTarget: boolean;
    readonly asNoteMinGasPriceTarget: {
      readonly target: U256;
    } & Struct;
    readonly type: 'NoteMinGasPriceTarget';
  }

  /** @name PalletBaseFeeCall (408) */
  interface PalletBaseFeeCall extends Enum {
    readonly isSetBaseFeePerGas: boolean;
    readonly asSetBaseFeePerGas: {
      readonly fee: U256;
    } & Struct;
    readonly isSetElasticity: boolean;
    readonly asSetElasticity: {
      readonly elasticity: Permill;
    } & Struct;
    readonly type: 'SetBaseFeePerGas' | 'SetElasticity';
  }

  /** @name PalletHotfixSufficientsCall (409) */
  interface PalletHotfixSufficientsCall extends Enum {
    readonly isHotfixIncAccountSufficients: boolean;
    readonly asHotfixIncAccountSufficients: {
      readonly addresses: Vec<H160>;
    } & Struct;
    readonly type: 'HotfixIncAccountSufficients';
  }

  /** @name PalletAirdropClaimsCall (411) */
  interface PalletAirdropClaimsCall extends Enum {
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly dest: Option<PalletAirdropClaimsUtilsMultiAddress>;
      readonly signer: Option<PalletAirdropClaimsUtilsMultiAddress>;
      readonly signature: PalletAirdropClaimsUtilsMultiAddressSignature;
    } & Struct;
    readonly isMintClaim: boolean;
    readonly asMintClaim: {
      readonly who: PalletAirdropClaimsUtilsMultiAddress;
      readonly value: u128;
      readonly vestingSchedule: Option<Vec<ITuple<[u128, u128, u64]>>>;
      readonly statement: Option<PalletAirdropClaimsStatementKind>;
    } & Struct;
    readonly isClaimAttest: boolean;
    readonly asClaimAttest: {
      readonly dest: Option<PalletAirdropClaimsUtilsMultiAddress>;
      readonly signer: Option<PalletAirdropClaimsUtilsMultiAddress>;
      readonly signature: PalletAirdropClaimsUtilsMultiAddressSignature;
      readonly statement: Bytes;
    } & Struct;
    readonly isMoveClaim: boolean;
    readonly asMoveClaim: {
      readonly old: PalletAirdropClaimsUtilsMultiAddress;
      readonly new_: PalletAirdropClaimsUtilsMultiAddress;
    } & Struct;
    readonly isForceSetExpiryConfig: boolean;
    readonly asForceSetExpiryConfig: {
      readonly expiryBlock: u64;
      readonly dest: PalletAirdropClaimsUtilsMultiAddress;
    } & Struct;
    readonly isClaimSigned: boolean;
    readonly asClaimSigned: {
      readonly dest: Option<PalletAirdropClaimsUtilsMultiAddress>;
    } & Struct;
    readonly type: 'Claim' | 'MintClaim' | 'ClaimAttest' | 'MoveClaim' | 'ForceSetExpiryConfig' | 'ClaimSigned';
  }

  /** @name PalletAirdropClaimsUtilsMultiAddressSignature (413) */
  interface PalletAirdropClaimsUtilsMultiAddressSignature extends Enum {
    readonly isEvm: boolean;
    readonly asEvm: PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature;
    readonly isNative: boolean;
    readonly asNative: PalletAirdropClaimsUtilsSr25519Signature;
    readonly type: 'Evm' | 'Native';
  }

  /** @name PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature (414) */
  interface PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature extends U8aFixed {}

  /** @name PalletAirdropClaimsUtilsSr25519Signature (415) */
  interface PalletAirdropClaimsUtilsSr25519Signature extends U8aFixed {}

  /** @name PalletAirdropClaimsStatementKind (421) */
  interface PalletAirdropClaimsStatementKind extends Enum {
    readonly isRegular: boolean;
    readonly isSafe: boolean;
    readonly type: 'Regular' | 'Safe';
  }

  /** @name PalletProxyCall (422) */
  interface PalletProxyCall extends Enum {
    readonly isProxy: boolean;
    readonly asProxy: {
      readonly real: MultiAddress;
      readonly forceProxyType: Option<TangleTestnetRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly isAddProxy: boolean;
    readonly asAddProxy: {
      readonly delegate: MultiAddress;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly delay: u64;
    } & Struct;
    readonly isRemoveProxy: boolean;
    readonly asRemoveProxy: {
      readonly delegate: MultiAddress;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly delay: u64;
    } & Struct;
    readonly isRemoveProxies: boolean;
    readonly isCreatePure: boolean;
    readonly asCreatePure: {
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly delay: u64;
      readonly index: u16;
    } & Struct;
    readonly isKillPure: boolean;
    readonly asKillPure: {
      readonly spawner: MultiAddress;
      readonly proxyType: TangleTestnetRuntimeProxyType;
      readonly index: u16;
      readonly height: Compact<u64>;
      readonly extIndex: Compact<u32>;
    } & Struct;
    readonly isAnnounce: boolean;
    readonly asAnnounce: {
      readonly real: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isRemoveAnnouncement: boolean;
    readonly asRemoveAnnouncement: {
      readonly real: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isRejectAnnouncement: boolean;
    readonly asRejectAnnouncement: {
      readonly delegate: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAnnounced: boolean;
    readonly asProxyAnnounced: {
      readonly delegate: MultiAddress;
      readonly real: MultiAddress;
      readonly forceProxyType: Option<TangleTestnetRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly type: 'Proxy' | 'AddProxy' | 'RemoveProxy' | 'RemoveProxies' | 'CreatePure' | 'KillPure' | 'Announce' | 'RemoveAnnouncement' | 'RejectAnnouncement' | 'ProxyAnnounced';
  }

  /** @name PalletMultiAssetDelegationCall (424) */
  interface PalletMultiAssetDelegationCall extends Enum {
    readonly isJoinOperators: boolean;
    readonly asJoinOperators: {
      readonly bondAmount: u128;
    } & Struct;
    readonly isScheduleLeaveOperators: boolean;
    readonly isCancelLeaveOperators: boolean;
    readonly isExecuteLeaveOperators: boolean;
    readonly isOperatorBondMore: boolean;
    readonly asOperatorBondMore: {
      readonly additionalBond: u128;
    } & Struct;
    readonly isScheduleOperatorUnstake: boolean;
    readonly asScheduleOperatorUnstake: {
      readonly unstakeAmount: u128;
    } & Struct;
    readonly isExecuteOperatorUnstake: boolean;
    readonly isCancelOperatorUnstake: boolean;
    readonly isGoOffline: boolean;
    readonly isGoOnline: boolean;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
      readonly evmAddress: Option<H160>;
      readonly lockMultiplier: Option<TanglePrimitivesRewardsLockMultiplier>;
    } & Struct;
    readonly isScheduleWithdraw: boolean;
    readonly asScheduleWithdraw: {
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isExecuteWithdraw: boolean;
    readonly asExecuteWithdraw: {
      readonly evmAddress: Option<H160>;
    } & Struct;
    readonly isCancelWithdraw: boolean;
    readonly asCancelWithdraw: {
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
      readonly blueprintSelection: PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection;
    } & Struct;
    readonly isScheduleDelegatorUnstake: boolean;
    readonly asScheduleDelegatorUnstake: {
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isExecuteDelegatorUnstake: boolean;
    readonly isCancelDelegatorUnstake: boolean;
    readonly asCancelDelegatorUnstake: {
      readonly operator: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly amount: u128;
    } & Struct;
    readonly isDelegateNomination: boolean;
    readonly asDelegateNomination: {
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly blueprintSelection: PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection;
    } & Struct;
    readonly isScheduleNominationUnstake: boolean;
    readonly asScheduleNominationUnstake: {
      readonly operator: AccountId32;
      readonly amount: u128;
      readonly blueprintSelection: PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection;
    } & Struct;
    readonly isExecuteNominationUnstake: boolean;
    readonly asExecuteNominationUnstake: {
      readonly operator: AccountId32;
    } & Struct;
    readonly isCancelNominationUnstake: boolean;
    readonly asCancelNominationUnstake: {
      readonly operator: AccountId32;
    } & Struct;
    readonly isAddBlueprintId: boolean;
    readonly asAddBlueprintId: {
      readonly blueprintId: u64;
    } & Struct;
    readonly isRemoveBlueprintId: boolean;
    readonly asRemoveBlueprintId: {
      readonly blueprintId: u64;
    } & Struct;
    readonly type: 'JoinOperators' | 'ScheduleLeaveOperators' | 'CancelLeaveOperators' | 'ExecuteLeaveOperators' | 'OperatorBondMore' | 'ScheduleOperatorUnstake' | 'ExecuteOperatorUnstake' | 'CancelOperatorUnstake' | 'GoOffline' | 'GoOnline' | 'Deposit' | 'ScheduleWithdraw' | 'ExecuteWithdraw' | 'CancelWithdraw' | 'Delegate' | 'ScheduleDelegatorUnstake' | 'ExecuteDelegatorUnstake' | 'CancelDelegatorUnstake' | 'DelegateNomination' | 'ScheduleNominationUnstake' | 'ExecuteNominationUnstake' | 'CancelNominationUnstake' | 'AddBlueprintId' | 'RemoveBlueprintId';
  }

  /** @name PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection (426) */
  interface PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection extends Enum {
    readonly isFixed: boolean;
    readonly asFixed: Vec<u64>;
    readonly isAll: boolean;
    readonly type: 'Fixed' | 'All';
  }

  /** @name TangleTestnetRuntimeMaxDelegatorBlueprints (427) */
  type TangleTestnetRuntimeMaxDelegatorBlueprints = Null;

  /** @name PalletServicesModuleCall (430) */
  interface PalletServicesModuleCall extends Enum {
    readonly isCreateBlueprint: boolean;
    readonly asCreateBlueprint: {
      readonly blueprint: TanglePrimitivesServicesServiceServiceBlueprint;
    } & Struct;
    readonly isPreRegister: boolean;
    readonly asPreRegister: {
      readonly blueprintId: Compact<u64>;
    } & Struct;
    readonly isRegister: boolean;
    readonly asRegister: {
      readonly blueprintId: Compact<u64>;
      readonly preferences: TanglePrimitivesServicesTypesOperatorPreferences;
      readonly registrationArgs: Vec<TanglePrimitivesServicesField>;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isUnregister: boolean;
    readonly asUnregister: {
      readonly blueprintId: Compact<u64>;
    } & Struct;
    readonly isRequest: boolean;
    readonly asRequest: {
      readonly evmOrigin: Option<H160>;
      readonly blueprintId: Compact<u64>;
      readonly permittedCallers: Vec<AccountId32>;
      readonly operators: Vec<AccountId32>;
      readonly requestArgs: Vec<TanglePrimitivesServicesField>;
      readonly assetSecurityRequirements: Vec<TanglePrimitivesServicesTypesAssetSecurityRequirement>;
      readonly ttl: Compact<u64>;
      readonly paymentAsset: TanglePrimitivesServicesTypesAssetU128;
      readonly value: Compact<u128>;
      readonly membershipModel: TanglePrimitivesServicesTypesMembershipModel;
    } & Struct;
    readonly isApprove: boolean;
    readonly asApprove: {
      readonly requestId: Compact<u64>;
      readonly securityCommitments: Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>;
    } & Struct;
    readonly isReject: boolean;
    readonly asReject: {
      readonly requestId: Compact<u64>;
    } & Struct;
    readonly isTerminate: boolean;
    readonly asTerminate: {
      readonly serviceId: Compact<u64>;
    } & Struct;
    readonly isCall: boolean;
    readonly asCall: {
      readonly serviceId: Compact<u64>;
      readonly job: Compact<u8>;
      readonly args: Vec<TanglePrimitivesServicesField>;
    } & Struct;
    readonly isSubmitResult: boolean;
    readonly asSubmitResult: {
      readonly serviceId: Compact<u64>;
      readonly callId: Compact<u64>;
      readonly result: Vec<TanglePrimitivesServicesField>;
    } & Struct;
    readonly isSlash: boolean;
    readonly asSlash: {
      readonly offender: AccountId32;
      readonly serviceId: Compact<u64>;
      readonly slashPercent: Compact<Percent>;
    } & Struct;
    readonly isDispute: boolean;
    readonly asDispute: {
      readonly era: Compact<u32>;
      readonly index: Compact<u32>;
    } & Struct;
    readonly isUpdateMasterBlueprintServiceManager: boolean;
    readonly asUpdateMasterBlueprintServiceManager: {
      readonly address: H160;
    } & Struct;
    readonly isJoinService: boolean;
    readonly asJoinService: {
      readonly instanceId: u64;
      readonly securityCommitments: Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>;
    } & Struct;
    readonly isLeaveService: boolean;
    readonly asLeaveService: {
      readonly instanceId: u64;
    } & Struct;
    readonly isUpdateRpcAddress: boolean;
    readonly asUpdateRpcAddress: {
      readonly blueprintId: Compact<u64>;
      readonly rpcAddress: Bytes;
    } & Struct;
    readonly isRequestWithSignedPriceQuotes: boolean;
    readonly asRequestWithSignedPriceQuotes: {
      readonly evmOrigin: Option<H160>;
      readonly blueprintId: Compact<u64>;
      readonly permittedCallers: Vec<AccountId32>;
      readonly operators: Vec<AccountId32>;
      readonly requestArgs: Vec<TanglePrimitivesServicesField>;
      readonly assetSecurityRequirements: Vec<TanglePrimitivesServicesTypesAssetSecurityRequirement>;
      readonly ttl: Compact<u64>;
      readonly paymentAsset: TanglePrimitivesServicesTypesAssetU128;
      readonly membershipModel: TanglePrimitivesServicesTypesMembershipModel;
      readonly pricingQuotes: Vec<TanglePrimitivesServicesPricingPricingQuote>;
      readonly operatorSignatures: Vec<U8aFixed>;
      readonly securityCommitments: Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>;
    } & Struct;
    readonly isHeartbeat: boolean;
    readonly asHeartbeat: {
      readonly serviceId: Compact<u64>;
      readonly blueprintId: Compact<u64>;
      readonly metricsData: Bytes;
      readonly signature: U8aFixed;
    } & Struct;
    readonly isUpdateDefaultHeartbeatThreshold: boolean;
    readonly asUpdateDefaultHeartbeatThreshold: {
      readonly threshold: u8;
    } & Struct;
    readonly isUpdateDefaultHeartbeatInterval: boolean;
    readonly asUpdateDefaultHeartbeatInterval: {
      readonly interval: u64;
    } & Struct;
    readonly isUpdateDefaultHeartbeatSlashingWindow: boolean;
    readonly asUpdateDefaultHeartbeatSlashingWindow: {
      readonly window: u64;
    } & Struct;
    readonly type: 'CreateBlueprint' | 'PreRegister' | 'Register' | 'Unregister' | 'Request' | 'Approve' | 'Reject' | 'Terminate' | 'Call' | 'SubmitResult' | 'Slash' | 'Dispute' | 'UpdateMasterBlueprintServiceManager' | 'JoinService' | 'LeaveService' | 'UpdateRpcAddress' | 'RequestWithSignedPriceQuotes' | 'Heartbeat' | 'UpdateDefaultHeartbeatThreshold' | 'UpdateDefaultHeartbeatInterval' | 'UpdateDefaultHeartbeatSlashingWindow';
  }

  /** @name TanglePrimitivesServicesServiceServiceBlueprint (431) */
  interface TanglePrimitivesServicesServiceServiceBlueprint extends Struct {
    readonly metadata: TanglePrimitivesServicesServiceServiceMetadata;
    readonly jobs: Vec<TanglePrimitivesServicesJobsJobDefinition>;
    readonly registrationParams: Vec<TanglePrimitivesServicesFieldFieldType>;
    readonly requestParams: Vec<TanglePrimitivesServicesFieldFieldType>;
    readonly manager: TanglePrimitivesServicesServiceBlueprintServiceManager;
    readonly masterManagerRevision: TanglePrimitivesServicesServiceMasterBlueprintServiceManagerRevision;
    readonly sources: Vec<TanglePrimitivesServicesSourcesBlueprintSource>;
    readonly supportedMembershipModels: Vec<TanglePrimitivesServicesTypesMembershipModelType>;
  }

  /** @name TanglePrimitivesServicesServiceServiceMetadata (432) */
  interface TanglePrimitivesServicesServiceServiceMetadata extends Struct {
    readonly name: Bytes;
    readonly description: Option<Bytes>;
    readonly author: Option<Bytes>;
    readonly category: Option<Bytes>;
    readonly codeRepository: Option<Bytes>;
    readonly logo: Option<Bytes>;
    readonly website: Option<Bytes>;
    readonly license: Option<Bytes>;
  }

  /** @name TanglePrimitivesServicesJobsJobDefinition (437) */
  interface TanglePrimitivesServicesJobsJobDefinition extends Struct {
    readonly metadata: TanglePrimitivesServicesJobsJobMetadata;
    readonly params: Vec<TanglePrimitivesServicesFieldFieldType>;
    readonly result: Vec<TanglePrimitivesServicesFieldFieldType>;
    readonly pricingModel: TanglePrimitivesServicesTypesPricingModelU32;
  }

  /** @name TanglePrimitivesServicesJobsJobMetadata (438) */
  interface TanglePrimitivesServicesJobsJobMetadata extends Struct {
    readonly name: Bytes;
    readonly description: Option<Bytes>;
  }

  /** @name TanglePrimitivesServicesTypesPricingModelU32 (441) */
  interface TanglePrimitivesServicesTypesPricingModelU32 extends Enum {
    readonly isPayOnce: boolean;
    readonly asPayOnce: {
      readonly amount: u128;
    } & Struct;
    readonly isSubscription: boolean;
    readonly asSubscription: {
      readonly ratePerInterval: u128;
      readonly interval: u32;
      readonly maybeEnd: Option<u32>;
    } & Struct;
    readonly isEventDriven: boolean;
    readonly asEventDriven: {
      readonly rewardPerEvent: u128;
    } & Struct;
    readonly type: 'PayOnce' | 'Subscription' | 'EventDriven';
  }

  /** @name TanglePrimitivesServicesServiceBlueprintServiceManager (443) */
  interface TanglePrimitivesServicesServiceBlueprintServiceManager extends Enum {
    readonly isEvm: boolean;
    readonly asEvm: H160;
    readonly type: 'Evm';
  }

  /** @name TanglePrimitivesServicesServiceMasterBlueprintServiceManagerRevision (444) */
  interface TanglePrimitivesServicesServiceMasterBlueprintServiceManagerRevision extends Enum {
    readonly isLatest: boolean;
    readonly isSpecific: boolean;
    readonly asSpecific: u32;
    readonly type: 'Latest' | 'Specific';
  }

  /** @name TanglePrimitivesServicesSourcesBlueprintSource (446) */
  interface TanglePrimitivesServicesSourcesBlueprintSource extends Enum {
    readonly isWasm: boolean;
    readonly asWasm: {
      readonly runtime: TanglePrimitivesServicesSourcesWasmRuntime;
      readonly fetcher: TanglePrimitivesServicesSourcesWasmFetcher;
    } & Struct;
    readonly isNative: boolean;
    readonly asNative: TanglePrimitivesServicesSourcesNativeFetcher;
    readonly isContainer: boolean;
    readonly asContainer: TanglePrimitivesServicesSourcesImageRegistryFetcher;
    readonly isTesting: boolean;
    readonly asTesting: TanglePrimitivesServicesSourcesTestFetcher;
    readonly type: 'Wasm' | 'Native' | 'Container' | 'Testing';
  }

  /** @name TanglePrimitivesServicesSourcesWasmRuntime (447) */
  interface TanglePrimitivesServicesSourcesWasmRuntime extends Enum {
    readonly isWasmtime: boolean;
    readonly isWasmer: boolean;
    readonly type: 'Wasmtime' | 'Wasmer';
  }

  /** @name TanglePrimitivesServicesSourcesWasmFetcher (448) */
  interface TanglePrimitivesServicesSourcesWasmFetcher extends Enum {
    readonly isIpfs: boolean;
    readonly asIpfs: Bytes;
    readonly isGithub: boolean;
    readonly asGithub: TanglePrimitivesServicesSourcesGithubFetcher;
    readonly type: 'Ipfs' | 'Github';
  }

  /** @name TanglePrimitivesServicesSourcesGithubFetcher (450) */
  interface TanglePrimitivesServicesSourcesGithubFetcher extends Struct {
    readonly owner: Bytes;
    readonly repo: Bytes;
    readonly tag: Bytes;
    readonly binaries: Vec<TanglePrimitivesServicesSourcesBlueprintBinary>;
  }

  /** @name TanglePrimitivesServicesSourcesBlueprintBinary (458) */
  interface TanglePrimitivesServicesSourcesBlueprintBinary extends Struct {
    readonly arch: TanglePrimitivesServicesSourcesArchitecture;
    readonly os: TanglePrimitivesServicesSourcesOperatingSystem;
    readonly name: Bytes;
    readonly sha256: U8aFixed;
  }

  /** @name TanglePrimitivesServicesSourcesArchitecture (459) */
  interface TanglePrimitivesServicesSourcesArchitecture extends Enum {
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

  /** @name TanglePrimitivesServicesSourcesOperatingSystem (460) */
  interface TanglePrimitivesServicesSourcesOperatingSystem extends Enum {
    readonly isUnknown: boolean;
    readonly isLinux: boolean;
    readonly isWindows: boolean;
    readonly isMacOS: boolean;
    readonly isBsd: boolean;
    readonly type: 'Unknown' | 'Linux' | 'Windows' | 'MacOS' | 'Bsd';
  }

  /** @name TanglePrimitivesServicesSourcesNativeFetcher (464) */
  interface TanglePrimitivesServicesSourcesNativeFetcher extends Enum {
    readonly isIpfs: boolean;
    readonly asIpfs: Bytes;
    readonly isGithub: boolean;
    readonly asGithub: TanglePrimitivesServicesSourcesGithubFetcher;
    readonly type: 'Ipfs' | 'Github';
  }

  /** @name TanglePrimitivesServicesSourcesImageRegistryFetcher (465) */
  interface TanglePrimitivesServicesSourcesImageRegistryFetcher extends Struct {
    readonly registry_: Bytes;
    readonly image: Bytes;
    readonly tag: Bytes;
  }

  /** @name TanglePrimitivesServicesSourcesTestFetcher (472) */
  interface TanglePrimitivesServicesSourcesTestFetcher extends Struct {
    readonly cargoPackage: Bytes;
    readonly cargoBin: Bytes;
    readonly basePath: Bytes;
  }

  /** @name TanglePrimitivesServicesTypesMembershipModelType (475) */
  interface TanglePrimitivesServicesTypesMembershipModelType extends Enum {
    readonly isFixed: boolean;
    readonly isDynamic: boolean;
    readonly type: 'Fixed' | 'Dynamic';
  }

  /** @name TanglePrimitivesServicesTypesMembershipModel (477) */
  interface TanglePrimitivesServicesTypesMembershipModel extends Enum {
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

  /** @name TanglePrimitivesServicesPricingPricingQuote (481) */
  interface TanglePrimitivesServicesPricingPricingQuote extends Struct {
    readonly blueprintId: u64;
    readonly ttlBlocks: u64;
    readonly totalCostRate: u128;
    readonly timestamp: u64;
    readonly expiry: u64;
    readonly resources: Vec<TanglePrimitivesServicesPricingResourcePricing>;
    readonly securityCommitments: Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>;
  }

  /** @name TanglePrimitivesServicesPricingResourcePricing (483) */
  interface TanglePrimitivesServicesPricingResourcePricing extends Struct {
    readonly kind: Bytes;
    readonly count: u64;
    readonly pricePerUnitRate: u128;
  }

  /** @name PalletTangleLstCall (489) */
  interface PalletTangleLstCall extends Enum {
    readonly isJoin: boolean;
    readonly asJoin: {
      readonly amount: Compact<u128>;
      readonly poolId: u32;
    } & Struct;
    readonly isBondExtra: boolean;
    readonly asBondExtra: {
      readonly poolId: u32;
      readonly extra: PalletTangleLstBondExtra;
    } & Struct;
    readonly isUnbond: boolean;
    readonly asUnbond: {
      readonly memberAccount: MultiAddress;
      readonly poolId: u32;
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
      readonly poolId: u32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly amount: Compact<u128>;
      readonly root: MultiAddress;
      readonly nominator: MultiAddress;
      readonly bouncer: MultiAddress;
      readonly name: Option<Bytes>;
      readonly icon: Option<Bytes>;
    } & Struct;
    readonly isCreateWithPoolId: boolean;
    readonly asCreateWithPoolId: {
      readonly amount: Compact<u128>;
      readonly root: MultiAddress;
      readonly nominator: MultiAddress;
      readonly bouncer: MultiAddress;
      readonly poolId: u32;
      readonly name: Option<Bytes>;
      readonly icon: Option<Bytes>;
    } & Struct;
    readonly isNominate: boolean;
    readonly asNominate: {
      readonly poolId: u32;
      readonly validators: Vec<AccountId32>;
    } & Struct;
    readonly isSetState: boolean;
    readonly asSetState: {
      readonly poolId: u32;
      readonly state: PalletTangleLstPoolsPoolState;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly poolId: u32;
      readonly metadata: Bytes;
    } & Struct;
    readonly isSetConfigs: boolean;
    readonly asSetConfigs: {
      readonly minJoinBond: PalletTangleLstConfigOpU128;
      readonly minCreateBond: PalletTangleLstConfigOpU128;
      readonly maxPools: PalletTangleLstConfigOpU32;
      readonly globalMaxCommission: PalletTangleLstConfigOpPerbill;
    } & Struct;
    readonly isUpdateRoles: boolean;
    readonly asUpdateRoles: {
      readonly poolId: u32;
      readonly newRoot: PalletTangleLstConfigOpAccountId32;
      readonly newNominator: PalletTangleLstConfigOpAccountId32;
      readonly newBouncer: PalletTangleLstConfigOpAccountId32;
    } & Struct;
    readonly isChill: boolean;
    readonly asChill: {
      readonly poolId: u32;
    } & Struct;
    readonly isBondExtraOther: boolean;
    readonly asBondExtraOther: {
      readonly member: MultiAddress;
      readonly poolId: u32;
      readonly extra: PalletTangleLstBondExtra;
    } & Struct;
    readonly isSetCommission: boolean;
    readonly asSetCommission: {
      readonly poolId: u32;
      readonly newCommission: Option<ITuple<[Perbill, AccountId32]>>;
    } & Struct;
    readonly isSetCommissionMax: boolean;
    readonly asSetCommissionMax: {
      readonly poolId: u32;
      readonly maxCommission: Perbill;
    } & Struct;
    readonly isSetCommissionChangeRate: boolean;
    readonly asSetCommissionChangeRate: {
      readonly poolId: u32;
      readonly changeRate: PalletTangleLstCommissionCommissionChangeRate;
    } & Struct;
    readonly isClaimCommission: boolean;
    readonly asClaimCommission: {
      readonly poolId: u32;
    } & Struct;
    readonly isAdjustPoolDeposit: boolean;
    readonly asAdjustPoolDeposit: {
      readonly poolId: u32;
    } & Struct;
    readonly isSetCommissionClaimPermission: boolean;
    readonly asSetCommissionClaimPermission: {
      readonly poolId: u32;
      readonly permission: Option<PalletTangleLstCommissionCommissionClaimPermission>;
    } & Struct;
    readonly isSetLastPoolId: boolean;
    readonly asSetLastPoolId: {
      readonly poolId: u32;
    } & Struct;
    readonly type: 'Join' | 'BondExtra' | 'Unbond' | 'PoolWithdrawUnbonded' | 'WithdrawUnbonded' | 'Create' | 'CreateWithPoolId' | 'Nominate' | 'SetState' | 'SetMetadata' | 'SetConfigs' | 'UpdateRoles' | 'Chill' | 'BondExtraOther' | 'SetCommission' | 'SetCommissionMax' | 'SetCommissionChangeRate' | 'ClaimCommission' | 'AdjustPoolDeposit' | 'SetCommissionClaimPermission' | 'SetLastPoolId';
  }

  /** @name PalletTangleLstBondExtra (490) */
  interface PalletTangleLstBondExtra extends Enum {
    readonly isFreeBalance: boolean;
    readonly asFreeBalance: u128;
    readonly type: 'FreeBalance';
  }

  /** @name PalletTangleLstConfigOpU128 (495) */
  interface PalletTangleLstConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletTangleLstConfigOpU32 (496) */
  interface PalletTangleLstConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletTangleLstConfigOpPerbill (497) */
  interface PalletTangleLstConfigOpPerbill extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Perbill;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletTangleLstConfigOpAccountId32 (498) */
  interface PalletTangleLstConfigOpAccountId32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: AccountId32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletRewardsCall (499) */
  interface PalletRewardsCall extends Enum {
    readonly isClaimRewardsOther: boolean;
    readonly asClaimRewardsOther: {
      readonly who: AccountId32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
    } & Struct;
    readonly isManageAssetRewardVault: boolean;
    readonly asManageAssetRewardVault: {
      readonly vaultId: u32;
      readonly asset: TanglePrimitivesServicesTypesAssetU128;
      readonly action: PalletRewardsAssetAction;
    } & Struct;
    readonly isCreateRewardVault: boolean;
    readonly asCreateRewardVault: {
      readonly vaultId: u32;
      readonly newConfig: PalletRewardsRewardConfigForAssetVault;
    } & Struct;
    readonly isUpdateVaultRewardConfig: boolean;
    readonly asUpdateVaultRewardConfig: {
      readonly vaultId: u32;
      readonly newConfig: PalletRewardsRewardConfigForAssetVault;
    } & Struct;
    readonly isUpdateDecayConfig: boolean;
    readonly asUpdateDecayConfig: {
      readonly startPeriod: u64;
      readonly rate: Perbill;
    } & Struct;
    readonly isUpdateApyBlocks: boolean;
    readonly asUpdateApyBlocks: {
      readonly blocks: u64;
    } & Struct;
    readonly isSetVaultMetadata: boolean;
    readonly asSetVaultMetadata: {
      readonly vaultId: u32;
      readonly name: Bytes;
      readonly logo: Bytes;
    } & Struct;
    readonly isRemoveVaultMetadata: boolean;
    readonly asRemoveVaultMetadata: {
      readonly vaultId: u32;
    } & Struct;
    readonly isClaimRewards: boolean;
    readonly type: 'ClaimRewardsOther' | 'ManageAssetRewardVault' | 'CreateRewardVault' | 'UpdateVaultRewardConfig' | 'UpdateDecayConfig' | 'UpdateApyBlocks' | 'SetVaultMetadata' | 'RemoveVaultMetadata' | 'ClaimRewards';
  }

  /** @name PalletIsmpCall (500) */
  interface PalletIsmpCall extends Enum {
    readonly isHandleUnsigned: boolean;
    readonly asHandleUnsigned: {
      readonly messages: Vec<IsmpMessagingMessage>;
    } & Struct;
    readonly isCreateConsensusClient: boolean;
    readonly asCreateConsensusClient: {
      readonly message: IsmpMessagingCreateConsensusState;
    } & Struct;
    readonly isUpdateConsensusState: boolean;
    readonly asUpdateConsensusState: {
      readonly message: PalletIsmpUtilsUpdateConsensusState;
    } & Struct;
    readonly isFundMessage: boolean;
    readonly asFundMessage: {
      readonly message: PalletIsmpUtilsFundMessageParams;
    } & Struct;
    readonly type: 'HandleUnsigned' | 'CreateConsensusClient' | 'UpdateConsensusState' | 'FundMessage';
  }

  /** @name IsmpMessagingMessage (502) */
  interface IsmpMessagingMessage extends Enum {
    readonly isConsensus: boolean;
    readonly asConsensus: IsmpMessagingConsensusMessage;
    readonly isFraudProof: boolean;
    readonly asFraudProof: IsmpMessagingFraudProofMessage;
    readonly isRequest: boolean;
    readonly asRequest: IsmpMessagingRequestMessage;
    readonly isResponse: boolean;
    readonly asResponse: IsmpMessagingResponseMessage;
    readonly isTimeout: boolean;
    readonly asTimeout: IsmpMessagingTimeoutMessage;
    readonly type: 'Consensus' | 'FraudProof' | 'Request' | 'Response' | 'Timeout';
  }

  /** @name IsmpMessagingConsensusMessage (503) */
  interface IsmpMessagingConsensusMessage extends Struct {
    readonly consensusProof: Bytes;
    readonly consensusStateId: U8aFixed;
    readonly signer: Bytes;
  }

  /** @name IsmpMessagingFraudProofMessage (504) */
  interface IsmpMessagingFraudProofMessage extends Struct {
    readonly proof1: Bytes;
    readonly proof2: Bytes;
    readonly consensusStateId: U8aFixed;
  }

  /** @name IsmpMessagingRequestMessage (505) */
  interface IsmpMessagingRequestMessage extends Struct {
    readonly requests: Vec<IsmpRouterPostRequest>;
    readonly proof: IsmpMessagingProof;
    readonly signer: Bytes;
  }

  /** @name IsmpRouterPostRequest (507) */
  interface IsmpRouterPostRequest extends Struct {
    readonly source: IsmpHostStateMachine;
    readonly dest: IsmpHostStateMachine;
    readonly nonce: u64;
    readonly from: Bytes;
    readonly to: Bytes;
    readonly timeoutTimestamp: u64;
    readonly body: Bytes;
  }

  /** @name IsmpMessagingProof (508) */
  interface IsmpMessagingProof extends Struct {
    readonly height: IsmpConsensusStateMachineHeight;
    readonly proof: Bytes;
  }

  /** @name IsmpMessagingResponseMessage (509) */
  interface IsmpMessagingResponseMessage extends Struct {
    readonly datagram: IsmpRouterRequestResponse;
    readonly proof: IsmpMessagingProof;
    readonly signer: Bytes;
  }

  /** @name IsmpRouterRequestResponse (510) */
  interface IsmpRouterRequestResponse extends Enum {
    readonly isRequest: boolean;
    readonly asRequest: Vec<IsmpRouterRequest>;
    readonly isResponse: boolean;
    readonly asResponse: Vec<IsmpRouterResponse>;
    readonly type: 'Request' | 'Response';
  }

  /** @name IsmpRouterRequest (512) */
  interface IsmpRouterRequest extends Enum {
    readonly isPost: boolean;
    readonly asPost: IsmpRouterPostRequest;
    readonly isGet: boolean;
    readonly asGet: IsmpRouterGetRequest;
    readonly type: 'Post' | 'Get';
  }

  /** @name IsmpRouterGetRequest (513) */
  interface IsmpRouterGetRequest extends Struct {
    readonly source: IsmpHostStateMachine;
    readonly dest: IsmpHostStateMachine;
    readonly nonce: u64;
    readonly from: Bytes;
    readonly keys_: Vec<Bytes>;
    readonly height: u64;
    readonly context: Bytes;
    readonly timeoutTimestamp: u64;
  }

  /** @name IsmpRouterResponse (515) */
  interface IsmpRouterResponse extends Enum {
    readonly isPost: boolean;
    readonly asPost: IsmpRouterPostResponse;
    readonly isGet: boolean;
    readonly asGet: IsmpRouterGetResponse;
    readonly type: 'Post' | 'Get';
  }

  /** @name IsmpRouterPostResponse (516) */
  interface IsmpRouterPostResponse extends Struct {
    readonly post: IsmpRouterPostRequest;
    readonly response: Bytes;
    readonly timeoutTimestamp: u64;
  }

  /** @name IsmpRouterGetResponse (517) */
  interface IsmpRouterGetResponse extends Struct {
    readonly getRequest: IsmpRouterGetRequest;
    readonly getValues: Vec<IsmpRouterStorageValue>;
  }

  /** @name IsmpRouterStorageValue (519) */
  interface IsmpRouterStorageValue extends Struct {
    readonly key: Bytes;
    readonly value: Option<Bytes>;
  }

  /** @name IsmpMessagingTimeoutMessage (521) */
  interface IsmpMessagingTimeoutMessage extends Enum {
    readonly isPost: boolean;
    readonly asPost: {
      readonly requests: Vec<IsmpRouterRequest>;
      readonly timeoutProof: IsmpMessagingProof;
    } & Struct;
    readonly isPostResponse: boolean;
    readonly asPostResponse: {
      readonly responses: Vec<IsmpRouterPostResponse>;
      readonly timeoutProof: IsmpMessagingProof;
    } & Struct;
    readonly isGet: boolean;
    readonly asGet: {
      readonly requests: Vec<IsmpRouterRequest>;
    } & Struct;
    readonly type: 'Post' | 'PostResponse' | 'Get';
  }

  /** @name IsmpMessagingCreateConsensusState (523) */
  interface IsmpMessagingCreateConsensusState extends Struct {
    readonly consensusState: Bytes;
    readonly consensusClientId: U8aFixed;
    readonly consensusStateId: U8aFixed;
    readonly unbondingPeriod: u64;
    readonly challengePeriods: BTreeMap<IsmpHostStateMachine, u64>;
    readonly stateMachineCommitments: Vec<ITuple<[IsmpConsensusStateMachineId, IsmpMessagingStateCommitmentHeight]>>;
  }

  /** @name IsmpMessagingStateCommitmentHeight (529) */
  interface IsmpMessagingStateCommitmentHeight extends Struct {
    readonly commitment: IsmpConsensusStateCommitment;
    readonly height: u64;
  }

  /** @name IsmpConsensusStateCommitment (530) */
  interface IsmpConsensusStateCommitment extends Struct {
    readonly timestamp: u64;
    readonly overlayRoot: Option<H256>;
    readonly stateRoot: H256;
  }

  /** @name PalletIsmpUtilsUpdateConsensusState (531) */
  interface PalletIsmpUtilsUpdateConsensusState extends Struct {
    readonly consensusStateId: U8aFixed;
    readonly unbondingPeriod: Option<u64>;
    readonly challengePeriods: BTreeMap<IsmpHostStateMachine, u64>;
  }

  /** @name PalletIsmpUtilsFundMessageParams (532) */
  interface PalletIsmpUtilsFundMessageParams extends Struct {
    readonly commitment: PalletIsmpUtilsMessageCommitment;
    readonly amount: u128;
  }

  /** @name PalletIsmpUtilsMessageCommitment (533) */
  interface PalletIsmpUtilsMessageCommitment extends Enum {
    readonly isRequest: boolean;
    readonly asRequest: H256;
    readonly isResponse: boolean;
    readonly asResponse: H256;
    readonly type: 'Request' | 'Response';
  }

  /** @name IsmpGrandpaCall (534) */
  interface IsmpGrandpaCall extends Enum {
    readonly isAddStateMachines: boolean;
    readonly asAddStateMachines: {
      readonly newStateMachines: Vec<IsmpGrandpaAddStateMachine>;
    } & Struct;
    readonly isRemoveStateMachines: boolean;
    readonly asRemoveStateMachines: {
      readonly stateMachines: Vec<IsmpHostStateMachine>;
    } & Struct;
    readonly type: 'AddStateMachines' | 'RemoveStateMachines';
  }

  /** @name IsmpGrandpaAddStateMachine (536) */
  interface IsmpGrandpaAddStateMachine extends Struct {
    readonly stateMachine: IsmpHostStateMachine;
    readonly slotDuration: u64;
  }

  /** @name PalletTokenGatewayCall (537) */
  interface PalletTokenGatewayCall extends Enum {
    readonly isTeleport: boolean;
    readonly asTeleport: {
      readonly params: PalletTokenGatewayTeleportParams;
    } & Struct;
    readonly isSetTokenGatewayAddresses: boolean;
    readonly asSetTokenGatewayAddresses: {
      readonly addresses: BTreeMap<IsmpHostStateMachine, Bytes>;
    } & Struct;
    readonly isCreateErc6160Asset: boolean;
    readonly asCreateErc6160Asset: {
      readonly asset: PalletTokenGatewayAssetRegistration;
    } & Struct;
    readonly isUpdateErc6160Asset: boolean;
    readonly asUpdateErc6160Asset: {
      readonly asset: TokenGatewayPrimitivesGatewayAssetUpdate;
    } & Struct;
    readonly isUpdateAssetPrecision: boolean;
    readonly asUpdateAssetPrecision: {
      readonly update: PalletTokenGatewayPrecisionUpdate;
    } & Struct;
    readonly type: 'Teleport' | 'SetTokenGatewayAddresses' | 'CreateErc6160Asset' | 'UpdateErc6160Asset' | 'UpdateAssetPrecision';
  }

  /** @name PalletTokenGatewayTeleportParams (538) */
  interface PalletTokenGatewayTeleportParams extends Struct {
    readonly assetId: u128;
    readonly destination: IsmpHostStateMachine;
    readonly recepient: H256;
    readonly amount: u128;
    readonly timeout: u64;
    readonly tokenGateway: Bytes;
    readonly relayerFee: u128;
    readonly callData: Option<Bytes>;
    readonly redeem: bool;
  }

  /** @name PalletTokenGatewayAssetRegistration (542) */
  interface PalletTokenGatewayAssetRegistration extends Struct {
    readonly localId: u128;
    readonly reg: TokenGatewayPrimitivesGatewayAssetRegistration;
    readonly native: bool;
    readonly precision: BTreeMap<IsmpHostStateMachine, u8>;
  }

  /** @name TokenGatewayPrimitivesGatewayAssetRegistration (543) */
  interface TokenGatewayPrimitivesGatewayAssetRegistration extends Struct {
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly chains: Vec<IsmpHostStateMachine>;
    readonly minimumBalance: Option<u128>;
  }

  /** @name TokenGatewayPrimitivesGatewayAssetUpdate (548) */
  interface TokenGatewayPrimitivesGatewayAssetUpdate extends Struct {
    readonly assetId: H256;
    readonly addChains: Vec<IsmpHostStateMachine>;
    readonly removeChains: Vec<IsmpHostStateMachine>;
    readonly newAdmins: Vec<ITuple<[IsmpHostStateMachine, H160]>>;
  }

  /** @name PalletTokenGatewayPrecisionUpdate (553) */
  interface PalletTokenGatewayPrecisionUpdate extends Struct {
    readonly assetId: u128;
    readonly precisions: BTreeMap<IsmpHostStateMachine, u8>;
  }

  /** @name PalletCreditsCall (554) */
  interface PalletCreditsCall extends Enum {
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isClaimCredits: boolean;
    readonly asClaimCredits: {
      readonly amountToClaim: Compact<u128>;
      readonly offchainAccountId: Bytes;
    } & Struct;
    readonly isClaimCreditsWithAsset: boolean;
    readonly asClaimCreditsWithAsset: {
      readonly amountToClaim: Compact<u128>;
      readonly offchainAccountId: Bytes;
      readonly assetId: u128;
    } & Struct;
    readonly isSetStakeTiers: boolean;
    readonly asSetStakeTiers: {
      readonly newTiers: Vec<PalletCreditsStakeTier>;
    } & Struct;
    readonly isSetAssetStakeTiers: boolean;
    readonly asSetAssetStakeTiers: {
      readonly assetId: u128;
      readonly newTiers: Vec<PalletCreditsStakeTier>;
    } & Struct;
    readonly type: 'Burn' | 'ClaimCredits' | 'ClaimCreditsWithAsset' | 'SetStakeTiers' | 'SetAssetStakeTiers';
  }

  /** @name PalletCreditsStakeTier (556) */
  interface PalletCreditsStakeTier extends Struct {
    readonly threshold: Compact<u128>;
    readonly ratePerBlock: Compact<u128>;
  }

  /** @name PalletSudoError (557) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name PalletAssetsAssetDetails (559) */
  interface PalletAssetsAssetDetails extends Struct {
    readonly owner: AccountId32;
    readonly issuer: AccountId32;
    readonly admin: AccountId32;
    readonly freezer: AccountId32;
    readonly supply: u128;
    readonly deposit: u128;
    readonly minBalance: u128;
    readonly isSufficient: bool;
    readonly accounts: u32;
    readonly sufficients: u32;
    readonly approvals: u32;
    readonly status: PalletAssetsAssetStatus;
  }

  /** @name PalletAssetsAssetStatus (560) */
  interface PalletAssetsAssetStatus extends Enum {
    readonly isLive: boolean;
    readonly isFrozen: boolean;
    readonly isDestroying: boolean;
    readonly type: 'Live' | 'Frozen' | 'Destroying';
  }

  /** @name PalletAssetsAssetAccount (562) */
  interface PalletAssetsAssetAccount extends Struct {
    readonly balance: u128;
    readonly status: PalletAssetsAccountStatus;
    readonly reason: PalletAssetsExistenceReason;
    readonly extra: Null;
  }

  /** @name PalletAssetsAccountStatus (563) */
  interface PalletAssetsAccountStatus extends Enum {
    readonly isLiquid: boolean;
    readonly isFrozen: boolean;
    readonly isBlocked: boolean;
    readonly type: 'Liquid' | 'Frozen' | 'Blocked';
  }

  /** @name PalletAssetsExistenceReason (564) */
  interface PalletAssetsExistenceReason extends Enum {
    readonly isConsumer: boolean;
    readonly isSufficient: boolean;
    readonly isDepositHeld: boolean;
    readonly asDepositHeld: u128;
    readonly isDepositRefunded: boolean;
    readonly isDepositFrom: boolean;
    readonly asDepositFrom: ITuple<[AccountId32, u128]>;
    readonly type: 'Consumer' | 'Sufficient' | 'DepositHeld' | 'DepositRefunded' | 'DepositFrom';
  }

  /** @name PalletAssetsApproval (566) */
  interface PalletAssetsApproval extends Struct {
    readonly amount: u128;
    readonly deposit: u128;
  }

  /** @name PalletAssetsAssetMetadata (567) */
  interface PalletAssetsAssetMetadata extends Struct {
    readonly deposit: u128;
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly isFrozen: bool;
  }

  /** @name PalletAssetsError (569) */
  interface PalletAssetsError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isNoAccount: boolean;
    readonly isNoPermission: boolean;
    readonly isUnknown: boolean;
    readonly isFrozen: boolean;
    readonly isInUse: boolean;
    readonly isBadWitness: boolean;
    readonly isMinBalanceZero: boolean;
    readonly isUnavailableConsumer: boolean;
    readonly isBadMetadata: boolean;
    readonly isUnapproved: boolean;
    readonly isWouldDie: boolean;
    readonly isAlreadyExists: boolean;
    readonly isNoDeposit: boolean;
    readonly isWouldBurn: boolean;
    readonly isLiveAsset: boolean;
    readonly isAssetNotLive: boolean;
    readonly isIncorrectStatus: boolean;
    readonly isNotFrozen: boolean;
    readonly isCallbackFailed: boolean;
    readonly isBadAssetId: boolean;
    readonly type: 'BalanceLow' | 'NoAccount' | 'NoPermission' | 'Unknown' | 'Frozen' | 'InUse' | 'BadWitness' | 'MinBalanceZero' | 'UnavailableConsumer' | 'BadMetadata' | 'Unapproved' | 'WouldDie' | 'AlreadyExists' | 'NoDeposit' | 'WouldBurn' | 'LiveAsset' | 'AssetNotLive' | 'IncorrectStatus' | 'NotFrozen' | 'CallbackFailed' | 'BadAssetId';
  }

  /** @name PalletBalancesBalanceLock (571) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (572) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesReserveData (575) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name FrameSupportTokensMiscIdAmountRuntimeHoldReason (578) */
  interface FrameSupportTokensMiscIdAmountRuntimeHoldReason extends Struct {
    readonly id: TangleTestnetRuntimeRuntimeHoldReason;
    readonly amount: u128;
  }

  /** @name TangleTestnetRuntimeRuntimeHoldReason (579) */
  interface TangleTestnetRuntimeRuntimeHoldReason extends Enum {
    readonly isPreimage: boolean;
    readonly asPreimage: PalletPreimageHoldReason;
    readonly type: 'Preimage';
  }

  /** @name PalletPreimageHoldReason (580) */
  interface PalletPreimageHoldReason extends Enum {
    readonly isPreimage: boolean;
    readonly type: 'Preimage';
  }

  /** @name FrameSupportTokensMiscIdAmountRuntimeFreezeReason (583) */
  interface FrameSupportTokensMiscIdAmountRuntimeFreezeReason extends Struct {
    readonly id: TangleTestnetRuntimeRuntimeFreezeReason;
    readonly amount: u128;
  }

  /** @name TangleTestnetRuntimeRuntimeFreezeReason (584) */
  interface TangleTestnetRuntimeRuntimeFreezeReason extends Enum {
    readonly isNominationPools: boolean;
    readonly asNominationPools: PalletNominationPoolsFreezeReason;
    readonly isLst: boolean;
    readonly asLst: PalletTangleLstFreezeReason;
    readonly type: 'NominationPools' | 'Lst';
  }

  /** @name PalletNominationPoolsFreezeReason (585) */
  interface PalletNominationPoolsFreezeReason extends Enum {
    readonly isPoolMinBalance: boolean;
    readonly type: 'PoolMinBalance';
  }

  /** @name PalletTangleLstFreezeReason (586) */
  interface PalletTangleLstFreezeReason extends Enum {
    readonly isPoolMinBalance: boolean;
    readonly type: 'PoolMinBalance';
  }

  /** @name PalletBalancesError (588) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isExpendability: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly isTooManyHolds: boolean;
    readonly isTooManyFreezes: boolean;
    readonly isIssuanceDeactivated: boolean;
    readonly isDeltaZero: boolean;
    readonly type: 'VestingBalance' | 'LiquidityRestrictions' | 'InsufficientBalance' | 'ExistentialDeposit' | 'Expendability' | 'ExistingVestingSchedule' | 'DeadAccount' | 'TooManyReserves' | 'TooManyHolds' | 'TooManyFreezes' | 'IssuanceDeactivated' | 'DeltaZero';
  }

  /** @name PalletTransactionPaymentReleases (590) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name SpConsensusBabeDigestsPreDigest (597) */
  interface SpConsensusBabeDigestsPreDigest extends Enum {
    readonly isPrimary: boolean;
    readonly asPrimary: SpConsensusBabeDigestsPrimaryPreDigest;
    readonly isSecondaryPlain: boolean;
    readonly asSecondaryPlain: SpConsensusBabeDigestsSecondaryPlainPreDigest;
    readonly isSecondaryVRF: boolean;
    readonly asSecondaryVRF: SpConsensusBabeDigestsSecondaryVRFPreDigest;
    readonly type: 'Primary' | 'SecondaryPlain' | 'SecondaryVRF';
  }

  /** @name SpConsensusBabeDigestsPrimaryPreDigest (598) */
  interface SpConsensusBabeDigestsPrimaryPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature;
  }

  /** @name SpCoreSr25519VrfVrfSignature (599) */
  interface SpCoreSr25519VrfVrfSignature extends Struct {
    readonly preOutput: U8aFixed;
    readonly proof: U8aFixed;
  }

  /** @name SpConsensusBabeDigestsSecondaryPlainPreDigest (600) */
  interface SpConsensusBabeDigestsSecondaryPlainPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
  }

  /** @name SpConsensusBabeDigestsSecondaryVRFPreDigest (601) */
  interface SpConsensusBabeDigestsSecondaryVRFPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature;
  }

  /** @name SpConsensusBabeBabeEpochConfiguration (602) */
  interface SpConsensusBabeBabeEpochConfiguration extends Struct {
    readonly c: ITuple<[u64, u64]>;
    readonly allowedSlots: SpConsensusBabeAllowedSlots;
  }

  /** @name PalletBabeError (604) */
  interface PalletBabeError extends Enum {
    readonly isInvalidEquivocationProof: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly isInvalidConfiguration: boolean;
    readonly type: 'InvalidEquivocationProof' | 'InvalidKeyOwnershipProof' | 'DuplicateOffenceReport' | 'InvalidConfiguration';
  }

  /** @name PalletGrandpaStoredState (605) */
  interface PalletGrandpaStoredState extends Enum {
    readonly isLive: boolean;
    readonly isPendingPause: boolean;
    readonly asPendingPause: {
      readonly scheduledAt: u64;
      readonly delay: u64;
    } & Struct;
    readonly isPaused: boolean;
    readonly isPendingResume: boolean;
    readonly asPendingResume: {
      readonly scheduledAt: u64;
      readonly delay: u64;
    } & Struct;
    readonly type: 'Live' | 'PendingPause' | 'Paused' | 'PendingResume';
  }

  /** @name PalletGrandpaStoredPendingChange (606) */
  interface PalletGrandpaStoredPendingChange extends Struct {
    readonly scheduledAt: u64;
    readonly delay: u64;
    readonly nextAuthorities: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>;
    readonly forced: Option<u64>;
  }

  /** @name PalletGrandpaError (608) */
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

  /** @name PalletIndicesError (610) */
  interface PalletIndicesError extends Enum {
    readonly isNotAssigned: boolean;
    readonly isNotOwner: boolean;
    readonly isInUse: boolean;
    readonly isNotTransfer: boolean;
    readonly isPermanent: boolean;
    readonly type: 'NotAssigned' | 'NotOwner' | 'InUse' | 'NotTransfer' | 'Permanent';
  }

  /** @name PalletDemocracyReferendumInfo (615) */
  interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletDemocracyReferendumStatus;
    readonly isFinished: boolean;
    readonly asFinished: {
      readonly approved: bool;
      readonly end: u64;
    } & Struct;
    readonly type: 'Ongoing' | 'Finished';
  }

  /** @name PalletDemocracyReferendumStatus (616) */
  interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u64;
    readonly proposal: FrameSupportPreimagesBounded;
    readonly threshold: PalletDemocracyVoteThreshold;
    readonly delay: u64;
    readonly tally: PalletDemocracyTally;
  }

  /** @name PalletDemocracyTally (617) */
  interface PalletDemocracyTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly turnout: u128;
  }

  /** @name PalletDemocracyVoteVoting (618) */
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

  /** @name PalletDemocracyDelegations (622) */
  interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /** @name PalletDemocracyVotePriorLock (623) */
  interface PalletDemocracyVotePriorLock extends ITuple<[u64, u128]> {}

  /** @name PalletDemocracyError (626) */
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

  /** @name PalletCollectiveVotes (628) */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId32>;
    readonly nays: Vec<AccountId32>;
    readonly end: u64;
  }

  /** @name PalletCollectiveError (629) */
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
    readonly isPrimeAccountNotMember: boolean;
    readonly type: 'NotMember' | 'DuplicateProposal' | 'ProposalMissing' | 'WrongIndex' | 'DuplicateVote' | 'AlreadyInitialized' | 'TooEarly' | 'TooManyProposals' | 'WrongProposalWeight' | 'WrongProposalLength' | 'PrimeAccountNotMember';
  }

  /** @name PalletVestingReleases (632) */
  interface PalletVestingReleases extends Enum {
    readonly isV0: boolean;
    readonly isV1: boolean;
    readonly type: 'V0' | 'V1';
  }

  /** @name PalletVestingError (633) */
  interface PalletVestingError extends Enum {
    readonly isNotVesting: boolean;
    readonly isAtMaxVestingSchedules: boolean;
    readonly isAmountLow: boolean;
    readonly isScheduleIndexOutOfBounds: boolean;
    readonly isInvalidScheduleParams: boolean;
    readonly type: 'NotVesting' | 'AtMaxVestingSchedules' | 'AmountLow' | 'ScheduleIndexOutOfBounds' | 'InvalidScheduleParams';
  }

  /** @name PalletElectionsPhragmenSeatHolder (635) */
  interface PalletElectionsPhragmenSeatHolder extends Struct {
    readonly who: AccountId32;
    readonly stake: u128;
    readonly deposit: u128;
  }

  /** @name PalletElectionsPhragmenVoter (636) */
  interface PalletElectionsPhragmenVoter extends Struct {
    readonly votes: Vec<AccountId32>;
    readonly stake: u128;
    readonly deposit: u128;
  }

  /** @name PalletElectionsPhragmenError (637) */
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

  /** @name PalletElectionProviderMultiPhaseReadySolution (638) */
  interface PalletElectionProviderMultiPhaseReadySolution extends Struct {
    readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>;
    readonly score: SpNposElectionsElectionScore;
    readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
  }

  /** @name PalletElectionProviderMultiPhaseRoundSnapshot (640) */
  interface PalletElectionProviderMultiPhaseRoundSnapshot extends Struct {
    readonly voters: Vec<ITuple<[AccountId32, u64, Vec<AccountId32>]>>;
    readonly targets: Vec<AccountId32>;
  }

  /** @name PalletElectionProviderMultiPhaseSignedSignedSubmission (647) */
  interface PalletElectionProviderMultiPhaseSignedSignedSubmission extends Struct {
    readonly who: AccountId32;
    readonly deposit: u128;
    readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
    readonly callFee: u128;
  }

  /** @name PalletElectionProviderMultiPhaseError (648) */
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
    readonly isPreDispatchDifferentRound: boolean;
    readonly type: 'PreDispatchEarlySubmission' | 'PreDispatchWrongWinnerCount' | 'PreDispatchWeakSubmission' | 'SignedQueueFull' | 'SignedCannotPayDeposit' | 'SignedInvalidWitness' | 'SignedTooMuchWeight' | 'OcwCallWrongEra' | 'MissingSnapshotMetadata' | 'InvalidSubmissionIndex' | 'CallNotAllowed' | 'FallbackFailed' | 'BoundNotMet' | 'TooManyWinners' | 'PreDispatchDifferentRound';
  }

  /** @name PalletStakingStakingLedger (649) */
  interface PalletStakingStakingLedger extends Struct {
    readonly stash: AccountId32;
    readonly total: Compact<u128>;
    readonly active: Compact<u128>;
    readonly unlocking: Vec<PalletStakingUnlockChunk>;
    readonly legacyClaimedRewards: Vec<u32>;
  }

  /** @name PalletStakingNominations (651) */
  interface PalletStakingNominations extends Struct {
    readonly targets: Vec<AccountId32>;
    readonly submittedIn: u32;
    readonly suppressed: bool;
  }

  /** @name PalletStakingActiveEraInfo (652) */
  interface PalletStakingActiveEraInfo extends Struct {
    readonly index: u32;
    readonly start: Option<u64>;
  }

  /** @name SpStakingPagedExposureMetadata (654) */
  interface SpStakingPagedExposureMetadata extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly nominatorCount: u32;
    readonly pageCount: u32;
  }

  /** @name SpStakingExposurePage (656) */
  interface SpStakingExposurePage extends Struct {
    readonly pageTotal: Compact<u128>;
    readonly others: Vec<SpStakingIndividualExposure>;
  }

  /** @name PalletStakingEraRewardPoints (657) */
  interface PalletStakingEraRewardPoints extends Struct {
    readonly total: u32;
    readonly individual: BTreeMap<AccountId32, u32>;
  }

  /** @name PalletStakingUnappliedSlash (662) */
  interface PalletStakingUnappliedSlash extends Struct {
    readonly validator: AccountId32;
    readonly own: u128;
    readonly others: Vec<ITuple<[AccountId32, u128]>>;
    readonly reporters: Vec<AccountId32>;
    readonly payout: u128;
  }

  /** @name PalletStakingSlashingSlashingSpans (666) */
  interface PalletStakingSlashingSlashingSpans extends Struct {
    readonly spanIndex: u32;
    readonly lastStart: u32;
    readonly lastNonzeroSlash: u32;
    readonly prior: Vec<u32>;
  }

  /** @name PalletStakingSlashingSpanRecord (667) */
  interface PalletStakingSlashingSpanRecord extends Struct {
    readonly slashed: u128;
    readonly paidOut: u128;
  }

  /** @name PalletStakingPalletError (668) */
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
    readonly isInvalidPage: boolean;
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
    readonly isControllerDeprecated: boolean;
    readonly isCannotRestoreLedger: boolean;
    readonly isRewardDestinationRestricted: boolean;
    readonly isNotEnoughFunds: boolean;
    readonly isVirtualStakerNotAllowed: boolean;
    readonly type: 'NotController' | 'NotStash' | 'AlreadyBonded' | 'AlreadyPaired' | 'EmptyTargets' | 'DuplicateIndex' | 'InvalidSlashIndex' | 'InsufficientBond' | 'NoMoreChunks' | 'NoUnlockChunk' | 'FundedTarget' | 'InvalidEraToReward' | 'InvalidNumberOfNominations' | 'NotSortedAndUnique' | 'AlreadyClaimed' | 'InvalidPage' | 'IncorrectHistoryDepth' | 'IncorrectSlashingSpans' | 'BadState' | 'TooManyTargets' | 'BadTarget' | 'CannotChillOther' | 'TooManyNominators' | 'TooManyValidators' | 'CommissionTooLow' | 'BoundNotMet' | 'ControllerDeprecated' | 'CannotRestoreLedger' | 'RewardDestinationRestricted' | 'NotEnoughFunds' | 'VirtualStakerNotAllowed';
  }

  /** @name SpCoreCryptoKeyTypeId (672) */
  interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (673) */
  interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean;
    readonly isNoAssociatedValidatorId: boolean;
    readonly isDuplicatedKey: boolean;
    readonly isNoKeys: boolean;
    readonly isNoAccount: boolean;
    readonly type: 'InvalidProof' | 'NoAssociatedValidatorId' | 'DuplicatedKey' | 'NoKeys' | 'NoAccount';
  }

  /** @name PalletTreasuryProposal (675) */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly beneficiary: AccountId32;
    readonly bond: u128;
  }

  /** @name PalletTreasurySpendStatus (677) */
  interface PalletTreasurySpendStatus extends Struct {
    readonly assetKind: Null;
    readonly amount: u128;
    readonly beneficiary: AccountId32;
    readonly validFrom: u64;
    readonly expireAt: u64;
    readonly status: PalletTreasuryPaymentState;
  }

  /** @name PalletTreasuryPaymentState (678) */
  interface PalletTreasuryPaymentState extends Enum {
    readonly isPending: boolean;
    readonly isAttempted: boolean;
    readonly asAttempted: {
      readonly id: Null;
    } & Struct;
    readonly isFailed: boolean;
    readonly type: 'Pending' | 'Attempted' | 'Failed';
  }

  /** @name FrameSupportPalletId (679) */
  interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (680) */
  interface PalletTreasuryError extends Enum {
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly isInsufficientPermission: boolean;
    readonly isProposalNotApproved: boolean;
    readonly isFailedToConvertBalance: boolean;
    readonly isSpendExpired: boolean;
    readonly isEarlyPayout: boolean;
    readonly isAlreadyAttempted: boolean;
    readonly isPayoutError: boolean;
    readonly isNotAttempted: boolean;
    readonly isInconclusive: boolean;
    readonly type: 'InvalidIndex' | 'TooManyApprovals' | 'InsufficientPermission' | 'ProposalNotApproved' | 'FailedToConvertBalance' | 'SpendExpired' | 'EarlyPayout' | 'AlreadyAttempted' | 'PayoutError' | 'NotAttempted' | 'Inconclusive';
  }

  /** @name PalletBountiesBounty (681) */
  interface PalletBountiesBounty extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly fee: u128;
    readonly curatorDeposit: u128;
    readonly bond: u128;
    readonly status: PalletBountiesBountyStatus;
  }

  /** @name PalletBountiesBountyStatus (682) */
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
      readonly updateDue: u64;
    } & Struct;
    readonly isPendingPayout: boolean;
    readonly asPendingPayout: {
      readonly curator: AccountId32;
      readonly beneficiary: AccountId32;
      readonly unlockAt: u64;
    } & Struct;
    readonly type: 'Proposed' | 'Approved' | 'Funded' | 'CuratorProposed' | 'Active' | 'PendingPayout';
  }

  /** @name PalletBountiesError (684) */
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

  /** @name PalletChildBountiesChildBounty (685) */
  interface PalletChildBountiesChildBounty extends Struct {
    readonly parentBounty: u32;
    readonly value: u128;
    readonly fee: u128;
    readonly curatorDeposit: u128;
    readonly status: PalletChildBountiesChildBountyStatus;
  }

  /** @name PalletChildBountiesChildBountyStatus (686) */
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
      readonly unlockAt: u64;
    } & Struct;
    readonly type: 'Added' | 'CuratorProposed' | 'Active' | 'PendingPayout';
  }

  /** @name PalletChildBountiesError (687) */
  interface PalletChildBountiesError extends Enum {
    readonly isParentBountyNotActive: boolean;
    readonly isInsufficientBountyBalance: boolean;
    readonly isTooManyChildBounties: boolean;
    readonly type: 'ParentBountyNotActive' | 'InsufficientBountyBalance' | 'TooManyChildBounties';
  }

  /** @name PalletBagsListListNode (688) */
  interface PalletBagsListListNode extends Struct {
    readonly id: AccountId32;
    readonly prev: Option<AccountId32>;
    readonly next: Option<AccountId32>;
    readonly bagUpper: u64;
    readonly score: u64;
  }

  /** @name PalletBagsListListBag (689) */
  interface PalletBagsListListBag extends Struct {
    readonly head: Option<AccountId32>;
    readonly tail: Option<AccountId32>;
  }

  /** @name PalletBagsListError (690) */
  interface PalletBagsListError extends Enum {
    readonly isList: boolean;
    readonly asList: PalletBagsListListListError;
    readonly type: 'List';
  }

  /** @name PalletBagsListListListError (691) */
  interface PalletBagsListListListError extends Enum {
    readonly isDuplicate: boolean;
    readonly isNotHeavier: boolean;
    readonly isNotInSameBag: boolean;
    readonly isNodeNotFound: boolean;
    readonly type: 'Duplicate' | 'NotHeavier' | 'NotInSameBag' | 'NodeNotFound';
  }

  /** @name PalletNominationPoolsPoolMember (692) */
  interface PalletNominationPoolsPoolMember extends Struct {
    readonly poolId: u32;
    readonly points: u128;
    readonly lastRecordedRewardCounter: u128;
    readonly unbondingEras: BTreeMap<u32, u128>;
  }

  /** @name PalletNominationPoolsBondedPoolInner (697) */
  interface PalletNominationPoolsBondedPoolInner extends Struct {
    readonly commission: PalletNominationPoolsCommission;
    readonly memberCounter: u32;
    readonly points: u128;
    readonly roles: PalletNominationPoolsPoolRoles;
    readonly state: PalletNominationPoolsPoolState;
  }

  /** @name PalletNominationPoolsCommission (698) */
  interface PalletNominationPoolsCommission extends Struct {
    readonly current: Option<ITuple<[Perbill, AccountId32]>>;
    readonly max: Option<Perbill>;
    readonly changeRate: Option<PalletNominationPoolsCommissionChangeRate>;
    readonly throttleFrom: Option<u64>;
    readonly claimPermission: Option<PalletNominationPoolsCommissionClaimPermission>;
  }

  /** @name PalletNominationPoolsPoolRoles (701) */
  interface PalletNominationPoolsPoolRoles extends Struct {
    readonly depositor: AccountId32;
    readonly root: Option<AccountId32>;
    readonly nominator: Option<AccountId32>;
    readonly bouncer: Option<AccountId32>;
  }

  /** @name PalletNominationPoolsRewardPool (702) */
  interface PalletNominationPoolsRewardPool extends Struct {
    readonly lastRecordedRewardCounter: u128;
    readonly lastRecordedTotalPayouts: u128;
    readonly totalRewardsClaimed: u128;
    readonly totalCommissionPending: u128;
    readonly totalCommissionClaimed: u128;
  }

  /** @name PalletNominationPoolsSubPools (703) */
  interface PalletNominationPoolsSubPools extends Struct {
    readonly noEra: PalletNominationPoolsUnbondPool;
    readonly withEra: BTreeMap<u32, PalletNominationPoolsUnbondPool>;
  }

  /** @name PalletNominationPoolsUnbondPool (704) */
  interface PalletNominationPoolsUnbondPool extends Struct {
    readonly points: u128;
    readonly balance: u128;
  }

  /** @name PalletNominationPoolsError (709) */
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
    readonly isMaxCommissionRestricted: boolean;
    readonly isCommissionExceedsMaximum: boolean;
    readonly isCommissionExceedsGlobalMaximum: boolean;
    readonly isCommissionChangeThrottled: boolean;
    readonly isCommissionChangeRateNotAllowed: boolean;
    readonly isNoPendingCommission: boolean;
    readonly isNoCommissionCurrentSet: boolean;
    readonly isPoolIdInUse: boolean;
    readonly isInvalidPoolId: boolean;
    readonly isBondExtraRestricted: boolean;
    readonly isNothingToAdjust: boolean;
    readonly isNothingToSlash: boolean;
    readonly isSlashTooLow: boolean;
    readonly isAlreadyMigrated: boolean;
    readonly isNotMigrated: boolean;
    readonly isNotSupported: boolean;
    readonly type: 'PoolNotFound' | 'PoolMemberNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'AccountBelongsToOtherPool' | 'FullyUnbonding' | 'MaxUnbondingLimit' | 'CannotWithdrawAny' | 'MinimumBondNotMet' | 'OverflowRisk' | 'NotDestroying' | 'NotNominator' | 'NotKickerOrDestroying' | 'NotOpen' | 'MaxPools' | 'MaxPoolMembers' | 'CanNotChangeState' | 'DoesNotHavePermission' | 'MetadataExceedsMaxLen' | 'Defensive' | 'PartialUnbondNotAllowedPermissionlessly' | 'MaxCommissionRestricted' | 'CommissionExceedsMaximum' | 'CommissionExceedsGlobalMaximum' | 'CommissionChangeThrottled' | 'CommissionChangeRateNotAllowed' | 'NoPendingCommission' | 'NoCommissionCurrentSet' | 'PoolIdInUse' | 'InvalidPoolId' | 'BondExtraRestricted' | 'NothingToAdjust' | 'NothingToSlash' | 'SlashTooLow' | 'AlreadyMigrated' | 'NotMigrated' | 'NotSupported';
  }

  /** @name PalletNominationPoolsDefensiveError (710) */
  interface PalletNominationPoolsDefensiveError extends Enum {
    readonly isNotEnoughSpaceInUnbondPool: boolean;
    readonly isPoolNotFound: boolean;
    readonly isRewardPoolNotFound: boolean;
    readonly isSubPoolsNotFound: boolean;
    readonly isBondedStashKilledPrematurely: boolean;
    readonly isDelegationUnsupported: boolean;
    readonly isSlashNotApplied: boolean;
    readonly type: 'NotEnoughSpaceInUnbondPool' | 'PoolNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'BondedStashKilledPrematurely' | 'DelegationUnsupported' | 'SlashNotApplied';
  }

  /** @name PalletSchedulerScheduled (713) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u64, u32]>>;
    readonly origin: TangleTestnetRuntimeOriginCaller;
  }

  /** @name PalletSchedulerRetryConfig (715) */
  interface PalletSchedulerRetryConfig extends Struct {
    readonly totalRetries: u8;
    readonly remaining: u8;
    readonly period: u64;
  }

  /** @name PalletSchedulerError (716) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly isNamed: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange' | 'Named';
  }

  /** @name PalletPreimageOldRequestStatus (717) */
  interface PalletPreimageOldRequestStatus extends Enum {
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

  /** @name PalletPreimageRequestStatus (719) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly ticket: ITuple<[AccountId32, Null]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly maybeTicket: Option<ITuple<[AccountId32, Null]>>;
      readonly count: u32;
      readonly maybeLen: Option<u32>;
    } & Struct;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageError (723) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly isTooMany: boolean;
    readonly isTooFew: boolean;
    readonly isNoCost: boolean;
    readonly type: 'TooBig' | 'AlreadyNoted' | 'NotAuthorized' | 'NotNoted' | 'Requested' | 'NotRequested' | 'TooMany' | 'TooFew' | 'NoCost';
  }

  /** @name SpStakingOffenceOffenceDetails (724) */
  interface SpStakingOffenceOffenceDetails extends Struct {
    readonly offender: ITuple<[AccountId32, SpStakingExposure]>;
    readonly reporters: Vec<AccountId32>;
  }

  /** @name PalletTxPauseError (726) */
  interface PalletTxPauseError extends Enum {
    readonly isIsPaused: boolean;
    readonly isIsUnpaused: boolean;
    readonly isUnpausable: boolean;
    readonly isNotFound: boolean;
    readonly type: 'IsPaused' | 'IsUnpaused' | 'Unpausable' | 'NotFound';
  }

  /** @name PalletImOnlineError (729) */
  interface PalletImOnlineError extends Enum {
    readonly isInvalidKey: boolean;
    readonly isDuplicatedHeartbeat: boolean;
    readonly type: 'InvalidKey' | 'DuplicatedHeartbeat';
  }

  /** @name PalletIdentityRegistration (731) */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityLegacyIdentityInfo;
  }

  /** @name PalletIdentityRegistrarInfo (740) */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId32;
    readonly fee: u128;
    readonly fields: u64;
  }

  /** @name PalletIdentityAuthorityProperties (742) */
  interface PalletIdentityAuthorityProperties extends Struct {
    readonly suffix: Bytes;
    readonly allocation: u32;
  }

  /** @name PalletIdentityError (745) */
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
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly isJudgementForDifferentIdentity: boolean;
    readonly isJudgementPaymentFailed: boolean;
    readonly isInvalidSuffix: boolean;
    readonly isNotUsernameAuthority: boolean;
    readonly isNoAllocation: boolean;
    readonly isInvalidSignature: boolean;
    readonly isRequiresSignature: boolean;
    readonly isInvalidUsername: boolean;
    readonly isUsernameTaken: boolean;
    readonly isNoUsername: boolean;
    readonly isNotExpired: boolean;
    readonly type: 'TooManySubAccounts' | 'NotFound' | 'NotNamed' | 'EmptyIndex' | 'FeeChanged' | 'NoIdentity' | 'StickyJudgement' | 'JudgementGiven' | 'InvalidJudgement' | 'InvalidIndex' | 'InvalidTarget' | 'TooManyRegistrars' | 'AlreadyClaimed' | 'NotSub' | 'NotOwned' | 'JudgementForDifferentIdentity' | 'JudgementPaymentFailed' | 'InvalidSuffix' | 'NotUsernameAuthority' | 'NoAllocation' | 'InvalidSignature' | 'RequiresSignature' | 'InvalidUsername' | 'UsernameTaken' | 'NoUsername' | 'NotExpired';
  }

  /** @name PalletUtilityError (746) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: 'TooManyCalls';
  }

  /** @name PalletMultisigMultisig (748) */
  interface PalletMultisigMultisig extends Struct {
    readonly when: PalletMultisigTimepoint;
    readonly deposit: u128;
    readonly depositor: AccountId32;
    readonly approvals: Vec<AccountId32>;
  }

  /** @name PalletMultisigError (749) */
  interface PalletMultisigError extends Enum {
    readonly isMinimumThreshold: boolean;
    readonly isAlreadyApproved: boolean;
    readonly isNoApprovalsNeeded: boolean;
    readonly isTooFewSignatories: boolean;
    readonly isTooManySignatories: boolean;
    readonly isSignatoriesOutOfOrder: boolean;
    readonly isSenderInSignatories: boolean;
    readonly isNotFound: boolean;
    readonly isNotOwner: boolean;
    readonly isNoTimepoint: boolean;
    readonly isWrongTimepoint: boolean;
    readonly isUnexpectedTimepoint: boolean;
    readonly isMaxWeightTooLow: boolean;
    readonly isAlreadyStored: boolean;
    readonly type: 'MinimumThreshold' | 'AlreadyApproved' | 'NoApprovalsNeeded' | 'TooFewSignatories' | 'TooManySignatories' | 'SignatoriesOutOfOrder' | 'SenderInSignatories' | 'NotFound' | 'NotOwner' | 'NoTimepoint' | 'WrongTimepoint' | 'UnexpectedTimepoint' | 'MaxWeightTooLow' | 'AlreadyStored';
  }

  /** @name FpRpcTransactionStatus (752) */
  interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256;
    readonly transactionIndex: u32;
    readonly from: H160;
    readonly to: Option<H160>;
    readonly contractAddress: Option<H160>;
    readonly logs: Vec<EthereumLog>;
    readonly logsBloom: EthbloomBloom;
  }

  /** @name EthbloomBloom (754) */
  interface EthbloomBloom extends U8aFixed {}

  /** @name EthereumReceiptReceiptV3 (756) */
  interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumReceiptEip658ReceiptData;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumReceiptEip658ReceiptData;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumReceiptEip658ReceiptData;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumReceiptEip658ReceiptData (757) */
  interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8;
    readonly usedGas: U256;
    readonly logsBloom: EthbloomBloom;
    readonly logs: Vec<EthereumLog>;
  }

  /** @name EthereumBlock (758) */
  interface EthereumBlock extends Struct {
    readonly header: EthereumHeader;
    readonly transactions: Vec<EthereumTransactionTransactionV2>;
    readonly ommers: Vec<EthereumHeader>;
  }

  /** @name EthereumHeader (759) */
  interface EthereumHeader extends Struct {
    readonly parentHash: H256;
    readonly ommersHash: H256;
    readonly beneficiary: H160;
    readonly stateRoot: H256;
    readonly transactionsRoot: H256;
    readonly receiptsRoot: H256;
    readonly logsBloom: EthbloomBloom;
    readonly difficulty: U256;
    readonly number: U256;
    readonly gasLimit: U256;
    readonly gasUsed: U256;
    readonly timestamp: u64;
    readonly extraData: Bytes;
    readonly mixHash: H256;
    readonly nonce: EthereumTypesHashH64;
  }

  /** @name EthereumTypesHashH64 (760) */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name PalletEthereumError (765) */
  interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isPreLogExists: boolean;
    readonly type: 'InvalidSignature' | 'PreLogExists';
  }

  /** @name PalletEvmCodeMetadata (766) */
  interface PalletEvmCodeMetadata extends Struct {
    readonly size_: u64;
    readonly hash_: H256;
  }

  /** @name PalletEvmError (768) */
  interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isFeeOverflow: boolean;
    readonly isPaymentOverflow: boolean;
    readonly isWithdrawFailed: boolean;
    readonly isGasPriceTooLow: boolean;
    readonly isInvalidNonce: boolean;
    readonly isGasLimitTooLow: boolean;
    readonly isGasLimitTooHigh: boolean;
    readonly isInvalidChainId: boolean;
    readonly isInvalidSignature: boolean;
    readonly isReentrancy: boolean;
    readonly isTransactionMustComeFromEOA: boolean;
    readonly isUndefined: boolean;
    readonly type: 'BalanceLow' | 'FeeOverflow' | 'PaymentOverflow' | 'WithdrawFailed' | 'GasPriceTooLow' | 'InvalidNonce' | 'GasLimitTooLow' | 'GasLimitTooHigh' | 'InvalidChainId' | 'InvalidSignature' | 'Reentrancy' | 'TransactionMustComeFromEOA' | 'Undefined';
  }

  /** @name PalletHotfixSufficientsError (769) */
  interface PalletHotfixSufficientsError extends Enum {
    readonly isMaxAddressCountExceeded: boolean;
    readonly type: 'MaxAddressCountExceeded';
  }

  /** @name PalletAirdropClaimsError (771) */
  interface PalletAirdropClaimsError extends Enum {
    readonly isInvalidEthereumSignature: boolean;
    readonly isInvalidNativeSignature: boolean;
    readonly isInvalidNativeAccount: boolean;
    readonly isSignerHasNoClaim: boolean;
    readonly isSenderHasNoClaim: boolean;
    readonly isPotUnderflow: boolean;
    readonly isInvalidStatement: boolean;
    readonly isVestedBalanceExists: boolean;
    readonly type: 'InvalidEthereumSignature' | 'InvalidNativeSignature' | 'InvalidNativeAccount' | 'SignerHasNoClaim' | 'SenderHasNoClaim' | 'PotUnderflow' | 'InvalidStatement' | 'VestedBalanceExists';
  }

  /** @name PalletProxyProxyDefinition (774) */
  interface PalletProxyProxyDefinition extends Struct {
    readonly delegate: AccountId32;
    readonly proxyType: TangleTestnetRuntimeProxyType;
    readonly delay: u64;
  }

  /** @name PalletProxyAnnouncement (778) */
  interface PalletProxyAnnouncement extends Struct {
    readonly real: AccountId32;
    readonly callHash: H256;
    readonly height: u64;
  }

  /** @name PalletProxyError (780) */
  interface PalletProxyError extends Enum {
    readonly isTooMany: boolean;
    readonly isNotFound: boolean;
    readonly isNotProxy: boolean;
    readonly isUnproxyable: boolean;
    readonly isDuplicate: boolean;
    readonly isNoPermission: boolean;
    readonly isUnannounced: boolean;
    readonly isNoSelfProxy: boolean;
    readonly type: 'TooMany' | 'NotFound' | 'NotProxy' | 'Unproxyable' | 'Duplicate' | 'NoPermission' | 'Unannounced' | 'NoSelfProxy';
  }

  /** @name PalletMultiAssetDelegationOperatorOperatorMetadata (781) */
  interface PalletMultiAssetDelegationOperatorOperatorMetadata extends Struct {
    readonly stake: u128;
    readonly delegationCount: u32;
    readonly request: Option<PalletMultiAssetDelegationOperatorOperatorBondLessRequest>;
    readonly delegations: Vec<PalletMultiAssetDelegationOperatorDelegatorBond>;
    readonly status: PalletMultiAssetDelegationOperatorOperatorStatus;
    readonly blueprintIds: Vec<u32>;
  }

  /** @name TangleTestnetRuntimeMaxDelegations (782) */
  type TangleTestnetRuntimeMaxDelegations = Null;

  /** @name TangleTestnetRuntimeMaxOperatorBlueprints (783) */
  type TangleTestnetRuntimeMaxOperatorBlueprints = Null;

  /** @name PalletMultiAssetDelegationOperatorOperatorBondLessRequest (785) */
  interface PalletMultiAssetDelegationOperatorOperatorBondLessRequest extends Struct {
    readonly amount: u128;
    readonly requestTime: u32;
  }

  /** @name PalletMultiAssetDelegationOperatorDelegatorBond (787) */
  interface PalletMultiAssetDelegationOperatorDelegatorBond extends Struct {
    readonly delegator: AccountId32;
    readonly amount: u128;
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
  }

  /** @name PalletMultiAssetDelegationOperatorOperatorStatus (789) */
  interface PalletMultiAssetDelegationOperatorOperatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isInactive: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: 'Active' | 'Inactive' | 'Leaving';
  }

  /** @name PalletMultiAssetDelegationOperatorOperatorSnapshot (791) */
  interface PalletMultiAssetDelegationOperatorOperatorSnapshot extends Struct {
    readonly stake: u128;
    readonly delegations: Vec<PalletMultiAssetDelegationOperatorDelegatorBond>;
  }

  /** @name PalletMultiAssetDelegationDelegatorDelegatorMetadata (792) */
  interface PalletMultiAssetDelegationDelegatorDelegatorMetadata extends Struct {
    readonly deposits: BTreeMap<TanglePrimitivesServicesTypesAssetU128, PalletMultiAssetDelegationDelegatorDeposit>;
    readonly withdrawRequests: Vec<PalletMultiAssetDelegationDelegatorWithdrawRequest>;
    readonly delegations: Vec<PalletMultiAssetDelegationDelegatorBondInfoDelegator>;
    readonly delegatorUnstakeRequests: Vec<PalletMultiAssetDelegationDelegatorBondLessRequest>;
    readonly status: PalletMultiAssetDelegationDelegatorDelegatorStatus;
  }

  /** @name TangleTestnetRuntimeMaxWithdrawRequests (793) */
  type TangleTestnetRuntimeMaxWithdrawRequests = Null;

  /** @name TangleTestnetRuntimeMaxUnstakeRequests (794) */
  type TangleTestnetRuntimeMaxUnstakeRequests = Null;

  /** @name PalletMultiAssetDelegationDelegatorDeposit (796) */
  interface PalletMultiAssetDelegationDelegatorDeposit extends Struct {
    readonly amount: u128;
    readonly delegatedAmount: u128;
    readonly locks: Option<Vec<TanglePrimitivesRewardsLockInfo>>;
  }

  /** @name TanglePrimitivesRewardsLockInfo (799) */
  interface TanglePrimitivesRewardsLockInfo extends Struct {
    readonly amount: u128;
    readonly lockMultiplier: TanglePrimitivesRewardsLockMultiplier;
    readonly expiryBlock: u64;
  }

  /** @name PalletMultiAssetDelegationDelegatorWithdrawRequest (804) */
  interface PalletMultiAssetDelegationDelegatorWithdrawRequest extends Struct {
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly amount: u128;
    readonly requestedRound: u32;
  }

  /** @name PalletMultiAssetDelegationDelegatorBondInfoDelegator (807) */
  interface PalletMultiAssetDelegationDelegatorBondInfoDelegator extends Struct {
    readonly operator: AccountId32;
    readonly amount: u128;
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly blueprintSelection: PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection;
    readonly isNomination: bool;
  }

  /** @name PalletMultiAssetDelegationDelegatorBondLessRequest (810) */
  interface PalletMultiAssetDelegationDelegatorBondLessRequest extends Struct {
    readonly operator: AccountId32;
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly amount: u128;
    readonly requestedRound: u32;
    readonly blueprintSelection: PalletMultiAssetDelegationDelegatorDelegatorBlueprintSelection;
    readonly isNomination: bool;
  }

  /** @name PalletMultiAssetDelegationDelegatorDelegatorStatus (812) */
  interface PalletMultiAssetDelegationDelegatorDelegatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isLeavingScheduled: boolean;
    readonly asLeavingScheduled: u32;
    readonly type: 'Active' | 'LeavingScheduled';
  }

  /** @name PalletMultiAssetDelegationError (813) */
  interface PalletMultiAssetDelegationError extends Enum {
    readonly isAlreadyOperator: boolean;
    readonly isBondTooLow: boolean;
    readonly isInvalidAmount: boolean;
    readonly isNotAnOperator: boolean;
    readonly isCannotExit: boolean;
    readonly isAlreadyLeaving: boolean;
    readonly isNotLeavingOperator: boolean;
    readonly isLeavingRoundNotReached: boolean;
    readonly isNoScheduledBondLess: boolean;
    readonly isBondLessRequestNotSatisfied: boolean;
    readonly isNotActiveOperator: boolean;
    readonly isNotOfflineOperator: boolean;
    readonly isAlreadyDelegator: boolean;
    readonly isNotDelegator: boolean;
    readonly isWithdrawRequestAlreadyExists: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isNoWithdrawRequest: boolean;
    readonly isNoBondLessRequest: boolean;
    readonly isBondLessNotReady: boolean;
    readonly isBondLessRequestAlreadyExists: boolean;
    readonly isActiveServicesUsingAsset: boolean;
    readonly isNoActiveDelegation: boolean;
    readonly isAssetNotWhitelisted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isMaxBlueprintsExceeded: boolean;
    readonly isAssetNotFound: boolean;
    readonly isBlueprintAlreadyWhitelisted: boolean;
    readonly isNoWithdrawRequests: boolean;
    readonly isNoMatchingwithdrawRequest: boolean;
    readonly isAssetAlreadyInVault: boolean;
    readonly isAssetNotInVault: boolean;
    readonly isVaultNotFound: boolean;
    readonly isDuplicateBlueprintId: boolean;
    readonly isBlueprintIdNotFound: boolean;
    readonly isNotInFixedMode: boolean;
    readonly isMaxDelegationsExceeded: boolean;
    readonly isMaxUnstakeRequestsExceeded: boolean;
    readonly isMaxWithdrawRequestsExceeded: boolean;
    readonly isDepositOverflow: boolean;
    readonly isUnstakeAmountTooLarge: boolean;
    readonly isStakeOverflow: boolean;
    readonly isInsufficientStakeRemaining: boolean;
    readonly isApyExceedsMaximum: boolean;
    readonly isCapCannotBeZero: boolean;
    readonly isCapExceedsTotalSupply: boolean;
    readonly isPendingUnstakeRequestExists: boolean;
    readonly isBlueprintNotSelected: boolean;
    readonly isErc20TransferFailed: boolean;
    readonly isSlashAlertFailed: boolean;
    readonly isEvmAbiEncode: boolean;
    readonly isEvmAbiDecode: boolean;
    readonly isLockViolation: boolean;
    readonly isDepositExceedsCapForAsset: boolean;
    readonly isOverflowRisk: boolean;
    readonly isAssetConfigNotFound: boolean;
    readonly isCannotGoOfflineWithActiveServices: boolean;
    readonly isNotNominator: boolean;
    readonly type: 'AlreadyOperator' | 'BondTooLow' | 'InvalidAmount' | 'NotAnOperator' | 'CannotExit' | 'AlreadyLeaving' | 'NotLeavingOperator' | 'LeavingRoundNotReached' | 'NoScheduledBondLess' | 'BondLessRequestNotSatisfied' | 'NotActiveOperator' | 'NotOfflineOperator' | 'AlreadyDelegator' | 'NotDelegator' | 'WithdrawRequestAlreadyExists' | 'InsufficientBalance' | 'NoWithdrawRequest' | 'NoBondLessRequest' | 'BondLessNotReady' | 'BondLessRequestAlreadyExists' | 'ActiveServicesUsingAsset' | 'NoActiveDelegation' | 'AssetNotWhitelisted' | 'NotAuthorized' | 'MaxBlueprintsExceeded' | 'AssetNotFound' | 'BlueprintAlreadyWhitelisted' | 'NoWithdrawRequests' | 'NoMatchingwithdrawRequest' | 'AssetAlreadyInVault' | 'AssetNotInVault' | 'VaultNotFound' | 'DuplicateBlueprintId' | 'BlueprintIdNotFound' | 'NotInFixedMode' | 'MaxDelegationsExceeded' | 'MaxUnstakeRequestsExceeded' | 'MaxWithdrawRequestsExceeded' | 'DepositOverflow' | 'UnstakeAmountTooLarge' | 'StakeOverflow' | 'InsufficientStakeRemaining' | 'ApyExceedsMaximum' | 'CapCannotBeZero' | 'CapExceedsTotalSupply' | 'PendingUnstakeRequestExists' | 'BlueprintNotSelected' | 'Erc20TransferFailed' | 'SlashAlertFailed' | 'EvmAbiEncode' | 'EvmAbiDecode' | 'LockViolation' | 'DepositExceedsCapForAsset' | 'OverflowRisk' | 'AssetConfigNotFound' | 'CannotGoOfflineWithActiveServices' | 'NotNominator';
  }

  /** @name TanglePrimitivesServicesQosHeartbeatStats (817) */
  interface TanglePrimitivesServicesQosHeartbeatStats extends Struct {
    readonly expectedHeartbeats: u32;
    readonly receivedHeartbeats: u32;
    readonly lastCheckBlock: u32;
    readonly lastHeartbeatBlock: u32;
  }

  /** @name TanglePrimitivesServicesServiceServiceRequest (819) */
  interface TanglePrimitivesServicesServiceServiceRequest extends Struct {
    readonly blueprint: u64;
    readonly owner: AccountId32;
    readonly securityRequirements: Vec<TanglePrimitivesServicesTypesAssetSecurityRequirement>;
    readonly ttl: u64;
    readonly args: Vec<TanglePrimitivesServicesField>;
    readonly permittedCallers: Vec<AccountId32>;
    readonly operatorsWithApprovalState: Vec<ITuple<[AccountId32, TanglePrimitivesServicesTypesApprovalState]>>;
    readonly membershipModel: TanglePrimitivesServicesTypesMembershipModel;
  }

  /** @name TanglePrimitivesServicesTypesApprovalState (824) */
  interface TanglePrimitivesServicesTypesApprovalState extends Enum {
    readonly isPending: boolean;
    readonly isApproved: boolean;
    readonly asApproved: {
      readonly securityCommitments: Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>;
    } & Struct;
    readonly isRejected: boolean;
    readonly type: 'Pending' | 'Approved' | 'Rejected';
  }

  /** @name TanglePrimitivesServicesService (826) */
  interface TanglePrimitivesServicesService extends Struct {
    readonly id: u64;
    readonly blueprint: u64;
    readonly owner: AccountId32;
    readonly args: Vec<TanglePrimitivesServicesField>;
    readonly operatorSecurityCommitments: Vec<ITuple<[AccountId32, Vec<TanglePrimitivesServicesTypesAssetSecurityCommitment>]>>;
    readonly securityRequirements: Vec<TanglePrimitivesServicesTypesAssetSecurityRequirement>;
    readonly permittedCallers: Vec<AccountId32>;
    readonly ttl: u64;
    readonly membershipModel: TanglePrimitivesServicesTypesMembershipModel;
  }

  /** @name TanglePrimitivesServicesJobsJobCall (829) */
  interface TanglePrimitivesServicesJobsJobCall extends Struct {
    readonly serviceId: u64;
    readonly job: u8;
    readonly args: Vec<TanglePrimitivesServicesField>;
  }

  /** @name TanglePrimitivesServicesJobsJobCallResult (830) */
  interface TanglePrimitivesServicesJobsJobCallResult extends Struct {
    readonly serviceId: u64;
    readonly callId: u64;
    readonly result: Vec<TanglePrimitivesServicesField>;
  }

  /** @name TanglePrimitivesServicesTypesUnappliedSlash (831) */
  interface TanglePrimitivesServicesTypesUnappliedSlash extends Struct {
    readonly era: u32;
    readonly blueprintId: u64;
    readonly serviceId: u64;
    readonly operator: AccountId32;
    readonly slashPercent: Percent;
  }

  /** @name TanglePrimitivesServicesTypesOperatorProfile (833) */
  interface TanglePrimitivesServicesTypesOperatorProfile extends Struct {
    readonly services: BTreeSet<u64>;
    readonly blueprints: BTreeSet<u64>;
  }

  /** @name TanglePrimitivesServicesServiceStagingServicePayment (836) */
  interface TanglePrimitivesServicesServiceStagingServicePayment extends Struct {
    readonly requestId: u64;
    readonly refundTo: TanglePrimitivesAccount;
    readonly asset: TanglePrimitivesServicesTypesAssetU128;
    readonly amount: u128;
  }

  /** @name TanglePrimitivesAccount (837) */
  interface TanglePrimitivesAccount extends Enum {
    readonly isId: boolean;
    readonly asId: AccountId32;
    readonly isAddress: boolean;
    readonly asAddress: H160;
    readonly type: 'Id' | 'Address';
  }

  /** @name TanglePrimitivesServicesJobsJobSubscriptionBilling (839) */
  interface TanglePrimitivesServicesJobsJobSubscriptionBilling extends Struct {
    readonly serviceId: u64;
    readonly jobIndex: u8;
    readonly subscriber: AccountId32;
    readonly lastBilled: u64;
    readonly endBlock: Option<u64>;
  }

  /** @name TanglePrimitivesServicesJobsJobPayment (840) */
  interface TanglePrimitivesServicesJobsJobPayment extends Struct {
    readonly serviceId: u64;
    readonly jobIndex: u8;
    readonly callId: u64;
    readonly payer: AccountId32;
    readonly asset: TanglePrimitivesServicesTypesAssetU32;
    readonly amount: u128;
  }

  /** @name TanglePrimitivesServicesTypesAssetU32 (841) */
  interface TanglePrimitivesServicesTypesAssetU32 extends Enum {
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly isErc20: boolean;
    readonly asErc20: H160;
    readonly type: 'Custom' | 'Erc20';
  }

  /** @name PalletServicesModuleError (842) */
  interface PalletServicesModuleError extends Enum {
    readonly isBlueprintNotFound: boolean;
    readonly isBlueprintCreationInterrupted: boolean;
    readonly isAlreadyRegistered: boolean;
    readonly isNotRegistered: boolean;
    readonly isOperatorNotActive: boolean;
    readonly isInvalidRegistrationInput: boolean;
    readonly isNotAllowedToUnregister: boolean;
    readonly isNotAllowedToUpdateRpcAddress: boolean;
    readonly isInvalidRequestInput: boolean;
    readonly isInvalidJobCallInput: boolean;
    readonly isInvalidJobResult: boolean;
    readonly isApprovalInterrupted: boolean;
    readonly isRejectionInterrupted: boolean;
    readonly isServiceRequestNotFound: boolean;
    readonly isServiceInitializationInterrupted: boolean;
    readonly isServiceNotFound: boolean;
    readonly isTerminationInterrupted: boolean;
    readonly isTypeCheck: boolean;
    readonly asTypeCheck: TanglePrimitivesServicesTypesTypeCheckError;
    readonly isMaxPermittedCallersExceeded: boolean;
    readonly isMaxServiceProvidersExceeded: boolean;
    readonly isMaxServicesPerUserExceeded: boolean;
    readonly isMaxFieldsExceeded: boolean;
    readonly isApprovalNotRequested: boolean;
    readonly isJobDefinitionNotFound: boolean;
    readonly isServiceOrJobCallNotFound: boolean;
    readonly isJobCallResultNotFound: boolean;
    readonly isEvmAbiEncode: boolean;
    readonly isEvmAbiDecode: boolean;
    readonly isOperatorProfileNotFound: boolean;
    readonly isMaxServicesPerOperatorExceeded: boolean;
    readonly isMaxBlueprintsPerOperatorExceeded: boolean;
    readonly isDuplicateOperator: boolean;
    readonly isDuplicateKey: boolean;
    readonly isTooManyOperators: boolean;
    readonly isTooFewOperators: boolean;
    readonly isNoAssetsProvided: boolean;
    readonly isDuplicateAsset: boolean;
    readonly isMaxAssetsPerServiceExceeded: boolean;
    readonly isNativeAssetExposureTooLow: boolean;
    readonly isNoNativeAsset: boolean;
    readonly isOffenderNotOperator: boolean;
    readonly isNoSlashingOrigin: boolean;
    readonly isNoDisputeOrigin: boolean;
    readonly isUnappliedSlashNotFound: boolean;
    readonly isMasterBlueprintServiceManagerRevisionNotFound: boolean;
    readonly isDuplicateMembershipModel: boolean;
    readonly isMaxMasterBlueprintServiceManagerVersionsExceeded: boolean;
    readonly isErc20TransferFailed: boolean;
    readonly isMissingEVMOrigin: boolean;
    readonly isExpectedEVMAddress: boolean;
    readonly isExpectedAccountId: boolean;
    readonly isOnRequestFailure: boolean;
    readonly isOnRegisterHookFailed: boolean;
    readonly isOnApproveFailure: boolean;
    readonly isOnRejectFailure: boolean;
    readonly isOnServiceInitHook: boolean;
    readonly isUnsupportedMembershipModel: boolean;
    readonly isDynamicMembershipNotSupported: boolean;
    readonly isJoinRejected: boolean;
    readonly isLeaveRejected: boolean;
    readonly isMaxOperatorsReached: boolean;
    readonly isOnCanJoinFailure: boolean;
    readonly isOnCanLeaveFailure: boolean;
    readonly isOnOperatorJoinFailure: boolean;
    readonly isOnOperatorLeaveFailure: boolean;
    readonly isAlreadyJoined: boolean;
    readonly isNotAnOperator: boolean;
    readonly isInvalidSlashPercentage: boolean;
    readonly isInvalidKey: boolean;
    readonly isInvalidSecurityCommitments: boolean;
    readonly isInvalidSecurityRequirements: boolean;
    readonly isInvalidQuoteSignature: boolean;
    readonly isSignatureCountMismatch: boolean;
    readonly isMissingQuoteSignature: boolean;
    readonly isInvalidKeyForQuote: boolean;
    readonly isSignatureVerificationFailed: boolean;
    readonly isInvalidSignatureBytes: boolean;
    readonly isGetHeartbeatIntervalFailure: boolean;
    readonly isGetHeartbeatThresholdFailure: boolean;
    readonly isGetSlashingWindowFailure: boolean;
    readonly isHeartbeatTooEarly: boolean;
    readonly isHeartbeatSignatureVerificationFailed: boolean;
    readonly isInvalidHeartbeatData: boolean;
    readonly isServiceNotActive: boolean;
    readonly isInvalidJobId: boolean;
    readonly isPaymentAlreadyProcessed: boolean;
    readonly isPaymentCalculationOverflow: boolean;
    readonly isTooManySubscriptions: boolean;
    readonly isCustomAssetTransferFailed: boolean;
    readonly isAssetNotFound: boolean;
    readonly isInvalidErc20Address: boolean;
    readonly isInsufficientDelegatedStake: boolean;
    readonly isUnexpectedAssetCommitment: boolean;
    readonly isNoOperatorStake: boolean;
    readonly isCommitmentBelowMinimum: boolean;
    readonly isCommitmentAboveMaximum: boolean;
    readonly isMissingAssetCommitment: boolean;
    readonly isOperatorHasNoAssetStake: boolean;
    readonly isInvalidEventCount: boolean;
    readonly isMetricsDataTooLarge: boolean;
    readonly isSubscriptionNotValid: boolean;
    readonly isServiceNotOwned: boolean;
    readonly type: 'BlueprintNotFound' | 'BlueprintCreationInterrupted' | 'AlreadyRegistered' | 'NotRegistered' | 'OperatorNotActive' | 'InvalidRegistrationInput' | 'NotAllowedToUnregister' | 'NotAllowedToUpdateRpcAddress' | 'InvalidRequestInput' | 'InvalidJobCallInput' | 'InvalidJobResult' | 'ApprovalInterrupted' | 'RejectionInterrupted' | 'ServiceRequestNotFound' | 'ServiceInitializationInterrupted' | 'ServiceNotFound' | 'TerminationInterrupted' | 'TypeCheck' | 'MaxPermittedCallersExceeded' | 'MaxServiceProvidersExceeded' | 'MaxServicesPerUserExceeded' | 'MaxFieldsExceeded' | 'ApprovalNotRequested' | 'JobDefinitionNotFound' | 'ServiceOrJobCallNotFound' | 'JobCallResultNotFound' | 'EvmAbiEncode' | 'EvmAbiDecode' | 'OperatorProfileNotFound' | 'MaxServicesPerOperatorExceeded' | 'MaxBlueprintsPerOperatorExceeded' | 'DuplicateOperator' | 'DuplicateKey' | 'TooManyOperators' | 'TooFewOperators' | 'NoAssetsProvided' | 'DuplicateAsset' | 'MaxAssetsPerServiceExceeded' | 'NativeAssetExposureTooLow' | 'NoNativeAsset' | 'OffenderNotOperator' | 'NoSlashingOrigin' | 'NoDisputeOrigin' | 'UnappliedSlashNotFound' | 'MasterBlueprintServiceManagerRevisionNotFound' | 'DuplicateMembershipModel' | 'MaxMasterBlueprintServiceManagerVersionsExceeded' | 'Erc20TransferFailed' | 'MissingEVMOrigin' | 'ExpectedEVMAddress' | 'ExpectedAccountId' | 'OnRequestFailure' | 'OnRegisterHookFailed' | 'OnApproveFailure' | 'OnRejectFailure' | 'OnServiceInitHook' | 'UnsupportedMembershipModel' | 'DynamicMembershipNotSupported' | 'JoinRejected' | 'LeaveRejected' | 'MaxOperatorsReached' | 'OnCanJoinFailure' | 'OnCanLeaveFailure' | 'OnOperatorJoinFailure' | 'OnOperatorLeaveFailure' | 'AlreadyJoined' | 'NotAnOperator' | 'InvalidSlashPercentage' | 'InvalidKey' | 'InvalidSecurityCommitments' | 'InvalidSecurityRequirements' | 'InvalidQuoteSignature' | 'SignatureCountMismatch' | 'MissingQuoteSignature' | 'InvalidKeyForQuote' | 'SignatureVerificationFailed' | 'InvalidSignatureBytes' | 'GetHeartbeatIntervalFailure' | 'GetHeartbeatThresholdFailure' | 'GetSlashingWindowFailure' | 'HeartbeatTooEarly' | 'HeartbeatSignatureVerificationFailed' | 'InvalidHeartbeatData' | 'ServiceNotActive' | 'InvalidJobId' | 'PaymentAlreadyProcessed' | 'PaymentCalculationOverflow' | 'TooManySubscriptions' | 'CustomAssetTransferFailed' | 'AssetNotFound' | 'InvalidErc20Address' | 'InsufficientDelegatedStake' | 'UnexpectedAssetCommitment' | 'NoOperatorStake' | 'CommitmentBelowMinimum' | 'CommitmentAboveMaximum' | 'MissingAssetCommitment' | 'OperatorHasNoAssetStake' | 'InvalidEventCount' | 'MetricsDataTooLarge' | 'SubscriptionNotValid' | 'ServiceNotOwned';
  }

  /** @name TanglePrimitivesServicesTypesTypeCheckError (843) */
  interface TanglePrimitivesServicesTypesTypeCheckError extends Enum {
    readonly isArgumentTypeMismatch: boolean;
    readonly asArgumentTypeMismatch: {
      readonly index: u8;
      readonly expected: TanglePrimitivesServicesFieldFieldType;
      readonly actual: TanglePrimitivesServicesFieldFieldType;
    } & Struct;
    readonly isNotEnoughArguments: boolean;
    readonly asNotEnoughArguments: {
      readonly expected: u8;
      readonly actual: u8;
    } & Struct;
    readonly isResultTypeMismatch: boolean;
    readonly asResultTypeMismatch: {
      readonly index: u8;
      readonly expected: TanglePrimitivesServicesFieldFieldType;
      readonly actual: TanglePrimitivesServicesFieldFieldType;
    } & Struct;
    readonly type: 'ArgumentTypeMismatch' | 'NotEnoughArguments' | 'ResultTypeMismatch';
  }

  /** @name PalletTangleLstBondedPoolBondedPoolInner (844) */
  interface PalletTangleLstBondedPoolBondedPoolInner extends Struct {
    readonly commission: PalletTangleLstCommission;
    readonly roles: PalletTangleLstPoolsPoolRoles;
    readonly state: PalletTangleLstPoolsPoolState;
    readonly metadata: PalletTangleLstBondedPoolPoolMetadata;
  }

  /** @name PalletTangleLstCommission (845) */
  interface PalletTangleLstCommission extends Struct {
    readonly current: Option<ITuple<[Perbill, AccountId32]>>;
    readonly max: Option<Perbill>;
    readonly changeRate: Option<PalletTangleLstCommissionCommissionChangeRate>;
    readonly throttleFrom: Option<u64>;
    readonly claimPermission: Option<PalletTangleLstCommissionCommissionClaimPermission>;
  }

  /** @name PalletTangleLstPoolsPoolRoles (847) */
  interface PalletTangleLstPoolsPoolRoles extends Struct {
    readonly depositor: AccountId32;
    readonly root: Option<AccountId32>;
    readonly nominator: Option<AccountId32>;
    readonly bouncer: Option<AccountId32>;
  }

  /** @name PalletTangleLstBondedPoolPoolMetadata (848) */
  interface PalletTangleLstBondedPoolPoolMetadata extends Struct {
    readonly name: Option<Bytes>;
    readonly icon: Option<Bytes>;
  }

  /** @name PalletTangleLstSubPoolsRewardPool (849) */
  interface PalletTangleLstSubPoolsRewardPool extends Struct {
    readonly lastRecordedRewardCounter: u128;
    readonly lastRecordedTotalPayouts: u128;
    readonly totalRewardsClaimed: u128;
    readonly totalCommissionPending: u128;
    readonly totalCommissionClaimed: u128;
  }

  /** @name PalletTangleLstSubPools (850) */
  interface PalletTangleLstSubPools extends Struct {
    readonly noEra: PalletTangleLstSubPoolsUnbondPool;
    readonly withEra: BTreeMap<u32, PalletTangleLstSubPoolsUnbondPool>;
  }

  /** @name PalletTangleLstSubPoolsUnbondPool (851) */
  interface PalletTangleLstSubPoolsUnbondPool extends Struct {
    readonly points: u128;
    readonly balance: u128;
  }

  /** @name PalletTangleLstPoolsPoolMember (857) */
  interface PalletTangleLstPoolsPoolMember extends Struct {
    readonly unbondingEras: BTreeMap<u32, ITuple<[u32, u128]>>;
  }

  /** @name PalletTangleLstClaimPermission (862) */
  interface PalletTangleLstClaimPermission extends Enum {
    readonly isPermissioned: boolean;
    readonly isPermissionlessCompound: boolean;
    readonly isPermissionlessWithdraw: boolean;
    readonly isPermissionlessAll: boolean;
    readonly type: 'Permissioned' | 'PermissionlessCompound' | 'PermissionlessWithdraw' | 'PermissionlessAll';
  }

  /** @name PalletTangleLstError (863) */
  interface PalletTangleLstError extends Enum {
    readonly isPoolNotFound: boolean;
    readonly isPoolMemberNotFound: boolean;
    readonly isRewardPoolNotFound: boolean;
    readonly isSubPoolsNotFound: boolean;
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
    readonly asDefensive: PalletTangleLstDefensiveError;
    readonly isPartialUnbondNotAllowedPermissionlessly: boolean;
    readonly isMaxCommissionRestricted: boolean;
    readonly isCommissionExceedsMaximum: boolean;
    readonly isCommissionExceedsGlobalMaximum: boolean;
    readonly isCommissionChangeThrottled: boolean;
    readonly isCommissionChangeRateNotAllowed: boolean;
    readonly isNoPendingCommission: boolean;
    readonly isNoCommissionCurrentSet: boolean;
    readonly isPoolIdInUse: boolean;
    readonly isInvalidPoolId: boolean;
    readonly isBondExtraRestricted: boolean;
    readonly isNothingToAdjust: boolean;
    readonly isPoolTokenCreationFailed: boolean;
    readonly isNoBalanceToUnbond: boolean;
    readonly type: 'PoolNotFound' | 'PoolMemberNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'FullyUnbonding' | 'MaxUnbondingLimit' | 'CannotWithdrawAny' | 'MinimumBondNotMet' | 'OverflowRisk' | 'NotDestroying' | 'NotNominator' | 'NotKickerOrDestroying' | 'NotOpen' | 'MaxPools' | 'MaxPoolMembers' | 'CanNotChangeState' | 'DoesNotHavePermission' | 'MetadataExceedsMaxLen' | 'Defensive' | 'PartialUnbondNotAllowedPermissionlessly' | 'MaxCommissionRestricted' | 'CommissionExceedsMaximum' | 'CommissionExceedsGlobalMaximum' | 'CommissionChangeThrottled' | 'CommissionChangeRateNotAllowed' | 'NoPendingCommission' | 'NoCommissionCurrentSet' | 'PoolIdInUse' | 'InvalidPoolId' | 'BondExtraRestricted' | 'NothingToAdjust' | 'PoolTokenCreationFailed' | 'NoBalanceToUnbond';
  }

  /** @name PalletTangleLstDefensiveError (864) */
  interface PalletTangleLstDefensiveError extends Enum {
    readonly isNotEnoughSpaceInUnbondPool: boolean;
    readonly isPoolNotFound: boolean;
    readonly isRewardPoolNotFound: boolean;
    readonly isSubPoolsNotFound: boolean;
    readonly isBondedStashKilledPrematurely: boolean;
    readonly type: 'NotEnoughSpaceInUnbondPool' | 'PoolNotFound' | 'RewardPoolNotFound' | 'SubPoolsNotFound' | 'BondedStashKilledPrematurely';
  }

  /** @name PalletRewardsVaultMetadata (868) */
  interface PalletRewardsVaultMetadata extends Struct {
    readonly name: Bytes;
    readonly logo: Bytes;
  }

  /** @name PalletRewardsError (871) */
  interface PalletRewardsError extends Enum {
    readonly isNoRewardsAvailable: boolean;
    readonly isInsufficientRewardsBalance: boolean;
    readonly isAssetNotWhitelisted: boolean;
    readonly isAssetAlreadyWhitelisted: boolean;
    readonly isInvalidAPY: boolean;
    readonly isAssetAlreadyInVault: boolean;
    readonly isAssetNotInVault: boolean;
    readonly isVaultNotFound: boolean;
    readonly isDuplicateBlueprintId: boolean;
    readonly isBlueprintIdNotFound: boolean;
    readonly isRewardConfigNotFound: boolean;
    readonly isCannotCalculatePropotionalApy: boolean;
    readonly isCannotCalculateRewardPerBlock: boolean;
    readonly isIncentiveCapGreaterThanDepositCap: boolean;
    readonly isBoostMultiplierMustBeOne: boolean;
    readonly isVaultAlreadyExists: boolean;
    readonly isTotalDepositLessThanIncentiveCap: boolean;
    readonly isPotAlreadyExists: boolean;
    readonly isPotAccountNotFound: boolean;
    readonly isInvalidDecayRate: boolean;
    readonly isIncentiveCapGreaterThanMaxIncentiveCap: boolean;
    readonly isDepositCapGreaterThanMaxDepositCap: boolean;
    readonly isIncentiveCapLessThanMinIncentiveCap: boolean;
    readonly isDepositCapLessThanMinDepositCap: boolean;
    readonly isNameTooLong: boolean;
    readonly isLogoTooLong: boolean;
    readonly isVaultMetadataNotFound: boolean;
    readonly isNoRewardsToClaim: boolean;
    readonly isArithmeticOverflow: boolean;
    readonly isTransferFailed: boolean;
    readonly isTooManyPendingRewards: boolean;
    readonly type: 'NoRewardsAvailable' | 'InsufficientRewardsBalance' | 'AssetNotWhitelisted' | 'AssetAlreadyWhitelisted' | 'InvalidAPY' | 'AssetAlreadyInVault' | 'AssetNotInVault' | 'VaultNotFound' | 'DuplicateBlueprintId' | 'BlueprintIdNotFound' | 'RewardConfigNotFound' | 'CannotCalculatePropotionalApy' | 'CannotCalculateRewardPerBlock' | 'IncentiveCapGreaterThanDepositCap' | 'BoostMultiplierMustBeOne' | 'VaultAlreadyExists' | 'TotalDepositLessThanIncentiveCap' | 'PotAlreadyExists' | 'PotAccountNotFound' | 'InvalidDecayRate' | 'IncentiveCapGreaterThanMaxIncentiveCap' | 'DepositCapGreaterThanMaxDepositCap' | 'IncentiveCapLessThanMinIncentiveCap' | 'DepositCapLessThanMinDepositCap' | 'NameTooLong' | 'LogoTooLong' | 'VaultMetadataNotFound' | 'NoRewardsToClaim' | 'ArithmeticOverflow' | 'TransferFailed' | 'TooManyPendingRewards';
  }

  /** @name PalletIsmpError (872) */
  interface PalletIsmpError extends Enum {
    readonly isInvalidMessage: boolean;
    readonly isMessageNotFound: boolean;
    readonly isConsensusClientCreationFailed: boolean;
    readonly isUnbondingPeriodUpdateFailed: boolean;
    readonly isChallengePeriodUpdateFailed: boolean;
    readonly type: 'InvalidMessage' | 'MessageNotFound' | 'ConsensusClientCreationFailed' | 'UnbondingPeriodUpdateFailed' | 'ChallengePeriodUpdateFailed';
  }

  /** @name PalletHyperbridgeError (873) */
  type PalletHyperbridgeError = Null;

  /** @name PalletTokenGatewayError (875) */
  interface PalletTokenGatewayError extends Enum {
    readonly isUnregisteredAsset: boolean;
    readonly isAssetTeleportError: boolean;
    readonly isCoprocessorNotConfigured: boolean;
    readonly isDispatchError: boolean;
    readonly isAssetCreationError: boolean;
    readonly isAssetDecimalsNotFound: boolean;
    readonly isNotInitialized: boolean;
    readonly isUnknownAsset: boolean;
    readonly isNotAssetOwner: boolean;
    readonly type: 'UnregisteredAsset' | 'AssetTeleportError' | 'CoprocessorNotConfigured' | 'DispatchError' | 'AssetCreationError' | 'AssetDecimalsNotFound' | 'NotInitialized' | 'UnknownAsset' | 'NotAssetOwner';
  }

  /** @name PalletCreditsError (877) */
  interface PalletCreditsError extends Enum {
    readonly isInsufficientTntBalance: boolean;
    readonly isClaimAmountExceedsWindowAllowance: boolean;
    readonly isInvalidClaimId: boolean;
    readonly isNoValidTier: boolean;
    readonly isAmountZero: boolean;
    readonly isBurnTransferNotImplemented: boolean;
    readonly isStakeTiersNotSorted: boolean;
    readonly isEmptyStakeTiers: boolean;
    readonly isOverflow: boolean;
    readonly isStakeTiersOverflow: boolean;
    readonly isAssetRatesNotConfigured: boolean;
    readonly isRateTooHigh: boolean;
    readonly type: 'InsufficientTntBalance' | 'ClaimAmountExceedsWindowAllowance' | 'InvalidClaimId' | 'NoValidTier' | 'AmountZero' | 'BurnTransferNotImplemented' | 'StakeTiersNotSorted' | 'EmptyStakeTiers' | 'Overflow' | 'StakeTiersOverflow' | 'AssetRatesNotConfigured' | 'RateTooHigh';
  }

  /** @name FrameSystemExtensionsCheckNonZeroSender (880) */
  type FrameSystemExtensionsCheckNonZeroSender = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (881) */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (882) */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (883) */
  type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (886) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (887) */
  type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (888) */
  interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name FrameMetadataHashExtensionCheckMetadataHash (889) */
  interface FrameMetadataHashExtensionCheckMetadataHash extends Struct {
    readonly mode: FrameMetadataHashExtensionMode;
  }

  /** @name FrameMetadataHashExtensionMode (890) */
  interface FrameMetadataHashExtensionMode extends Enum {
    readonly isDisabled: boolean;
    readonly isEnabled: boolean;
    readonly type: 'Disabled' | 'Enabled';
  }

  /** @name TangleTestnetRuntimeExtensionCheckNominatedRestaked (891) */
  type TangleTestnetRuntimeExtensionCheckNominatedRestaked = Null;

  /** @name TangleTestnetRuntimeRuntime (893) */
  type TangleTestnetRuntimeRuntime = Null;

} // declare module
