// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/submittable'

import type {
  ApiTypes,
  AugmentedSubmittable,
  SubmittableExtrinsic,
  SubmittableExtrinsicFunction,
} from '@polkadot/api-base/types'
import type { Data } from '@polkadot/types'
import type {
  Bytes,
  Compact,
  Null,
  Option,
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
import type { AnyNumber, IMethod, ITuple } from '@polkadot/types-codec/types'
import type {
  AccountId32,
  Call,
  H160,
  H256,
  MultiAddress,
  Perbill,
  Percent,
  Permill,
} from '@polkadot/types/interfaces/runtime'
import {
  SpConsensusBabeDigestsNextConfigDescriptor,
  SpConsensusSlotsEquivocationProof,
  SpSessionMembershipProof,
  PalletBalancesAdjustmentDirection,
  PalletAirdropClaimsUtilsMultiAddress,
  PalletAirdropClaimsUtilsMultiAddressSignature,
  PalletAirdropClaimsStatementKind,
  SpWeightsWeightV2Weight,
  PalletDemocracyConviction,
  FrameSupportPreimagesBounded,
  PalletDemocracyMetadataOwner,
  PalletDemocracyVoteAccountVote,
  PalletDkgFeeInfo,
  SpNposElectionsSupport,
  SpNposElectionsElectionScore,
  PalletElectionProviderMultiPhaseRawSolution,
  PalletElectionProviderMultiPhaseSolutionOrSnapshotSize,
  PalletElectionsPhragmenRenouncing,
  EthereumTransactionTransactionV2,
  SpConsensusGrandpaEquivocationProof,
  SpCoreVoid,
  PalletIdentityJudgement,
  PalletIdentityLegacyIdentityInfo,
  SpRuntimeMultiSignature,
  PalletImOnlineHeartbeat,
  PalletImOnlineSr25519AppSr25519Signature,
  TanglePrimitivesRolesRoleType,
  TanglePrimitivesJobsValidatorOffenceType,
  TanglePrimitivesJobsJobSubmission,
  TanglePrimitivesJobsJobResult,
  TanglePrimitivesMisbehaviorMisbehaviorSubmission,
  PalletMultisigTimepoint,
  PalletNominationPoolsBondExtra,
  PalletNominationPoolsClaimPermission,
  PalletNominationPoolsCommissionChangeRate,
  PalletNominationPoolsCommissionClaimPermission,
  PalletNominationPoolsConfigOpU128,
  PalletNominationPoolsConfigOpU32,
  PalletNominationPoolsConfigOpPerbill,
  PalletNominationPoolsPoolState,
  PalletNominationPoolsConfigOpAccountId32,
  PalletRolesProfile,
  TangleTestnetRuntimeOpaqueSessionKeys,
  PalletStakingRewardDestination,
  PalletStakingPalletConfigOpU128,
  PalletStakingPalletConfigOpU32,
  PalletStakingPalletConfigOpPercent,
  PalletStakingPalletConfigOpPerbill,
  PalletStakingValidatorPrefs,
  TangleTestnetRuntimeOriginCaller,
  PalletVestingVestingInfo,
  PalletZksaasFeeInfo,
} from '@polkadot/types/lookup'

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>
export type __SubmittableExtrinsic<ApiType extends ApiTypes> =
  SubmittableExtrinsic<ApiType>
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> =
  SubmittableExtrinsicFunction<ApiType>

declare module '@polkadot/api-base/types/submittable' {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    babe: {
      /**
       * See [`Pallet::plan_config_change`].
       **/
      planConfigChange: AugmentedSubmittable<
        (
          config:
            | SpConsensusBabeDigestsNextConfigDescriptor
            | { V1: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusBabeDigestsNextConfigDescriptor]
      >
      /**
       * See [`Pallet::report_equivocation`].
       **/
      reportEquivocation: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusSlotsEquivocationProof
            | {
                offender?: any
                slot?: any
                firstHeader?: any
                secondHeader?: any
              }
            | string
            | Uint8Array,
          keyOwnerProof:
            | SpSessionMembershipProof
            | { session?: any; trieNodes?: any; validatorCount?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]
      >
      /**
       * See [`Pallet::report_equivocation_unsigned`].
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusSlotsEquivocationProof
            | {
                offender?: any
                slot?: any
                firstHeader?: any
                secondHeader?: any
              }
            | string
            | Uint8Array,
          keyOwnerProof:
            | SpSessionMembershipProof
            | { session?: any; trieNodes?: any; validatorCount?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    bagsList: {
      /**
       * See [`Pallet::put_in_front_of`].
       **/
      putInFrontOf: AugmentedSubmittable<
        (
          lighter:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::put_in_front_of_other`].
       **/
      putInFrontOfOther: AugmentedSubmittable<
        (
          heavier:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          lighter:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, MultiAddress]
      >
      /**
       * See [`Pallet::rebag`].
       **/
      rebag: AugmentedSubmittable<
        (
          dislocated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    balances: {
      /**
       * See [`Pallet::force_adjust_total_issuance`].
       **/
      forceAdjustTotalIssuance: AugmentedSubmittable<
        (
          direction:
            | PalletBalancesAdjustmentDirection
            | 'Increase'
            | 'Decrease'
            | number
            | Uint8Array,
          delta: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletBalancesAdjustmentDirection, Compact<u128>]
      >
      /**
       * See [`Pallet::force_set_balance`].
       **/
      forceSetBalance: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          newFree: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::force_transfer`].
       **/
      forceTransfer: AugmentedSubmittable<
        (
          source:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::force_unreserve`].
       **/
      forceUnreserve: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u128]
      >
      /**
       * See [`Pallet::transfer_all`].
       **/
      transferAll: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          keepAlive: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, bool]
      >
      /**
       * See [`Pallet::transfer_allow_death`].
       **/
      transferAllowDeath: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::transfer_keep_alive`].
       **/
      transferKeepAlive: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::upgrade_accounts`].
       **/
      upgradeAccounts: AugmentedSubmittable<
        (
          who: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    baseFee: {
      /**
       * See [`Pallet::set_base_fee_per_gas`].
       **/
      setBaseFeePerGas: AugmentedSubmittable<
        (fee: U256 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U256]
      >
      /**
       * See [`Pallet::set_elasticity`].
       **/
      setElasticity: AugmentedSubmittable<
        (
          elasticity: Permill | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Permill]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    bounties: {
      /**
       * See [`Pallet::accept_curator`].
       **/
      acceptCurator: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::approve_bounty`].
       **/
      approveBounty: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::award_bounty`].
       **/
      awardBounty: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array,
          beneficiary:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, MultiAddress]
      >
      /**
       * See [`Pallet::claim_bounty`].
       **/
      claimBounty: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::close_bounty`].
       **/
      closeBounty: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::extend_bounty_expiry`].
       **/
      extendBountyExpiry: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array,
          remark: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Bytes]
      >
      /**
       * See [`Pallet::propose_bounty`].
       **/
      proposeBounty: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          description: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, Bytes]
      >
      /**
       * See [`Pallet::propose_curator`].
       **/
      proposeCurator: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array,
          curator:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          fee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::unassign_curator`].
       **/
      unassignCurator: AugmentedSubmittable<
        (
          bountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    childBounties: {
      /**
       * See [`Pallet::accept_curator`].
       **/
      acceptCurator: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>]
      >
      /**
       * See [`Pallet::add_child_bounty`].
       **/
      addChildBounty: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array,
          description: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>, Bytes]
      >
      /**
       * See [`Pallet::award_child_bounty`].
       **/
      awardChildBounty: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array,
          beneficiary:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>, MultiAddress]
      >
      /**
       * See [`Pallet::claim_child_bounty`].
       **/
      claimChildBounty: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>]
      >
      /**
       * See [`Pallet::close_child_bounty`].
       **/
      closeChildBounty: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>]
      >
      /**
       * See [`Pallet::propose_curator`].
       **/
      proposeCurator: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array,
          curator:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          fee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>, MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::unassign_curator`].
       **/
      unassignCurator: AugmentedSubmittable<
        (
          parentBountyId: Compact<u32> | AnyNumber | Uint8Array,
          childBountyId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u32>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    claims: {
      /**
       * See [`Pallet::claim`].
       **/
      claim: AugmentedSubmittable<
        (
          dest:
            | Option<PalletAirdropClaimsUtilsMultiAddress>
            | null
            | Uint8Array
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string,
          signer:
            | Option<PalletAirdropClaimsUtilsMultiAddress>
            | null
            | Uint8Array
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string,
          signature:
            | PalletAirdropClaimsUtilsMultiAddressSignature
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Option<PalletAirdropClaimsUtilsMultiAddress>,
          Option<PalletAirdropClaimsUtilsMultiAddress>,
          PalletAirdropClaimsUtilsMultiAddressSignature
        ]
      >
      /**
       * See [`Pallet::claim_attest`].
       **/
      claimAttest: AugmentedSubmittable<
        (
          dest:
            | Option<PalletAirdropClaimsUtilsMultiAddress>
            | null
            | Uint8Array
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string,
          signer:
            | Option<PalletAirdropClaimsUtilsMultiAddress>
            | null
            | Uint8Array
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string,
          signature:
            | PalletAirdropClaimsUtilsMultiAddressSignature
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array,
          statement: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          Option<PalletAirdropClaimsUtilsMultiAddress>,
          Option<PalletAirdropClaimsUtilsMultiAddress>,
          PalletAirdropClaimsUtilsMultiAddressSignature,
          Bytes
        ]
      >
      /**
       * See [`Pallet::force_set_expiry_config`].
       **/
      forceSetExpiryConfig: AugmentedSubmittable<
        (
          expiryBlock: u64 | AnyNumber | Uint8Array,
          dest:
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, PalletAirdropClaimsUtilsMultiAddress]
      >
      /**
       * See [`Pallet::mint_claim`].
       **/
      mintClaim: AugmentedSubmittable<
        (
          who:
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array,
          value: u128 | AnyNumber | Uint8Array,
          vestingSchedule:
            | Option<Vec<ITuple<[u128, u128, u64]>>>
            | null
            | Uint8Array
            | Vec<ITuple<[u128, u128, u64]>>
            | [
                u128 | AnyNumber | Uint8Array,
                u128 | AnyNumber | Uint8Array,
                u64 | AnyNumber | Uint8Array
              ][],
          statement:
            | Option<PalletAirdropClaimsStatementKind>
            | null
            | Uint8Array
            | PalletAirdropClaimsStatementKind
            | 'Regular'
            | 'Safe'
            | number
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletAirdropClaimsUtilsMultiAddress,
          u128,
          Option<Vec<ITuple<[u128, u128, u64]>>>,
          Option<PalletAirdropClaimsStatementKind>
        ]
      >
      /**
       * See [`Pallet::move_claim`].
       **/
      moveClaim: AugmentedSubmittable<
        (
          old:
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array,
          updated:
            | PalletAirdropClaimsUtilsMultiAddress
            | { EVM: any }
            | { Native: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletAirdropClaimsUtilsMultiAddress,
          PalletAirdropClaimsUtilsMultiAddress
        ]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    council: {
      /**
       * See [`Pallet::close`].
       **/
      close: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          proposalWeightBound:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, SpWeightsWeightV2Weight, Compact<u32>]
      >
      /**
       * See [`Pallet::disapprove_proposal`].
       **/
      disapproveProposal: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::execute`].
       **/
      execute: AugmentedSubmittable<
        (
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, Compact<u32>]
      >
      /**
       * See [`Pallet::propose`].
       **/
      propose: AugmentedSubmittable<
        (
          threshold: Compact<u32> | AnyNumber | Uint8Array,
          proposal: Call | IMethod | string | Uint8Array,
          lengthBound: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Call, Compact<u32>]
      >
      /**
       * See [`Pallet::set_members`].
       **/
      setMembers: AugmentedSubmittable<
        (
          newMembers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[],
          prime: Option<AccountId32> | null | Uint8Array | AccountId32 | string,
          oldCount: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>, Option<AccountId32>, u32]
      >
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<
        (
          proposal: H256 | string | Uint8Array,
          index: Compact<u32> | AnyNumber | Uint8Array,
          approve: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Compact<u32>, bool]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    democracy: {
      /**
       * See [`Pallet::blacklist`].
       **/
      blacklist: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          maybeRefIndex: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [H256, Option<u32>]
      >
      /**
       * See [`Pallet::cancel_proposal`].
       **/
      cancelProposal: AugmentedSubmittable<
        (
          propIndex: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::cancel_referendum`].
       **/
      cancelReferendum: AugmentedSubmittable<
        (
          refIndex: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::clear_public_proposals`].
       **/
      clearPublicProposals: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * See [`Pallet::delegate`].
       **/
      delegate: AugmentedSubmittable<
        (
          to:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          conviction:
            | PalletDemocracyConviction
            | 'None'
            | 'Locked1x'
            | 'Locked2x'
            | 'Locked3x'
            | 'Locked4x'
            | 'Locked5x'
            | 'Locked6x'
            | number
            | Uint8Array,
          balance: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, PalletDemocracyConviction, u128]
      >
      /**
       * See [`Pallet::emergency_cancel`].
       **/
      emergencyCancel: AugmentedSubmittable<
        (
          refIndex: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::external_propose`].
       **/
      externalPropose: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >
      /**
       * See [`Pallet::external_propose_default`].
       **/
      externalProposeDefault: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >
      /**
       * See [`Pallet::external_propose_majority`].
       **/
      externalProposeMajority: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded]
      >
      /**
       * See [`Pallet::fast_track`].
       **/
      fastTrack: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array,
          votingPeriod: u64 | AnyNumber | Uint8Array,
          delay: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256, u64, u64]
      >
      /**
       * See [`Pallet::propose`].
       **/
      propose: AugmentedSubmittable<
        (
          proposal:
            | FrameSupportPreimagesBounded
            | { Legacy: any }
            | { Inline: any }
            | { Lookup: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [FrameSupportPreimagesBounded, Compact<u128>]
      >
      /**
       * See [`Pallet::remove_other_vote`].
       **/
      removeOtherVote: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u32]
      >
      /**
       * See [`Pallet::remove_vote`].
       **/
      removeVote: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::second`].
       **/
      second: AugmentedSubmittable<
        (
          proposal: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::set_metadata`].
       **/
      setMetadata: AugmentedSubmittable<
        (
          owner:
            | PalletDemocracyMetadataOwner
            | { External: any }
            | { Proposal: any }
            | { Referendum: any }
            | string
            | Uint8Array,
          maybeHash: Option<H256> | null | Uint8Array | H256 | string
        ) => SubmittableExtrinsic<ApiType>,
        [PalletDemocracyMetadataOwner, Option<H256>]
      >
      /**
       * See [`Pallet::undelegate`].
       **/
      undelegate: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::unlock`].
       **/
      unlock: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::veto_external`].
       **/
      vetoExternal: AugmentedSubmittable<
        (
          proposalHash: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<
        (
          refIndex: Compact<u32> | AnyNumber | Uint8Array,
          vote:
            | PalletDemocracyVoteAccountVote
            | { Standard: any }
            | { Split: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, PalletDemocracyVoteAccountVote]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    dkg: {
      /**
       * See [`Pallet::set_fee`].
       **/
      setFee: AugmentedSubmittable<
        (
          feeInfo:
            | PalletDkgFeeInfo
            | {
                baseFee?: any
                dkgValidatorFee?: any
                sigValidatorFee?: any
                refreshValidatorFee?: any
                storageFeePerByte?: any
                storageFeePerBlock?: any
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletDkgFeeInfo]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    dynamicFee: {
      /**
       * See [`Pallet::note_min_gas_price_target`].
       **/
      noteMinGasPriceTarget: AugmentedSubmittable<
        (
          target: U256 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U256]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    electionProviderMultiPhase: {
      /**
       * See [`Pallet::governance_fallback`].
       **/
      governanceFallback: AugmentedSubmittable<
        (
          maybeMaxVoters: Option<u32> | null | Uint8Array | u32 | AnyNumber,
          maybeMaxTargets: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Option<u32>, Option<u32>]
      >
      /**
       * See [`Pallet::set_emergency_election_result`].
       **/
      setEmergencyElectionResult: AugmentedSubmittable<
        (
          supports:
            | Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>
            | [
                AccountId32 | string | Uint8Array,
                (
                  | SpNposElectionsSupport
                  | { total?: any; voters?: any }
                  | string
                  | Uint8Array
                )
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>]
      >
      /**
       * See [`Pallet::set_minimum_untrusted_score`].
       **/
      setMinimumUntrustedScore: AugmentedSubmittable<
        (
          maybeNextScore:
            | Option<SpNposElectionsElectionScore>
            | null
            | Uint8Array
            | SpNposElectionsElectionScore
            | { minimalStake?: any; sumStake?: any; sumStakeSquared?: any }
            | string
        ) => SubmittableExtrinsic<ApiType>,
        [Option<SpNposElectionsElectionScore>]
      >
      /**
       * See [`Pallet::submit`].
       **/
      submit: AugmentedSubmittable<
        (
          rawSolution:
            | PalletElectionProviderMultiPhaseRawSolution
            | { solution?: any; score?: any; round?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletElectionProviderMultiPhaseRawSolution]
      >
      /**
       * See [`Pallet::submit_unsigned`].
       **/
      submitUnsigned: AugmentedSubmittable<
        (
          rawSolution:
            | PalletElectionProviderMultiPhaseRawSolution
            | { solution?: any; score?: any; round?: any }
            | string
            | Uint8Array,
          witness:
            | PalletElectionProviderMultiPhaseSolutionOrSnapshotSize
            | { voters?: any; targets?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletElectionProviderMultiPhaseRawSolution,
          PalletElectionProviderMultiPhaseSolutionOrSnapshotSize
        ]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    elections: {
      /**
       * See [`Pallet::clean_defunct_voters`].
       **/
      cleanDefunctVoters: AugmentedSubmittable<
        (
          numVoters: u32 | AnyNumber | Uint8Array,
          numDefunct: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >
      /**
       * See [`Pallet::remove_member`].
       **/
      removeMember: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          slashBond: bool | boolean | Uint8Array,
          rerunElection: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, bool, bool]
      >
      /**
       * See [`Pallet::remove_voter`].
       **/
      removeVoter: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::renounce_candidacy`].
       **/
      renounceCandidacy: AugmentedSubmittable<
        (
          renouncing:
            | PalletElectionsPhragmenRenouncing
            | { Member: any }
            | { RunnerUp: any }
            | { Candidate: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletElectionsPhragmenRenouncing]
      >
      /**
       * See [`Pallet::submit_candidacy`].
       **/
      submitCandidacy: AugmentedSubmittable<
        (
          candidateCount: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<
        (
          votes: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[],
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>, Compact<u128>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    ethereum: {
      /**
       * See [`Pallet::transact`].
       **/
      transact: AugmentedSubmittable<
        (
          transaction:
            | EthereumTransactionTransactionV2
            | { Legacy: any }
            | { EIP2930: any }
            | { EIP1559: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [EthereumTransactionTransactionV2]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    evm: {
      /**
       * See [`Pallet::call`].
       **/
      call: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          target: H160 | string | Uint8Array,
          input: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas:
            | Option<U256>
            | null
            | Uint8Array
            | U256
            | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [
                H160 | string | Uint8Array,
                Vec<H256> | (H256 | string | Uint8Array)[]
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [
          H160,
          H160,
          Bytes,
          U256,
          u64,
          U256,
          Option<U256>,
          Option<U256>,
          Vec<ITuple<[H160, Vec<H256>]>>
        ]
      >
      /**
       * See [`Pallet::create`].
       **/
      create: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas:
            | Option<U256>
            | null
            | Uint8Array
            | U256
            | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [
                H160 | string | Uint8Array,
                Vec<H256> | (H256 | string | Uint8Array)[]
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [
          H160,
          Bytes,
          U256,
          u64,
          U256,
          Option<U256>,
          Option<U256>,
          Vec<ITuple<[H160, Vec<H256>]>>
        ]
      >
      /**
       * See [`Pallet::create2`].
       **/
      create2: AugmentedSubmittable<
        (
          source: H160 | string | Uint8Array,
          init: Bytes | string | Uint8Array,
          salt: H256 | string | Uint8Array,
          value: U256 | AnyNumber | Uint8Array,
          gasLimit: u64 | AnyNumber | Uint8Array,
          maxFeePerGas: U256 | AnyNumber | Uint8Array,
          maxPriorityFeePerGas:
            | Option<U256>
            | null
            | Uint8Array
            | U256
            | AnyNumber,
          nonce: Option<U256> | null | Uint8Array | U256 | AnyNumber,
          accessList:
            | Vec<ITuple<[H160, Vec<H256>]>>
            | [
                H160 | string | Uint8Array,
                Vec<H256> | (H256 | string | Uint8Array)[]
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [
          H160,
          Bytes,
          H256,
          U256,
          u64,
          U256,
          Option<U256>,
          Option<U256>,
          Vec<ITuple<[H160, Vec<H256>]>>
        ]
      >
      /**
       * See [`Pallet::withdraw`].
       **/
      withdraw: AugmentedSubmittable<
        (
          address: H160 | string | Uint8Array,
          value: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [H160, u128]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    grandpa: {
      /**
       * See [`Pallet::note_stalled`].
       **/
      noteStalled: AugmentedSubmittable<
        (
          delay: u64 | AnyNumber | Uint8Array,
          bestFinalizedBlockNumber: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, u64]
      >
      /**
       * See [`Pallet::report_equivocation`].
       **/
      reportEquivocation: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusGrandpaEquivocationProof
            | { setId?: any; equivocation?: any }
            | string
            | Uint8Array,
          keyOwnerProof: SpCoreVoid | null
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusGrandpaEquivocationProof, SpCoreVoid]
      >
      /**
       * See [`Pallet::report_equivocation_unsigned`].
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusGrandpaEquivocationProof
            | { setId?: any; equivocation?: any }
            | string
            | Uint8Array,
          keyOwnerProof: SpCoreVoid | null
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusGrandpaEquivocationProof, SpCoreVoid]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    hotfixSufficients: {
      /**
       * See [`Pallet::hotfix_inc_account_sufficients`].
       **/
      hotfixIncAccountSufficients: AugmentedSubmittable<
        (
          addresses: Vec<H160> | (H160 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<H160>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    identity: {
      /**
       * See [`Pallet::accept_username`].
       **/
      acceptUsername: AugmentedSubmittable<
        (
          username: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::add_registrar`].
       **/
      addRegistrar: AugmentedSubmittable<
        (
          account:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::add_sub`].
       **/
      addSub: AugmentedSubmittable<
        (
          sub:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          data:
            | Data
            | { None: any }
            | { Raw: any }
            | { BlakeTwo256: any }
            | { Sha256: any }
            | { Keccak256: any }
            | { ShaThree256: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Data]
      >
      /**
       * See [`Pallet::add_username_authority`].
       **/
      addUsernameAuthority: AugmentedSubmittable<
        (
          authority:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          suffix: Bytes | string | Uint8Array,
          allocation: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Bytes, u32]
      >
      /**
       * See [`Pallet::cancel_request`].
       **/
      cancelRequest: AugmentedSubmittable<
        (
          regIndex: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::clear_identity`].
       **/
      clearIdentity: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * See [`Pallet::kill_identity`].
       **/
      killIdentity: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::provide_judgement`].
       **/
      provideJudgement: AugmentedSubmittable<
        (
          regIndex: Compact<u32> | AnyNumber | Uint8Array,
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          judgement:
            | PalletIdentityJudgement
            | { Unknown: any }
            | { FeePaid: any }
            | { Reasonable: any }
            | { KnownGood: any }
            | { OutOfDate: any }
            | { LowQuality: any }
            | { Erroneous: any }
            | string
            | Uint8Array,
          identity: H256 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, MultiAddress, PalletIdentityJudgement, H256]
      >
      /**
       * See [`Pallet::quit_sub`].
       **/
      quitSub: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::remove_dangling_username`].
       **/
      removeDanglingUsername: AugmentedSubmittable<
        (
          username: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::remove_expired_approval`].
       **/
      removeExpiredApproval: AugmentedSubmittable<
        (
          username: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::remove_sub`].
       **/
      removeSub: AugmentedSubmittable<
        (
          sub:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::remove_username_authority`].
       **/
      removeUsernameAuthority: AugmentedSubmittable<
        (
          authority:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::rename_sub`].
       **/
      renameSub: AugmentedSubmittable<
        (
          sub:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          data:
            | Data
            | { None: any }
            | { Raw: any }
            | { BlakeTwo256: any }
            | { Sha256: any }
            | { Keccak256: any }
            | { ShaThree256: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Data]
      >
      /**
       * See [`Pallet::request_judgement`].
       **/
      requestJudgement: AugmentedSubmittable<
        (
          regIndex: Compact<u32> | AnyNumber | Uint8Array,
          maxFee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >
      /**
       * See [`Pallet::set_account_id`].
       **/
      setAccountId: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          updated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, MultiAddress]
      >
      /**
       * See [`Pallet::set_fee`].
       **/
      setFee: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fee: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, Compact<u128>]
      >
      /**
       * See [`Pallet::set_fields`].
       **/
      setFields: AugmentedSubmittable<
        (
          index: Compact<u32> | AnyNumber | Uint8Array,
          fields: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>, u64]
      >
      /**
       * See [`Pallet::set_identity`].
       **/
      setIdentity: AugmentedSubmittable<
        (
          info:
            | PalletIdentityLegacyIdentityInfo
            | {
                additional?: any
                display?: any
                legal?: any
                web?: any
                riot?: any
                email?: any
                pgpFingerprint?: any
                image?: any
                twitter?: any
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletIdentityLegacyIdentityInfo]
      >
      /**
       * See [`Pallet::set_primary_username`].
       **/
      setPrimaryUsername: AugmentedSubmittable<
        (
          username: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::set_subs`].
       **/
      setSubs: AugmentedSubmittable<
        (
          subs:
            | Vec<ITuple<[AccountId32, Data]>>
            | [
                AccountId32 | string | Uint8Array,
                (
                  | Data
                  | { None: any }
                  | { Raw: any }
                  | { BlakeTwo256: any }
                  | { Sha256: any }
                  | { Keccak256: any }
                  | { ShaThree256: any }
                  | string
                  | Uint8Array
                )
              ][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[AccountId32, Data]>>]
      >
      /**
       * See [`Pallet::set_username_for`].
       **/
      setUsernameFor: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          username: Bytes | string | Uint8Array,
          signature:
            | Option<SpRuntimeMultiSignature>
            | null
            | Uint8Array
            | SpRuntimeMultiSignature
            | { Ed25519: any }
            | { Sr25519: any }
            | { Ecdsa: any }
            | string
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Bytes, Option<SpRuntimeMultiSignature>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    imOnline: {
      /**
       * See [`Pallet::heartbeat`].
       **/
      heartbeat: AugmentedSubmittable<
        (
          heartbeat:
            | PalletImOnlineHeartbeat
            | {
                blockNumber?: any
                sessionIndex?: any
                authorityIndex?: any
                validatorsLen?: any
              }
            | string
            | Uint8Array,
          signature:
            | PalletImOnlineSr25519AppSr25519Signature
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    indices: {
      /**
       * See [`Pallet::claim`].
       **/
      claim: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::force_transfer`].
       **/
      forceTransfer: AugmentedSubmittable<
        (
          updated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          index: u32 | AnyNumber | Uint8Array,
          freeze: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u32, bool]
      >
      /**
       * See [`Pallet::free`].
       **/
      free: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::freeze`].
       **/
      freeze: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::transfer`].
       **/
      transfer: AugmentedSubmittable<
        (
          updated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u32]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    jobs: {
      /**
       * See [`Pallet::extend_job_result_ttl`].
       **/
      extendJobResultTtl: AugmentedSubmittable<
        (
          roleType:
            | TanglePrimitivesRolesRoleType
            | { Tss: any }
            | { ZkSaaS: any }
            | { LightClientRelaying: any }
            | string
            | Uint8Array,
          jobId: u64 | AnyNumber | Uint8Array,
          extendBy: u64 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TanglePrimitivesRolesRoleType, u64, u64]
      >
      /**
       * See [`Pallet::report_inactive_validator`].
       **/
      reportInactiveValidator: AugmentedSubmittable<
        (
          roleType:
            | TanglePrimitivesRolesRoleType
            | { Tss: any }
            | { ZkSaaS: any }
            | { LightClientRelaying: any }
            | string
            | Uint8Array,
          jobId: u64 | AnyNumber | Uint8Array,
          validator: AccountId32 | string | Uint8Array,
          offence:
            | TanglePrimitivesJobsValidatorOffenceType
            | 'Inactivity'
            | 'InvalidSignatureSubmitted'
            | 'RejectedValidAction'
            | 'ApprovedInvalidAction'
            | number
            | Uint8Array,
          signatures: Vec<Bytes> | (Bytes | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [
          TanglePrimitivesRolesRoleType,
          u64,
          AccountId32,
          TanglePrimitivesJobsValidatorOffenceType,
          Vec<Bytes>
        ]
      >
      /**
       * See [`Pallet::set_permitted_caller`].
       **/
      setPermittedCaller: AugmentedSubmittable<
        (
          roleType:
            | TanglePrimitivesRolesRoleType
            | { Tss: any }
            | { ZkSaaS: any }
            | { LightClientRelaying: any }
            | string
            | Uint8Array,
          jobId: u64 | AnyNumber | Uint8Array,
          newPermittedCaller: AccountId32 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TanglePrimitivesRolesRoleType, u64, AccountId32]
      >
      /**
       * See [`Pallet::set_time_fee`].
       **/
      setTimeFee: AugmentedSubmittable<
        (
          newFee: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128]
      >
      /**
       * See [`Pallet::submit_job`].
       **/
      submitJob: AugmentedSubmittable<
        (
          job:
            | TanglePrimitivesJobsJobSubmission
            | { expiry?: any; ttl?: any; jobType?: any; fallback?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TanglePrimitivesJobsJobSubmission]
      >
      /**
       * See [`Pallet::submit_job_result`].
       **/
      submitJobResult: AugmentedSubmittable<
        (
          roleType:
            | TanglePrimitivesRolesRoleType
            | { Tss: any }
            | { ZkSaaS: any }
            | { LightClientRelaying: any }
            | string
            | Uint8Array,
          jobId: u64 | AnyNumber | Uint8Array,
          result:
            | TanglePrimitivesJobsJobResult
            | { DKGPhaseOne: any }
            | { DKGPhaseTwo: any }
            | { DKGPhaseThree: any }
            | { DKGPhaseFour: any }
            | { ZkSaaSPhaseOne: any }
            | { ZkSaaSPhaseTwo: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TanglePrimitivesRolesRoleType, u64, TanglePrimitivesJobsJobResult]
      >
      /**
       * See [`Pallet::submit_misbehavior`].
       **/
      submitMisbehavior: AugmentedSubmittable<
        (
          misbehavior:
            | TanglePrimitivesMisbehaviorMisbehaviorSubmission
            | {
                roleType?: any
                offender?: any
                jobId?: any
                justification?: any
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TanglePrimitivesMisbehaviorMisbehaviorSubmission]
      >
      /**
       * See [`Pallet::withdraw_rewards`].
       **/
      withdrawRewards: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    multisig: {
      /**
       * See [`Pallet::approve_as_multi`].
       **/
      approveAsMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories:
            | Vec<AccountId32>
            | (AccountId32 | string | Uint8Array)[],
          maybeTimepoint:
            | Option<PalletMultisigTimepoint>
            | null
            | Uint8Array
            | PalletMultisigTimepoint
            | { height?: any; index?: any }
            | string,
          callHash: U8aFixed | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          u16,
          Vec<AccountId32>,
          Option<PalletMultisigTimepoint>,
          U8aFixed,
          SpWeightsWeightV2Weight
        ]
      >
      /**
       * See [`Pallet::as_multi`].
       **/
      asMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories:
            | Vec<AccountId32>
            | (AccountId32 | string | Uint8Array)[],
          maybeTimepoint:
            | Option<PalletMultisigTimepoint>
            | null
            | Uint8Array
            | PalletMultisigTimepoint
            | { height?: any; index?: any }
            | string,
          call: Call | IMethod | string | Uint8Array,
          maxWeight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          u16,
          Vec<AccountId32>,
          Option<PalletMultisigTimepoint>,
          Call,
          SpWeightsWeightV2Weight
        ]
      >
      /**
       * See [`Pallet::as_multi_threshold_1`].
       **/
      asMultiThreshold1: AugmentedSubmittable<
        (
          otherSignatories:
            | Vec<AccountId32>
            | (AccountId32 | string | Uint8Array)[],
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>, Call]
      >
      /**
       * See [`Pallet::cancel_as_multi`].
       **/
      cancelAsMulti: AugmentedSubmittable<
        (
          threshold: u16 | AnyNumber | Uint8Array,
          otherSignatories:
            | Vec<AccountId32>
            | (AccountId32 | string | Uint8Array)[],
          timepoint:
            | PalletMultisigTimepoint
            | { height?: any; index?: any }
            | string
            | Uint8Array,
          callHash: U8aFixed | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Vec<AccountId32>, PalletMultisigTimepoint, U8aFixed]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    nominationPools: {
      /**
       * See [`Pallet::adjust_pool_deposit`].
       **/
      adjustPoolDeposit: AugmentedSubmittable<
        (poolId: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::bond_extra`].
       **/
      bondExtra: AugmentedSubmittable<
        (
          extra:
            | PalletNominationPoolsBondExtra
            | { FreeBalance: any }
            | { Rewards: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletNominationPoolsBondExtra]
      >
      /**
       * See [`Pallet::bond_extra_other`].
       **/
      bondExtraOther: AugmentedSubmittable<
        (
          member:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          extra:
            | PalletNominationPoolsBondExtra
            | { FreeBalance: any }
            | { Rewards: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, PalletNominationPoolsBondExtra]
      >
      /**
       * See [`Pallet::chill`].
       **/
      chill: AugmentedSubmittable<
        (poolId: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::claim_commission`].
       **/
      claimCommission: AugmentedSubmittable<
        (poolId: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::claim_payout`].
       **/
      claimPayout: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::claim_payout_other`].
       **/
      claimPayoutOther: AugmentedSubmittable<
        (
          other: AccountId32 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32]
      >
      /**
       * See [`Pallet::create`].
       **/
      create: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          root:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          nominator:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          bouncer:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, MultiAddress, MultiAddress, MultiAddress]
      >
      /**
       * See [`Pallet::create_with_pool_id`].
       **/
      createWithPoolId: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          root:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          nominator:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          bouncer:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          poolId: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, MultiAddress, MultiAddress, MultiAddress, u32]
      >
      /**
       * See [`Pallet::join`].
       **/
      join: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          poolId: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, u32]
      >
      /**
       * See [`Pallet::nominate`].
       **/
      nominate: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          validators: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Vec<AccountId32>]
      >
      /**
       * See [`Pallet::pool_withdraw_unbonded`].
       **/
      poolWithdrawUnbonded: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          numSlashingSpans: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >
      /**
       * See [`Pallet::set_claim_permission`].
       **/
      setClaimPermission: AugmentedSubmittable<
        (
          permission:
            | PalletNominationPoolsClaimPermission
            | 'Permissioned'
            | 'PermissionlessCompound'
            | 'PermissionlessWithdraw'
            | 'PermissionlessAll'
            | number
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletNominationPoolsClaimPermission]
      >
      /**
       * See [`Pallet::set_commission`].
       **/
      setCommission: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          newCommission:
            | Option<ITuple<[Perbill, AccountId32]>>
            | null
            | Uint8Array
            | ITuple<[Perbill, AccountId32]>
            | [
                Perbill | AnyNumber | Uint8Array,
                AccountId32 | string | Uint8Array
              ]
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<ITuple<[Perbill, AccountId32]>>]
      >
      /**
       * See [`Pallet::set_commission_change_rate`].
       **/
      setCommissionChangeRate: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          changeRate:
            | PalletNominationPoolsCommissionChangeRate
            | { maxIncrease?: any; minDelay?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, PalletNominationPoolsCommissionChangeRate]
      >
      /**
       * See [`Pallet::set_commission_claim_permission`].
       **/
      setCommissionClaimPermission: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          permission:
            | Option<PalletNominationPoolsCommissionClaimPermission>
            | null
            | Uint8Array
            | PalletNominationPoolsCommissionClaimPermission
            | { Permissionless: any }
            | { Account: any }
            | string
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Option<PalletNominationPoolsCommissionClaimPermission>]
      >
      /**
       * See [`Pallet::set_commission_max`].
       **/
      setCommissionMax: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          maxCommission: Perbill | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Perbill]
      >
      /**
       * See [`Pallet::set_configs`].
       **/
      setConfigs: AugmentedSubmittable<
        (
          minJoinBond:
            | PalletNominationPoolsConfigOpU128
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          minCreateBond:
            | PalletNominationPoolsConfigOpU128
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          maxPools:
            | PalletNominationPoolsConfigOpU32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          maxMembers:
            | PalletNominationPoolsConfigOpU32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          maxMembersPerPool:
            | PalletNominationPoolsConfigOpU32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          globalMaxCommission:
            | PalletNominationPoolsConfigOpPerbill
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletNominationPoolsConfigOpU128,
          PalletNominationPoolsConfigOpU128,
          PalletNominationPoolsConfigOpU32,
          PalletNominationPoolsConfigOpU32,
          PalletNominationPoolsConfigOpU32,
          PalletNominationPoolsConfigOpPerbill
        ]
      >
      /**
       * See [`Pallet::set_metadata`].
       **/
      setMetadata: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          metadata: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Bytes]
      >
      /**
       * See [`Pallet::set_state`].
       **/
      setState: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          state:
            | PalletNominationPoolsPoolState
            | 'Open'
            | 'Blocked'
            | 'Destroying'
            | number
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, PalletNominationPoolsPoolState]
      >
      /**
       * See [`Pallet::unbond`].
       **/
      unbond: AugmentedSubmittable<
        (
          memberAccount:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          unbondingPoints: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >
      /**
       * See [`Pallet::update_roles`].
       **/
      updateRoles: AugmentedSubmittable<
        (
          poolId: u32 | AnyNumber | Uint8Array,
          newRoot:
            | PalletNominationPoolsConfigOpAccountId32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          newNominator:
            | PalletNominationPoolsConfigOpAccountId32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          newBouncer:
            | PalletNominationPoolsConfigOpAccountId32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          u32,
          PalletNominationPoolsConfigOpAccountId32,
          PalletNominationPoolsConfigOpAccountId32,
          PalletNominationPoolsConfigOpAccountId32
        ]
      >
      /**
       * See [`Pallet::withdraw_unbonded`].
       **/
      withdrawUnbonded: AugmentedSubmittable<
        (
          memberAccount:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          numSlashingSpans: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u32]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    preimage: {
      /**
       * See [`Pallet::ensure_updated`].
       **/
      ensureUpdated: AugmentedSubmittable<
        (
          hashes: Vec<H256> | (H256 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<H256>]
      >
      /**
       * See [`Pallet::note_preimage`].
       **/
      notePreimage: AugmentedSubmittable<
        (bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::request_preimage`].
       **/
      requestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::unnote_preimage`].
       **/
      unnotePreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::unrequest_preimage`].
       **/
      unrequestPreimage: AugmentedSubmittable<
        (hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    roles: {
      /**
       * See [`Pallet::chill`].
       **/
      chill: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::create_profile`].
       **/
      createProfile: AugmentedSubmittable<
        (
          profile:
            | PalletRolesProfile
            | { Independent: any }
            | { Shared: any }
            | string
            | Uint8Array,
          maxActiveServices: Option<u32> | null | Uint8Array | u32 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [PalletRolesProfile, Option<u32>]
      >
      /**
       * See [`Pallet::delete_profile`].
       **/
      deleteProfile: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * See [`Pallet::payout_stakers`].
       **/
      payoutStakers: AugmentedSubmittable<
        (
          validatorStash: AccountId32 | string | Uint8Array,
          era: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32, u32]
      >
      /**
       * See [`Pallet::set_min_restaking_bond`].
       **/
      setMinRestakingBond: AugmentedSubmittable<
        (
          minRestakingBond: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u128]
      >
      /**
       * See [`Pallet::unbond_funds`].
       **/
      unbondFunds: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >
      /**
       * See [`Pallet::update_profile`].
       **/
      updateProfile: AugmentedSubmittable<
        (
          updatedProfile:
            | PalletRolesProfile
            | { Independent: any }
            | { Shared: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletRolesProfile]
      >
      /**
       * See [`Pallet::withdraw_unbonded`].
       **/
      withdrawUnbonded: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    scheduler: {
      /**
       * See [`Pallet::cancel`].
       **/
      cancel: AugmentedSubmittable<
        (
          when: u64 | AnyNumber | Uint8Array,
          index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, u32]
      >
      /**
       * See [`Pallet::cancel_named`].
       **/
      cancelNamed: AugmentedSubmittable<
        (id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [U8aFixed]
      >
      /**
       * See [`Pallet::schedule`].
       **/
      schedule: AugmentedSubmittable<
        (
          when: u64 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u64, u32]>>
            | null
            | Uint8Array
            | ITuple<[u64, u32]>
            | [u64 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, Option<ITuple<[u64, u32]>>, u8, Call]
      >
      /**
       * See [`Pallet::schedule_after`].
       **/
      scheduleAfter: AugmentedSubmittable<
        (
          after: u64 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u64, u32]>>
            | null
            | Uint8Array
            | ITuple<[u64, u32]>
            | [u64 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u64, Option<ITuple<[u64, u32]>>, u8, Call]
      >
      /**
       * See [`Pallet::schedule_named`].
       **/
      scheduleNamed: AugmentedSubmittable<
        (
          id: U8aFixed | string | Uint8Array,
          when: u64 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u64, u32]>>
            | null
            | Uint8Array
            | ITuple<[u64, u32]>
            | [u64 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed, u64, Option<ITuple<[u64, u32]>>, u8, Call]
      >
      /**
       * See [`Pallet::schedule_named_after`].
       **/
      scheduleNamedAfter: AugmentedSubmittable<
        (
          id: U8aFixed | string | Uint8Array,
          after: u64 | AnyNumber | Uint8Array,
          maybePeriodic:
            | Option<ITuple<[u64, u32]>>
            | null
            | Uint8Array
            | ITuple<[u64, u32]>
            | [u64 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array],
          priority: u8 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed, u64, Option<ITuple<[u64, u32]>>, u8, Call]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    session: {
      /**
       * See [`Pallet::purge_keys`].
       **/
      purgeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::set_keys`].
       **/
      setKeys: AugmentedSubmittable<
        (
          keys:
            | TangleTestnetRuntimeOpaqueSessionKeys
            | { babe?: any; grandpa?: any; imOnline?: any; role?: any }
            | string
            | Uint8Array,
          proof: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TangleTestnetRuntimeOpaqueSessionKeys, Bytes]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    staking: {
      /**
       * See [`Pallet::bond`].
       **/
      bond: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          payee:
            | PalletStakingRewardDestination
            | { Staked: any }
            | { Stash: any }
            | { Controller: any }
            | { Account: any }
            | { None: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, PalletStakingRewardDestination]
      >
      /**
       * See [`Pallet::bond_extra`].
       **/
      bondExtra: AugmentedSubmittable<
        (
          maxAdditional: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >
      /**
       * See [`Pallet::cancel_deferred_slash`].
       **/
      cancelDeferredSlash: AugmentedSubmittable<
        (
          era: u32 | AnyNumber | Uint8Array,
          slashIndices: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [u32, Vec<u32>]
      >
      /**
       * See [`Pallet::chill`].
       **/
      chill: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::chill_other`].
       **/
      chillOther: AugmentedSubmittable<
        (
          stash: AccountId32 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32]
      >
      /**
       * See [`Pallet::deprecate_controller_batch`].
       **/
      deprecateControllerBatch: AugmentedSubmittable<
        (
          controllers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>]
      >
      /**
       * See [`Pallet::force_apply_min_commission`].
       **/
      forceApplyMinCommission: AugmentedSubmittable<
        (
          validatorStash: AccountId32 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32]
      >
      /**
       * See [`Pallet::force_new_era`].
       **/
      forceNewEra: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::force_new_era_always`].
       **/
      forceNewEraAlways: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * See [`Pallet::force_no_eras`].
       **/
      forceNoEras: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::force_unstake`].
       **/
      forceUnstake: AugmentedSubmittable<
        (
          stash: AccountId32 | string | Uint8Array,
          numSlashingSpans: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32, u32]
      >
      /**
       * See [`Pallet::increase_validator_count`].
       **/
      increaseValidatorCount: AugmentedSubmittable<
        (
          additional: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::kick`].
       **/
      kick: AugmentedSubmittable<
        (
          who:
            | Vec<MultiAddress>
            | (
                | MultiAddress
                | { Id: any }
                | { Index: any }
                | { Raw: any }
                | { Address32: any }
                | { Address20: any }
                | string
                | Uint8Array
              )[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<MultiAddress>]
      >
      /**
       * See [`Pallet::nominate`].
       **/
      nominate: AugmentedSubmittable<
        (
          targets:
            | Vec<MultiAddress>
            | (
                | MultiAddress
                | { Id: any }
                | { Index: any }
                | { Raw: any }
                | { Address32: any }
                | { Address20: any }
                | string
                | Uint8Array
              )[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<MultiAddress>]
      >
      /**
       * See [`Pallet::payout_stakers`].
       **/
      payoutStakers: AugmentedSubmittable<
        (
          validatorStash: AccountId32 | string | Uint8Array,
          era: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32, u32]
      >
      /**
       * See [`Pallet::payout_stakers_by_page`].
       **/
      payoutStakersByPage: AugmentedSubmittable<
        (
          validatorStash: AccountId32 | string | Uint8Array,
          era: u32 | AnyNumber | Uint8Array,
          page: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32, u32, u32]
      >
      /**
       * See [`Pallet::reap_stash`].
       **/
      reapStash: AugmentedSubmittable<
        (
          stash: AccountId32 | string | Uint8Array,
          numSlashingSpans: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32, u32]
      >
      /**
       * See [`Pallet::rebond`].
       **/
      rebond: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >
      /**
       * See [`Pallet::scale_validator_count`].
       **/
      scaleValidatorCount: AugmentedSubmittable<
        (
          factor: Percent | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Percent]
      >
      /**
       * See [`Pallet::set_controller`].
       **/
      setController: AugmentedSubmittable<
        () => SubmittableExtrinsic<ApiType>,
        []
      >
      /**
       * See [`Pallet::set_invulnerables`].
       **/
      setInvulnerables: AugmentedSubmittable<
        (
          invulnerables:
            | Vec<AccountId32>
            | (AccountId32 | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<AccountId32>]
      >
      /**
       * See [`Pallet::set_min_commission`].
       **/
      setMinCommission: AugmentedSubmittable<
        (
          updated: Perbill | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Perbill]
      >
      /**
       * See [`Pallet::set_payee`].
       **/
      setPayee: AugmentedSubmittable<
        (
          payee:
            | PalletStakingRewardDestination
            | { Staked: any }
            | { Stash: any }
            | { Controller: any }
            | { Account: any }
            | { None: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletStakingRewardDestination]
      >
      /**
       * See [`Pallet::set_staking_configs`].
       **/
      setStakingConfigs: AugmentedSubmittable<
        (
          minNominatorBond:
            | PalletStakingPalletConfigOpU128
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          minValidatorBond:
            | PalletStakingPalletConfigOpU128
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          maxNominatorCount:
            | PalletStakingPalletConfigOpU32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          maxValidatorCount:
            | PalletStakingPalletConfigOpU32
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          chillThreshold:
            | PalletStakingPalletConfigOpPercent
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array,
          minCommission:
            | PalletStakingPalletConfigOpPerbill
            | { Noop: any }
            | { Set: any }
            | { Remove: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [
          PalletStakingPalletConfigOpU128,
          PalletStakingPalletConfigOpU128,
          PalletStakingPalletConfigOpU32,
          PalletStakingPalletConfigOpU32,
          PalletStakingPalletConfigOpPercent,
          PalletStakingPalletConfigOpPerbill
        ]
      >
      /**
       * See [`Pallet::set_validator_count`].
       **/
      setValidatorCount: AugmentedSubmittable<
        (
          updated: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::unbond`].
       **/
      unbond: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>]
      >
      /**
       * See [`Pallet::update_payee`].
       **/
      updatePayee: AugmentedSubmittable<
        (
          controller: AccountId32 | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [AccountId32]
      >
      /**
       * See [`Pallet::validate`].
       **/
      validate: AugmentedSubmittable<
        (
          prefs:
            | PalletStakingValidatorPrefs
            | { commission?: any; blocked?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletStakingValidatorPrefs]
      >
      /**
       * See [`Pallet::withdraw_unbonded`].
       **/
      withdrawUnbonded: AugmentedSubmittable<
        (
          numSlashingSpans: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    sudo: {
      /**
       * See [`Pallet::remove_key`].
       **/
      removeKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::set_key`].
       **/
      setKey: AugmentedSubmittable<
        (
          updated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * See [`Pallet::sudo`].
       **/
      sudo: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call]
      >
      /**
       * See [`Pallet::sudo_as`].
       **/
      sudoAs: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Call]
      >
      /**
       * See [`Pallet::sudo_unchecked_weight`].
       **/
      sudoUncheckedWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    system: {
      /**
       * See [`Pallet::apply_authorized_upgrade`].
       **/
      applyAuthorizedUpgrade: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::authorize_upgrade`].
       **/
      authorizeUpgrade: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::authorize_upgrade_without_checks`].
       **/
      authorizeUpgradeWithoutChecks: AugmentedSubmittable<
        (codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [H256]
      >
      /**
       * See [`Pallet::kill_prefix`].
       **/
      killPrefix: AugmentedSubmittable<
        (
          prefix: Bytes | string | Uint8Array,
          subkeys: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32]
      >
      /**
       * See [`Pallet::kill_storage`].
       **/
      killStorage: AugmentedSubmittable<
        (
          keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Bytes>]
      >
      /**
       * See [`Pallet::remark`].
       **/
      remark: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::remark_with_event`].
       **/
      remarkWithEvent: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::set_code`].
       **/
      setCode: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::set_code_without_checks`].
       **/
      setCodeWithoutChecks: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >
      /**
       * See [`Pallet::set_heap_pages`].
       **/
      setHeapPages: AugmentedSubmittable<
        (pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u64]
      >
      /**
       * See [`Pallet::set_storage`].
       **/
      setStorage: AugmentedSubmittable<
        (
          items:
            | Vec<ITuple<[Bytes, Bytes]>>
            | [Bytes | string | Uint8Array, Bytes | string | Uint8Array][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[Bytes, Bytes]>>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    timestamp: {
      /**
       * See [`Pallet::set`].
       **/
      set: AugmentedSubmittable<
        (
          now: Compact<u64> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u64>]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    transactionPause: {
      /**
       * See [`Pallet::pause_transaction`].
       **/
      pauseTransaction: AugmentedSubmittable<
        (
          palletName: Bytes | string | Uint8Array,
          functionName: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, Bytes]
      >
      /**
       * See [`Pallet::unpause_transaction`].
       **/
      unpauseTransaction: AugmentedSubmittable<
        (
          palletName: Bytes | string | Uint8Array,
          functionName: Bytes | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, Bytes]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    treasury: {
      /**
       * See [`Pallet::approve_proposal`].
       **/
      approveProposal: AugmentedSubmittable<
        (
          proposalId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::check_status`].
       **/
      checkStatus: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::payout`].
       **/
      payout: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * See [`Pallet::propose_spend`].
       **/
      proposeSpend: AugmentedSubmittable<
        (
          value: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, MultiAddress]
      >
      /**
       * See [`Pallet::reject_proposal`].
       **/
      rejectProposal: AugmentedSubmittable<
        (
          proposalId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::remove_approval`].
       **/
      removeApproval: AugmentedSubmittable<
        (
          proposalId: Compact<u32> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u32>]
      >
      /**
       * See [`Pallet::spend`].
       **/
      spend: AugmentedSubmittable<
        (
          assetKind: Null | null,
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary: AccountId32 | string | Uint8Array,
          validFrom: Option<u64> | null | Uint8Array | u64 | AnyNumber
        ) => SubmittableExtrinsic<ApiType>,
        [Null, Compact<u128>, AccountId32, Option<u64>]
      >
      /**
       * See [`Pallet::spend_local`].
       **/
      spendLocal: AugmentedSubmittable<
        (
          amount: Compact<u128> | AnyNumber | Uint8Array,
          beneficiary:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u128>, MultiAddress]
      >
      /**
       * See [`Pallet::void_spend`].
       **/
      voidSpend: AugmentedSubmittable<
        (index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u32]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    utility: {
      /**
       * See [`Pallet::as_derivative`].
       **/
      asDerivative: AugmentedSubmittable<
        (
          index: u16 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Call]
      >
      /**
       * See [`Pallet::batch`].
       **/
      batch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >
      /**
       * See [`Pallet::batch_all`].
       **/
      batchAll: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >
      /**
       * See [`Pallet::dispatch_as`].
       **/
      dispatchAs: AugmentedSubmittable<
        (
          asOrigin:
            | TangleTestnetRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | { Council: any }
            | { Ethereum: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [TangleTestnetRuntimeOriginCaller, Call]
      >
      /**
       * See [`Pallet::force_batch`].
       **/
      forceBatch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >
      /**
       * See [`Pallet::with_weight`].
       **/
      withWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    vesting: {
      /**
       * See [`Pallet::force_remove_vesting_schedule`].
       **/
      forceRemoveVestingSchedule: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          scheduleIndex: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u32]
      >
      /**
       * See [`Pallet::force_vested_transfer`].
       **/
      forceVestedTransfer: AugmentedSubmittable<
        (
          source:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          schedule:
            | PalletVestingVestingInfo
            | { locked?: any; perBlock?: any; startingBlock?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, MultiAddress, PalletVestingVestingInfo]
      >
      /**
       * See [`Pallet::merge_schedules`].
       **/
      mergeSchedules: AugmentedSubmittable<
        (
          schedule1Index: u32 | AnyNumber | Uint8Array,
          schedule2Index: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >
      /**
       * See [`Pallet::vest`].
       **/
      vest: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>
      /**
       * See [`Pallet::vested_transfer`].
       **/
      vestedTransfer: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          schedule:
            | PalletVestingVestingInfo
            | { locked?: any; perBlock?: any; startingBlock?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, PalletVestingVestingInfo]
      >
      /**
       * See [`Pallet::vest_other`].
       **/
      vestOther: AugmentedSubmittable<
        (
          target:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
    zkSaaS: {
      /**
       * See [`Pallet::set_fee`].
       **/
      setFee: AugmentedSubmittable<
        (
          feeInfo:
            | PalletZksaasFeeInfo
            | {
                baseFee?: any
                circuitFee?: any
                proveFee?: any
                storageFeePerByte?: any
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [PalletZksaasFeeInfo]
      >
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>
    }
  } // AugmentedSubmittables
} // declare module
