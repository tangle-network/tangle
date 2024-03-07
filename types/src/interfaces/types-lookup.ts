// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/types/lookup'

import type { Data } from '@polkadot/types'
import type {
  BTreeMap,
  Bytes,
  Compact,
  Enum,
  Null,
  Option,
  Result,
  Struct,
  Text,
  U256,
  U8aFixed,
  Vec,
  bool,
  u128,
  u16,
  u32,
  u64,
  u8,
} from '@polkadot/types-codec'
import type { ITuple } from '@polkadot/types-codec/types'
import type { Vote } from '@polkadot/types/interfaces/elections'
import type {
  AccountId32,
  Call,
  H160,
  H256,
  MultiAddress,
  PerU16,
  Perbill,
  Percent,
  Permill,
} from '@polkadot/types/interfaces/runtime'
import type { Event } from '@polkadot/types/interfaces/system'

declare module '@polkadot/types/lookup' {
  /** @name FrameSystemAccountInfo (3) */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32
    readonly consumers: u32
    readonly providers: u32
    readonly sufficients: u32
    readonly data: PalletBalancesAccountData
  }

  /** @name PalletBalancesAccountData (5) */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128
    readonly reserved: u128
    readonly frozen: u128
    readonly flags: u128
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (8) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight
    readonly operational: SpWeightsWeightV2Weight
    readonly mandatory: SpWeightsWeightV2Weight
  }

  /** @name SpWeightsWeightV2Weight (9) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>
    readonly proofSize: Compact<u64>
  }

  /** @name SpRuntimeDigest (14) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>
  }

  /** @name SpRuntimeDigestDigestItem (16) */
  interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean
    readonly asOther: Bytes
    readonly isConsensus: boolean
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>
    readonly isSeal: boolean
    readonly asSeal: ITuple<[U8aFixed, Bytes]>
    readonly isPreRuntime: boolean
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>
    readonly isRuntimeEnvironmentUpdated: boolean
    readonly type:
      | 'Other'
      | 'Consensus'
      | 'Seal'
      | 'PreRuntime'
      | 'RuntimeEnvironmentUpdated'
  }

  /** @name FrameSystemEventRecord (19) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase
    readonly event: Event
    readonly topics: Vec<H256>
  }

  /** @name FrameSystemEvent (21) */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo
    } & Struct
    readonly isExtrinsicFailed: boolean
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo
    } & Struct
    readonly isCodeUpdated: boolean
    readonly isNewAccount: boolean
    readonly asNewAccount: {
      readonly account: AccountId32
    } & Struct
    readonly isKilledAccount: boolean
    readonly asKilledAccount: {
      readonly account: AccountId32
    } & Struct
    readonly isRemarked: boolean
    readonly asRemarked: {
      readonly sender: AccountId32
      readonly hash_: H256
    } & Struct
    readonly isUpgradeAuthorized: boolean
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256
      readonly checkVersion: bool
    } & Struct
    readonly type:
      | 'ExtrinsicSuccess'
      | 'ExtrinsicFailed'
      | 'CodeUpdated'
      | 'NewAccount'
      | 'KilledAccount'
      | 'Remarked'
      | 'UpgradeAuthorized'
  }

  /** @name FrameSupportDispatchDispatchInfo (22) */
  interface FrameSupportDispatchDispatchInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight
    readonly class: FrameSupportDispatchDispatchClass
    readonly paysFee: FrameSupportDispatchPays
  }

  /** @name FrameSupportDispatchDispatchClass (23) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean
    readonly isOperational: boolean
    readonly isMandatory: boolean
    readonly type: 'Normal' | 'Operational' | 'Mandatory'
  }

  /** @name FrameSupportDispatchPays (24) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean
    readonly isNo: boolean
    readonly type: 'Yes' | 'No'
  }

  /** @name SpRuntimeDispatchError (25) */
  interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean
    readonly isCannotLookup: boolean
    readonly isBadOrigin: boolean
    readonly isModule: boolean
    readonly asModule: SpRuntimeModuleError
    readonly isConsumerRemaining: boolean
    readonly isNoProviders: boolean
    readonly isTooManyConsumers: boolean
    readonly isToken: boolean
    readonly asToken: SpRuntimeTokenError
    readonly isArithmetic: boolean
    readonly asArithmetic: SpArithmeticArithmeticError
    readonly isTransactional: boolean
    readonly asTransactional: SpRuntimeTransactionalError
    readonly isExhausted: boolean
    readonly isCorruption: boolean
    readonly isUnavailable: boolean
    readonly isRootNotAllowed: boolean
    readonly type:
      | 'Other'
      | 'CannotLookup'
      | 'BadOrigin'
      | 'Module'
      | 'ConsumerRemaining'
      | 'NoProviders'
      | 'TooManyConsumers'
      | 'Token'
      | 'Arithmetic'
      | 'Transactional'
      | 'Exhausted'
      | 'Corruption'
      | 'Unavailable'
      | 'RootNotAllowed'
  }

  /** @name SpRuntimeModuleError (26) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8
    readonly error: U8aFixed
  }

  /** @name SpRuntimeTokenError (27) */
  interface SpRuntimeTokenError extends Enum {
    readonly isFundsUnavailable: boolean
    readonly isOnlyProvider: boolean
    readonly isBelowMinimum: boolean
    readonly isCannotCreate: boolean
    readonly isUnknownAsset: boolean
    readonly isFrozen: boolean
    readonly isUnsupported: boolean
    readonly isCannotCreateHold: boolean
    readonly isNotExpendable: boolean
    readonly isBlocked: boolean
    readonly type:
      | 'FundsUnavailable'
      | 'OnlyProvider'
      | 'BelowMinimum'
      | 'CannotCreate'
      | 'UnknownAsset'
      | 'Frozen'
      | 'Unsupported'
      | 'CannotCreateHold'
      | 'NotExpendable'
      | 'Blocked'
  }

  /** @name SpArithmeticArithmeticError (28) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean
    readonly isOverflow: boolean
    readonly isDivisionByZero: boolean
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero'
  }

  /** @name SpRuntimeTransactionalError (29) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean
    readonly isNoLayer: boolean
    readonly type: 'LimitReached' | 'NoLayer'
  }

  /** @name PalletSudoEvent (31) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly isKeyChanged: boolean
    readonly asKeyChanged: {
      readonly old: Option<AccountId32>
      readonly new_: AccountId32
    } & Struct
    readonly isKeyRemoved: boolean
    readonly isSudoAsDone: boolean
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly type: 'Sudid' | 'KeyChanged' | 'KeyRemoved' | 'SudoAsDone'
  }

  /** @name PalletBalancesEvent (35) */
  interface PalletBalancesEvent extends Enum {
    readonly isEndowed: boolean
    readonly asEndowed: {
      readonly account: AccountId32
      readonly freeBalance: u128
    } & Struct
    readonly isDustLost: boolean
    readonly asDustLost: {
      readonly account: AccountId32
      readonly amount: u128
    } & Struct
    readonly isTransfer: boolean
    readonly asTransfer: {
      readonly from: AccountId32
      readonly to: AccountId32
      readonly amount: u128
    } & Struct
    readonly isBalanceSet: boolean
    readonly asBalanceSet: {
      readonly who: AccountId32
      readonly free: u128
    } & Struct
    readonly isReserved: boolean
    readonly asReserved: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isUnreserved: boolean
    readonly asUnreserved: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isReserveRepatriated: boolean
    readonly asReserveRepatriated: {
      readonly from: AccountId32
      readonly to: AccountId32
      readonly amount: u128
      readonly destinationStatus: FrameSupportTokensMiscBalanceStatus
    } & Struct
    readonly isDeposit: boolean
    readonly asDeposit: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isWithdraw: boolean
    readonly asWithdraw: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isSlashed: boolean
    readonly asSlashed: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isMinted: boolean
    readonly asMinted: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isBurned: boolean
    readonly asBurned: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isSuspended: boolean
    readonly asSuspended: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isRestored: boolean
    readonly asRestored: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isUpgraded: boolean
    readonly asUpgraded: {
      readonly who: AccountId32
    } & Struct
    readonly isIssued: boolean
    readonly asIssued: {
      readonly amount: u128
    } & Struct
    readonly isRescinded: boolean
    readonly asRescinded: {
      readonly amount: u128
    } & Struct
    readonly isLocked: boolean
    readonly asLocked: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isUnlocked: boolean
    readonly asUnlocked: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isFrozen: boolean
    readonly asFrozen: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isThawed: boolean
    readonly asThawed: {
      readonly who: AccountId32
      readonly amount: u128
    } & Struct
    readonly isTotalIssuanceForced: boolean
    readonly asTotalIssuanceForced: {
      readonly old: u128
      readonly new_: u128
    } & Struct
    readonly type:
      | 'Endowed'
      | 'DustLost'
      | 'Transfer'
      | 'BalanceSet'
      | 'Reserved'
      | 'Unreserved'
      | 'ReserveRepatriated'
      | 'Deposit'
      | 'Withdraw'
      | 'Slashed'
      | 'Minted'
      | 'Burned'
      | 'Suspended'
      | 'Restored'
      | 'Upgraded'
      | 'Issued'
      | 'Rescinded'
      | 'Locked'
      | 'Unlocked'
      | 'Frozen'
      | 'Thawed'
      | 'TotalIssuanceForced'
  }

  /** @name FrameSupportTokensMiscBalanceStatus (36) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean
    readonly isReserved: boolean
    readonly type: 'Free' | 'Reserved'
  }

  /** @name PalletTransactionPaymentEvent (37) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean
    readonly asTransactionFeePaid: {
      readonly who: AccountId32
      readonly actualFee: u128
      readonly tip: u128
    } & Struct
    readonly type: 'TransactionFeePaid'
  }

  /** @name PalletGrandpaEvent (38) */
  interface PalletGrandpaEvent extends Enum {
    readonly isNewAuthorities: boolean
    readonly asNewAuthorities: {
      readonly authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>
    } & Struct
    readonly isPaused: boolean
    readonly isResumed: boolean
    readonly type: 'NewAuthorities' | 'Paused' | 'Resumed'
  }

  /** @name SpConsensusGrandpaAppPublic (41) */
  interface SpConsensusGrandpaAppPublic extends SpCoreEd25519Public {}

  /** @name SpCoreEd25519Public (42) */
  interface SpCoreEd25519Public extends U8aFixed {}

  /** @name PalletIndicesEvent (43) */
  interface PalletIndicesEvent extends Enum {
    readonly isIndexAssigned: boolean
    readonly asIndexAssigned: {
      readonly who: AccountId32
      readonly index: u32
    } & Struct
    readonly isIndexFreed: boolean
    readonly asIndexFreed: {
      readonly index: u32
    } & Struct
    readonly isIndexFrozen: boolean
    readonly asIndexFrozen: {
      readonly index: u32
      readonly who: AccountId32
    } & Struct
    readonly type: 'IndexAssigned' | 'IndexFreed' | 'IndexFrozen'
  }

  /** @name PalletDemocracyEvent (44) */
  interface PalletDemocracyEvent extends Enum {
    readonly isProposed: boolean
    readonly asProposed: {
      readonly proposalIndex: u32
      readonly deposit: u128
    } & Struct
    readonly isTabled: boolean
    readonly asTabled: {
      readonly proposalIndex: u32
      readonly deposit: u128
    } & Struct
    readonly isExternalTabled: boolean
    readonly isStarted: boolean
    readonly asStarted: {
      readonly refIndex: u32
      readonly threshold: PalletDemocracyVoteThreshold
    } & Struct
    readonly isPassed: boolean
    readonly asPassed: {
      readonly refIndex: u32
    } & Struct
    readonly isNotPassed: boolean
    readonly asNotPassed: {
      readonly refIndex: u32
    } & Struct
    readonly isCancelled: boolean
    readonly asCancelled: {
      readonly refIndex: u32
    } & Struct
    readonly isDelegated: boolean
    readonly asDelegated: {
      readonly who: AccountId32
      readonly target: AccountId32
    } & Struct
    readonly isUndelegated: boolean
    readonly asUndelegated: {
      readonly account: AccountId32
    } & Struct
    readonly isVetoed: boolean
    readonly asVetoed: {
      readonly who: AccountId32
      readonly proposalHash: H256
      readonly until: u64
    } & Struct
    readonly isBlacklisted: boolean
    readonly asBlacklisted: {
      readonly proposalHash: H256
    } & Struct
    readonly isVoted: boolean
    readonly asVoted: {
      readonly voter: AccountId32
      readonly refIndex: u32
      readonly vote: PalletDemocracyVoteAccountVote
    } & Struct
    readonly isSeconded: boolean
    readonly asSeconded: {
      readonly seconder: AccountId32
      readonly propIndex: u32
    } & Struct
    readonly isProposalCanceled: boolean
    readonly asProposalCanceled: {
      readonly propIndex: u32
    } & Struct
    readonly isMetadataSet: boolean
    readonly asMetadataSet: {
      readonly owner: PalletDemocracyMetadataOwner
      readonly hash_: H256
    } & Struct
    readonly isMetadataCleared: boolean
    readonly asMetadataCleared: {
      readonly owner: PalletDemocracyMetadataOwner
      readonly hash_: H256
    } & Struct
    readonly isMetadataTransferred: boolean
    readonly asMetadataTransferred: {
      readonly prevOwner: PalletDemocracyMetadataOwner
      readonly owner: PalletDemocracyMetadataOwner
      readonly hash_: H256
    } & Struct
    readonly type:
      | 'Proposed'
      | 'Tabled'
      | 'ExternalTabled'
      | 'Started'
      | 'Passed'
      | 'NotPassed'
      | 'Cancelled'
      | 'Delegated'
      | 'Undelegated'
      | 'Vetoed'
      | 'Blacklisted'
      | 'Voted'
      | 'Seconded'
      | 'ProposalCanceled'
      | 'MetadataSet'
      | 'MetadataCleared'
      | 'MetadataTransferred'
  }

  /** @name PalletDemocracyVoteThreshold (45) */
  interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean
    readonly isSuperMajorityAgainst: boolean
    readonly isSimpleMajority: boolean
    readonly type:
      | 'SuperMajorityApprove'
      | 'SuperMajorityAgainst'
      | 'SimpleMajority'
  }

  /** @name PalletDemocracyVoteAccountVote (46) */
  interface PalletDemocracyVoteAccountVote extends Enum {
    readonly isStandard: boolean
    readonly asStandard: {
      readonly vote: Vote
      readonly balance: u128
    } & Struct
    readonly isSplit: boolean
    readonly asSplit: {
      readonly aye: u128
      readonly nay: u128
    } & Struct
    readonly type: 'Standard' | 'Split'
  }

  /** @name PalletDemocracyMetadataOwner (48) */
  interface PalletDemocracyMetadataOwner extends Enum {
    readonly isExternal: boolean
    readonly isProposal: boolean
    readonly asProposal: u32
    readonly isReferendum: boolean
    readonly asReferendum: u32
    readonly type: 'External' | 'Proposal' | 'Referendum'
  }

  /** @name PalletCollectiveEvent (49) */
  interface PalletCollectiveEvent extends Enum {
    readonly isProposed: boolean
    readonly asProposed: {
      readonly account: AccountId32
      readonly proposalIndex: u32
      readonly proposalHash: H256
      readonly threshold: u32
    } & Struct
    readonly isVoted: boolean
    readonly asVoted: {
      readonly account: AccountId32
      readonly proposalHash: H256
      readonly voted: bool
      readonly yes: u32
      readonly no: u32
    } & Struct
    readonly isApproved: boolean
    readonly asApproved: {
      readonly proposalHash: H256
    } & Struct
    readonly isDisapproved: boolean
    readonly asDisapproved: {
      readonly proposalHash: H256
    } & Struct
    readonly isExecuted: boolean
    readonly asExecuted: {
      readonly proposalHash: H256
      readonly result: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly isMemberExecuted: boolean
    readonly asMemberExecuted: {
      readonly proposalHash: H256
      readonly result: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly isClosed: boolean
    readonly asClosed: {
      readonly proposalHash: H256
      readonly yes: u32
      readonly no: u32
    } & Struct
    readonly type:
      | 'Proposed'
      | 'Voted'
      | 'Approved'
      | 'Disapproved'
      | 'Executed'
      | 'MemberExecuted'
      | 'Closed'
  }

  /** @name PalletVestingEvent (50) */
  interface PalletVestingEvent extends Enum {
    readonly isVestingUpdated: boolean
    readonly asVestingUpdated: {
      readonly account: AccountId32
      readonly unvested: u128
    } & Struct
    readonly isVestingCompleted: boolean
    readonly asVestingCompleted: {
      readonly account: AccountId32
    } & Struct
    readonly type: 'VestingUpdated' | 'VestingCompleted'
  }

  /** @name PalletElectionsPhragmenEvent (51) */
  interface PalletElectionsPhragmenEvent extends Enum {
    readonly isNewTerm: boolean
    readonly asNewTerm: {
      readonly newMembers: Vec<ITuple<[AccountId32, u128]>>
    } & Struct
    readonly isEmptyTerm: boolean
    readonly isElectionError: boolean
    readonly isMemberKicked: boolean
    readonly asMemberKicked: {
      readonly member: AccountId32
    } & Struct
    readonly isRenounced: boolean
    readonly asRenounced: {
      readonly candidate: AccountId32
    } & Struct
    readonly isCandidateSlashed: boolean
    readonly asCandidateSlashed: {
      readonly candidate: AccountId32
      readonly amount: u128
    } & Struct
    readonly isSeatHolderSlashed: boolean
    readonly asSeatHolderSlashed: {
      readonly seatHolder: AccountId32
      readonly amount: u128
    } & Struct
    readonly type:
      | 'NewTerm'
      | 'EmptyTerm'
      | 'ElectionError'
      | 'MemberKicked'
      | 'Renounced'
      | 'CandidateSlashed'
      | 'SeatHolderSlashed'
  }

  /** @name PalletElectionProviderMultiPhaseEvent (54) */
  interface PalletElectionProviderMultiPhaseEvent extends Enum {
    readonly isSolutionStored: boolean
    readonly asSolutionStored: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute
      readonly origin: Option<AccountId32>
      readonly prevEjected: bool
    } & Struct
    readonly isElectionFinalized: boolean
    readonly asElectionFinalized: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute
      readonly score: SpNposElectionsElectionScore
    } & Struct
    readonly isElectionFailed: boolean
    readonly isRewarded: boolean
    readonly asRewarded: {
      readonly account: AccountId32
      readonly value: u128
    } & Struct
    readonly isSlashed: boolean
    readonly asSlashed: {
      readonly account: AccountId32
      readonly value: u128
    } & Struct
    readonly isPhaseTransitioned: boolean
    readonly asPhaseTransitioned: {
      readonly from: PalletElectionProviderMultiPhasePhase
      readonly to: PalletElectionProviderMultiPhasePhase
      readonly round: u32
    } & Struct
    readonly type:
      | 'SolutionStored'
      | 'ElectionFinalized'
      | 'ElectionFailed'
      | 'Rewarded'
      | 'Slashed'
      | 'PhaseTransitioned'
  }

  /** @name PalletElectionProviderMultiPhaseElectionCompute (55) */
  interface PalletElectionProviderMultiPhaseElectionCompute extends Enum {
    readonly isOnChain: boolean
    readonly isSigned: boolean
    readonly isUnsigned: boolean
    readonly isFallback: boolean
    readonly isEmergency: boolean
    readonly type: 'OnChain' | 'Signed' | 'Unsigned' | 'Fallback' | 'Emergency'
  }

  /** @name SpNposElectionsElectionScore (56) */
  interface SpNposElectionsElectionScore extends Struct {
    readonly minimalStake: u128
    readonly sumStake: u128
    readonly sumStakeSquared: u128
  }

  /** @name PalletElectionProviderMultiPhasePhase (57) */
  interface PalletElectionProviderMultiPhasePhase extends Enum {
    readonly isOff: boolean
    readonly isSigned: boolean
    readonly isUnsigned: boolean
    readonly asUnsigned: ITuple<[bool, u64]>
    readonly isEmergency: boolean
    readonly type: 'Off' | 'Signed' | 'Unsigned' | 'Emergency'
  }

  /** @name PalletStakingPalletEvent (59) */
  interface PalletStakingPalletEvent extends Enum {
    readonly isEraPaid: boolean
    readonly asEraPaid: {
      readonly eraIndex: u32
      readonly validatorPayout: u128
      readonly remainder: u128
    } & Struct
    readonly isRewarded: boolean
    readonly asRewarded: {
      readonly stash: AccountId32
      readonly dest: PalletStakingRewardDestination
      readonly amount: u128
    } & Struct
    readonly isSlashed: boolean
    readonly asSlashed: {
      readonly staker: AccountId32
      readonly amount: u128
    } & Struct
    readonly isSlashReported: boolean
    readonly asSlashReported: {
      readonly validator: AccountId32
      readonly fraction: Perbill
      readonly slashEra: u32
    } & Struct
    readonly isOldSlashingReportDiscarded: boolean
    readonly asOldSlashingReportDiscarded: {
      readonly sessionIndex: u32
    } & Struct
    readonly isStakersElected: boolean
    readonly isBonded: boolean
    readonly asBonded: {
      readonly stash: AccountId32
      readonly amount: u128
    } & Struct
    readonly isUnbonded: boolean
    readonly asUnbonded: {
      readonly stash: AccountId32
      readonly amount: u128
    } & Struct
    readonly isWithdrawn: boolean
    readonly asWithdrawn: {
      readonly stash: AccountId32
      readonly amount: u128
    } & Struct
    readonly isKicked: boolean
    readonly asKicked: {
      readonly nominator: AccountId32
      readonly stash: AccountId32
    } & Struct
    readonly isStakingElectionFailed: boolean
    readonly isChilled: boolean
    readonly asChilled: {
      readonly stash: AccountId32
    } & Struct
    readonly isPayoutStarted: boolean
    readonly asPayoutStarted: {
      readonly eraIndex: u32
      readonly validatorStash: AccountId32
    } & Struct
    readonly isValidatorPrefsSet: boolean
    readonly asValidatorPrefsSet: {
      readonly stash: AccountId32
      readonly prefs: PalletStakingValidatorPrefs
    } & Struct
    readonly isSnapshotVotersSizeExceeded: boolean
    readonly asSnapshotVotersSizeExceeded: {
      readonly size_: u32
    } & Struct
    readonly isSnapshotTargetsSizeExceeded: boolean
    readonly asSnapshotTargetsSizeExceeded: {
      readonly size_: u32
    } & Struct
    readonly isForceEra: boolean
    readonly asForceEra: {
      readonly mode: PalletStakingForcing
    } & Struct
    readonly type:
      | 'EraPaid'
      | 'Rewarded'
      | 'Slashed'
      | 'SlashReported'
      | 'OldSlashingReportDiscarded'
      | 'StakersElected'
      | 'Bonded'
      | 'Unbonded'
      | 'Withdrawn'
      | 'Kicked'
      | 'StakingElectionFailed'
      | 'Chilled'
      | 'PayoutStarted'
      | 'ValidatorPrefsSet'
      | 'SnapshotVotersSizeExceeded'
      | 'SnapshotTargetsSizeExceeded'
      | 'ForceEra'
  }

  /** @name PalletStakingRewardDestination (60) */
  interface PalletStakingRewardDestination extends Enum {
    readonly isStaked: boolean
    readonly isStash: boolean
    readonly isController: boolean
    readonly isAccount: boolean
    readonly asAccount: AccountId32
    readonly isNone: boolean
    readonly type: 'Staked' | 'Stash' | 'Controller' | 'Account' | 'None'
  }

  /** @name PalletStakingValidatorPrefs (62) */
  interface PalletStakingValidatorPrefs extends Struct {
    readonly commission: Compact<Perbill>
    readonly blocked: bool
  }

  /** @name PalletStakingForcing (64) */
  interface PalletStakingForcing extends Enum {
    readonly isNotForcing: boolean
    readonly isForceNew: boolean
    readonly isForceNone: boolean
    readonly isForceAlways: boolean
    readonly type: 'NotForcing' | 'ForceNew' | 'ForceNone' | 'ForceAlways'
  }

  /** @name PalletSessionEvent (65) */
  interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean
    readonly asNewSession: {
      readonly sessionIndex: u32
    } & Struct
    readonly type: 'NewSession'
  }

  /** @name PalletTreasuryEvent (66) */
  interface PalletTreasuryEvent extends Enum {
    readonly isProposed: boolean
    readonly asProposed: {
      readonly proposalIndex: u32
    } & Struct
    readonly isSpending: boolean
    readonly asSpending: {
      readonly budgetRemaining: u128
    } & Struct
    readonly isAwarded: boolean
    readonly asAwarded: {
      readonly proposalIndex: u32
      readonly award: u128
      readonly account: AccountId32
    } & Struct
    readonly isRejected: boolean
    readonly asRejected: {
      readonly proposalIndex: u32
      readonly slashed: u128
    } & Struct
    readonly isBurnt: boolean
    readonly asBurnt: {
      readonly burntFunds: u128
    } & Struct
    readonly isRollover: boolean
    readonly asRollover: {
      readonly rolloverBalance: u128
    } & Struct
    readonly isDeposit: boolean
    readonly asDeposit: {
      readonly value: u128
    } & Struct
    readonly isSpendApproved: boolean
    readonly asSpendApproved: {
      readonly proposalIndex: u32
      readonly amount: u128
      readonly beneficiary: AccountId32
    } & Struct
    readonly isUpdatedInactive: boolean
    readonly asUpdatedInactive: {
      readonly reactivated: u128
      readonly deactivated: u128
    } & Struct
    readonly isAssetSpendApproved: boolean
    readonly asAssetSpendApproved: {
      readonly index: u32
      readonly assetKind: Null
      readonly amount: u128
      readonly beneficiary: AccountId32
      readonly validFrom: u64
      readonly expireAt: u64
    } & Struct
    readonly isAssetSpendVoided: boolean
    readonly asAssetSpendVoided: {
      readonly index: u32
    } & Struct
    readonly isPaid: boolean
    readonly asPaid: {
      readonly index: u32
      readonly paymentId: Null
    } & Struct
    readonly isPaymentFailed: boolean
    readonly asPaymentFailed: {
      readonly index: u32
      readonly paymentId: Null
    } & Struct
    readonly isSpendProcessed: boolean
    readonly asSpendProcessed: {
      readonly index: u32
    } & Struct
    readonly type:
      | 'Proposed'
      | 'Spending'
      | 'Awarded'
      | 'Rejected'
      | 'Burnt'
      | 'Rollover'
      | 'Deposit'
      | 'SpendApproved'
      | 'UpdatedInactive'
      | 'AssetSpendApproved'
      | 'AssetSpendVoided'
      | 'Paid'
      | 'PaymentFailed'
      | 'SpendProcessed'
  }

  /** @name PalletBountiesEvent (67) */
  interface PalletBountiesEvent extends Enum {
    readonly isBountyProposed: boolean
    readonly asBountyProposed: {
      readonly index: u32
    } & Struct
    readonly isBountyRejected: boolean
    readonly asBountyRejected: {
      readonly index: u32
      readonly bond: u128
    } & Struct
    readonly isBountyBecameActive: boolean
    readonly asBountyBecameActive: {
      readonly index: u32
    } & Struct
    readonly isBountyAwarded: boolean
    readonly asBountyAwarded: {
      readonly index: u32
      readonly beneficiary: AccountId32
    } & Struct
    readonly isBountyClaimed: boolean
    readonly asBountyClaimed: {
      readonly index: u32
      readonly payout: u128
      readonly beneficiary: AccountId32
    } & Struct
    readonly isBountyCanceled: boolean
    readonly asBountyCanceled: {
      readonly index: u32
    } & Struct
    readonly isBountyExtended: boolean
    readonly asBountyExtended: {
      readonly index: u32
    } & Struct
    readonly isBountyApproved: boolean
    readonly asBountyApproved: {
      readonly index: u32
    } & Struct
    readonly isCuratorProposed: boolean
    readonly asCuratorProposed: {
      readonly bountyId: u32
      readonly curator: AccountId32
    } & Struct
    readonly isCuratorUnassigned: boolean
    readonly asCuratorUnassigned: {
      readonly bountyId: u32
    } & Struct
    readonly isCuratorAccepted: boolean
    readonly asCuratorAccepted: {
      readonly bountyId: u32
      readonly curator: AccountId32
    } & Struct
    readonly type:
      | 'BountyProposed'
      | 'BountyRejected'
      | 'BountyBecameActive'
      | 'BountyAwarded'
      | 'BountyClaimed'
      | 'BountyCanceled'
      | 'BountyExtended'
      | 'BountyApproved'
      | 'CuratorProposed'
      | 'CuratorUnassigned'
      | 'CuratorAccepted'
  }

  /** @name PalletChildBountiesEvent (68) */
  interface PalletChildBountiesEvent extends Enum {
    readonly isAdded: boolean
    readonly asAdded: {
      readonly index: u32
      readonly childIndex: u32
    } & Struct
    readonly isAwarded: boolean
    readonly asAwarded: {
      readonly index: u32
      readonly childIndex: u32
      readonly beneficiary: AccountId32
    } & Struct
    readonly isClaimed: boolean
    readonly asClaimed: {
      readonly index: u32
      readonly childIndex: u32
      readonly payout: u128
      readonly beneficiary: AccountId32
    } & Struct
    readonly isCanceled: boolean
    readonly asCanceled: {
      readonly index: u32
      readonly childIndex: u32
    } & Struct
    readonly type: 'Added' | 'Awarded' | 'Claimed' | 'Canceled'
  }

  /** @name PalletBagsListEvent (69) */
  interface PalletBagsListEvent extends Enum {
    readonly isRebagged: boolean
    readonly asRebagged: {
      readonly who: AccountId32
      readonly from: u64
      readonly to: u64
    } & Struct
    readonly isScoreUpdated: boolean
    readonly asScoreUpdated: {
      readonly who: AccountId32
      readonly newScore: u64
    } & Struct
    readonly type: 'Rebagged' | 'ScoreUpdated'
  }

  /** @name PalletNominationPoolsEvent (70) */
  interface PalletNominationPoolsEvent extends Enum {
    readonly isCreated: boolean
    readonly asCreated: {
      readonly depositor: AccountId32
      readonly poolId: u32
    } & Struct
    readonly isBonded: boolean
    readonly asBonded: {
      readonly member: AccountId32
      readonly poolId: u32
      readonly bonded: u128
      readonly joined: bool
    } & Struct
    readonly isPaidOut: boolean
    readonly asPaidOut: {
      readonly member: AccountId32
      readonly poolId: u32
      readonly payout: u128
    } & Struct
    readonly isUnbonded: boolean
    readonly asUnbonded: {
      readonly member: AccountId32
      readonly poolId: u32
      readonly balance: u128
      readonly points: u128
      readonly era: u32
    } & Struct
    readonly isWithdrawn: boolean
    readonly asWithdrawn: {
      readonly member: AccountId32
      readonly poolId: u32
      readonly balance: u128
      readonly points: u128
    } & Struct
    readonly isDestroyed: boolean
    readonly asDestroyed: {
      readonly poolId: u32
    } & Struct
    readonly isStateChanged: boolean
    readonly asStateChanged: {
      readonly poolId: u32
      readonly newState: PalletNominationPoolsPoolState
    } & Struct
    readonly isMemberRemoved: boolean
    readonly asMemberRemoved: {
      readonly poolId: u32
      readonly member: AccountId32
    } & Struct
    readonly isRolesUpdated: boolean
    readonly asRolesUpdated: {
      readonly root: Option<AccountId32>
      readonly bouncer: Option<AccountId32>
      readonly nominator: Option<AccountId32>
    } & Struct
    readonly isPoolSlashed: boolean
    readonly asPoolSlashed: {
      readonly poolId: u32
      readonly balance: u128
    } & Struct
    readonly isUnbondingPoolSlashed: boolean
    readonly asUnbondingPoolSlashed: {
      readonly poolId: u32
      readonly era: u32
      readonly balance: u128
    } & Struct
    readonly isPoolCommissionUpdated: boolean
    readonly asPoolCommissionUpdated: {
      readonly poolId: u32
      readonly current: Option<ITuple<[Perbill, AccountId32]>>
    } & Struct
    readonly isPoolMaxCommissionUpdated: boolean
    readonly asPoolMaxCommissionUpdated: {
      readonly poolId: u32
      readonly maxCommission: Perbill
    } & Struct
    readonly isPoolCommissionChangeRateUpdated: boolean
    readonly asPoolCommissionChangeRateUpdated: {
      readonly poolId: u32
      readonly changeRate: PalletNominationPoolsCommissionChangeRate
    } & Struct
    readonly isPoolCommissionClaimPermissionUpdated: boolean
    readonly asPoolCommissionClaimPermissionUpdated: {
      readonly poolId: u32
      readonly permission: Option<PalletNominationPoolsCommissionClaimPermission>
    } & Struct
    readonly isPoolCommissionClaimed: boolean
    readonly asPoolCommissionClaimed: {
      readonly poolId: u32
      readonly commission: u128
    } & Struct
    readonly isMinBalanceDeficitAdjusted: boolean
    readonly asMinBalanceDeficitAdjusted: {
      readonly poolId: u32
      readonly amount: u128
    } & Struct
    readonly isMinBalanceExcessAdjusted: boolean
    readonly asMinBalanceExcessAdjusted: {
      readonly poolId: u32
      readonly amount: u128
    } & Struct
    readonly type:
      | 'Created'
      | 'Bonded'
      | 'PaidOut'
      | 'Unbonded'
      | 'Withdrawn'
      | 'Destroyed'
      | 'StateChanged'
      | 'MemberRemoved'
      | 'RolesUpdated'
      | 'PoolSlashed'
      | 'UnbondingPoolSlashed'
      | 'PoolCommissionUpdated'
      | 'PoolMaxCommissionUpdated'
      | 'PoolCommissionChangeRateUpdated'
      | 'PoolCommissionClaimPermissionUpdated'
      | 'PoolCommissionClaimed'
      | 'MinBalanceDeficitAdjusted'
      | 'MinBalanceExcessAdjusted'
  }

  /** @name PalletNominationPoolsPoolState (71) */
  interface PalletNominationPoolsPoolState extends Enum {
    readonly isOpen: boolean
    readonly isBlocked: boolean
    readonly isDestroying: boolean
    readonly type: 'Open' | 'Blocked' | 'Destroying'
  }

  /** @name PalletNominationPoolsCommissionChangeRate (74) */
  interface PalletNominationPoolsCommissionChangeRate extends Struct {
    readonly maxIncrease: Perbill
    readonly minDelay: u64
  }

  /** @name PalletNominationPoolsCommissionClaimPermission (76) */
  interface PalletNominationPoolsCommissionClaimPermission extends Enum {
    readonly isPermissionless: boolean
    readonly isAccount: boolean
    readonly asAccount: AccountId32
    readonly type: 'Permissionless' | 'Account'
  }

  /** @name PalletSchedulerEvent (77) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean
    readonly asScheduled: {
      readonly when: u64
      readonly index: u32
    } & Struct
    readonly isCanceled: boolean
    readonly asCanceled: {
      readonly when: u64
      readonly index: u32
    } & Struct
    readonly isDispatched: boolean
    readonly asDispatched: {
      readonly task: ITuple<[u64, u32]>
      readonly id: Option<U8aFixed>
      readonly result: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly isCallUnavailable: boolean
    readonly asCallUnavailable: {
      readonly task: ITuple<[u64, u32]>
      readonly id: Option<U8aFixed>
    } & Struct
    readonly isPeriodicFailed: boolean
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u64, u32]>
      readonly id: Option<U8aFixed>
    } & Struct
    readonly isPermanentlyOverweight: boolean
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u64, u32]>
      readonly id: Option<U8aFixed>
    } & Struct
    readonly type:
      | 'Scheduled'
      | 'Canceled'
      | 'Dispatched'
      | 'CallUnavailable'
      | 'PeriodicFailed'
      | 'PermanentlyOverweight'
  }

  /** @name PalletPreimageEvent (80) */
  interface PalletPreimageEvent extends Enum {
    readonly isNoted: boolean
    readonly asNoted: {
      readonly hash_: H256
    } & Struct
    readonly isRequested: boolean
    readonly asRequested: {
      readonly hash_: H256
    } & Struct
    readonly isCleared: boolean
    readonly asCleared: {
      readonly hash_: H256
    } & Struct
    readonly type: 'Noted' | 'Requested' | 'Cleared'
  }

  /** @name PalletOffencesEvent (81) */
  interface PalletOffencesEvent extends Enum {
    readonly isOffence: boolean
    readonly asOffence: {
      readonly kind: U8aFixed
      readonly timeslot: Bytes
    } & Struct
    readonly type: 'Offence'
  }

  /** @name PalletTransactionPauseModuleEvent (83) */
  interface PalletTransactionPauseModuleEvent extends Enum {
    readonly isTransactionPaused: boolean
    readonly asTransactionPaused: {
      readonly palletNameBytes: Bytes
      readonly functionNameBytes: Bytes
    } & Struct
    readonly isTransactionUnpaused: boolean
    readonly asTransactionUnpaused: {
      readonly palletNameBytes: Bytes
      readonly functionNameBytes: Bytes
    } & Struct
    readonly type: 'TransactionPaused' | 'TransactionUnpaused'
  }

  /** @name PalletImOnlineEvent (84) */
  interface PalletImOnlineEvent extends Enum {
    readonly isHeartbeatReceived: boolean
    readonly asHeartbeatReceived: {
      readonly authorityId: PalletImOnlineSr25519AppSr25519Public
    } & Struct
    readonly isAllGood: boolean
    readonly isSomeOffline: boolean
    readonly asSomeOffline: {
      readonly offline: Vec<ITuple<[AccountId32, SpStakingExposure]>>
    } & Struct
    readonly type: 'HeartbeatReceived' | 'AllGood' | 'SomeOffline'
  }

  /** @name PalletImOnlineSr25519AppSr25519Public (85) */
  interface PalletImOnlineSr25519AppSr25519Public extends SpCoreSr25519Public {}

  /** @name SpCoreSr25519Public (86) */
  interface SpCoreSr25519Public extends U8aFixed {}

  /** @name SpStakingExposure (89) */
  interface SpStakingExposure extends Struct {
    readonly total: Compact<u128>
    readonly own: Compact<u128>
    readonly others: Vec<SpStakingIndividualExposure>
  }

  /** @name SpStakingIndividualExposure (92) */
  interface SpStakingIndividualExposure extends Struct {
    readonly who: AccountId32
    readonly value: Compact<u128>
  }

  /** @name PalletIdentityEvent (93) */
  interface PalletIdentityEvent extends Enum {
    readonly isIdentitySet: boolean
    readonly asIdentitySet: {
      readonly who: AccountId32
    } & Struct
    readonly isIdentityCleared: boolean
    readonly asIdentityCleared: {
      readonly who: AccountId32
      readonly deposit: u128
    } & Struct
    readonly isIdentityKilled: boolean
    readonly asIdentityKilled: {
      readonly who: AccountId32
      readonly deposit: u128
    } & Struct
    readonly isJudgementRequested: boolean
    readonly asJudgementRequested: {
      readonly who: AccountId32
      readonly registrarIndex: u32
    } & Struct
    readonly isJudgementUnrequested: boolean
    readonly asJudgementUnrequested: {
      readonly who: AccountId32
      readonly registrarIndex: u32
    } & Struct
    readonly isJudgementGiven: boolean
    readonly asJudgementGiven: {
      readonly target: AccountId32
      readonly registrarIndex: u32
    } & Struct
    readonly isRegistrarAdded: boolean
    readonly asRegistrarAdded: {
      readonly registrarIndex: u32
    } & Struct
    readonly isSubIdentityAdded: boolean
    readonly asSubIdentityAdded: {
      readonly sub: AccountId32
      readonly main: AccountId32
      readonly deposit: u128
    } & Struct
    readonly isSubIdentityRemoved: boolean
    readonly asSubIdentityRemoved: {
      readonly sub: AccountId32
      readonly main: AccountId32
      readonly deposit: u128
    } & Struct
    readonly isSubIdentityRevoked: boolean
    readonly asSubIdentityRevoked: {
      readonly sub: AccountId32
      readonly main: AccountId32
      readonly deposit: u128
    } & Struct
    readonly isAuthorityAdded: boolean
    readonly asAuthorityAdded: {
      readonly authority: AccountId32
    } & Struct
    readonly isAuthorityRemoved: boolean
    readonly asAuthorityRemoved: {
      readonly authority: AccountId32
    } & Struct
    readonly isUsernameSet: boolean
    readonly asUsernameSet: {
      readonly who: AccountId32
      readonly username: Bytes
    } & Struct
    readonly isUsernameQueued: boolean
    readonly asUsernameQueued: {
      readonly who: AccountId32
      readonly username: Bytes
      readonly expiration: u64
    } & Struct
    readonly isPreapprovalExpired: boolean
    readonly asPreapprovalExpired: {
      readonly whose: AccountId32
    } & Struct
    readonly isPrimaryUsernameSet: boolean
    readonly asPrimaryUsernameSet: {
      readonly who: AccountId32
      readonly username: Bytes
    } & Struct
    readonly isDanglingUsernameRemoved: boolean
    readonly asDanglingUsernameRemoved: {
      readonly who: AccountId32
      readonly username: Bytes
    } & Struct
    readonly type:
      | 'IdentitySet'
      | 'IdentityCleared'
      | 'IdentityKilled'
      | 'JudgementRequested'
      | 'JudgementUnrequested'
      | 'JudgementGiven'
      | 'RegistrarAdded'
      | 'SubIdentityAdded'
      | 'SubIdentityRemoved'
      | 'SubIdentityRevoked'
      | 'AuthorityAdded'
      | 'AuthorityRemoved'
      | 'UsernameSet'
      | 'UsernameQueued'
      | 'PreapprovalExpired'
      | 'PrimaryUsernameSet'
      | 'DanglingUsernameRemoved'
  }

  /** @name PalletUtilityEvent (95) */
  interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean
    readonly asBatchInterrupted: {
      readonly index: u32
      readonly error: SpRuntimeDispatchError
    } & Struct
    readonly isBatchCompleted: boolean
    readonly isBatchCompletedWithErrors: boolean
    readonly isItemCompleted: boolean
    readonly isItemFailed: boolean
    readonly asItemFailed: {
      readonly error: SpRuntimeDispatchError
    } & Struct
    readonly isDispatchedAs: boolean
    readonly asDispatchedAs: {
      readonly result: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly type:
      | 'BatchInterrupted'
      | 'BatchCompleted'
      | 'BatchCompletedWithErrors'
      | 'ItemCompleted'
      | 'ItemFailed'
      | 'DispatchedAs'
  }

  /** @name PalletMultisigEvent (96) */
  interface PalletMultisigEvent extends Enum {
    readonly isNewMultisig: boolean
    readonly asNewMultisig: {
      readonly approving: AccountId32
      readonly multisig: AccountId32
      readonly callHash: U8aFixed
    } & Struct
    readonly isMultisigApproval: boolean
    readonly asMultisigApproval: {
      readonly approving: AccountId32
      readonly timepoint: PalletMultisigTimepoint
      readonly multisig: AccountId32
      readonly callHash: U8aFixed
    } & Struct
    readonly isMultisigExecuted: boolean
    readonly asMultisigExecuted: {
      readonly approving: AccountId32
      readonly timepoint: PalletMultisigTimepoint
      readonly multisig: AccountId32
      readonly callHash: U8aFixed
      readonly result: Result<Null, SpRuntimeDispatchError>
    } & Struct
    readonly isMultisigCancelled: boolean
    readonly asMultisigCancelled: {
      readonly cancelling: AccountId32
      readonly timepoint: PalletMultisigTimepoint
      readonly multisig: AccountId32
      readonly callHash: U8aFixed
    } & Struct
    readonly type:
      | 'NewMultisig'
      | 'MultisigApproval'
      | 'MultisigExecuted'
      | 'MultisigCancelled'
  }

  /** @name PalletMultisigTimepoint (97) */
  interface PalletMultisigTimepoint extends Struct {
    readonly height: u64
    readonly index: u32
  }

  /** @name PalletEthereumEvent (98) */
  interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean
    readonly asExecuted: {
      readonly from: H160
      readonly to: H160
      readonly transactionHash: H256
      readonly exitReason: EvmCoreErrorExitReason
      readonly extraData: Bytes
    } & Struct
    readonly type: 'Executed'
  }

  /** @name EvmCoreErrorExitReason (101) */
  interface EvmCoreErrorExitReason extends Enum {
    readonly isSucceed: boolean
    readonly asSucceed: EvmCoreErrorExitSucceed
    readonly isError: boolean
    readonly asError: EvmCoreErrorExitError
    readonly isRevert: boolean
    readonly asRevert: EvmCoreErrorExitRevert
    readonly isFatal: boolean
    readonly asFatal: EvmCoreErrorExitFatal
    readonly type: 'Succeed' | 'Error' | 'Revert' | 'Fatal'
  }

  /** @name EvmCoreErrorExitSucceed (102) */
  interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean
    readonly isReturned: boolean
    readonly isSuicided: boolean
    readonly type: 'Stopped' | 'Returned' | 'Suicided'
  }

  /** @name EvmCoreErrorExitError (103) */
  interface EvmCoreErrorExitError extends Enum {
    readonly isStackUnderflow: boolean
    readonly isStackOverflow: boolean
    readonly isInvalidJump: boolean
    readonly isInvalidRange: boolean
    readonly isDesignatedInvalid: boolean
    readonly isCallTooDeep: boolean
    readonly isCreateCollision: boolean
    readonly isCreateContractLimit: boolean
    readonly isOutOfOffset: boolean
    readonly isOutOfGas: boolean
    readonly isOutOfFund: boolean
    readonly isPcUnderflow: boolean
    readonly isCreateEmpty: boolean
    readonly isOther: boolean
    readonly asOther: Text
    readonly isMaxNonce: boolean
    readonly isInvalidCode: boolean
    readonly asInvalidCode: u8
    readonly type:
      | 'StackUnderflow'
      | 'StackOverflow'
      | 'InvalidJump'
      | 'InvalidRange'
      | 'DesignatedInvalid'
      | 'CallTooDeep'
      | 'CreateCollision'
      | 'CreateContractLimit'
      | 'OutOfOffset'
      | 'OutOfGas'
      | 'OutOfFund'
      | 'PcUnderflow'
      | 'CreateEmpty'
      | 'Other'
      | 'MaxNonce'
      | 'InvalidCode'
  }

  /** @name EvmCoreErrorExitRevert (107) */
  interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean
    readonly type: 'Reverted'
  }

  /** @name EvmCoreErrorExitFatal (108) */
  interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean
    readonly isUnhandledInterrupt: boolean
    readonly isCallErrorAsFatal: boolean
    readonly asCallErrorAsFatal: EvmCoreErrorExitError
    readonly isOther: boolean
    readonly asOther: Text
    readonly type:
      | 'NotSupported'
      | 'UnhandledInterrupt'
      | 'CallErrorAsFatal'
      | 'Other'
  }

  /** @name PalletEvmEvent (109) */
  interface PalletEvmEvent extends Enum {
    readonly isLog: boolean
    readonly asLog: {
      readonly log: EthereumLog
    } & Struct
    readonly isCreated: boolean
    readonly asCreated: {
      readonly address: H160
    } & Struct
    readonly isCreatedFailed: boolean
    readonly asCreatedFailed: {
      readonly address: H160
    } & Struct
    readonly isExecuted: boolean
    readonly asExecuted: {
      readonly address: H160
    } & Struct
    readonly isExecutedFailed: boolean
    readonly asExecutedFailed: {
      readonly address: H160
    } & Struct
    readonly type:
      | 'Log'
      | 'Created'
      | 'CreatedFailed'
      | 'Executed'
      | 'ExecutedFailed'
  }

  /** @name EthereumLog (110) */
  interface EthereumLog extends Struct {
    readonly address: H160
    readonly topics: Vec<H256>
    readonly data: Bytes
  }

  /** @name PalletBaseFeeEvent (112) */
  interface PalletBaseFeeEvent extends Enum {
    readonly isNewBaseFeePerGas: boolean
    readonly asNewBaseFeePerGas: {
      readonly fee: U256
    } & Struct
    readonly isBaseFeeOverflow: boolean
    readonly isNewElasticity: boolean
    readonly asNewElasticity: {
      readonly elasticity: Permill
    } & Struct
    readonly type: 'NewBaseFeePerGas' | 'BaseFeeOverflow' | 'NewElasticity'
  }

  /** @name PalletAirdropClaimsEvent (116) */
  interface PalletAirdropClaimsEvent extends Enum {
    readonly isClaimed: boolean
    readonly asClaimed: {
      readonly recipient: AccountId32
      readonly source: PalletAirdropClaimsUtilsMultiAddress
      readonly amount: u128
    } & Struct
    readonly type: 'Claimed'
  }

  /** @name PalletAirdropClaimsUtilsMultiAddress (117) */
  interface PalletAirdropClaimsUtilsMultiAddress extends Enum {
    readonly isEvm: boolean
    readonly asEvm: PalletAirdropClaimsUtilsEthereumAddress
    readonly isNative: boolean
    readonly asNative: AccountId32
    readonly type: 'Evm' | 'Native'
  }

  /** @name PalletAirdropClaimsUtilsEthereumAddress (118) */
  interface PalletAirdropClaimsUtilsEthereumAddress extends U8aFixed {}

  /** @name PalletRolesEvent (119) */
  interface PalletRolesEvent extends Enum {
    readonly isRoleAssigned: boolean
    readonly asRoleAssigned: {
      readonly account: AccountId32
      readonly role: TanglePrimitivesRolesRoleType
    } & Struct
    readonly isRoleRemoved: boolean
    readonly asRoleRemoved: {
      readonly account: AccountId32
      readonly role: TanglePrimitivesRolesRoleType
    } & Struct
    readonly isSlashed: boolean
    readonly asSlashed: {
      readonly account: AccountId32
      readonly amount: u128
    } & Struct
    readonly isProfileCreated: boolean
    readonly asProfileCreated: {
      readonly account: AccountId32
      readonly totalProfileRestake: u128
      readonly roles: Vec<TanglePrimitivesRolesRoleType>
    } & Struct
    readonly isProfileUpdated: boolean
    readonly asProfileUpdated: {
      readonly account: AccountId32
      readonly totalProfileRestake: u128
      readonly roles: Vec<TanglePrimitivesRolesRoleType>
    } & Struct
    readonly isProfileDeleted: boolean
    readonly asProfileDeleted: {
      readonly account: AccountId32
    } & Struct
    readonly isPendingJobs: boolean
    readonly asPendingJobs: {
      readonly pendingJobs: Vec<ITuple<[TanglePrimitivesRolesRoleType, u64]>>
    } & Struct
    readonly isRolesRewardSet: boolean
    readonly asRolesRewardSet: {
      readonly totalRewards: u128
    } & Struct
    readonly isPayoutStarted: boolean
    readonly asPayoutStarted: {
      readonly eraIndex: u32
      readonly validatorStash: AccountId32
    } & Struct
    readonly isRewarded: boolean
    readonly asRewarded: {
      readonly stash: AccountId32
      readonly amount: u128
    } & Struct
    readonly isMinRestakingBondUpdated: boolean
    readonly asMinRestakingBondUpdated: {
      readonly value: u128
    } & Struct
    readonly type:
      | 'RoleAssigned'
      | 'RoleRemoved'
      | 'Slashed'
      | 'ProfileCreated'
      | 'ProfileUpdated'
      | 'ProfileDeleted'
      | 'PendingJobs'
      | 'RolesRewardSet'
      | 'PayoutStarted'
      | 'Rewarded'
      | 'MinRestakingBondUpdated'
  }

  /** @name TanglePrimitivesRolesRoleType (120) */
  interface TanglePrimitivesRolesRoleType extends Enum {
    readonly isTss: boolean
    readonly asTss: TanglePrimitivesRolesTssThresholdSignatureRoleType
    readonly isZkSaaS: boolean
    readonly asZkSaaS: TanglePrimitivesRolesZksaasZeroKnowledgeRoleType
    readonly isLightClientRelaying: boolean
    readonly type: 'Tss' | 'ZkSaaS' | 'LightClientRelaying'
  }

  /** @name TanglePrimitivesRolesTssThresholdSignatureRoleType (121) */
  interface TanglePrimitivesRolesTssThresholdSignatureRoleType extends Enum {
    readonly isZengoGG20Secp256k1: boolean
    readonly isDfnsCGGMP21Secp256k1: boolean
    readonly isDfnsCGGMP21Secp256r1: boolean
    readonly isDfnsCGGMP21Stark: boolean
    readonly isZcashFrostP256: boolean
    readonly isZcashFrostP384: boolean
    readonly isZcashFrostSecp256k1: boolean
    readonly isZcashFrostRistretto255: boolean
    readonly isZcashFrostEd25519: boolean
    readonly isZcashFrostEd448: boolean
    readonly isGennaroDKGBls381: boolean
    readonly type:
      | 'ZengoGG20Secp256k1'
      | 'DfnsCGGMP21Secp256k1'
      | 'DfnsCGGMP21Secp256r1'
      | 'DfnsCGGMP21Stark'
      | 'ZcashFrostP256'
      | 'ZcashFrostP384'
      | 'ZcashFrostSecp256k1'
      | 'ZcashFrostRistretto255'
      | 'ZcashFrostEd25519'
      | 'ZcashFrostEd448'
      | 'GennaroDKGBls381'
  }

  /** @name TanglePrimitivesRolesZksaasZeroKnowledgeRoleType (122) */
  interface TanglePrimitivesRolesZksaasZeroKnowledgeRoleType extends Enum {
    readonly isZkSaaSGroth16: boolean
    readonly isZkSaaSMarlin: boolean
    readonly type: 'ZkSaaSGroth16' | 'ZkSaaSMarlin'
  }

  /** @name PalletJobsModuleEvent (126) */
  interface PalletJobsModuleEvent extends Enum {
    readonly isJobSubmitted: boolean
    readonly asJobSubmitted: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly details: TanglePrimitivesJobsJobSubmission
    } & Struct
    readonly isJobResultSubmitted: boolean
    readonly asJobResultSubmitted: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
    } & Struct
    readonly isValidatorRewarded: boolean
    readonly asValidatorRewarded: {
      readonly id: AccountId32
      readonly reward: u128
    } & Struct
    readonly isJobRefunded: boolean
    readonly asJobRefunded: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
    } & Struct
    readonly isJobParticipantsUpdated: boolean
    readonly asJobParticipantsUpdated: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly details: TanglePrimitivesJobsJobInfo
    } & Struct
    readonly isJobReSubmitted: boolean
    readonly asJobReSubmitted: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly details: TanglePrimitivesJobsJobInfo
    } & Struct
    readonly isJobResultExtended: boolean
    readonly asJobResultExtended: {
      readonly jobId: u64
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly newExpiry: u64
    } & Struct
    readonly type:
      | 'JobSubmitted'
      | 'JobResultSubmitted'
      | 'ValidatorRewarded'
      | 'JobRefunded'
      | 'JobParticipantsUpdated'
      | 'JobReSubmitted'
      | 'JobResultExtended'
  }

  /** @name TanglePrimitivesJobsJobSubmission (127) */
  interface TanglePrimitivesJobsJobSubmission extends Struct {
    readonly expiry: u64
    readonly ttl: u64
    readonly jobType: TanglePrimitivesJobsJobType
    readonly fallback: TanglePrimitivesJobsFallbackOptions
  }

  /** @name TangleTestnetRuntimeMaxParticipants (128) */
  type TangleTestnetRuntimeMaxParticipants = Null

  /** @name TangleTestnetRuntimeMaxSubmissionLen (129) */
  type TangleTestnetRuntimeMaxSubmissionLen = Null

  /** @name TanglePrimitivesJobsJobType (130) */
  interface TanglePrimitivesJobsJobType extends Enum {
    readonly isDkgtssPhaseOne: boolean
    readonly asDkgtssPhaseOne: TanglePrimitivesJobsTssDkgtssPhaseOneJobType
    readonly isDkgtssPhaseTwo: boolean
    readonly asDkgtssPhaseTwo: TanglePrimitivesJobsTssDkgtssPhaseTwoJobType
    readonly isDkgtssPhaseThree: boolean
    readonly asDkgtssPhaseThree: TanglePrimitivesJobsTssDkgtssPhaseThreeJobType
    readonly isDkgtssPhaseFour: boolean
    readonly asDkgtssPhaseFour: TanglePrimitivesJobsTssDkgtssPhaseFourJobType
    readonly isZkSaaSPhaseOne: boolean
    readonly asZkSaaSPhaseOne: TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType
    readonly isZkSaaSPhaseTwo: boolean
    readonly asZkSaaSPhaseTwo: TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType
    readonly type:
      | 'DkgtssPhaseOne'
      | 'DkgtssPhaseTwo'
      | 'DkgtssPhaseThree'
      | 'DkgtssPhaseFour'
      | 'ZkSaaSPhaseOne'
      | 'ZkSaaSPhaseTwo'
  }

  /** @name TanglePrimitivesJobsTssDkgtssPhaseOneJobType (131) */
  interface TanglePrimitivesJobsTssDkgtssPhaseOneJobType extends Struct {
    readonly participants: Vec<AccountId32>
    readonly threshold: u8
    readonly permittedCaller: Option<AccountId32>
    readonly roleType: TanglePrimitivesRolesTssThresholdSignatureRoleType
  }

  /** @name TanglePrimitivesJobsTssDkgtssPhaseTwoJobType (134) */
  interface TanglePrimitivesJobsTssDkgtssPhaseTwoJobType extends Struct {
    readonly phaseOneId: u64
    readonly submission: Bytes
    readonly roleType: TanglePrimitivesRolesTssThresholdSignatureRoleType
  }

  /** @name TanglePrimitivesJobsTssDkgtssPhaseThreeJobType (136) */
  interface TanglePrimitivesJobsTssDkgtssPhaseThreeJobType extends Struct {
    readonly phaseOneId: u64
    readonly roleType: TanglePrimitivesRolesTssThresholdSignatureRoleType
  }

  /** @name TanglePrimitivesJobsTssDkgtssPhaseFourJobType (137) */
  interface TanglePrimitivesJobsTssDkgtssPhaseFourJobType extends Struct {
    readonly phaseOneId: u64
    readonly newPhaseOneId: u64
    readonly roleType: TanglePrimitivesRolesTssThresholdSignatureRoleType
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType (138) */
  interface TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType extends Struct {
    readonly participants: Vec<AccountId32>
    readonly permittedCaller: Option<AccountId32>
    readonly system: TanglePrimitivesJobsZksaasZkSaaSSystem
    readonly roleType: TanglePrimitivesRolesZksaasZeroKnowledgeRoleType
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSSystem (139) */
  interface TanglePrimitivesJobsZksaasZkSaaSSystem extends Enum {
    readonly isGroth16: boolean
    readonly asGroth16: TanglePrimitivesJobsZksaasGroth16System
    readonly type: 'Groth16'
  }

  /** @name TanglePrimitivesJobsZksaasGroth16System (140) */
  interface TanglePrimitivesJobsZksaasGroth16System extends Struct {
    readonly circuit: TanglePrimitivesJobsZksaasHyperData
    readonly numInputs: u64
    readonly numConstraints: u64
    readonly provingKey: TanglePrimitivesJobsZksaasHyperData
    readonly verifyingKey: Bytes
    readonly wasm: TanglePrimitivesJobsZksaasHyperData
  }

  /** @name TanglePrimitivesJobsZksaasHyperData (141) */
  interface TanglePrimitivesJobsZksaasHyperData extends Enum {
    readonly isRaw: boolean
    readonly asRaw: Bytes
    readonly isIpfs: boolean
    readonly asIpfs: Bytes
    readonly isHttp: boolean
    readonly asHttp: Bytes
    readonly type: 'Raw' | 'Ipfs' | 'Http'
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType (142) */
  interface TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType extends Struct {
    readonly phaseOneId: u64
    readonly request: TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest
    readonly roleType: TanglePrimitivesRolesZksaasZeroKnowledgeRoleType
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest (143) */
  interface TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest extends Enum {
    readonly isGroth16: boolean
    readonly asGroth16: TanglePrimitivesJobsZksaasGroth16ProveRequest
    readonly type: 'Groth16'
  }

  /** @name TanglePrimitivesJobsZksaasGroth16ProveRequest (144) */
  interface TanglePrimitivesJobsZksaasGroth16ProveRequest extends Struct {
    readonly publicInput: Bytes
    readonly aShares: Vec<TanglePrimitivesJobsZksaasHyperData>
    readonly axShares: Vec<TanglePrimitivesJobsZksaasHyperData>
    readonly qapShares: Vec<TanglePrimitivesJobsZksaasQapShare>
  }

  /** @name TanglePrimitivesJobsZksaasQapShare (148) */
  interface TanglePrimitivesJobsZksaasQapShare extends Struct {
    readonly a: TanglePrimitivesJobsZksaasHyperData
    readonly b: TanglePrimitivesJobsZksaasHyperData
    readonly c: TanglePrimitivesJobsZksaasHyperData
  }

  /** @name TanglePrimitivesJobsFallbackOptions (150) */
  interface TanglePrimitivesJobsFallbackOptions extends Enum {
    readonly isDestroy: boolean
    readonly isRegenerateWithThreshold: boolean
    readonly asRegenerateWithThreshold: u8
    readonly type: 'Destroy' | 'RegenerateWithThreshold'
  }

  /** @name TanglePrimitivesJobsJobInfo (151) */
  interface TanglePrimitivesJobsJobInfo extends Struct {
    readonly owner: AccountId32
    readonly expiry: u64
    readonly ttl: u64
    readonly jobType: TanglePrimitivesJobsJobType
    readonly fee: u128
    readonly fallback: TanglePrimitivesJobsFallbackOptions
  }

  /** @name PalletDkgEvent (152) */
  interface PalletDkgEvent extends Enum {
    readonly isFeeUpdated: boolean
    readonly asFeeUpdated: PalletDkgFeeInfo
    readonly isKeyRotated: boolean
    readonly asKeyRotated: {
      readonly fromJobId: u64
      readonly toJobId: u64
      readonly signature: Bytes
    } & Struct
    readonly type: 'FeeUpdated' | 'KeyRotated'
  }

  /** @name PalletDkgFeeInfo (153) */
  interface PalletDkgFeeInfo extends Struct {
    readonly baseFee: u128
    readonly dkgValidatorFee: u128
    readonly sigValidatorFee: u128
    readonly refreshValidatorFee: u128
    readonly storageFeePerByte: u128
    readonly storageFeePerBlock: u128
  }

  /** @name PalletZksaasEvent (154) */
  interface PalletZksaasEvent extends Enum {
    readonly isFeeUpdated: boolean
    readonly asFeeUpdated: PalletZksaasFeeInfo
    readonly type: 'FeeUpdated'
  }

  /** @name PalletZksaasFeeInfo (155) */
  interface PalletZksaasFeeInfo extends Struct {
    readonly baseFee: u128
    readonly circuitFee: u128
    readonly proveFee: u128
    readonly storageFeePerByte: u128
  }

  /** @name FrameSystemPhase (156) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean
    readonly asApplyExtrinsic: u32
    readonly isFinalization: boolean
    readonly isInitialization: boolean
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization'
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (158) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>
    readonly specName: Text
  }

  /** @name FrameSystemCodeUpgradeAuthorization (160) */
  interface FrameSystemCodeUpgradeAuthorization extends Struct {
    readonly codeHash: H256
    readonly checkVersion: bool
  }

  /** @name FrameSystemCall (161) */
  interface FrameSystemCall extends Enum {
    readonly isRemark: boolean
    readonly asRemark: {
      readonly remark: Bytes
    } & Struct
    readonly isSetHeapPages: boolean
    readonly asSetHeapPages: {
      readonly pages: u64
    } & Struct
    readonly isSetCode: boolean
    readonly asSetCode: {
      readonly code: Bytes
    } & Struct
    readonly isSetCodeWithoutChecks: boolean
    readonly asSetCodeWithoutChecks: {
      readonly code: Bytes
    } & Struct
    readonly isSetStorage: boolean
    readonly asSetStorage: {
      readonly items: Vec<ITuple<[Bytes, Bytes]>>
    } & Struct
    readonly isKillStorage: boolean
    readonly asKillStorage: {
      readonly keys_: Vec<Bytes>
    } & Struct
    readonly isKillPrefix: boolean
    readonly asKillPrefix: {
      readonly prefix: Bytes
      readonly subkeys: u32
    } & Struct
    readonly isRemarkWithEvent: boolean
    readonly asRemarkWithEvent: {
      readonly remark: Bytes
    } & Struct
    readonly isAuthorizeUpgrade: boolean
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256
    } & Struct
    readonly isAuthorizeUpgradeWithoutChecks: boolean
    readonly asAuthorizeUpgradeWithoutChecks: {
      readonly codeHash: H256
    } & Struct
    readonly isApplyAuthorizedUpgrade: boolean
    readonly asApplyAuthorizedUpgrade: {
      readonly code: Bytes
    } & Struct
    readonly type:
      | 'Remark'
      | 'SetHeapPages'
      | 'SetCode'
      | 'SetCodeWithoutChecks'
      | 'SetStorage'
      | 'KillStorage'
      | 'KillPrefix'
      | 'RemarkWithEvent'
      | 'AuthorizeUpgrade'
      | 'AuthorizeUpgradeWithoutChecks'
      | 'ApplyAuthorizedUpgrade'
  }

  /** @name FrameSystemLimitsBlockWeights (165) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight
    readonly maxBlock: SpWeightsWeightV2Weight
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (166) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass
    readonly operational: FrameSystemLimitsWeightsPerClass
    readonly mandatory: FrameSystemLimitsWeightsPerClass
  }

  /** @name FrameSystemLimitsWeightsPerClass (167) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>
    readonly maxTotal: Option<SpWeightsWeightV2Weight>
    readonly reserved: Option<SpWeightsWeightV2Weight>
  }

  /** @name FrameSystemLimitsBlockLength (169) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (170) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32
    readonly operational: u32
    readonly mandatory: u32
  }

  /** @name SpWeightsRuntimeDbWeight (171) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64
    readonly write: u64
  }

  /** @name SpVersionRuntimeVersion (172) */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text
    readonly implName: Text
    readonly authoringVersion: u32
    readonly specVersion: u32
    readonly implVersion: u32
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>
    readonly transactionVersion: u32
    readonly stateVersion: u8
  }

  /** @name FrameSystemError (178) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean
    readonly isSpecVersionNeedsToIncrease: boolean
    readonly isFailedToExtractRuntimeVersion: boolean
    readonly isNonDefaultComposite: boolean
    readonly isNonZeroRefCount: boolean
    readonly isCallFiltered: boolean
    readonly isNothingAuthorized: boolean
    readonly isUnauthorized: boolean
    readonly type:
      | 'InvalidSpecName'
      | 'SpecVersionNeedsToIncrease'
      | 'FailedToExtractRuntimeVersion'
      | 'NonDefaultComposite'
      | 'NonZeroRefCount'
      | 'CallFiltered'
      | 'NothingAuthorized'
      | 'Unauthorized'
  }

  /** @name PalletTimestampCall (179) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean
    readonly asSet: {
      readonly now: Compact<u64>
    } & Struct
    readonly type: 'Set'
  }

  /** @name PalletSudoCall (180) */
  interface PalletSudoCall extends Enum {
    readonly isSudo: boolean
    readonly asSudo: {
      readonly call: Call
    } & Struct
    readonly isSudoUncheckedWeight: boolean
    readonly asSudoUncheckedWeight: {
      readonly call: Call
      readonly weight: SpWeightsWeightV2Weight
    } & Struct
    readonly isSetKey: boolean
    readonly asSetKey: {
      readonly new_: MultiAddress
    } & Struct
    readonly isSudoAs: boolean
    readonly asSudoAs: {
      readonly who: MultiAddress
      readonly call: Call
    } & Struct
    readonly isRemoveKey: boolean
    readonly type:
      | 'Sudo'
      | 'SudoUncheckedWeight'
      | 'SetKey'
      | 'SudoAs'
      | 'RemoveKey'
  }

  /** @name PalletBalancesCall (182) */
  interface PalletBalancesCall extends Enum {
    readonly isTransferAllowDeath: boolean
    readonly asTransferAllowDeath: {
      readonly dest: MultiAddress
      readonly value: Compact<u128>
    } & Struct
    readonly isForceTransfer: boolean
    readonly asForceTransfer: {
      readonly source: MultiAddress
      readonly dest: MultiAddress
      readonly value: Compact<u128>
    } & Struct
    readonly isTransferKeepAlive: boolean
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress
      readonly value: Compact<u128>
    } & Struct
    readonly isTransferAll: boolean
    readonly asTransferAll: {
      readonly dest: MultiAddress
      readonly keepAlive: bool
    } & Struct
    readonly isForceUnreserve: boolean
    readonly asForceUnreserve: {
      readonly who: MultiAddress
      readonly amount: u128
    } & Struct
    readonly isUpgradeAccounts: boolean
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId32>
    } & Struct
    readonly isForceSetBalance: boolean
    readonly asForceSetBalance: {
      readonly who: MultiAddress
      readonly newFree: Compact<u128>
    } & Struct
    readonly isForceAdjustTotalIssuance: boolean
    readonly asForceAdjustTotalIssuance: {
      readonly direction: PalletBalancesAdjustmentDirection
      readonly delta: Compact<u128>
    } & Struct
    readonly type:
      | 'TransferAllowDeath'
      | 'ForceTransfer'
      | 'TransferKeepAlive'
      | 'TransferAll'
      | 'ForceUnreserve'
      | 'UpgradeAccounts'
      | 'ForceSetBalance'
      | 'ForceAdjustTotalIssuance'
  }

  /** @name PalletBalancesAdjustmentDirection (184) */
  interface PalletBalancesAdjustmentDirection extends Enum {
    readonly isIncrease: boolean
    readonly isDecrease: boolean
    readonly type: 'Increase' | 'Decrease'
  }

  /** @name PalletBabeCall (185) */
  interface PalletBabeCall extends Enum {
    readonly isReportEquivocation: boolean
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof
      readonly keyOwnerProof: SpSessionMembershipProof
    } & Struct
    readonly isReportEquivocationUnsigned: boolean
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof
      readonly keyOwnerProof: SpSessionMembershipProof
    } & Struct
    readonly isPlanConfigChange: boolean
    readonly asPlanConfigChange: {
      readonly config: SpConsensusBabeDigestsNextConfigDescriptor
    } & Struct
    readonly type:
      | 'ReportEquivocation'
      | 'ReportEquivocationUnsigned'
      | 'PlanConfigChange'
  }

  /** @name SpConsensusSlotsEquivocationProof (186) */
  interface SpConsensusSlotsEquivocationProof extends Struct {
    readonly offender: SpConsensusBabeAppPublic
    readonly slot: u64
    readonly firstHeader: SpRuntimeHeader
    readonly secondHeader: SpRuntimeHeader
  }

  /** @name SpRuntimeHeader (187) */
  interface SpRuntimeHeader extends Struct {
    readonly parentHash: H256
    readonly number: Compact<u64>
    readonly stateRoot: H256
    readonly extrinsicsRoot: H256
    readonly digest: SpRuntimeDigest
  }

  /** @name SpConsensusBabeAppPublic (188) */
  interface SpConsensusBabeAppPublic extends SpCoreSr25519Public {}

  /** @name SpSessionMembershipProof (190) */
  interface SpSessionMembershipProof extends Struct {
    readonly session: u32
    readonly trieNodes: Vec<Bytes>
    readonly validatorCount: u32
  }

  /** @name SpConsensusBabeDigestsNextConfigDescriptor (191) */
  interface SpConsensusBabeDigestsNextConfigDescriptor extends Enum {
    readonly isV1: boolean
    readonly asV1: {
      readonly c: ITuple<[u64, u64]>
      readonly allowedSlots: SpConsensusBabeAllowedSlots
    } & Struct
    readonly type: 'V1'
  }

  /** @name SpConsensusBabeAllowedSlots (193) */
  interface SpConsensusBabeAllowedSlots extends Enum {
    readonly isPrimarySlots: boolean
    readonly isPrimaryAndSecondaryPlainSlots: boolean
    readonly isPrimaryAndSecondaryVRFSlots: boolean
    readonly type:
      | 'PrimarySlots'
      | 'PrimaryAndSecondaryPlainSlots'
      | 'PrimaryAndSecondaryVRFSlots'
  }

  /** @name PalletGrandpaCall (194) */
  interface PalletGrandpaCall extends Enum {
    readonly isReportEquivocation: boolean
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof
      readonly keyOwnerProof: SpCoreVoid
    } & Struct
    readonly isReportEquivocationUnsigned: boolean
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof
      readonly keyOwnerProof: SpCoreVoid
    } & Struct
    readonly isNoteStalled: boolean
    readonly asNoteStalled: {
      readonly delay: u64
      readonly bestFinalizedBlockNumber: u64
    } & Struct
    readonly type:
      | 'ReportEquivocation'
      | 'ReportEquivocationUnsigned'
      | 'NoteStalled'
  }

  /** @name SpConsensusGrandpaEquivocationProof (195) */
  interface SpConsensusGrandpaEquivocationProof extends Struct {
    readonly setId: u64
    readonly equivocation: SpConsensusGrandpaEquivocation
  }

  /** @name SpConsensusGrandpaEquivocation (196) */
  interface SpConsensusGrandpaEquivocation extends Enum {
    readonly isPrevote: boolean
    readonly asPrevote: FinalityGrandpaEquivocationPrevote
    readonly isPrecommit: boolean
    readonly asPrecommit: FinalityGrandpaEquivocationPrecommit
    readonly type: 'Prevote' | 'Precommit'
  }

  /** @name FinalityGrandpaEquivocationPrevote (197) */
  interface FinalityGrandpaEquivocationPrevote extends Struct {
    readonly roundNumber: u64
    readonly identity: SpConsensusGrandpaAppPublic
    readonly first: ITuple<
      [FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]
    >
    readonly second: ITuple<
      [FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]
    >
  }

  /** @name FinalityGrandpaPrevote (198) */
  interface FinalityGrandpaPrevote extends Struct {
    readonly targetHash: H256
    readonly targetNumber: u64
  }

  /** @name SpConsensusGrandpaAppSignature (199) */
  interface SpConsensusGrandpaAppSignature extends SpCoreEd25519Signature {}

  /** @name SpCoreEd25519Signature (200) */
  interface SpCoreEd25519Signature extends U8aFixed {}

  /** @name FinalityGrandpaEquivocationPrecommit (203) */
  interface FinalityGrandpaEquivocationPrecommit extends Struct {
    readonly roundNumber: u64
    readonly identity: SpConsensusGrandpaAppPublic
    readonly first: ITuple<
      [FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]
    >
    readonly second: ITuple<
      [FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]
    >
  }

  /** @name FinalityGrandpaPrecommit (204) */
  interface FinalityGrandpaPrecommit extends Struct {
    readonly targetHash: H256
    readonly targetNumber: u64
  }

  /** @name SpCoreVoid (206) */
  type SpCoreVoid = Null

  /** @name PalletIndicesCall (207) */
  interface PalletIndicesCall extends Enum {
    readonly isClaim: boolean
    readonly asClaim: {
      readonly index: u32
    } & Struct
    readonly isTransfer: boolean
    readonly asTransfer: {
      readonly new_: MultiAddress
      readonly index: u32
    } & Struct
    readonly isFree: boolean
    readonly asFree: {
      readonly index: u32
    } & Struct
    readonly isForceTransfer: boolean
    readonly asForceTransfer: {
      readonly new_: MultiAddress
      readonly index: u32
      readonly freeze: bool
    } & Struct
    readonly isFreeze: boolean
    readonly asFreeze: {
      readonly index: u32
    } & Struct
    readonly type: 'Claim' | 'Transfer' | 'Free' | 'ForceTransfer' | 'Freeze'
  }

  /** @name PalletDemocracyCall (208) */
  interface PalletDemocracyCall extends Enum {
    readonly isPropose: boolean
    readonly asPropose: {
      readonly proposal: FrameSupportPreimagesBounded
      readonly value: Compact<u128>
    } & Struct
    readonly isSecond: boolean
    readonly asSecond: {
      readonly proposal: Compact<u32>
    } & Struct
    readonly isVote: boolean
    readonly asVote: {
      readonly refIndex: Compact<u32>
      readonly vote: PalletDemocracyVoteAccountVote
    } & Struct
    readonly isEmergencyCancel: boolean
    readonly asEmergencyCancel: {
      readonly refIndex: u32
    } & Struct
    readonly isExternalPropose: boolean
    readonly asExternalPropose: {
      readonly proposal: FrameSupportPreimagesBounded
    } & Struct
    readonly isExternalProposeMajority: boolean
    readonly asExternalProposeMajority: {
      readonly proposal: FrameSupportPreimagesBounded
    } & Struct
    readonly isExternalProposeDefault: boolean
    readonly asExternalProposeDefault: {
      readonly proposal: FrameSupportPreimagesBounded
    } & Struct
    readonly isFastTrack: boolean
    readonly asFastTrack: {
      readonly proposalHash: H256
      readonly votingPeriod: u64
      readonly delay: u64
    } & Struct
    readonly isVetoExternal: boolean
    readonly asVetoExternal: {
      readonly proposalHash: H256
    } & Struct
    readonly isCancelReferendum: boolean
    readonly asCancelReferendum: {
      readonly refIndex: Compact<u32>
    } & Struct
    readonly isDelegate: boolean
    readonly asDelegate: {
      readonly to: MultiAddress
      readonly conviction: PalletDemocracyConviction
      readonly balance: u128
    } & Struct
    readonly isUndelegate: boolean
    readonly isClearPublicProposals: boolean
    readonly isUnlock: boolean
    readonly asUnlock: {
      readonly target: MultiAddress
    } & Struct
    readonly isRemoveVote: boolean
    readonly asRemoveVote: {
      readonly index: u32
    } & Struct
    readonly isRemoveOtherVote: boolean
    readonly asRemoveOtherVote: {
      readonly target: MultiAddress
      readonly index: u32
    } & Struct
    readonly isBlacklist: boolean
    readonly asBlacklist: {
      readonly proposalHash: H256
      readonly maybeRefIndex: Option<u32>
    } & Struct
    readonly isCancelProposal: boolean
    readonly asCancelProposal: {
      readonly propIndex: Compact<u32>
    } & Struct
    readonly isSetMetadata: boolean
    readonly asSetMetadata: {
      readonly owner: PalletDemocracyMetadataOwner
      readonly maybeHash: Option<H256>
    } & Struct
    readonly type:
      | 'Propose'
      | 'Second'
      | 'Vote'
      | 'EmergencyCancel'
      | 'ExternalPropose'
      | 'ExternalProposeMajority'
      | 'ExternalProposeDefault'
      | 'FastTrack'
      | 'VetoExternal'
      | 'CancelReferendum'
      | 'Delegate'
      | 'Undelegate'
      | 'ClearPublicProposals'
      | 'Unlock'
      | 'RemoveVote'
      | 'RemoveOtherVote'
      | 'Blacklist'
      | 'CancelProposal'
      | 'SetMetadata'
  }

  /** @name FrameSupportPreimagesBounded (209) */
  interface FrameSupportPreimagesBounded extends Enum {
    readonly isLegacy: boolean
    readonly asLegacy: {
      readonly hash_: H256
    } & Struct
    readonly isInline: boolean
    readonly asInline: Bytes
    readonly isLookup: boolean
    readonly asLookup: {
      readonly hash_: H256
      readonly len: u32
    } & Struct
    readonly type: 'Legacy' | 'Inline' | 'Lookup'
  }

  /** @name SpRuntimeBlakeTwo256 (210) */
  type SpRuntimeBlakeTwo256 = Null

  /** @name PalletDemocracyConviction (212) */
  interface PalletDemocracyConviction extends Enum {
    readonly isNone: boolean
    readonly isLocked1x: boolean
    readonly isLocked2x: boolean
    readonly isLocked3x: boolean
    readonly isLocked4x: boolean
    readonly isLocked5x: boolean
    readonly isLocked6x: boolean
    readonly type:
      | 'None'
      | 'Locked1x'
      | 'Locked2x'
      | 'Locked3x'
      | 'Locked4x'
      | 'Locked5x'
      | 'Locked6x'
  }

  /** @name PalletCollectiveCall (215) */
  interface PalletCollectiveCall extends Enum {
    readonly isSetMembers: boolean
    readonly asSetMembers: {
      readonly newMembers: Vec<AccountId32>
      readonly prime: Option<AccountId32>
      readonly oldCount: u32
    } & Struct
    readonly isExecute: boolean
    readonly asExecute: {
      readonly proposal: Call
      readonly lengthBound: Compact<u32>
    } & Struct
    readonly isPropose: boolean
    readonly asPropose: {
      readonly threshold: Compact<u32>
      readonly proposal: Call
      readonly lengthBound: Compact<u32>
    } & Struct
    readonly isVote: boolean
    readonly asVote: {
      readonly proposal: H256
      readonly index: Compact<u32>
      readonly approve: bool
    } & Struct
    readonly isDisapproveProposal: boolean
    readonly asDisapproveProposal: {
      readonly proposalHash: H256
    } & Struct
    readonly isClose: boolean
    readonly asClose: {
      readonly proposalHash: H256
      readonly index: Compact<u32>
      readonly proposalWeightBound: SpWeightsWeightV2Weight
      readonly lengthBound: Compact<u32>
    } & Struct
    readonly type:
      | 'SetMembers'
      | 'Execute'
      | 'Propose'
      | 'Vote'
      | 'DisapproveProposal'
      | 'Close'
  }

  /** @name PalletVestingCall (216) */
  interface PalletVestingCall extends Enum {
    readonly isVest: boolean
    readonly isVestOther: boolean
    readonly asVestOther: {
      readonly target: MultiAddress
    } & Struct
    readonly isVestedTransfer: boolean
    readonly asVestedTransfer: {
      readonly target: MultiAddress
      readonly schedule: PalletVestingVestingInfo
    } & Struct
    readonly isForceVestedTransfer: boolean
    readonly asForceVestedTransfer: {
      readonly source: MultiAddress
      readonly target: MultiAddress
      readonly schedule: PalletVestingVestingInfo
    } & Struct
    readonly isMergeSchedules: boolean
    readonly asMergeSchedules: {
      readonly schedule1Index: u32
      readonly schedule2Index: u32
    } & Struct
    readonly isForceRemoveVestingSchedule: boolean
    readonly asForceRemoveVestingSchedule: {
      readonly target: MultiAddress
      readonly scheduleIndex: u32
    } & Struct
    readonly type:
      | 'Vest'
      | 'VestOther'
      | 'VestedTransfer'
      | 'ForceVestedTransfer'
      | 'MergeSchedules'
      | 'ForceRemoveVestingSchedule'
  }

  /** @name PalletVestingVestingInfo (217) */
  interface PalletVestingVestingInfo extends Struct {
    readonly locked: u128
    readonly perBlock: u128
    readonly startingBlock: u64
  }

  /** @name PalletElectionsPhragmenCall (218) */
  interface PalletElectionsPhragmenCall extends Enum {
    readonly isVote: boolean
    readonly asVote: {
      readonly votes: Vec<AccountId32>
      readonly value: Compact<u128>
    } & Struct
    readonly isRemoveVoter: boolean
    readonly isSubmitCandidacy: boolean
    readonly asSubmitCandidacy: {
      readonly candidateCount: Compact<u32>
    } & Struct
    readonly isRenounceCandidacy: boolean
    readonly asRenounceCandidacy: {
      readonly renouncing: PalletElectionsPhragmenRenouncing
    } & Struct
    readonly isRemoveMember: boolean
    readonly asRemoveMember: {
      readonly who: MultiAddress
      readonly slashBond: bool
      readonly rerunElection: bool
    } & Struct
    readonly isCleanDefunctVoters: boolean
    readonly asCleanDefunctVoters: {
      readonly numVoters: u32
      readonly numDefunct: u32
    } & Struct
    readonly type:
      | 'Vote'
      | 'RemoveVoter'
      | 'SubmitCandidacy'
      | 'RenounceCandidacy'
      | 'RemoveMember'
      | 'CleanDefunctVoters'
  }

  /** @name PalletElectionsPhragmenRenouncing (219) */
  interface PalletElectionsPhragmenRenouncing extends Enum {
    readonly isMember: boolean
    readonly isRunnerUp: boolean
    readonly isCandidate: boolean
    readonly asCandidate: Compact<u32>
    readonly type: 'Member' | 'RunnerUp' | 'Candidate'
  }

  /** @name PalletElectionProviderMultiPhaseCall (220) */
  interface PalletElectionProviderMultiPhaseCall extends Enum {
    readonly isSubmitUnsigned: boolean
    readonly asSubmitUnsigned: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution
      readonly witness: PalletElectionProviderMultiPhaseSolutionOrSnapshotSize
    } & Struct
    readonly isSetMinimumUntrustedScore: boolean
    readonly asSetMinimumUntrustedScore: {
      readonly maybeNextScore: Option<SpNposElectionsElectionScore>
    } & Struct
    readonly isSetEmergencyElectionResult: boolean
    readonly asSetEmergencyElectionResult: {
      readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>
    } & Struct
    readonly isSubmit: boolean
    readonly asSubmit: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution
    } & Struct
    readonly isGovernanceFallback: boolean
    readonly asGovernanceFallback: {
      readonly maybeMaxVoters: Option<u32>
      readonly maybeMaxTargets: Option<u32>
    } & Struct
    readonly type:
      | 'SubmitUnsigned'
      | 'SetMinimumUntrustedScore'
      | 'SetEmergencyElectionResult'
      | 'Submit'
      | 'GovernanceFallback'
  }

  /** @name PalletElectionProviderMultiPhaseRawSolution (221) */
  interface PalletElectionProviderMultiPhaseRawSolution extends Struct {
    readonly solution: TangleTestnetRuntimeNposSolution16
    readonly score: SpNposElectionsElectionScore
    readonly round: u32
  }

  /** @name TangleTestnetRuntimeNposSolution16 (222) */
  interface TangleTestnetRuntimeNposSolution16 extends Struct {
    readonly votes1: Vec<ITuple<[Compact<u32>, Compact<u16>]>>
    readonly votes2: Vec<
      ITuple<
        [Compact<u32>, ITuple<[Compact<u16>, Compact<PerU16>]>, Compact<u16>]
      >
    >
    readonly votes3: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes4: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes5: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes6: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes7: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes8: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes9: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes10: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes11: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes12: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes13: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes14: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes15: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
    readonly votes16: Vec<
      ITuple<
        [
          Compact<u32>,
          Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>,
          Compact<u16>
        ]
      >
    >
  }

  /** @name PalletElectionProviderMultiPhaseSolutionOrSnapshotSize (273) */
  interface PalletElectionProviderMultiPhaseSolutionOrSnapshotSize
    extends Struct {
    readonly voters: Compact<u32>
    readonly targets: Compact<u32>
  }

  /** @name SpNposElectionsSupport (277) */
  interface SpNposElectionsSupport extends Struct {
    readonly total: u128
    readonly voters: Vec<ITuple<[AccountId32, u128]>>
  }

  /** @name PalletStakingPalletCall (278) */
  interface PalletStakingPalletCall extends Enum {
    readonly isBond: boolean
    readonly asBond: {
      readonly value: Compact<u128>
      readonly payee: PalletStakingRewardDestination
    } & Struct
    readonly isBondExtra: boolean
    readonly asBondExtra: {
      readonly maxAdditional: Compact<u128>
    } & Struct
    readonly isUnbond: boolean
    readonly asUnbond: {
      readonly value: Compact<u128>
    } & Struct
    readonly isWithdrawUnbonded: boolean
    readonly asWithdrawUnbonded: {
      readonly numSlashingSpans: u32
    } & Struct
    readonly isValidate: boolean
    readonly asValidate: {
      readonly prefs: PalletStakingValidatorPrefs
    } & Struct
    readonly isNominate: boolean
    readonly asNominate: {
      readonly targets: Vec<MultiAddress>
    } & Struct
    readonly isChill: boolean
    readonly isSetPayee: boolean
    readonly asSetPayee: {
      readonly payee: PalletStakingRewardDestination
    } & Struct
    readonly isSetController: boolean
    readonly isSetValidatorCount: boolean
    readonly asSetValidatorCount: {
      readonly new_: Compact<u32>
    } & Struct
    readonly isIncreaseValidatorCount: boolean
    readonly asIncreaseValidatorCount: {
      readonly additional: Compact<u32>
    } & Struct
    readonly isScaleValidatorCount: boolean
    readonly asScaleValidatorCount: {
      readonly factor: Percent
    } & Struct
    readonly isForceNoEras: boolean
    readonly isForceNewEra: boolean
    readonly isSetInvulnerables: boolean
    readonly asSetInvulnerables: {
      readonly invulnerables: Vec<AccountId32>
    } & Struct
    readonly isForceUnstake: boolean
    readonly asForceUnstake: {
      readonly stash: AccountId32
      readonly numSlashingSpans: u32
    } & Struct
    readonly isForceNewEraAlways: boolean
    readonly isCancelDeferredSlash: boolean
    readonly asCancelDeferredSlash: {
      readonly era: u32
      readonly slashIndices: Vec<u32>
    } & Struct
    readonly isPayoutStakers: boolean
    readonly asPayoutStakers: {
      readonly validatorStash: AccountId32
      readonly era: u32
    } & Struct
    readonly isRebond: boolean
    readonly asRebond: {
      readonly value: Compact<u128>
    } & Struct
    readonly isReapStash: boolean
    readonly asReapStash: {
      readonly stash: AccountId32
      readonly numSlashingSpans: u32
    } & Struct
    readonly isKick: boolean
    readonly asKick: {
      readonly who: Vec<MultiAddress>
    } & Struct
    readonly isSetStakingConfigs: boolean
    readonly asSetStakingConfigs: {
      readonly minNominatorBond: PalletStakingPalletConfigOpU128
      readonly minValidatorBond: PalletStakingPalletConfigOpU128
      readonly maxNominatorCount: PalletStakingPalletConfigOpU32
      readonly maxValidatorCount: PalletStakingPalletConfigOpU32
      readonly chillThreshold: PalletStakingPalletConfigOpPercent
      readonly minCommission: PalletStakingPalletConfigOpPerbill
    } & Struct
    readonly isChillOther: boolean
    readonly asChillOther: {
      readonly stash: AccountId32
    } & Struct
    readonly isForceApplyMinCommission: boolean
    readonly asForceApplyMinCommission: {
      readonly validatorStash: AccountId32
    } & Struct
    readonly isSetMinCommission: boolean
    readonly asSetMinCommission: {
      readonly new_: Perbill
    } & Struct
    readonly isPayoutStakersByPage: boolean
    readonly asPayoutStakersByPage: {
      readonly validatorStash: AccountId32
      readonly era: u32
      readonly page: u32
    } & Struct
    readonly isUpdatePayee: boolean
    readonly asUpdatePayee: {
      readonly controller: AccountId32
    } & Struct
    readonly isDeprecateControllerBatch: boolean
    readonly asDeprecateControllerBatch: {
      readonly controllers: Vec<AccountId32>
    } & Struct
    readonly type:
      | 'Bond'
      | 'BondExtra'
      | 'Unbond'
      | 'WithdrawUnbonded'
      | 'Validate'
      | 'Nominate'
      | 'Chill'
      | 'SetPayee'
      | 'SetController'
      | 'SetValidatorCount'
      | 'IncreaseValidatorCount'
      | 'ScaleValidatorCount'
      | 'ForceNoEras'
      | 'ForceNewEra'
      | 'SetInvulnerables'
      | 'ForceUnstake'
      | 'ForceNewEraAlways'
      | 'CancelDeferredSlash'
      | 'PayoutStakers'
      | 'Rebond'
      | 'ReapStash'
      | 'Kick'
      | 'SetStakingConfigs'
      | 'ChillOther'
      | 'ForceApplyMinCommission'
      | 'SetMinCommission'
      | 'PayoutStakersByPage'
      | 'UpdatePayee'
      | 'DeprecateControllerBatch'
  }

  /** @name PalletStakingPalletConfigOpU128 (282) */
  interface PalletStakingPalletConfigOpU128 extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: u128
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletStakingPalletConfigOpU32 (283) */
  interface PalletStakingPalletConfigOpU32 extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: u32
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletStakingPalletConfigOpPercent (284) */
  interface PalletStakingPalletConfigOpPercent extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: Percent
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletStakingPalletConfigOpPerbill (285) */
  interface PalletStakingPalletConfigOpPerbill extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: Perbill
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletSessionCall (287) */
  interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean
    readonly asSetKeys: {
      readonly keys_: TangleTestnetRuntimeOpaqueSessionKeys
      readonly proof: Bytes
    } & Struct
    readonly isPurgeKeys: boolean
    readonly type: 'SetKeys' | 'PurgeKeys'
  }

  /** @name TangleTestnetRuntimeOpaqueSessionKeys (288) */
  interface TangleTestnetRuntimeOpaqueSessionKeys extends Struct {
    readonly babe: SpConsensusBabeAppPublic
    readonly grandpa: SpConsensusGrandpaAppPublic
    readonly imOnline: PalletImOnlineSr25519AppSr25519Public
    readonly role: TangleCryptoPrimitivesCryptoPublic
  }

  /** @name TangleCryptoPrimitivesCryptoPublic (289) */
  interface TangleCryptoPrimitivesCryptoPublic extends SpCoreEcdsaPublic {}

  /** @name SpCoreEcdsaPublic (290) */
  interface SpCoreEcdsaPublic extends U8aFixed {}

  /** @name PalletTreasuryCall (292) */
  interface PalletTreasuryCall extends Enum {
    readonly isProposeSpend: boolean
    readonly asProposeSpend: {
      readonly value: Compact<u128>
      readonly beneficiary: MultiAddress
    } & Struct
    readonly isRejectProposal: boolean
    readonly asRejectProposal: {
      readonly proposalId: Compact<u32>
    } & Struct
    readonly isApproveProposal: boolean
    readonly asApproveProposal: {
      readonly proposalId: Compact<u32>
    } & Struct
    readonly isSpendLocal: boolean
    readonly asSpendLocal: {
      readonly amount: Compact<u128>
      readonly beneficiary: MultiAddress
    } & Struct
    readonly isRemoveApproval: boolean
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>
    } & Struct
    readonly isSpend: boolean
    readonly asSpend: {
      readonly assetKind: Null
      readonly amount: Compact<u128>
      readonly beneficiary: AccountId32
      readonly validFrom: Option<u64>
    } & Struct
    readonly isPayout: boolean
    readonly asPayout: {
      readonly index: u32
    } & Struct
    readonly isCheckStatus: boolean
    readonly asCheckStatus: {
      readonly index: u32
    } & Struct
    readonly isVoidSpend: boolean
    readonly asVoidSpend: {
      readonly index: u32
    } & Struct
    readonly type:
      | 'ProposeSpend'
      | 'RejectProposal'
      | 'ApproveProposal'
      | 'SpendLocal'
      | 'RemoveApproval'
      | 'Spend'
      | 'Payout'
      | 'CheckStatus'
      | 'VoidSpend'
  }

  /** @name PalletBountiesCall (294) */
  interface PalletBountiesCall extends Enum {
    readonly isProposeBounty: boolean
    readonly asProposeBounty: {
      readonly value: Compact<u128>
      readonly description: Bytes
    } & Struct
    readonly isApproveBounty: boolean
    readonly asApproveBounty: {
      readonly bountyId: Compact<u32>
    } & Struct
    readonly isProposeCurator: boolean
    readonly asProposeCurator: {
      readonly bountyId: Compact<u32>
      readonly curator: MultiAddress
      readonly fee: Compact<u128>
    } & Struct
    readonly isUnassignCurator: boolean
    readonly asUnassignCurator: {
      readonly bountyId: Compact<u32>
    } & Struct
    readonly isAcceptCurator: boolean
    readonly asAcceptCurator: {
      readonly bountyId: Compact<u32>
    } & Struct
    readonly isAwardBounty: boolean
    readonly asAwardBounty: {
      readonly bountyId: Compact<u32>
      readonly beneficiary: MultiAddress
    } & Struct
    readonly isClaimBounty: boolean
    readonly asClaimBounty: {
      readonly bountyId: Compact<u32>
    } & Struct
    readonly isCloseBounty: boolean
    readonly asCloseBounty: {
      readonly bountyId: Compact<u32>
    } & Struct
    readonly isExtendBountyExpiry: boolean
    readonly asExtendBountyExpiry: {
      readonly bountyId: Compact<u32>
      readonly remark: Bytes
    } & Struct
    readonly type:
      | 'ProposeBounty'
      | 'ApproveBounty'
      | 'ProposeCurator'
      | 'UnassignCurator'
      | 'AcceptCurator'
      | 'AwardBounty'
      | 'ClaimBounty'
      | 'CloseBounty'
      | 'ExtendBountyExpiry'
  }

  /** @name PalletChildBountiesCall (295) */
  interface PalletChildBountiesCall extends Enum {
    readonly isAddChildBounty: boolean
    readonly asAddChildBounty: {
      readonly parentBountyId: Compact<u32>
      readonly value: Compact<u128>
      readonly description: Bytes
    } & Struct
    readonly isProposeCurator: boolean
    readonly asProposeCurator: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
      readonly curator: MultiAddress
      readonly fee: Compact<u128>
    } & Struct
    readonly isAcceptCurator: boolean
    readonly asAcceptCurator: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
    } & Struct
    readonly isUnassignCurator: boolean
    readonly asUnassignCurator: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
    } & Struct
    readonly isAwardChildBounty: boolean
    readonly asAwardChildBounty: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
      readonly beneficiary: MultiAddress
    } & Struct
    readonly isClaimChildBounty: boolean
    readonly asClaimChildBounty: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
    } & Struct
    readonly isCloseChildBounty: boolean
    readonly asCloseChildBounty: {
      readonly parentBountyId: Compact<u32>
      readonly childBountyId: Compact<u32>
    } & Struct
    readonly type:
      | 'AddChildBounty'
      | 'ProposeCurator'
      | 'AcceptCurator'
      | 'UnassignCurator'
      | 'AwardChildBounty'
      | 'ClaimChildBounty'
      | 'CloseChildBounty'
  }

  /** @name PalletBagsListCall (296) */
  interface PalletBagsListCall extends Enum {
    readonly isRebag: boolean
    readonly asRebag: {
      readonly dislocated: MultiAddress
    } & Struct
    readonly isPutInFrontOf: boolean
    readonly asPutInFrontOf: {
      readonly lighter: MultiAddress
    } & Struct
    readonly isPutInFrontOfOther: boolean
    readonly asPutInFrontOfOther: {
      readonly heavier: MultiAddress
      readonly lighter: MultiAddress
    } & Struct
    readonly type: 'Rebag' | 'PutInFrontOf' | 'PutInFrontOfOther'
  }

  /** @name PalletNominationPoolsCall (297) */
  interface PalletNominationPoolsCall extends Enum {
    readonly isJoin: boolean
    readonly asJoin: {
      readonly amount: Compact<u128>
      readonly poolId: u32
    } & Struct
    readonly isBondExtra: boolean
    readonly asBondExtra: {
      readonly extra: PalletNominationPoolsBondExtra
    } & Struct
    readonly isClaimPayout: boolean
    readonly isUnbond: boolean
    readonly asUnbond: {
      readonly memberAccount: MultiAddress
      readonly unbondingPoints: Compact<u128>
    } & Struct
    readonly isPoolWithdrawUnbonded: boolean
    readonly asPoolWithdrawUnbonded: {
      readonly poolId: u32
      readonly numSlashingSpans: u32
    } & Struct
    readonly isWithdrawUnbonded: boolean
    readonly asWithdrawUnbonded: {
      readonly memberAccount: MultiAddress
      readonly numSlashingSpans: u32
    } & Struct
    readonly isCreate: boolean
    readonly asCreate: {
      readonly amount: Compact<u128>
      readonly root: MultiAddress
      readonly nominator: MultiAddress
      readonly bouncer: MultiAddress
    } & Struct
    readonly isCreateWithPoolId: boolean
    readonly asCreateWithPoolId: {
      readonly amount: Compact<u128>
      readonly root: MultiAddress
      readonly nominator: MultiAddress
      readonly bouncer: MultiAddress
      readonly poolId: u32
    } & Struct
    readonly isNominate: boolean
    readonly asNominate: {
      readonly poolId: u32
      readonly validators: Vec<AccountId32>
    } & Struct
    readonly isSetState: boolean
    readonly asSetState: {
      readonly poolId: u32
      readonly state: PalletNominationPoolsPoolState
    } & Struct
    readonly isSetMetadata: boolean
    readonly asSetMetadata: {
      readonly poolId: u32
      readonly metadata: Bytes
    } & Struct
    readonly isSetConfigs: boolean
    readonly asSetConfigs: {
      readonly minJoinBond: PalletNominationPoolsConfigOpU128
      readonly minCreateBond: PalletNominationPoolsConfigOpU128
      readonly maxPools: PalletNominationPoolsConfigOpU32
      readonly maxMembers: PalletNominationPoolsConfigOpU32
      readonly maxMembersPerPool: PalletNominationPoolsConfigOpU32
      readonly globalMaxCommission: PalletNominationPoolsConfigOpPerbill
    } & Struct
    readonly isUpdateRoles: boolean
    readonly asUpdateRoles: {
      readonly poolId: u32
      readonly newRoot: PalletNominationPoolsConfigOpAccountId32
      readonly newNominator: PalletNominationPoolsConfigOpAccountId32
      readonly newBouncer: PalletNominationPoolsConfigOpAccountId32
    } & Struct
    readonly isChill: boolean
    readonly asChill: {
      readonly poolId: u32
    } & Struct
    readonly isBondExtraOther: boolean
    readonly asBondExtraOther: {
      readonly member: MultiAddress
      readonly extra: PalletNominationPoolsBondExtra
    } & Struct
    readonly isSetClaimPermission: boolean
    readonly asSetClaimPermission: {
      readonly permission: PalletNominationPoolsClaimPermission
    } & Struct
    readonly isClaimPayoutOther: boolean
    readonly asClaimPayoutOther: {
      readonly other: AccountId32
    } & Struct
    readonly isSetCommission: boolean
    readonly asSetCommission: {
      readonly poolId: u32
      readonly newCommission: Option<ITuple<[Perbill, AccountId32]>>
    } & Struct
    readonly isSetCommissionMax: boolean
    readonly asSetCommissionMax: {
      readonly poolId: u32
      readonly maxCommission: Perbill
    } & Struct
    readonly isSetCommissionChangeRate: boolean
    readonly asSetCommissionChangeRate: {
      readonly poolId: u32
      readonly changeRate: PalletNominationPoolsCommissionChangeRate
    } & Struct
    readonly isClaimCommission: boolean
    readonly asClaimCommission: {
      readonly poolId: u32
    } & Struct
    readonly isAdjustPoolDeposit: boolean
    readonly asAdjustPoolDeposit: {
      readonly poolId: u32
    } & Struct
    readonly isSetCommissionClaimPermission: boolean
    readonly asSetCommissionClaimPermission: {
      readonly poolId: u32
      readonly permission: Option<PalletNominationPoolsCommissionClaimPermission>
    } & Struct
    readonly type:
      | 'Join'
      | 'BondExtra'
      | 'ClaimPayout'
      | 'Unbond'
      | 'PoolWithdrawUnbonded'
      | 'WithdrawUnbonded'
      | 'Create'
      | 'CreateWithPoolId'
      | 'Nominate'
      | 'SetState'
      | 'SetMetadata'
      | 'SetConfigs'
      | 'UpdateRoles'
      | 'Chill'
      | 'BondExtraOther'
      | 'SetClaimPermission'
      | 'ClaimPayoutOther'
      | 'SetCommission'
      | 'SetCommissionMax'
      | 'SetCommissionChangeRate'
      | 'ClaimCommission'
      | 'AdjustPoolDeposit'
      | 'SetCommissionClaimPermission'
  }

  /** @name PalletNominationPoolsBondExtra (298) */
  interface PalletNominationPoolsBondExtra extends Enum {
    readonly isFreeBalance: boolean
    readonly asFreeBalance: u128
    readonly isRewards: boolean
    readonly type: 'FreeBalance' | 'Rewards'
  }

  /** @name PalletNominationPoolsConfigOpU128 (299) */
  interface PalletNominationPoolsConfigOpU128 extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: u128
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletNominationPoolsConfigOpU32 (300) */
  interface PalletNominationPoolsConfigOpU32 extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: u32
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletNominationPoolsConfigOpPerbill (301) */
  interface PalletNominationPoolsConfigOpPerbill extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: Perbill
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletNominationPoolsConfigOpAccountId32 (302) */
  interface PalletNominationPoolsConfigOpAccountId32 extends Enum {
    readonly isNoop: boolean
    readonly isSet: boolean
    readonly asSet: AccountId32
    readonly isRemove: boolean
    readonly type: 'Noop' | 'Set' | 'Remove'
  }

  /** @name PalletNominationPoolsClaimPermission (303) */
  interface PalletNominationPoolsClaimPermission extends Enum {
    readonly isPermissioned: boolean
    readonly isPermissionlessCompound: boolean
    readonly isPermissionlessWithdraw: boolean
    readonly isPermissionlessAll: boolean
    readonly type:
      | 'Permissioned'
      | 'PermissionlessCompound'
      | 'PermissionlessWithdraw'
      | 'PermissionlessAll'
  }

  /** @name PalletSchedulerCall (304) */
  interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean
    readonly asSchedule: {
      readonly when: u64
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>
      readonly priority: u8
      readonly call: Call
    } & Struct
    readonly isCancel: boolean
    readonly asCancel: {
      readonly when: u64
      readonly index: u32
    } & Struct
    readonly isScheduleNamed: boolean
    readonly asScheduleNamed: {
      readonly id: U8aFixed
      readonly when: u64
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>
      readonly priority: u8
      readonly call: Call
    } & Struct
    readonly isCancelNamed: boolean
    readonly asCancelNamed: {
      readonly id: U8aFixed
    } & Struct
    readonly isScheduleAfter: boolean
    readonly asScheduleAfter: {
      readonly after: u64
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>
      readonly priority: u8
      readonly call: Call
    } & Struct
    readonly isScheduleNamedAfter: boolean
    readonly asScheduleNamedAfter: {
      readonly id: U8aFixed
      readonly after: u64
      readonly maybePeriodic: Option<ITuple<[u64, u32]>>
      readonly priority: u8
      readonly call: Call
    } & Struct
    readonly type:
      | 'Schedule'
      | 'Cancel'
      | 'ScheduleNamed'
      | 'CancelNamed'
      | 'ScheduleAfter'
      | 'ScheduleNamedAfter'
  }

  /** @name PalletPreimageCall (306) */
  interface PalletPreimageCall extends Enum {
    readonly isNotePreimage: boolean
    readonly asNotePreimage: {
      readonly bytes: Bytes
    } & Struct
    readonly isUnnotePreimage: boolean
    readonly asUnnotePreimage: {
      readonly hash_: H256
    } & Struct
    readonly isRequestPreimage: boolean
    readonly asRequestPreimage: {
      readonly hash_: H256
    } & Struct
    readonly isUnrequestPreimage: boolean
    readonly asUnrequestPreimage: {
      readonly hash_: H256
    } & Struct
    readonly isEnsureUpdated: boolean
    readonly asEnsureUpdated: {
      readonly hashes: Vec<H256>
    } & Struct
    readonly type:
      | 'NotePreimage'
      | 'UnnotePreimage'
      | 'RequestPreimage'
      | 'UnrequestPreimage'
      | 'EnsureUpdated'
  }

  /** @name PalletTransactionPauseModuleCall (307) */
  interface PalletTransactionPauseModuleCall extends Enum {
    readonly isPauseTransaction: boolean
    readonly asPauseTransaction: {
      readonly palletName: Bytes
      readonly functionName: Bytes
    } & Struct
    readonly isUnpauseTransaction: boolean
    readonly asUnpauseTransaction: {
      readonly palletName: Bytes
      readonly functionName: Bytes
    } & Struct
    readonly type: 'PauseTransaction' | 'UnpauseTransaction'
  }

  /** @name PalletImOnlineCall (308) */
  interface PalletImOnlineCall extends Enum {
    readonly isHeartbeat: boolean
    readonly asHeartbeat: {
      readonly heartbeat: PalletImOnlineHeartbeat
      readonly signature: PalletImOnlineSr25519AppSr25519Signature
    } & Struct
    readonly type: 'Heartbeat'
  }

  /** @name PalletImOnlineHeartbeat (309) */
  interface PalletImOnlineHeartbeat extends Struct {
    readonly blockNumber: u64
    readonly sessionIndex: u32
    readonly authorityIndex: u32
    readonly validatorsLen: u32
  }

  /** @name PalletImOnlineSr25519AppSr25519Signature (310) */
  interface PalletImOnlineSr25519AppSr25519Signature
    extends SpCoreSr25519Signature {}

  /** @name SpCoreSr25519Signature (311) */
  interface SpCoreSr25519Signature extends U8aFixed {}

  /** @name PalletIdentityCall (312) */
  interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean
    readonly asAddRegistrar: {
      readonly account: MultiAddress
    } & Struct
    readonly isSetIdentity: boolean
    readonly asSetIdentity: {
      readonly info: PalletIdentityLegacyIdentityInfo
    } & Struct
    readonly isSetSubs: boolean
    readonly asSetSubs: {
      readonly subs: Vec<ITuple<[AccountId32, Data]>>
    } & Struct
    readonly isClearIdentity: boolean
    readonly isRequestJudgement: boolean
    readonly asRequestJudgement: {
      readonly regIndex: Compact<u32>
      readonly maxFee: Compact<u128>
    } & Struct
    readonly isCancelRequest: boolean
    readonly asCancelRequest: {
      readonly regIndex: u32
    } & Struct
    readonly isSetFee: boolean
    readonly asSetFee: {
      readonly index: Compact<u32>
      readonly fee: Compact<u128>
    } & Struct
    readonly isSetAccountId: boolean
    readonly asSetAccountId: {
      readonly index: Compact<u32>
      readonly new_: MultiAddress
    } & Struct
    readonly isSetFields: boolean
    readonly asSetFields: {
      readonly index: Compact<u32>
      readonly fields: u64
    } & Struct
    readonly isProvideJudgement: boolean
    readonly asProvideJudgement: {
      readonly regIndex: Compact<u32>
      readonly target: MultiAddress
      readonly judgement: PalletIdentityJudgement
      readonly identity: H256
    } & Struct
    readonly isKillIdentity: boolean
    readonly asKillIdentity: {
      readonly target: MultiAddress
    } & Struct
    readonly isAddSub: boolean
    readonly asAddSub: {
      readonly sub: MultiAddress
      readonly data: Data
    } & Struct
    readonly isRenameSub: boolean
    readonly asRenameSub: {
      readonly sub: MultiAddress
      readonly data: Data
    } & Struct
    readonly isRemoveSub: boolean
    readonly asRemoveSub: {
      readonly sub: MultiAddress
    } & Struct
    readonly isQuitSub: boolean
    readonly isAddUsernameAuthority: boolean
    readonly asAddUsernameAuthority: {
      readonly authority: MultiAddress
      readonly suffix: Bytes
      readonly allocation: u32
    } & Struct
    readonly isRemoveUsernameAuthority: boolean
    readonly asRemoveUsernameAuthority: {
      readonly authority: MultiAddress
    } & Struct
    readonly isSetUsernameFor: boolean
    readonly asSetUsernameFor: {
      readonly who: MultiAddress
      readonly username: Bytes
      readonly signature: Option<SpRuntimeMultiSignature>
    } & Struct
    readonly isAcceptUsername: boolean
    readonly asAcceptUsername: {
      readonly username: Bytes
    } & Struct
    readonly isRemoveExpiredApproval: boolean
    readonly asRemoveExpiredApproval: {
      readonly username: Bytes
    } & Struct
    readonly isSetPrimaryUsername: boolean
    readonly asSetPrimaryUsername: {
      readonly username: Bytes
    } & Struct
    readonly isRemoveDanglingUsername: boolean
    readonly asRemoveDanglingUsername: {
      readonly username: Bytes
    } & Struct
    readonly type:
      | 'AddRegistrar'
      | 'SetIdentity'
      | 'SetSubs'
      | 'ClearIdentity'
      | 'RequestJudgement'
      | 'CancelRequest'
      | 'SetFee'
      | 'SetAccountId'
      | 'SetFields'
      | 'ProvideJudgement'
      | 'KillIdentity'
      | 'AddSub'
      | 'RenameSub'
      | 'RemoveSub'
      | 'QuitSub'
      | 'AddUsernameAuthority'
      | 'RemoveUsernameAuthority'
      | 'SetUsernameFor'
      | 'AcceptUsername'
      | 'RemoveExpiredApproval'
      | 'SetPrimaryUsername'
      | 'RemoveDanglingUsername'
  }

  /** @name PalletIdentityLegacyIdentityInfo (313) */
  interface PalletIdentityLegacyIdentityInfo extends Struct {
    readonly additional: Vec<ITuple<[Data, Data]>>
    readonly display: Data
    readonly legal: Data
    readonly web: Data
    readonly riot: Data
    readonly email: Data
    readonly pgpFingerprint: Option<U8aFixed>
    readonly image: Data
    readonly twitter: Data
  }

  /** @name PalletIdentityJudgement (349) */
  interface PalletIdentityJudgement extends Enum {
    readonly isUnknown: boolean
    readonly isFeePaid: boolean
    readonly asFeePaid: u128
    readonly isReasonable: boolean
    readonly isKnownGood: boolean
    readonly isOutOfDate: boolean
    readonly isLowQuality: boolean
    readonly isErroneous: boolean
    readonly type:
      | 'Unknown'
      | 'FeePaid'
      | 'Reasonable'
      | 'KnownGood'
      | 'OutOfDate'
      | 'LowQuality'
      | 'Erroneous'
  }

  /** @name SpRuntimeMultiSignature (351) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean
    readonly asEd25519: SpCoreEd25519Signature
    readonly isSr25519: boolean
    readonly asSr25519: SpCoreSr25519Signature
    readonly isEcdsa: boolean
    readonly asEcdsa: SpCoreEcdsaSignature
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa'
  }

  /** @name SpCoreEcdsaSignature (352) */
  interface SpCoreEcdsaSignature extends U8aFixed {}

  /** @name PalletUtilityCall (354) */
  interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean
    readonly asBatch: {
      readonly calls: Vec<Call>
    } & Struct
    readonly isAsDerivative: boolean
    readonly asAsDerivative: {
      readonly index: u16
      readonly call: Call
    } & Struct
    readonly isBatchAll: boolean
    readonly asBatchAll: {
      readonly calls: Vec<Call>
    } & Struct
    readonly isDispatchAs: boolean
    readonly asDispatchAs: {
      readonly asOrigin: TangleTestnetRuntimeOriginCaller
      readonly call: Call
    } & Struct
    readonly isForceBatch: boolean
    readonly asForceBatch: {
      readonly calls: Vec<Call>
    } & Struct
    readonly isWithWeight: boolean
    readonly asWithWeight: {
      readonly call: Call
      readonly weight: SpWeightsWeightV2Weight
    } & Struct
    readonly type:
      | 'Batch'
      | 'AsDerivative'
      | 'BatchAll'
      | 'DispatchAs'
      | 'ForceBatch'
      | 'WithWeight'
  }

  /** @name TangleTestnetRuntimeOriginCaller (356) */
  interface TangleTestnetRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean
    readonly asSystem: FrameSupportDispatchRawOrigin
    readonly isVoid: boolean
    readonly isCouncil: boolean
    readonly asCouncil: PalletCollectiveRawOrigin
    readonly isEthereum: boolean
    readonly asEthereum: PalletEthereumRawOrigin
    readonly type: 'System' | 'Void' | 'Council' | 'Ethereum'
  }

  /** @name FrameSupportDispatchRawOrigin (357) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean
    readonly isSigned: boolean
    readonly asSigned: AccountId32
    readonly isNone: boolean
    readonly type: 'Root' | 'Signed' | 'None'
  }

  /** @name PalletCollectiveRawOrigin (358) */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean
    readonly asMembers: ITuple<[u32, u32]>
    readonly isMember: boolean
    readonly asMember: AccountId32
    readonly isPhantom: boolean
    readonly type: 'Members' | 'Member' | 'Phantom'
  }

  /** @name PalletEthereumRawOrigin (359) */
  interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean
    readonly asEthereumTransaction: H160
    readonly type: 'EthereumTransaction'
  }

  /** @name PalletMultisigCall (360) */
  interface PalletMultisigCall extends Enum {
    readonly isAsMultiThreshold1: boolean
    readonly asAsMultiThreshold1: {
      readonly otherSignatories: Vec<AccountId32>
      readonly call: Call
    } & Struct
    readonly isAsMulti: boolean
    readonly asAsMulti: {
      readonly threshold: u16
      readonly otherSignatories: Vec<AccountId32>
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>
      readonly call: Call
      readonly maxWeight: SpWeightsWeightV2Weight
    } & Struct
    readonly isApproveAsMulti: boolean
    readonly asApproveAsMulti: {
      readonly threshold: u16
      readonly otherSignatories: Vec<AccountId32>
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>
      readonly callHash: U8aFixed
      readonly maxWeight: SpWeightsWeightV2Weight
    } & Struct
    readonly isCancelAsMulti: boolean
    readonly asCancelAsMulti: {
      readonly threshold: u16
      readonly otherSignatories: Vec<AccountId32>
      readonly timepoint: PalletMultisigTimepoint
      readonly callHash: U8aFixed
    } & Struct
    readonly type:
      | 'AsMultiThreshold1'
      | 'AsMulti'
      | 'ApproveAsMulti'
      | 'CancelAsMulti'
  }

  /** @name PalletEthereumCall (362) */
  interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2
    } & Struct
    readonly type: 'Transact'
  }

  /** @name EthereumTransactionTransactionV2 (363) */
  interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean
    readonly asLegacy: EthereumTransactionLegacyTransaction
    readonly isEip2930: boolean
    readonly asEip2930: EthereumTransactionEip2930Transaction
    readonly isEip1559: boolean
    readonly asEip1559: EthereumTransactionEip1559Transaction
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559'
  }

  /** @name EthereumTransactionLegacyTransaction (364) */
  interface EthereumTransactionLegacyTransaction extends Struct {
    readonly nonce: U256
    readonly gasPrice: U256
    readonly gasLimit: U256
    readonly action: EthereumTransactionTransactionAction
    readonly value: U256
    readonly input: Bytes
    readonly signature: EthereumTransactionTransactionSignature
  }

  /** @name EthereumTransactionTransactionAction (365) */
  interface EthereumTransactionTransactionAction extends Enum {
    readonly isCall: boolean
    readonly asCall: H160
    readonly isCreate: boolean
    readonly type: 'Call' | 'Create'
  }

  /** @name EthereumTransactionTransactionSignature (366) */
  interface EthereumTransactionTransactionSignature extends Struct {
    readonly v: u64
    readonly r: H256
    readonly s: H256
  }

  /** @name EthereumTransactionEip2930Transaction (368) */
  interface EthereumTransactionEip2930Transaction extends Struct {
    readonly chainId: u64
    readonly nonce: U256
    readonly gasPrice: U256
    readonly gasLimit: U256
    readonly action: EthereumTransactionTransactionAction
    readonly value: U256
    readonly input: Bytes
    readonly accessList: Vec<EthereumTransactionAccessListItem>
    readonly oddYParity: bool
    readonly r: H256
    readonly s: H256
  }

  /** @name EthereumTransactionAccessListItem (370) */
  interface EthereumTransactionAccessListItem extends Struct {
    readonly address: H160
    readonly storageKeys: Vec<H256>
  }

  /** @name EthereumTransactionEip1559Transaction (371) */
  interface EthereumTransactionEip1559Transaction extends Struct {
    readonly chainId: u64
    readonly nonce: U256
    readonly maxPriorityFeePerGas: U256
    readonly maxFeePerGas: U256
    readonly gasLimit: U256
    readonly action: EthereumTransactionTransactionAction
    readonly value: U256
    readonly input: Bytes
    readonly accessList: Vec<EthereumTransactionAccessListItem>
    readonly oddYParity: bool
    readonly r: H256
    readonly s: H256
  }

  /** @name PalletEvmCall (372) */
  interface PalletEvmCall extends Enum {
    readonly isWithdraw: boolean
    readonly asWithdraw: {
      readonly address: H160
      readonly value: u128
    } & Struct
    readonly isCall: boolean
    readonly asCall: {
      readonly source: H160
      readonly target: H160
      readonly input: Bytes
      readonly value: U256
      readonly gasLimit: u64
      readonly maxFeePerGas: U256
      readonly maxPriorityFeePerGas: Option<U256>
      readonly nonce: Option<U256>
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>
    } & Struct
    readonly isCreate: boolean
    readonly asCreate: {
      readonly source: H160
      readonly init: Bytes
      readonly value: U256
      readonly gasLimit: u64
      readonly maxFeePerGas: U256
      readonly maxPriorityFeePerGas: Option<U256>
      readonly nonce: Option<U256>
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>
    } & Struct
    readonly isCreate2: boolean
    readonly asCreate2: {
      readonly source: H160
      readonly init: Bytes
      readonly salt: H256
      readonly value: U256
      readonly gasLimit: u64
      readonly maxFeePerGas: U256
      readonly maxPriorityFeePerGas: Option<U256>
      readonly nonce: Option<U256>
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>
    } & Struct
    readonly type: 'Withdraw' | 'Call' | 'Create' | 'Create2'
  }

  /** @name PalletDynamicFeeCall (376) */
  interface PalletDynamicFeeCall extends Enum {
    readonly isNoteMinGasPriceTarget: boolean
    readonly asNoteMinGasPriceTarget: {
      readonly target: U256
    } & Struct
    readonly type: 'NoteMinGasPriceTarget'
  }

  /** @name PalletBaseFeeCall (377) */
  interface PalletBaseFeeCall extends Enum {
    readonly isSetBaseFeePerGas: boolean
    readonly asSetBaseFeePerGas: {
      readonly fee: U256
    } & Struct
    readonly isSetElasticity: boolean
    readonly asSetElasticity: {
      readonly elasticity: Permill
    } & Struct
    readonly type: 'SetBaseFeePerGas' | 'SetElasticity'
  }

  /** @name PalletHotfixSufficientsCall (378) */
  interface PalletHotfixSufficientsCall extends Enum {
    readonly isHotfixIncAccountSufficients: boolean
    readonly asHotfixIncAccountSufficients: {
      readonly addresses: Vec<H160>
    } & Struct
    readonly type: 'HotfixIncAccountSufficients'
  }

  /** @name PalletAirdropClaimsCall (380) */
  interface PalletAirdropClaimsCall extends Enum {
    readonly isClaim: boolean
    readonly asClaim: {
      readonly dest: Option<PalletAirdropClaimsUtilsMultiAddress>
      readonly signer: Option<PalletAirdropClaimsUtilsMultiAddress>
      readonly signature: PalletAirdropClaimsUtilsMultiAddressSignature
    } & Struct
    readonly isMintClaim: boolean
    readonly asMintClaim: {
      readonly who: PalletAirdropClaimsUtilsMultiAddress
      readonly value: u128
      readonly vestingSchedule: Option<Vec<ITuple<[u128, u128, u64]>>>
      readonly statement: Option<PalletAirdropClaimsStatementKind>
    } & Struct
    readonly isClaimAttest: boolean
    readonly asClaimAttest: {
      readonly dest: Option<PalletAirdropClaimsUtilsMultiAddress>
      readonly signer: Option<PalletAirdropClaimsUtilsMultiAddress>
      readonly signature: PalletAirdropClaimsUtilsMultiAddressSignature
      readonly statement: Bytes
    } & Struct
    readonly isMoveClaim: boolean
    readonly asMoveClaim: {
      readonly old: PalletAirdropClaimsUtilsMultiAddress
      readonly new_: PalletAirdropClaimsUtilsMultiAddress
    } & Struct
    readonly isForceSetExpiryConfig: boolean
    readonly asForceSetExpiryConfig: {
      readonly expiryBlock: u64
      readonly dest: PalletAirdropClaimsUtilsMultiAddress
    } & Struct
    readonly type:
      | 'Claim'
      | 'MintClaim'
      | 'ClaimAttest'
      | 'MoveClaim'
      | 'ForceSetExpiryConfig'
  }

  /** @name PalletAirdropClaimsUtilsMultiAddressSignature (382) */
  interface PalletAirdropClaimsUtilsMultiAddressSignature extends Enum {
    readonly isEvm: boolean
    readonly asEvm: PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature
    readonly isNative: boolean
    readonly asNative: PalletAirdropClaimsUtilsSr25519Signature
    readonly type: 'Evm' | 'Native'
  }

  /** @name PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature (383) */
  interface PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature
    extends U8aFixed {}

  /** @name PalletAirdropClaimsUtilsSr25519Signature (384) */
  interface PalletAirdropClaimsUtilsSr25519Signature
    extends SpCoreSr25519Signature {}

  /** @name PalletAirdropClaimsStatementKind (390) */
  interface PalletAirdropClaimsStatementKind extends Enum {
    readonly isRegular: boolean
    readonly isSafe: boolean
    readonly type: 'Regular' | 'Safe'
  }

  /** @name PalletRolesCall (391) */
  interface PalletRolesCall extends Enum {
    readonly isCreateProfile: boolean
    readonly asCreateProfile: {
      readonly profile: PalletRolesProfile
      readonly maxActiveServices: Option<u32>
    } & Struct
    readonly isUpdateProfile: boolean
    readonly asUpdateProfile: {
      readonly updatedProfile: PalletRolesProfile
    } & Struct
    readonly isDeleteProfile: boolean
    readonly isChill: boolean
    readonly isUnbondFunds: boolean
    readonly asUnbondFunds: {
      readonly amount: Compact<u128>
    } & Struct
    readonly isWithdrawUnbonded: boolean
    readonly isPayoutStakers: boolean
    readonly asPayoutStakers: {
      readonly validatorStash: AccountId32
      readonly era: u32
    } & Struct
    readonly isSetMinRestakingBond: boolean
    readonly asSetMinRestakingBond: {
      readonly minRestakingBond: u128
    } & Struct
    readonly type:
      | 'CreateProfile'
      | 'UpdateProfile'
      | 'DeleteProfile'
      | 'Chill'
      | 'UnbondFunds'
      | 'WithdrawUnbonded'
      | 'PayoutStakers'
      | 'SetMinRestakingBond'
  }

  /** @name PalletRolesProfile (392) */
  interface PalletRolesProfile extends Enum {
    readonly isIndependent: boolean
    readonly asIndependent: PalletRolesProfileIndependentRestakeProfile
    readonly isShared: boolean
    readonly asShared: PalletRolesProfileSharedRestakeProfile
    readonly type: 'Independent' | 'Shared'
  }

  /** @name PalletRolesProfileIndependentRestakeProfile (393) */
  interface PalletRolesProfileIndependentRestakeProfile extends Struct {
    readonly records: Vec<PalletRolesProfileRecord>
  }

  /** @name PalletRolesProfileRecord (395) */
  interface PalletRolesProfileRecord extends Struct {
    readonly role: TanglePrimitivesRolesRoleType
    readonly amount: Option<u128>
  }

  /** @name PalletRolesProfileSharedRestakeProfile (398) */
  interface PalletRolesProfileSharedRestakeProfile extends Struct {
    readonly records: Vec<PalletRolesProfileRecord>
    readonly amount: u128
  }

  /** @name PalletJobsModuleCall (399) */
  interface PalletJobsModuleCall extends Enum {
    readonly isSubmitJob: boolean
    readonly asSubmitJob: {
      readonly job: TanglePrimitivesJobsJobSubmission
    } & Struct
    readonly isSubmitJobResult: boolean
    readonly asSubmitJobResult: {
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly jobId: u64
      readonly result: TanglePrimitivesJobsJobResult
    } & Struct
    readonly isWithdrawRewards: boolean
    readonly isReportInactiveValidator: boolean
    readonly asReportInactiveValidator: {
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly jobId: u64
      readonly validator: AccountId32
      readonly offence: TanglePrimitivesJobsValidatorOffenceType
      readonly signatures: Vec<Bytes>
    } & Struct
    readonly isSetPermittedCaller: boolean
    readonly asSetPermittedCaller: {
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly jobId: u64
      readonly newPermittedCaller: AccountId32
    } & Struct
    readonly isSetTimeFee: boolean
    readonly asSetTimeFee: {
      readonly newFee: u128
    } & Struct
    readonly isSubmitMisbehavior: boolean
    readonly asSubmitMisbehavior: {
      readonly misbehavior: TanglePrimitivesMisbehaviorMisbehaviorSubmission
    } & Struct
    readonly isExtendJobResultTtl: boolean
    readonly asExtendJobResultTtl: {
      readonly roleType: TanglePrimitivesRolesRoleType
      readonly jobId: u64
      readonly extendBy: u64
    } & Struct
    readonly type:
      | 'SubmitJob'
      | 'SubmitJobResult'
      | 'WithdrawRewards'
      | 'ReportInactiveValidator'
      | 'SetPermittedCaller'
      | 'SetTimeFee'
      | 'SubmitMisbehavior'
      | 'ExtendJobResultTtl'
  }

  /** @name TanglePrimitivesJobsJobResult (400) */
  interface TanglePrimitivesJobsJobResult extends Enum {
    readonly isDkgPhaseOne: boolean
    readonly asDkgPhaseOne: TanglePrimitivesJobsTssDkgtssKeySubmissionResult
    readonly isDkgPhaseTwo: boolean
    readonly asDkgPhaseTwo: TanglePrimitivesJobsTssDkgtssSignatureResult
    readonly isDkgPhaseThree: boolean
    readonly asDkgPhaseThree: TanglePrimitivesJobsTssDkgtssKeyRefreshResult
    readonly isDkgPhaseFour: boolean
    readonly asDkgPhaseFour: TanglePrimitivesJobsTssDkgtssKeyRotationResult
    readonly isZkSaaSPhaseOne: boolean
    readonly asZkSaaSPhaseOne: TanglePrimitivesJobsZksaasZkSaaSCircuitResult
    readonly isZkSaaSPhaseTwo: boolean
    readonly asZkSaaSPhaseTwo: TanglePrimitivesJobsZksaasZkSaaSProofResult
    readonly type:
      | 'DkgPhaseOne'
      | 'DkgPhaseTwo'
      | 'DkgPhaseThree'
      | 'DkgPhaseFour'
      | 'ZkSaaSPhaseOne'
      | 'ZkSaaSPhaseTwo'
  }

  /** @name TangleTestnetRuntimeMaxKeyLen (401) */
  type TangleTestnetRuntimeMaxKeyLen = Null

  /** @name TangleTestnetRuntimeMaxSignatureLen (402) */
  type TangleTestnetRuntimeMaxSignatureLen = Null

  /** @name TangleTestnetRuntimeMaxDataLen (403) */
  type TangleTestnetRuntimeMaxDataLen = Null

  /** @name TangleTestnetRuntimeMaxProofLen (404) */
  type TangleTestnetRuntimeMaxProofLen = Null

  /** @name TanglePrimitivesJobsTssDkgtssKeySubmissionResult (405) */
  interface TanglePrimitivesJobsTssDkgtssKeySubmissionResult extends Struct {
    readonly signatureScheme: TanglePrimitivesJobsTssDigitalSignatureScheme
    readonly key: Bytes
    readonly participants: Vec<Bytes>
    readonly signatures: Vec<Bytes>
    readonly threshold: u8
  }

  /** @name TanglePrimitivesJobsTssDigitalSignatureScheme (406) */
  interface TanglePrimitivesJobsTssDigitalSignatureScheme extends Enum {
    readonly isEcdsa: boolean
    readonly isSchnorrP256: boolean
    readonly isSchnorrP384: boolean
    readonly isSchnorrSecp256k1: boolean
    readonly isSchnorrSr25519: boolean
    readonly isSchnorrRistretto255: boolean
    readonly isSchnorrRedJubJub: boolean
    readonly isSchnorrEd25519: boolean
    readonly isSchnorrEd448: boolean
    readonly isBls381: boolean
    readonly type:
      | 'Ecdsa'
      | 'SchnorrP256'
      | 'SchnorrP384'
      | 'SchnorrSecp256k1'
      | 'SchnorrSr25519'
      | 'SchnorrRistretto255'
      | 'SchnorrRedJubJub'
      | 'SchnorrEd25519'
      | 'SchnorrEd448'
      | 'Bls381'
  }

  /** @name TanglePrimitivesJobsTssDkgtssSignatureResult (413) */
  interface TanglePrimitivesJobsTssDkgtssSignatureResult extends Struct {
    readonly signatureScheme: TanglePrimitivesJobsTssDigitalSignatureScheme
    readonly data: Bytes
    readonly signature: Bytes
    readonly verifyingKey: Bytes
  }

  /** @name TanglePrimitivesJobsTssDkgtssKeyRefreshResult (415) */
  interface TanglePrimitivesJobsTssDkgtssKeyRefreshResult extends Struct {
    readonly signatureScheme: TanglePrimitivesJobsTssDigitalSignatureScheme
  }

  /** @name TanglePrimitivesJobsTssDkgtssKeyRotationResult (416) */
  interface TanglePrimitivesJobsTssDkgtssKeyRotationResult extends Struct {
    readonly phaseOneId: u64
    readonly newPhaseOneId: u64
    readonly newKey: Bytes
    readonly key: Bytes
    readonly signature: Bytes
    readonly signatureScheme: TanglePrimitivesJobsTssDigitalSignatureScheme
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSCircuitResult (417) */
  interface TanglePrimitivesJobsZksaasZkSaaSCircuitResult extends Struct {
    readonly jobId: u64
    readonly participants: Vec<SpCoreEcdsaPublic>
  }

  /** @name TanglePrimitivesJobsZksaasZkSaaSProofResult (420) */
  interface TanglePrimitivesJobsZksaasZkSaaSProofResult extends Enum {
    readonly isArkworks: boolean
    readonly asArkworks: TanglePrimitivesJobsZksaasArkworksProofResult
    readonly isCircom: boolean
    readonly asCircom: TanglePrimitivesJobsZksaasCircomProofResult
    readonly type: 'Arkworks' | 'Circom'
  }

  /** @name TanglePrimitivesJobsZksaasArkworksProofResult (421) */
  interface TanglePrimitivesJobsZksaasArkworksProofResult extends Struct {
    readonly proof: Bytes
  }

  /** @name TanglePrimitivesJobsZksaasCircomProofResult (423) */
  interface TanglePrimitivesJobsZksaasCircomProofResult extends Struct {
    readonly proof: Bytes
  }

  /** @name TanglePrimitivesJobsValidatorOffenceType (424) */
  interface TanglePrimitivesJobsValidatorOffenceType extends Enum {
    readonly isInactivity: boolean
    readonly isInvalidSignatureSubmitted: boolean
    readonly isRejectedValidAction: boolean
    readonly isApprovedInvalidAction: boolean
    readonly type:
      | 'Inactivity'
      | 'InvalidSignatureSubmitted'
      | 'RejectedValidAction'
      | 'ApprovedInvalidAction'
  }

  /** @name TanglePrimitivesMisbehaviorMisbehaviorSubmission (425) */
  interface TanglePrimitivesMisbehaviorMisbehaviorSubmission extends Struct {
    readonly roleType: TanglePrimitivesRolesRoleType
    readonly offender: U8aFixed
    readonly jobId: u64
    readonly justification: TanglePrimitivesMisbehaviorMisbehaviorJustification
  }

  /** @name TanglePrimitivesMisbehaviorMisbehaviorJustification (426) */
  interface TanglePrimitivesMisbehaviorMisbehaviorJustification extends Enum {
    readonly isDkgtss: boolean
    readonly asDkgtss: TanglePrimitivesMisbehaviorDkgtssJustification
    readonly isZkSaaS: boolean
    readonly type: 'Dkgtss' | 'ZkSaaS'
  }

  /** @name TanglePrimitivesMisbehaviorDkgtssJustification (427) */
  interface TanglePrimitivesMisbehaviorDkgtssJustification extends Enum {
    readonly isDfnsCGGMP21: boolean
    readonly asDfnsCGGMP21: TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification
    readonly isZCashFrost: boolean
    readonly asZCashFrost: TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification
    readonly type: 'DfnsCGGMP21' | 'ZCashFrost'
  }

  /** @name TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification (428) */
  interface TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification
    extends Enum {
    readonly isKeygen: boolean
    readonly asKeygen: {
      readonly participants: Vec<U8aFixed>
      readonly t: u16
      readonly reason: TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted
    } & Struct
    readonly isKeyRefresh: boolean
    readonly asKeyRefresh: {
      readonly participants: Vec<U8aFixed>
      readonly t: u16
      readonly reason: TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted
    } & Struct
    readonly isSigning: boolean
    readonly asSigning: {
      readonly participants: Vec<U8aFixed>
      readonly t: u16
      readonly reason: TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted
    } & Struct
    readonly type: 'Keygen' | 'KeyRefresh' | 'Signing'
  }

  /** @name TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted (430) */
  interface TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted extends Enum {
    readonly isInvalidDecommitment: boolean
    readonly asInvalidDecommitment: {
      readonly round1: TanglePrimitivesMisbehaviorSignedRoundMessage
      readonly round2a: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidSchnorrProof: boolean
    readonly asInvalidSchnorrProof: {
      readonly round2a: Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>
      readonly round3: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isFeldmanVerificationFailed: boolean
    readonly asFeldmanVerificationFailed: {
      readonly round2a: TanglePrimitivesMisbehaviorSignedRoundMessage
      readonly round2b: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidDataSize: boolean
    readonly asInvalidDataSize: {
      readonly round2a: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly type:
      | 'InvalidDecommitment'
      | 'InvalidSchnorrProof'
      | 'FeldmanVerificationFailed'
      | 'InvalidDataSize'
  }

  /** @name TanglePrimitivesMisbehaviorSignedRoundMessage (431) */
  interface TanglePrimitivesMisbehaviorSignedRoundMessage extends Struct {
    readonly sender: u16
    readonly message: Bytes
    readonly signature: Bytes
  }

  /** @name TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted (433) */
  interface TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted
    extends Enum {
    readonly isInvalidDecommitment: boolean
    readonly asInvalidDecommitment: {
      readonly round1: TanglePrimitivesMisbehaviorSignedRoundMessage
      readonly round2: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidSchnorrProof: boolean
    readonly isInvalidModProof: boolean
    readonly asInvalidModProof: {
      readonly reason: TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason
      readonly round2: Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>
      readonly round3: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidFacProof: boolean
    readonly isInvalidRingPedersenParameters: boolean
    readonly asInvalidRingPedersenParameters: {
      readonly round2: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidX: boolean
    readonly isInvalidXShare: boolean
    readonly isInvalidDataSize: boolean
    readonly isPaillierDec: boolean
    readonly type:
      | 'InvalidDecommitment'
      | 'InvalidSchnorrProof'
      | 'InvalidModProof'
      | 'InvalidFacProof'
      | 'InvalidRingPedersenParameters'
      | 'InvalidX'
      | 'InvalidXShare'
      | 'InvalidDataSize'
      | 'PaillierDec'
  }

  /** @name TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason (434) */
  interface TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason
    extends Enum {
    readonly isEqualityCheck: boolean
    readonly asEqualityCheck: u8
    readonly isRangeCheck: boolean
    readonly asRangeCheck: u8
    readonly isEncryption: boolean
    readonly isPaillierEnc: boolean
    readonly isPaillierOp: boolean
    readonly isModPow: boolean
    readonly isModulusIsPrime: boolean
    readonly isModulusIsEven: boolean
    readonly isIncorrectNthRoot: boolean
    readonly asIncorrectNthRoot: u8
    readonly isIncorrectFourthRoot: boolean
    readonly asIncorrectFourthRoot: u8
    readonly type:
      | 'EqualityCheck'
      | 'RangeCheck'
      | 'Encryption'
      | 'PaillierEnc'
      | 'PaillierOp'
      | 'ModPow'
      | 'ModulusIsPrime'
      | 'ModulusIsEven'
      | 'IncorrectNthRoot'
      | 'IncorrectFourthRoot'
  }

  /** @name TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted (435) */
  interface TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted extends Enum {
    readonly isEncProofOfK: boolean
    readonly isInvalidPsi: boolean
    readonly isInvalidPsiPrimePrime: boolean
    readonly isMismatchedDelta: boolean
    readonly type:
      | 'EncProofOfK'
      | 'InvalidPsi'
      | 'InvalidPsiPrimePrime'
      | 'MismatchedDelta'
  }

  /** @name TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification (436) */
  interface TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification
    extends Enum {
    readonly isKeygen: boolean
    readonly asKeygen: {
      readonly participants: Vec<U8aFixed>
      readonly t: u16
      readonly reason: TanglePrimitivesMisbehaviorZcashFrostKeygenAborted
    } & Struct
    readonly isSigning: boolean
    readonly asSigning: {
      readonly participants: Vec<U8aFixed>
      readonly t: u16
      readonly reason: TanglePrimitivesMisbehaviorZcashFrostSigningAborted
    } & Struct
    readonly type: 'Keygen' | 'Signing'
  }

  /** @name TanglePrimitivesMisbehaviorZcashFrostKeygenAborted (437) */
  interface TanglePrimitivesMisbehaviorZcashFrostKeygenAborted extends Enum {
    readonly isInvalidProofOfKnowledge: boolean
    readonly asInvalidProofOfKnowledge: {
      readonly round1: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly isInvalidSecretShare: boolean
    readonly asInvalidSecretShare: {
      readonly round1: TanglePrimitivesMisbehaviorSignedRoundMessage
      readonly round2: TanglePrimitivesMisbehaviorSignedRoundMessage
    } & Struct
    readonly type: 'InvalidProofOfKnowledge' | 'InvalidSecretShare'
  }

  /** @name TanglePrimitivesMisbehaviorZcashFrostSigningAborted (438) */
  interface TanglePrimitivesMisbehaviorZcashFrostSigningAborted extends Enum {
    readonly isInvalidSignatureShare: boolean
    readonly asInvalidSignatureShare: {
      readonly round1: Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>
      readonly round2: Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>
    } & Struct
    readonly type: 'InvalidSignatureShare'
  }

  /** @name TanglePrimitivesMisbehaviorZkSaaSJustification (439) */
  type TanglePrimitivesMisbehaviorZkSaaSJustification = Null

  /** @name PalletDkgCall (440) */
  interface PalletDkgCall extends Enum {
    readonly isSetFee: boolean
    readonly asSetFee: {
      readonly feeInfo: PalletDkgFeeInfo
    } & Struct
    readonly type: 'SetFee'
  }

  /** @name PalletZksaasCall (441) */
  interface PalletZksaasCall extends Enum {
    readonly isSetFee: boolean
    readonly asSetFee: {
      readonly feeInfo: PalletZksaasFeeInfo
    } & Struct
    readonly type: 'SetFee'
  }

  /** @name PalletSudoError (442) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean
    readonly type: 'RequireSudo'
  }

  /** @name PalletBalancesBalanceLock (445) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed
    readonly amount: u128
    readonly reasons: PalletBalancesReasons
  }

  /** @name PalletBalancesReasons (446) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean
    readonly isMisc: boolean
    readonly isAll: boolean
    readonly type: 'Fee' | 'Misc' | 'All'
  }

  /** @name PalletBalancesReserveData (449) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed
    readonly amount: u128
  }

  /** @name PalletBalancesIdAmountRuntimeHoldReason (452) */
  interface PalletBalancesIdAmountRuntimeHoldReason extends Struct {
    readonly id: TangleTestnetRuntimeRuntimeHoldReason
    readonly amount: u128
  }

  /** @name TangleTestnetRuntimeRuntimeHoldReason (453) */
  interface TangleTestnetRuntimeRuntimeHoldReason extends Enum {
    readonly isPreimage: boolean
    readonly asPreimage: PalletPreimageHoldReason
    readonly type: 'Preimage'
  }

  /** @name PalletPreimageHoldReason (454) */
  interface PalletPreimageHoldReason extends Enum {
    readonly isPreimage: boolean
    readonly type: 'Preimage'
  }

  /** @name PalletBalancesIdAmountRuntimeFreezeReason (457) */
  interface PalletBalancesIdAmountRuntimeFreezeReason extends Struct {
    readonly id: TangleTestnetRuntimeRuntimeFreezeReason
    readonly amount: u128
  }

  /** @name TangleTestnetRuntimeRuntimeFreezeReason (458) */
  interface TangleTestnetRuntimeRuntimeFreezeReason extends Enum {
    readonly isNominationPools: boolean
    readonly asNominationPools: PalletNominationPoolsFreezeReason
    readonly type: 'NominationPools'
  }

  /** @name PalletNominationPoolsFreezeReason (459) */
  interface PalletNominationPoolsFreezeReason extends Enum {
    readonly isPoolMinBalance: boolean
    readonly type: 'PoolMinBalance'
  }

  /** @name PalletBalancesError (461) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean
    readonly isLiquidityRestrictions: boolean
    readonly isInsufficientBalance: boolean
    readonly isExistentialDeposit: boolean
    readonly isExpendability: boolean
    readonly isExistingVestingSchedule: boolean
    readonly isDeadAccount: boolean
    readonly isTooManyReserves: boolean
    readonly isTooManyHolds: boolean
    readonly isTooManyFreezes: boolean
    readonly isIssuanceDeactivated: boolean
    readonly isDeltaZero: boolean
    readonly type:
      | 'VestingBalance'
      | 'LiquidityRestrictions'
      | 'InsufficientBalance'
      | 'ExistentialDeposit'
      | 'Expendability'
      | 'ExistingVestingSchedule'
      | 'DeadAccount'
      | 'TooManyReserves'
      | 'TooManyHolds'
      | 'TooManyFreezes'
      | 'IssuanceDeactivated'
      | 'DeltaZero'
  }

  /** @name PalletTransactionPaymentReleases (463) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean
    readonly isV2: boolean
    readonly type: 'V1Ancient' | 'V2'
  }

  /** @name SpConsensusBabeDigestsPreDigest (470) */
  interface SpConsensusBabeDigestsPreDigest extends Enum {
    readonly isPrimary: boolean
    readonly asPrimary: SpConsensusBabeDigestsPrimaryPreDigest
    readonly isSecondaryPlain: boolean
    readonly asSecondaryPlain: SpConsensusBabeDigestsSecondaryPlainPreDigest
    readonly isSecondaryVRF: boolean
    readonly asSecondaryVRF: SpConsensusBabeDigestsSecondaryVRFPreDigest
    readonly type: 'Primary' | 'SecondaryPlain' | 'SecondaryVRF'
  }

  /** @name SpConsensusBabeDigestsPrimaryPreDigest (471) */
  interface SpConsensusBabeDigestsPrimaryPreDigest extends Struct {
    readonly authorityIndex: u32
    readonly slot: u64
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature
  }

  /** @name SpCoreSr25519VrfVrfSignature (472) */
  interface SpCoreSr25519VrfVrfSignature extends Struct {
    readonly preOutput: U8aFixed
    readonly proof: U8aFixed
  }

  /** @name SpConsensusBabeDigestsSecondaryPlainPreDigest (473) */
  interface SpConsensusBabeDigestsSecondaryPlainPreDigest extends Struct {
    readonly authorityIndex: u32
    readonly slot: u64
  }

  /** @name SpConsensusBabeDigestsSecondaryVRFPreDigest (474) */
  interface SpConsensusBabeDigestsSecondaryVRFPreDigest extends Struct {
    readonly authorityIndex: u32
    readonly slot: u64
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature
  }

  /** @name SpConsensusBabeBabeEpochConfiguration (475) */
  interface SpConsensusBabeBabeEpochConfiguration extends Struct {
    readonly c: ITuple<[u64, u64]>
    readonly allowedSlots: SpConsensusBabeAllowedSlots
  }

  /** @name PalletBabeError (477) */
  interface PalletBabeError extends Enum {
    readonly isInvalidEquivocationProof: boolean
    readonly isInvalidKeyOwnershipProof: boolean
    readonly isDuplicateOffenceReport: boolean
    readonly isInvalidConfiguration: boolean
    readonly type:
      | 'InvalidEquivocationProof'
      | 'InvalidKeyOwnershipProof'
      | 'DuplicateOffenceReport'
      | 'InvalidConfiguration'
  }

  /** @name PalletGrandpaStoredState (478) */
  interface PalletGrandpaStoredState extends Enum {
    readonly isLive: boolean
    readonly isPendingPause: boolean
    readonly asPendingPause: {
      readonly scheduledAt: u64
      readonly delay: u64
    } & Struct
    readonly isPaused: boolean
    readonly isPendingResume: boolean
    readonly asPendingResume: {
      readonly scheduledAt: u64
      readonly delay: u64
    } & Struct
    readonly type: 'Live' | 'PendingPause' | 'Paused' | 'PendingResume'
  }

  /** @name PalletGrandpaStoredPendingChange (479) */
  interface PalletGrandpaStoredPendingChange extends Struct {
    readonly scheduledAt: u64
    readonly delay: u64
    readonly nextAuthorities: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>
    readonly forced: Option<u64>
  }

  /** @name PalletGrandpaError (481) */
  interface PalletGrandpaError extends Enum {
    readonly isPauseFailed: boolean
    readonly isResumeFailed: boolean
    readonly isChangePending: boolean
    readonly isTooSoon: boolean
    readonly isInvalidKeyOwnershipProof: boolean
    readonly isInvalidEquivocationProof: boolean
    readonly isDuplicateOffenceReport: boolean
    readonly type:
      | 'PauseFailed'
      | 'ResumeFailed'
      | 'ChangePending'
      | 'TooSoon'
      | 'InvalidKeyOwnershipProof'
      | 'InvalidEquivocationProof'
      | 'DuplicateOffenceReport'
  }

  /** @name PalletIndicesError (483) */
  interface PalletIndicesError extends Enum {
    readonly isNotAssigned: boolean
    readonly isNotOwner: boolean
    readonly isInUse: boolean
    readonly isNotTransfer: boolean
    readonly isPermanent: boolean
    readonly type:
      | 'NotAssigned'
      | 'NotOwner'
      | 'InUse'
      | 'NotTransfer'
      | 'Permanent'
  }

  /** @name PalletDemocracyReferendumInfo (488) */
  interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean
    readonly asOngoing: PalletDemocracyReferendumStatus
    readonly isFinished: boolean
    readonly asFinished: {
      readonly approved: bool
      readonly end: u64
    } & Struct
    readonly type: 'Ongoing' | 'Finished'
  }

  /** @name PalletDemocracyReferendumStatus (489) */
  interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u64
    readonly proposal: FrameSupportPreimagesBounded
    readonly threshold: PalletDemocracyVoteThreshold
    readonly delay: u64
    readonly tally: PalletDemocracyTally
  }

  /** @name PalletDemocracyTally (490) */
  interface PalletDemocracyTally extends Struct {
    readonly ayes: u128
    readonly nays: u128
    readonly turnout: u128
  }

  /** @name PalletDemocracyVoteVoting (491) */
  interface PalletDemocracyVoteVoting extends Enum {
    readonly isDirect: boolean
    readonly asDirect: {
      readonly votes: Vec<ITuple<[u32, PalletDemocracyVoteAccountVote]>>
      readonly delegations: PalletDemocracyDelegations
      readonly prior: PalletDemocracyVotePriorLock
    } & Struct
    readonly isDelegating: boolean
    readonly asDelegating: {
      readonly balance: u128
      readonly target: AccountId32
      readonly conviction: PalletDemocracyConviction
      readonly delegations: PalletDemocracyDelegations
      readonly prior: PalletDemocracyVotePriorLock
    } & Struct
    readonly type: 'Direct' | 'Delegating'
  }

  /** @name PalletDemocracyDelegations (495) */
  interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128
    readonly capital: u128
  }

  /** @name PalletDemocracyVotePriorLock (496) */
  interface PalletDemocracyVotePriorLock extends ITuple<[u64, u128]> {}

  /** @name PalletDemocracyError (499) */
  interface PalletDemocracyError extends Enum {
    readonly isValueLow: boolean
    readonly isProposalMissing: boolean
    readonly isAlreadyCanceled: boolean
    readonly isDuplicateProposal: boolean
    readonly isProposalBlacklisted: boolean
    readonly isNotSimpleMajority: boolean
    readonly isInvalidHash: boolean
    readonly isNoProposal: boolean
    readonly isAlreadyVetoed: boolean
    readonly isReferendumInvalid: boolean
    readonly isNoneWaiting: boolean
    readonly isNotVoter: boolean
    readonly isNoPermission: boolean
    readonly isAlreadyDelegating: boolean
    readonly isInsufficientFunds: boolean
    readonly isNotDelegating: boolean
    readonly isVotesExist: boolean
    readonly isInstantNotAllowed: boolean
    readonly isNonsense: boolean
    readonly isWrongUpperBound: boolean
    readonly isMaxVotesReached: boolean
    readonly isTooMany: boolean
    readonly isVotingPeriodLow: boolean
    readonly isPreimageNotExist: boolean
    readonly type:
      | 'ValueLow'
      | 'ProposalMissing'
      | 'AlreadyCanceled'
      | 'DuplicateProposal'
      | 'ProposalBlacklisted'
      | 'NotSimpleMajority'
      | 'InvalidHash'
      | 'NoProposal'
      | 'AlreadyVetoed'
      | 'ReferendumInvalid'
      | 'NoneWaiting'
      | 'NotVoter'
      | 'NoPermission'
      | 'AlreadyDelegating'
      | 'InsufficientFunds'
      | 'NotDelegating'
      | 'VotesExist'
      | 'InstantNotAllowed'
      | 'Nonsense'
      | 'WrongUpperBound'
      | 'MaxVotesReached'
      | 'TooMany'
      | 'VotingPeriodLow'
      | 'PreimageNotExist'
  }

  /** @name PalletCollectiveVotes (501) */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32
    readonly threshold: u32
    readonly ayes: Vec<AccountId32>
    readonly nays: Vec<AccountId32>
    readonly end: u64
  }

  /** @name PalletCollectiveError (502) */
  interface PalletCollectiveError extends Enum {
    readonly isNotMember: boolean
    readonly isDuplicateProposal: boolean
    readonly isProposalMissing: boolean
    readonly isWrongIndex: boolean
    readonly isDuplicateVote: boolean
    readonly isAlreadyInitialized: boolean
    readonly isTooEarly: boolean
    readonly isTooManyProposals: boolean
    readonly isWrongProposalWeight: boolean
    readonly isWrongProposalLength: boolean
    readonly isPrimeAccountNotMember: boolean
    readonly type:
      | 'NotMember'
      | 'DuplicateProposal'
      | 'ProposalMissing'
      | 'WrongIndex'
      | 'DuplicateVote'
      | 'AlreadyInitialized'
      | 'TooEarly'
      | 'TooManyProposals'
      | 'WrongProposalWeight'
      | 'WrongProposalLength'
      | 'PrimeAccountNotMember'
  }

  /** @name PalletVestingReleases (505) */
  interface PalletVestingReleases extends Enum {
    readonly isV0: boolean
    readonly isV1: boolean
    readonly type: 'V0' | 'V1'
  }

  /** @name PalletVestingError (506) */
  interface PalletVestingError extends Enum {
    readonly isNotVesting: boolean
    readonly isAtMaxVestingSchedules: boolean
    readonly isAmountLow: boolean
    readonly isScheduleIndexOutOfBounds: boolean
    readonly isInvalidScheduleParams: boolean
    readonly type:
      | 'NotVesting'
      | 'AtMaxVestingSchedules'
      | 'AmountLow'
      | 'ScheduleIndexOutOfBounds'
      | 'InvalidScheduleParams'
  }

  /** @name PalletElectionsPhragmenSeatHolder (508) */
  interface PalletElectionsPhragmenSeatHolder extends Struct {
    readonly who: AccountId32
    readonly stake: u128
    readonly deposit: u128
  }

  /** @name PalletElectionsPhragmenVoter (509) */
  interface PalletElectionsPhragmenVoter extends Struct {
    readonly votes: Vec<AccountId32>
    readonly stake: u128
    readonly deposit: u128
  }

  /** @name PalletElectionsPhragmenError (510) */
  interface PalletElectionsPhragmenError extends Enum {
    readonly isUnableToVote: boolean
    readonly isNoVotes: boolean
    readonly isTooManyVotes: boolean
    readonly isMaximumVotesExceeded: boolean
    readonly isLowBalance: boolean
    readonly isUnableToPayBond: boolean
    readonly isMustBeVoter: boolean
    readonly isDuplicatedCandidate: boolean
    readonly isTooManyCandidates: boolean
    readonly isMemberSubmit: boolean
    readonly isRunnerUpSubmit: boolean
    readonly isInsufficientCandidateFunds: boolean
    readonly isNotMember: boolean
    readonly isInvalidWitnessData: boolean
    readonly isInvalidVoteCount: boolean
    readonly isInvalidRenouncing: boolean
    readonly isInvalidReplacement: boolean
    readonly type:
      | 'UnableToVote'
      | 'NoVotes'
      | 'TooManyVotes'
      | 'MaximumVotesExceeded'
      | 'LowBalance'
      | 'UnableToPayBond'
      | 'MustBeVoter'
      | 'DuplicatedCandidate'
      | 'TooManyCandidates'
      | 'MemberSubmit'
      | 'RunnerUpSubmit'
      | 'InsufficientCandidateFunds'
      | 'NotMember'
      | 'InvalidWitnessData'
      | 'InvalidVoteCount'
      | 'InvalidRenouncing'
      | 'InvalidReplacement'
  }

  /** @name PalletElectionProviderMultiPhaseReadySolution (511) */
  interface PalletElectionProviderMultiPhaseReadySolution extends Struct {
    readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>
    readonly score: SpNposElectionsElectionScore
    readonly compute: PalletElectionProviderMultiPhaseElectionCompute
  }

  /** @name PalletElectionProviderMultiPhaseRoundSnapshot (513) */
  interface PalletElectionProviderMultiPhaseRoundSnapshot extends Struct {
    readonly voters: Vec<ITuple<[AccountId32, u64, Vec<AccountId32>]>>
    readonly targets: Vec<AccountId32>
  }

  /** @name PalletElectionProviderMultiPhaseSignedSignedSubmission (520) */
  interface PalletElectionProviderMultiPhaseSignedSignedSubmission
    extends Struct {
    readonly who: AccountId32
    readonly deposit: u128
    readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution
    readonly callFee: u128
  }

  /** @name PalletElectionProviderMultiPhaseError (521) */
  interface PalletElectionProviderMultiPhaseError extends Enum {
    readonly isPreDispatchEarlySubmission: boolean
    readonly isPreDispatchWrongWinnerCount: boolean
    readonly isPreDispatchWeakSubmission: boolean
    readonly isSignedQueueFull: boolean
    readonly isSignedCannotPayDeposit: boolean
    readonly isSignedInvalidWitness: boolean
    readonly isSignedTooMuchWeight: boolean
    readonly isOcwCallWrongEra: boolean
    readonly isMissingSnapshotMetadata: boolean
    readonly isInvalidSubmissionIndex: boolean
    readonly isCallNotAllowed: boolean
    readonly isFallbackFailed: boolean
    readonly isBoundNotMet: boolean
    readonly isTooManyWinners: boolean
    readonly isPreDispatchDifferentRound: boolean
    readonly type:
      | 'PreDispatchEarlySubmission'
      | 'PreDispatchWrongWinnerCount'
      | 'PreDispatchWeakSubmission'
      | 'SignedQueueFull'
      | 'SignedCannotPayDeposit'
      | 'SignedInvalidWitness'
      | 'SignedTooMuchWeight'
      | 'OcwCallWrongEra'
      | 'MissingSnapshotMetadata'
      | 'InvalidSubmissionIndex'
      | 'CallNotAllowed'
      | 'FallbackFailed'
      | 'BoundNotMet'
      | 'TooManyWinners'
      | 'PreDispatchDifferentRound'
  }

  /** @name PalletStakingStakingLedger (522) */
  interface PalletStakingStakingLedger extends Struct {
    readonly stash: AccountId32
    readonly total: Compact<u128>
    readonly active: Compact<u128>
    readonly unlocking: Vec<PalletStakingUnlockChunk>
    readonly legacyClaimedRewards: Vec<u32>
  }

  /** @name PalletStakingUnlockChunk (524) */
  interface PalletStakingUnlockChunk extends Struct {
    readonly value: Compact<u128>
    readonly era: Compact<u32>
  }

  /** @name PalletStakingNominations (527) */
  interface PalletStakingNominations extends Struct {
    readonly targets: Vec<AccountId32>
    readonly submittedIn: u32
    readonly suppressed: bool
  }

  /** @name PalletStakingActiveEraInfo (528) */
  interface PalletStakingActiveEraInfo extends Struct {
    readonly index: u32
    readonly start: Option<u64>
  }

  /** @name SpStakingPagedExposureMetadata (530) */
  interface SpStakingPagedExposureMetadata extends Struct {
    readonly total: Compact<u128>
    readonly own: Compact<u128>
    readonly nominatorCount: u32
    readonly pageCount: u32
  }

  /** @name SpStakingExposurePage (532) */
  interface SpStakingExposurePage extends Struct {
    readonly pageTotal: Compact<u128>
    readonly others: Vec<SpStakingIndividualExposure>
  }

  /** @name PalletStakingEraRewardPoints (533) */
  interface PalletStakingEraRewardPoints extends Struct {
    readonly total: u32
    readonly individual: BTreeMap<AccountId32, u32>
  }

  /** @name PalletStakingUnappliedSlash (538) */
  interface PalletStakingUnappliedSlash extends Struct {
    readonly validator: AccountId32
    readonly own: u128
    readonly others: Vec<ITuple<[AccountId32, u128]>>
    readonly reporters: Vec<AccountId32>
    readonly payout: u128
  }

  /** @name PalletStakingSlashingSlashingSpans (542) */
  interface PalletStakingSlashingSlashingSpans extends Struct {
    readonly spanIndex: u32
    readonly lastStart: u32
    readonly lastNonzeroSlash: u32
    readonly prior: Vec<u32>
  }

  /** @name PalletStakingSlashingSpanRecord (543) */
  interface PalletStakingSlashingSpanRecord extends Struct {
    readonly slashed: u128
    readonly paidOut: u128
  }

  /** @name PalletStakingPalletError (546) */
  interface PalletStakingPalletError extends Enum {
    readonly isNotController: boolean
    readonly isNotStash: boolean
    readonly isAlreadyBonded: boolean
    readonly isAlreadyPaired: boolean
    readonly isEmptyTargets: boolean
    readonly isDuplicateIndex: boolean
    readonly isInvalidSlashIndex: boolean
    readonly isInsufficientBond: boolean
    readonly isNoMoreChunks: boolean
    readonly isNoUnlockChunk: boolean
    readonly isFundedTarget: boolean
    readonly isInvalidEraToReward: boolean
    readonly isInvalidNumberOfNominations: boolean
    readonly isNotSortedAndUnique: boolean
    readonly isAlreadyClaimed: boolean
    readonly isInvalidPage: boolean
    readonly isIncorrectHistoryDepth: boolean
    readonly isIncorrectSlashingSpans: boolean
    readonly isBadState: boolean
    readonly isTooManyTargets: boolean
    readonly isBadTarget: boolean
    readonly isCannotChillOther: boolean
    readonly isTooManyNominators: boolean
    readonly isTooManyValidators: boolean
    readonly isCommissionTooLow: boolean
    readonly isBoundNotMet: boolean
    readonly isControllerDeprecated: boolean
    readonly type:
      | 'NotController'
      | 'NotStash'
      | 'AlreadyBonded'
      | 'AlreadyPaired'
      | 'EmptyTargets'
      | 'DuplicateIndex'
      | 'InvalidSlashIndex'
      | 'InsufficientBond'
      | 'NoMoreChunks'
      | 'NoUnlockChunk'
      | 'FundedTarget'
      | 'InvalidEraToReward'
      | 'InvalidNumberOfNominations'
      | 'NotSortedAndUnique'
      | 'AlreadyClaimed'
      | 'InvalidPage'
      | 'IncorrectHistoryDepth'
      | 'IncorrectSlashingSpans'
      | 'BadState'
      | 'TooManyTargets'
      | 'BadTarget'
      | 'CannotChillOther'
      | 'TooManyNominators'
      | 'TooManyValidators'
      | 'CommissionTooLow'
      | 'BoundNotMet'
      | 'ControllerDeprecated'
  }

  /** @name SpCoreCryptoKeyTypeId (550) */
  interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (551) */
  interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean
    readonly isNoAssociatedValidatorId: boolean
    readonly isDuplicatedKey: boolean
    readonly isNoKeys: boolean
    readonly isNoAccount: boolean
    readonly type:
      | 'InvalidProof'
      | 'NoAssociatedValidatorId'
      | 'DuplicatedKey'
      | 'NoKeys'
      | 'NoAccount'
  }

  /** @name PalletTreasuryProposal (553) */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId32
    readonly value: u128
    readonly beneficiary: AccountId32
    readonly bond: u128
  }

  /** @name PalletTreasurySpendStatus (555) */
  interface PalletTreasurySpendStatus extends Struct {
    readonly assetKind: Null
    readonly amount: u128
    readonly beneficiary: AccountId32
    readonly validFrom: u64
    readonly expireAt: u64
    readonly status: PalletTreasuryPaymentState
  }

  /** @name PalletTreasuryPaymentState (556) */
  interface PalletTreasuryPaymentState extends Enum {
    readonly isPending: boolean
    readonly isAttempted: boolean
    readonly asAttempted: {
      readonly id: Null
    } & Struct
    readonly isFailed: boolean
    readonly type: 'Pending' | 'Attempted' | 'Failed'
  }

  /** @name FrameSupportPalletId (557) */
  interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (558) */
  interface PalletTreasuryError extends Enum {
    readonly isInsufficientProposersBalance: boolean
    readonly isInvalidIndex: boolean
    readonly isTooManyApprovals: boolean
    readonly isInsufficientPermission: boolean
    readonly isProposalNotApproved: boolean
    readonly isFailedToConvertBalance: boolean
    readonly isSpendExpired: boolean
    readonly isEarlyPayout: boolean
    readonly isAlreadyAttempted: boolean
    readonly isPayoutError: boolean
    readonly isNotAttempted: boolean
    readonly isInconclusive: boolean
    readonly type:
      | 'InsufficientProposersBalance'
      | 'InvalidIndex'
      | 'TooManyApprovals'
      | 'InsufficientPermission'
      | 'ProposalNotApproved'
      | 'FailedToConvertBalance'
      | 'SpendExpired'
      | 'EarlyPayout'
      | 'AlreadyAttempted'
      | 'PayoutError'
      | 'NotAttempted'
      | 'Inconclusive'
  }

  /** @name PalletBountiesBounty (559) */
  interface PalletBountiesBounty extends Struct {
    readonly proposer: AccountId32
    readonly value: u128
    readonly fee: u128
    readonly curatorDeposit: u128
    readonly bond: u128
    readonly status: PalletBountiesBountyStatus
  }

  /** @name PalletBountiesBountyStatus (560) */
  interface PalletBountiesBountyStatus extends Enum {
    readonly isProposed: boolean
    readonly isApproved: boolean
    readonly isFunded: boolean
    readonly isCuratorProposed: boolean
    readonly asCuratorProposed: {
      readonly curator: AccountId32
    } & Struct
    readonly isActive: boolean
    readonly asActive: {
      readonly curator: AccountId32
      readonly updateDue: u64
    } & Struct
    readonly isPendingPayout: boolean
    readonly asPendingPayout: {
      readonly curator: AccountId32
      readonly beneficiary: AccountId32
      readonly unlockAt: u64
    } & Struct
    readonly type:
      | 'Proposed'
      | 'Approved'
      | 'Funded'
      | 'CuratorProposed'
      | 'Active'
      | 'PendingPayout'
  }

  /** @name PalletBountiesError (562) */
  interface PalletBountiesError extends Enum {
    readonly isInsufficientProposersBalance: boolean
    readonly isInvalidIndex: boolean
    readonly isReasonTooBig: boolean
    readonly isUnexpectedStatus: boolean
    readonly isRequireCurator: boolean
    readonly isInvalidValue: boolean
    readonly isInvalidFee: boolean
    readonly isPendingPayout: boolean
    readonly isPremature: boolean
    readonly isHasActiveChildBounty: boolean
    readonly isTooManyQueued: boolean
    readonly type:
      | 'InsufficientProposersBalance'
      | 'InvalidIndex'
      | 'ReasonTooBig'
      | 'UnexpectedStatus'
      | 'RequireCurator'
      | 'InvalidValue'
      | 'InvalidFee'
      | 'PendingPayout'
      | 'Premature'
      | 'HasActiveChildBounty'
      | 'TooManyQueued'
  }

  /** @name PalletChildBountiesChildBounty (563) */
  interface PalletChildBountiesChildBounty extends Struct {
    readonly parentBounty: u32
    readonly value: u128
    readonly fee: u128
    readonly curatorDeposit: u128
    readonly status: PalletChildBountiesChildBountyStatus
  }

  /** @name PalletChildBountiesChildBountyStatus (564) */
  interface PalletChildBountiesChildBountyStatus extends Enum {
    readonly isAdded: boolean
    readonly isCuratorProposed: boolean
    readonly asCuratorProposed: {
      readonly curator: AccountId32
    } & Struct
    readonly isActive: boolean
    readonly asActive: {
      readonly curator: AccountId32
    } & Struct
    readonly isPendingPayout: boolean
    readonly asPendingPayout: {
      readonly curator: AccountId32
      readonly beneficiary: AccountId32
      readonly unlockAt: u64
    } & Struct
    readonly type: 'Added' | 'CuratorProposed' | 'Active' | 'PendingPayout'
  }

  /** @name PalletChildBountiesError (565) */
  interface PalletChildBountiesError extends Enum {
    readonly isParentBountyNotActive: boolean
    readonly isInsufficientBountyBalance: boolean
    readonly isTooManyChildBounties: boolean
    readonly type:
      | 'ParentBountyNotActive'
      | 'InsufficientBountyBalance'
      | 'TooManyChildBounties'
  }

  /** @name PalletBagsListListNode (566) */
  interface PalletBagsListListNode extends Struct {
    readonly id: AccountId32
    readonly prev: Option<AccountId32>
    readonly next: Option<AccountId32>
    readonly bagUpper: u64
    readonly score: u64
  }

  /** @name PalletBagsListListBag (567) */
  interface PalletBagsListListBag extends Struct {
    readonly head: Option<AccountId32>
    readonly tail: Option<AccountId32>
  }

  /** @name PalletBagsListError (569) */
  interface PalletBagsListError extends Enum {
    readonly isList: boolean
    readonly asList: PalletBagsListListListError
    readonly type: 'List'
  }

  /** @name PalletBagsListListListError (570) */
  interface PalletBagsListListListError extends Enum {
    readonly isDuplicate: boolean
    readonly isNotHeavier: boolean
    readonly isNotInSameBag: boolean
    readonly isNodeNotFound: boolean
    readonly type: 'Duplicate' | 'NotHeavier' | 'NotInSameBag' | 'NodeNotFound'
  }

  /** @name PalletNominationPoolsPoolMember (571) */
  interface PalletNominationPoolsPoolMember extends Struct {
    readonly poolId: u32
    readonly points: u128
    readonly lastRecordedRewardCounter: u128
    readonly unbondingEras: BTreeMap<u32, u128>
  }

  /** @name PalletNominationPoolsBondedPoolInner (576) */
  interface PalletNominationPoolsBondedPoolInner extends Struct {
    readonly commission: PalletNominationPoolsCommission
    readonly memberCounter: u32
    readonly points: u128
    readonly roles: PalletNominationPoolsPoolRoles
    readonly state: PalletNominationPoolsPoolState
  }

  /** @name PalletNominationPoolsCommission (577) */
  interface PalletNominationPoolsCommission extends Struct {
    readonly current: Option<ITuple<[Perbill, AccountId32]>>
    readonly max: Option<Perbill>
    readonly changeRate: Option<PalletNominationPoolsCommissionChangeRate>
    readonly throttleFrom: Option<u64>
    readonly claimPermission: Option<PalletNominationPoolsCommissionClaimPermission>
  }

  /** @name PalletNominationPoolsPoolRoles (580) */
  interface PalletNominationPoolsPoolRoles extends Struct {
    readonly depositor: AccountId32
    readonly root: Option<AccountId32>
    readonly nominator: Option<AccountId32>
    readonly bouncer: Option<AccountId32>
  }

  /** @name PalletNominationPoolsRewardPool (581) */
  interface PalletNominationPoolsRewardPool extends Struct {
    readonly lastRecordedRewardCounter: u128
    readonly lastRecordedTotalPayouts: u128
    readonly totalRewardsClaimed: u128
    readonly totalCommissionPending: u128
    readonly totalCommissionClaimed: u128
  }

  /** @name PalletNominationPoolsSubPools (582) */
  interface PalletNominationPoolsSubPools extends Struct {
    readonly noEra: PalletNominationPoolsUnbondPool
    readonly withEra: BTreeMap<u32, PalletNominationPoolsUnbondPool>
  }

  /** @name PalletNominationPoolsUnbondPool (583) */
  interface PalletNominationPoolsUnbondPool extends Struct {
    readonly points: u128
    readonly balance: u128
  }

  /** @name PalletNominationPoolsError (589) */
  interface PalletNominationPoolsError extends Enum {
    readonly isPoolNotFound: boolean
    readonly isPoolMemberNotFound: boolean
    readonly isRewardPoolNotFound: boolean
    readonly isSubPoolsNotFound: boolean
    readonly isAccountBelongsToOtherPool: boolean
    readonly isFullyUnbonding: boolean
    readonly isMaxUnbondingLimit: boolean
    readonly isCannotWithdrawAny: boolean
    readonly isMinimumBondNotMet: boolean
    readonly isOverflowRisk: boolean
    readonly isNotDestroying: boolean
    readonly isNotNominator: boolean
    readonly isNotKickerOrDestroying: boolean
    readonly isNotOpen: boolean
    readonly isMaxPools: boolean
    readonly isMaxPoolMembers: boolean
    readonly isCanNotChangeState: boolean
    readonly isDoesNotHavePermission: boolean
    readonly isMetadataExceedsMaxLen: boolean
    readonly isDefensive: boolean
    readonly asDefensive: PalletNominationPoolsDefensiveError
    readonly isPartialUnbondNotAllowedPermissionlessly: boolean
    readonly isMaxCommissionRestricted: boolean
    readonly isCommissionExceedsMaximum: boolean
    readonly isCommissionExceedsGlobalMaximum: boolean
    readonly isCommissionChangeThrottled: boolean
    readonly isCommissionChangeRateNotAllowed: boolean
    readonly isNoPendingCommission: boolean
    readonly isNoCommissionCurrentSet: boolean
    readonly isPoolIdInUse: boolean
    readonly isInvalidPoolId: boolean
    readonly isBondExtraRestricted: boolean
    readonly isNothingToAdjust: boolean
    readonly type:
      | 'PoolNotFound'
      | 'PoolMemberNotFound'
      | 'RewardPoolNotFound'
      | 'SubPoolsNotFound'
      | 'AccountBelongsToOtherPool'
      | 'FullyUnbonding'
      | 'MaxUnbondingLimit'
      | 'CannotWithdrawAny'
      | 'MinimumBondNotMet'
      | 'OverflowRisk'
      | 'NotDestroying'
      | 'NotNominator'
      | 'NotKickerOrDestroying'
      | 'NotOpen'
      | 'MaxPools'
      | 'MaxPoolMembers'
      | 'CanNotChangeState'
      | 'DoesNotHavePermission'
      | 'MetadataExceedsMaxLen'
      | 'Defensive'
      | 'PartialUnbondNotAllowedPermissionlessly'
      | 'MaxCommissionRestricted'
      | 'CommissionExceedsMaximum'
      | 'CommissionExceedsGlobalMaximum'
      | 'CommissionChangeThrottled'
      | 'CommissionChangeRateNotAllowed'
      | 'NoPendingCommission'
      | 'NoCommissionCurrentSet'
      | 'PoolIdInUse'
      | 'InvalidPoolId'
      | 'BondExtraRestricted'
      | 'NothingToAdjust'
  }

  /** @name PalletNominationPoolsDefensiveError (590) */
  interface PalletNominationPoolsDefensiveError extends Enum {
    readonly isNotEnoughSpaceInUnbondPool: boolean
    readonly isPoolNotFound: boolean
    readonly isRewardPoolNotFound: boolean
    readonly isSubPoolsNotFound: boolean
    readonly isBondedStashKilledPrematurely: boolean
    readonly type:
      | 'NotEnoughSpaceInUnbondPool'
      | 'PoolNotFound'
      | 'RewardPoolNotFound'
      | 'SubPoolsNotFound'
      | 'BondedStashKilledPrematurely'
  }

  /** @name PalletSchedulerScheduled (593) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>
    readonly priority: u8
    readonly call: FrameSupportPreimagesBounded
    readonly maybePeriodic: Option<ITuple<[u64, u32]>>
    readonly origin: TangleTestnetRuntimeOriginCaller
  }

  /** @name PalletSchedulerError (595) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean
    readonly isNotFound: boolean
    readonly isTargetBlockNumberInPast: boolean
    readonly isRescheduleNoChange: boolean
    readonly isNamed: boolean
    readonly type:
      | 'FailedToSchedule'
      | 'NotFound'
      | 'TargetBlockNumberInPast'
      | 'RescheduleNoChange'
      | 'Named'
  }

  /** @name PalletPreimageOldRequestStatus (596) */
  interface PalletPreimageOldRequestStatus extends Enum {
    readonly isUnrequested: boolean
    readonly asUnrequested: {
      readonly deposit: ITuple<[AccountId32, u128]>
      readonly len: u32
    } & Struct
    readonly isRequested: boolean
    readonly asRequested: {
      readonly deposit: Option<ITuple<[AccountId32, u128]>>
      readonly count: u32
      readonly len: Option<u32>
    } & Struct
    readonly type: 'Unrequested' | 'Requested'
  }

  /** @name PalletPreimageRequestStatus (598) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean
    readonly asUnrequested: {
      readonly ticket: ITuple<[AccountId32, Null]>
      readonly len: u32
    } & Struct
    readonly isRequested: boolean
    readonly asRequested: {
      readonly maybeTicket: Option<ITuple<[AccountId32, Null]>>
      readonly count: u32
      readonly maybeLen: Option<u32>
    } & Struct
    readonly type: 'Unrequested' | 'Requested'
  }

  /** @name PalletPreimageError (602) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean
    readonly isAlreadyNoted: boolean
    readonly isNotAuthorized: boolean
    readonly isNotNoted: boolean
    readonly isRequested: boolean
    readonly isNotRequested: boolean
    readonly isTooMany: boolean
    readonly isTooFew: boolean
    readonly type:
      | 'TooBig'
      | 'AlreadyNoted'
      | 'NotAuthorized'
      | 'NotNoted'
      | 'Requested'
      | 'NotRequested'
      | 'TooMany'
      | 'TooFew'
  }

  /** @name SpStakingOffenceOffenceDetails (603) */
  interface SpStakingOffenceOffenceDetails extends Struct {
    readonly offender: ITuple<[AccountId32, SpStakingExposure]>
    readonly reporters: Vec<AccountId32>
  }

  /** @name PalletTransactionPauseModuleError (605) */
  interface PalletTransactionPauseModuleError extends Enum {
    readonly isCannotPause: boolean
    readonly isInvalidCharacter: boolean
    readonly type: 'CannotPause' | 'InvalidCharacter'
  }

  /** @name PalletImOnlineError (608) */
  interface PalletImOnlineError extends Enum {
    readonly isInvalidKey: boolean
    readonly isDuplicatedHeartbeat: boolean
    readonly type: 'InvalidKey' | 'DuplicatedHeartbeat'
  }

  /** @name PalletIdentityRegistration (610) */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>
    readonly deposit: u128
    readonly info: PalletIdentityLegacyIdentityInfo
  }

  /** @name PalletIdentityRegistrarInfo (619) */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId32
    readonly fee: u128
    readonly fields: u64
  }

  /** @name PalletIdentityAuthorityProperties (621) */
  interface PalletIdentityAuthorityProperties extends Struct {
    readonly suffix: Bytes
    readonly allocation: u32
  }

  /** @name PalletIdentityError (624) */
  interface PalletIdentityError extends Enum {
    readonly isTooManySubAccounts: boolean
    readonly isNotFound: boolean
    readonly isNotNamed: boolean
    readonly isEmptyIndex: boolean
    readonly isFeeChanged: boolean
    readonly isNoIdentity: boolean
    readonly isStickyJudgement: boolean
    readonly isJudgementGiven: boolean
    readonly isInvalidJudgement: boolean
    readonly isInvalidIndex: boolean
    readonly isInvalidTarget: boolean
    readonly isTooManyRegistrars: boolean
    readonly isAlreadyClaimed: boolean
    readonly isNotSub: boolean
    readonly isNotOwned: boolean
    readonly isJudgementForDifferentIdentity: boolean
    readonly isJudgementPaymentFailed: boolean
    readonly isInvalidSuffix: boolean
    readonly isNotUsernameAuthority: boolean
    readonly isNoAllocation: boolean
    readonly isInvalidSignature: boolean
    readonly isRequiresSignature: boolean
    readonly isInvalidUsername: boolean
    readonly isUsernameTaken: boolean
    readonly isNoUsername: boolean
    readonly isNotExpired: boolean
    readonly type:
      | 'TooManySubAccounts'
      | 'NotFound'
      | 'NotNamed'
      | 'EmptyIndex'
      | 'FeeChanged'
      | 'NoIdentity'
      | 'StickyJudgement'
      | 'JudgementGiven'
      | 'InvalidJudgement'
      | 'InvalidIndex'
      | 'InvalidTarget'
      | 'TooManyRegistrars'
      | 'AlreadyClaimed'
      | 'NotSub'
      | 'NotOwned'
      | 'JudgementForDifferentIdentity'
      | 'JudgementPaymentFailed'
      | 'InvalidSuffix'
      | 'NotUsernameAuthority'
      | 'NoAllocation'
      | 'InvalidSignature'
      | 'RequiresSignature'
      | 'InvalidUsername'
      | 'UsernameTaken'
      | 'NoUsername'
      | 'NotExpired'
  }

  /** @name PalletUtilityError (625) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean
    readonly type: 'TooManyCalls'
  }

  /** @name PalletMultisigMultisig (627) */
  interface PalletMultisigMultisig extends Struct {
    readonly when: PalletMultisigTimepoint
    readonly deposit: u128
    readonly depositor: AccountId32
    readonly approvals: Vec<AccountId32>
  }

  /** @name PalletMultisigError (628) */
  interface PalletMultisigError extends Enum {
    readonly isMinimumThreshold: boolean
    readonly isAlreadyApproved: boolean
    readonly isNoApprovalsNeeded: boolean
    readonly isTooFewSignatories: boolean
    readonly isTooManySignatories: boolean
    readonly isSignatoriesOutOfOrder: boolean
    readonly isSenderInSignatories: boolean
    readonly isNotFound: boolean
    readonly isNotOwner: boolean
    readonly isNoTimepoint: boolean
    readonly isWrongTimepoint: boolean
    readonly isUnexpectedTimepoint: boolean
    readonly isMaxWeightTooLow: boolean
    readonly isAlreadyStored: boolean
    readonly type:
      | 'MinimumThreshold'
      | 'AlreadyApproved'
      | 'NoApprovalsNeeded'
      | 'TooFewSignatories'
      | 'TooManySignatories'
      | 'SignatoriesOutOfOrder'
      | 'SenderInSignatories'
      | 'NotFound'
      | 'NotOwner'
      | 'NoTimepoint'
      | 'WrongTimepoint'
      | 'UnexpectedTimepoint'
      | 'MaxWeightTooLow'
      | 'AlreadyStored'
  }

  /** @name FpRpcTransactionStatus (631) */
  interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256
    readonly transactionIndex: u32
    readonly from: H160
    readonly to: Option<H160>
    readonly contractAddress: Option<H160>
    readonly logs: Vec<EthereumLog>
    readonly logsBloom: EthbloomBloom
  }

  /** @name EthbloomBloom (634) */
  interface EthbloomBloom extends U8aFixed {}

  /** @name EthereumReceiptReceiptV3 (636) */
  interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean
    readonly asLegacy: EthereumReceiptEip658ReceiptData
    readonly isEip2930: boolean
    readonly asEip2930: EthereumReceiptEip658ReceiptData
    readonly isEip1559: boolean
    readonly asEip1559: EthereumReceiptEip658ReceiptData
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559'
  }

  /** @name EthereumReceiptEip658ReceiptData (637) */
  interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8
    readonly usedGas: U256
    readonly logsBloom: EthbloomBloom
    readonly logs: Vec<EthereumLog>
  }

  /** @name EthereumBlock (638) */
  interface EthereumBlock extends Struct {
    readonly header: EthereumHeader
    readonly transactions: Vec<EthereumTransactionTransactionV2>
    readonly ommers: Vec<EthereumHeader>
  }

  /** @name EthereumHeader (639) */
  interface EthereumHeader extends Struct {
    readonly parentHash: H256
    readonly ommersHash: H256
    readonly beneficiary: H160
    readonly stateRoot: H256
    readonly transactionsRoot: H256
    readonly receiptsRoot: H256
    readonly logsBloom: EthbloomBloom
    readonly difficulty: U256
    readonly number: U256
    readonly gasLimit: U256
    readonly gasUsed: U256
    readonly timestamp: u64
    readonly extraData: Bytes
    readonly mixHash: H256
    readonly nonce: EthereumTypesHashH64
  }

  /** @name EthereumTypesHashH64 (640) */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name PalletEthereumError (645) */
  interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean
    readonly isPreLogExists: boolean
    readonly type: 'InvalidSignature' | 'PreLogExists'
  }

  /** @name PalletEvmCodeMetadata (646) */
  interface PalletEvmCodeMetadata extends Struct {
    readonly size_: u64
    readonly hash_: H256
  }

  /** @name PalletEvmError (648) */
  interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean
    readonly isFeeOverflow: boolean
    readonly isPaymentOverflow: boolean
    readonly isWithdrawFailed: boolean
    readonly isGasPriceTooLow: boolean
    readonly isInvalidNonce: boolean
    readonly isGasLimitTooLow: boolean
    readonly isGasLimitTooHigh: boolean
    readonly isInvalidChainId: boolean
    readonly isInvalidSignature: boolean
    readonly isReentrancy: boolean
    readonly isTransactionMustComeFromEOA: boolean
    readonly isUndefined: boolean
    readonly type:
      | 'BalanceLow'
      | 'FeeOverflow'
      | 'PaymentOverflow'
      | 'WithdrawFailed'
      | 'GasPriceTooLow'
      | 'InvalidNonce'
      | 'GasLimitTooLow'
      | 'GasLimitTooHigh'
      | 'InvalidChainId'
      | 'InvalidSignature'
      | 'Reentrancy'
      | 'TransactionMustComeFromEOA'
      | 'Undefined'
  }

  /** @name PalletHotfixSufficientsError (649) */
  interface PalletHotfixSufficientsError extends Enum {
    readonly isMaxAddressCountExceeded: boolean
    readonly type: 'MaxAddressCountExceeded'
  }

  /** @name PalletAirdropClaimsError (651) */
  interface PalletAirdropClaimsError extends Enum {
    readonly isInvalidEthereumSignature: boolean
    readonly isInvalidNativeSignature: boolean
    readonly isInvalidNativeAccount: boolean
    readonly isSignerHasNoClaim: boolean
    readonly isSenderHasNoClaim: boolean
    readonly isPotUnderflow: boolean
    readonly isInvalidStatement: boolean
    readonly isVestedBalanceExists: boolean
    readonly type:
      | 'InvalidEthereumSignature'
      | 'InvalidNativeSignature'
      | 'InvalidNativeAccount'
      | 'SignerHasNoClaim'
      | 'SenderHasNoClaim'
      | 'PotUnderflow'
      | 'InvalidStatement'
      | 'VestedBalanceExists'
  }

  /** @name PalletRolesRestakingLedger (652) */
  interface PalletRolesRestakingLedger extends Struct {
    readonly stash: AccountId32
    readonly total: Compact<u128>
    readonly profile: PalletRolesProfile
    readonly roles: BTreeMap<
      TanglePrimitivesRolesRoleType,
      PalletRolesProfileRecord
    >
    readonly roleKey: Bytes
    readonly unlocking: Vec<PalletRolesUnlockChunk>
    readonly claimedRewards: Vec<u32>
    readonly maxActiveServices: u32
  }

  /** @name PalletRolesUnlockChunk (658) */
  interface PalletRolesUnlockChunk extends Struct {
    readonly value: Compact<u128>
    readonly era: Compact<u32>
  }

  /** @name PalletRolesError (662) */
  interface PalletRolesError extends Enum {
    readonly isNotValidator: boolean
    readonly isHasRoleAssigned: boolean
    readonly isRoleNotAssigned: boolean
    readonly isMaxRoles: boolean
    readonly isRoleCannotBeRemoved: boolean
    readonly isRestakingAmountCannotBeUpdated: boolean
    readonly isExceedsMaxRestakeValue: boolean
    readonly isInsufficientRestakingBond: boolean
    readonly isProfileUpdateFailed: boolean
    readonly isProfileAlreadyExists: boolean
    readonly isNoProfileFound: boolean
    readonly isProfileDeleteRequestFailed: boolean
    readonly isSessionKeysNotProvided: boolean
    readonly isKeySizeExceeded: boolean
    readonly isCannotGetCurrentEra: boolean
    readonly isInvalidEraToReward: boolean
    readonly isBoundNotMet: boolean
    readonly isAlreadyClaimed: boolean
    readonly isNoMoreChunks: boolean
    readonly type:
      | 'NotValidator'
      | 'HasRoleAssigned'
      | 'RoleNotAssigned'
      | 'MaxRoles'
      | 'RoleCannotBeRemoved'
      | 'RestakingAmountCannotBeUpdated'
      | 'ExceedsMaxRestakeValue'
      | 'InsufficientRestakingBond'
      | 'ProfileUpdateFailed'
      | 'ProfileAlreadyExists'
      | 'NoProfileFound'
      | 'ProfileDeleteRequestFailed'
      | 'SessionKeysNotProvided'
      | 'KeySizeExceeded'
      | 'CannotGetCurrentEra'
      | 'InvalidEraToReward'
      | 'BoundNotMet'
      | 'AlreadyClaimed'
      | 'NoMoreChunks'
  }

  /** @name TanglePrimitivesJobsPhaseResult (663) */
  interface TanglePrimitivesJobsPhaseResult extends Struct {
    readonly owner: AccountId32
    readonly result: TanglePrimitivesJobsJobResult
    readonly ttl: u64
    readonly permittedCaller: Option<AccountId32>
    readonly jobType: TanglePrimitivesJobsJobType
  }

  /** @name PalletJobsModuleError (665) */
  interface PalletJobsModuleError extends Enum {
    readonly isInvalidJobPhase: boolean
    readonly isInvalidValidator: boolean
    readonly isInvalidJobParams: boolean
    readonly isPreviousResultNotFound: boolean
    readonly isResultExpired: boolean
    readonly isJobAlreadyExpired: boolean
    readonly isJobNotFound: boolean
    readonly isPhaseOneResultNotFound: boolean
    readonly isNoRewards: boolean
    readonly isNotEnoughValidators: boolean
    readonly isEmptyResult: boolean
    readonly isEmptyJob: boolean
    readonly isValidatorRoleKeyNotFound: boolean
    readonly isResultNotExpectedType: boolean
    readonly isNoPermission: boolean
    readonly isTooManyParticipants: boolean
    readonly isExceedsMaxKeySize: boolean
    readonly isTooManyJobsForValidator: boolean
    readonly type:
      | 'InvalidJobPhase'
      | 'InvalidValidator'
      | 'InvalidJobParams'
      | 'PreviousResultNotFound'
      | 'ResultExpired'
      | 'JobAlreadyExpired'
      | 'JobNotFound'
      | 'PhaseOneResultNotFound'
      | 'NoRewards'
      | 'NotEnoughValidators'
      | 'EmptyResult'
      | 'EmptyJob'
      | 'ValidatorRoleKeyNotFound'
      | 'ResultNotExpectedType'
      | 'NoPermission'
      | 'TooManyParticipants'
      | 'ExceedsMaxKeySize'
      | 'TooManyJobsForValidator'
  }

  /** @name PalletDkgError (666) */
  interface PalletDkgError extends Enum {
    readonly isCannotRetreiveSigner: boolean
    readonly isNotEnoughSigners: boolean
    readonly isInvalidSignatureData: boolean
    readonly isNoParticipantsFound: boolean
    readonly isNoSignaturesFound: boolean
    readonly isInvalidJobType: boolean
    readonly isDuplicateSignature: boolean
    readonly isInvalidSignature: boolean
    readonly isInvalidSignatureScheme: boolean
    readonly isInvalidSignatureDeserialization: boolean
    readonly isInvalidVerifyingKey: boolean
    readonly isInvalidVerifyingKeyDeserialization: boolean
    readonly isSigningKeyMismatch: boolean
    readonly isInvalidParticipantPublicKey: boolean
    readonly isInvalidBlsPublicKey: boolean
    readonly isInvalidRoleType: boolean
    readonly isInvalidJustification: boolean
    readonly isMalformedRoundMessage: boolean
    readonly isNotSignedByOffender: boolean
    readonly isValidDecommitment: boolean
    readonly isValidDataSize: boolean
    readonly isValidFeldmanVerification: boolean
    readonly isValidSchnorrProof: boolean
    readonly isValidRingPedersenParameters: boolean
    readonly isValidModProof: boolean
    readonly isValidFrostSignatureShare: boolean
    readonly isInvalidFrostMessageSerialization: boolean
    readonly isInvalidFrostMessageDeserialization: boolean
    readonly isInvalidIdentifierDeserialization: boolean
    readonly isValidFrostSignature: boolean
    readonly isUnknownIdentifier: boolean
    readonly isDuplicateIdentifier: boolean
    readonly isIncorrectNumberOfIdentifiers: boolean
    readonly isIdentifierDerivationNotSupported: boolean
    readonly isMalformedFrostSignature: boolean
    readonly isInvalidFrostSignature: boolean
    readonly isInvalidFrostSignatureShare: boolean
    readonly isInvalidFrostSignatureScheme: boolean
    readonly isMalformedFrostVerifyingKey: boolean
    readonly isMalformedFrostSigningKey: boolean
    readonly isMissingFrostCommitment: boolean
    readonly isIdentityCommitment: boolean
    readonly isFrostFieldError: boolean
    readonly isFrostGroupError: boolean
    readonly type:
      | 'CannotRetreiveSigner'
      | 'NotEnoughSigners'
      | 'InvalidSignatureData'
      | 'NoParticipantsFound'
      | 'NoSignaturesFound'
      | 'InvalidJobType'
      | 'DuplicateSignature'
      | 'InvalidSignature'
      | 'InvalidSignatureScheme'
      | 'InvalidSignatureDeserialization'
      | 'InvalidVerifyingKey'
      | 'InvalidVerifyingKeyDeserialization'
      | 'SigningKeyMismatch'
      | 'InvalidParticipantPublicKey'
      | 'InvalidBlsPublicKey'
      | 'InvalidRoleType'
      | 'InvalidJustification'
      | 'MalformedRoundMessage'
      | 'NotSignedByOffender'
      | 'ValidDecommitment'
      | 'ValidDataSize'
      | 'ValidFeldmanVerification'
      | 'ValidSchnorrProof'
      | 'ValidRingPedersenParameters'
      | 'ValidModProof'
      | 'ValidFrostSignatureShare'
      | 'InvalidFrostMessageSerialization'
      | 'InvalidFrostMessageDeserialization'
      | 'InvalidIdentifierDeserialization'
      | 'ValidFrostSignature'
      | 'UnknownIdentifier'
      | 'DuplicateIdentifier'
      | 'IncorrectNumberOfIdentifiers'
      | 'IdentifierDerivationNotSupported'
      | 'MalformedFrostSignature'
      | 'InvalidFrostSignature'
      | 'InvalidFrostSignatureShare'
      | 'InvalidFrostSignatureScheme'
      | 'MalformedFrostVerifyingKey'
      | 'MalformedFrostSigningKey'
      | 'MissingFrostCommitment'
      | 'IdentityCommitment'
      | 'FrostFieldError'
      | 'FrostGroupError'
  }

  /** @name PalletZksaasError (667) */
  interface PalletZksaasError extends Enum {
    readonly isInvalidJobType: boolean
    readonly isInvalidProof: boolean
    readonly isMalformedProof: boolean
    readonly type: 'InvalidJobType' | 'InvalidProof' | 'MalformedProof'
  }

  /** @name FrameSystemExtensionsCheckNonZeroSender (670) */
  type FrameSystemExtensionsCheckNonZeroSender = Null

  /** @name FrameSystemExtensionsCheckSpecVersion (671) */
  type FrameSystemExtensionsCheckSpecVersion = Null

  /** @name FrameSystemExtensionsCheckTxVersion (672) */
  type FrameSystemExtensionsCheckTxVersion = Null

  /** @name FrameSystemExtensionsCheckGenesis (673) */
  type FrameSystemExtensionsCheckGenesis = Null

  /** @name FrameSystemExtensionsCheckNonce (676) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (677) */
  type FrameSystemExtensionsCheckWeight = Null

  /** @name PalletTransactionPaymentChargeTransactionPayment (678) */
  interface PalletTransactionPaymentChargeTransactionPayment
    extends Compact<u128> {}

  /** @name TangleTestnetRuntimeRuntime (680) */
  type TangleTestnetRuntimeRuntime = Null
} // declare module
