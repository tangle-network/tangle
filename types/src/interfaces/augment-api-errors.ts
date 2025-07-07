// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/errors';

import type { ApiTypes, AugmentedError } from '@polkadot/api-base/types';

export type __AugmentedError<ApiType extends ApiTypes> = AugmentedError<ApiType>;

declare module '@polkadot/api-base/types/errors' {
  interface AugmentedErrors<ApiType extends ApiTypes> {
    assets: {
      /**
       * The asset-account already exists.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * The asset is not live, and likely being destroyed.
       **/
      AssetNotLive: AugmentedError<ApiType>;
      /**
       * The asset ID must be equal to the [`NextAssetId`].
       **/
      BadAssetId: AugmentedError<ApiType>;
      /**
       * Invalid metadata given.
       **/
      BadMetadata: AugmentedError<ApiType>;
      /**
       * Invalid witness data given.
       **/
      BadWitness: AugmentedError<ApiType>;
      /**
       * Account balance must be greater than or equal to the transfer amount.
       **/
      BalanceLow: AugmentedError<ApiType>;
      /**
       * Callback action resulted in error
       **/
      CallbackFailed: AugmentedError<ApiType>;
      /**
       * The origin account is frozen.
       **/
      Frozen: AugmentedError<ApiType>;
      /**
       * The asset status is not the expected status.
       **/
      IncorrectStatus: AugmentedError<ApiType>;
      /**
       * The asset ID is already taken.
       **/
      InUse: AugmentedError<ApiType>;
      /**
       * The asset is a live asset and is actively being used. Usually emit for operations such
       * as `start_destroy` which require the asset to be in a destroying state.
       **/
      LiveAsset: AugmentedError<ApiType>;
      /**
       * Minimum balance should be non-zero.
       **/
      MinBalanceZero: AugmentedError<ApiType>;
      /**
       * The account to alter does not exist.
       **/
      NoAccount: AugmentedError<ApiType>;
      /**
       * The asset-account doesn't have an associated deposit.
       **/
      NoDeposit: AugmentedError<ApiType>;
      /**
       * The signing account has no permission to do the operation.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * The asset should be frozen before the given operation.
       **/
      NotFrozen: AugmentedError<ApiType>;
      /**
       * No approval exists that would allow the transfer.
       **/
      Unapproved: AugmentedError<ApiType>;
      /**
       * Unable to increment the consumer reference counters on the account. Either no provider
       * reference exists to allow a non-zero balance of a non-self-sufficient asset, or one
       * fewer then the maximum number of consumers has been reached.
       **/
      UnavailableConsumer: AugmentedError<ApiType>;
      /**
       * The given asset ID is unknown.
       **/
      Unknown: AugmentedError<ApiType>;
      /**
       * The operation would result in funds being burned.
       **/
      WouldBurn: AugmentedError<ApiType>;
      /**
       * The source account would not survive the transfer and it needs to stay alive.
       **/
      WouldDie: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    babe: {
      /**
       * A given equivocation report is valid but already previously reported.
       **/
      DuplicateOffenceReport: AugmentedError<ApiType>;
      /**
       * Submitted configuration is invalid.
       **/
      InvalidConfiguration: AugmentedError<ApiType>;
      /**
       * An equivocation proof provided as part of an equivocation report is invalid.
       **/
      InvalidEquivocationProof: AugmentedError<ApiType>;
      /**
       * A key ownership proof provided as part of an equivocation report is invalid.
       **/
      InvalidKeyOwnershipProof: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bagsList: {
      /**
       * A error in the list interface implementation.
       **/
      List: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    balances: {
      /**
       * Beneficiary account must pre-exist.
       **/
      DeadAccount: AugmentedError<ApiType>;
      /**
       * The delta cannot be zero.
       **/
      DeltaZero: AugmentedError<ApiType>;
      /**
       * Value too low to create account due to existential deposit.
       **/
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * A vesting schedule already exists for this account.
       **/
      ExistingVestingSchedule: AugmentedError<ApiType>;
      /**
       * Transfer/payment would kill account.
       **/
      Expendability: AugmentedError<ApiType>;
      /**
       * Balance too low to send value.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * The issuance cannot be modified since it is already deactivated.
       **/
      IssuanceDeactivated: AugmentedError<ApiType>;
      /**
       * Account liquidity restrictions prevent withdrawal.
       **/
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Number of freezes exceed `MaxFreezes`.
       **/
      TooManyFreezes: AugmentedError<ApiType>;
      /**
       * Number of holds exceed `VariantCountOf<T::RuntimeHoldReason>`.
       **/
      TooManyHolds: AugmentedError<ApiType>;
      /**
       * Number of named reserves exceed `MaxReserves`.
       **/
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * Vesting balance too high to send value.
       **/
      VestingBalance: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bounties: {
      /**
       * The bounty cannot be closed because it has active child bounties.
       **/
      HasActiveChildBounty: AugmentedError<ApiType>;
      /**
       * Proposer's balance is too low.
       **/
      InsufficientProposersBalance: AugmentedError<ApiType>;
      /**
       * Invalid bounty fee.
       **/
      InvalidFee: AugmentedError<ApiType>;
      /**
       * No proposal or bounty at that index.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Invalid bounty value.
       **/
      InvalidValue: AugmentedError<ApiType>;
      /**
       * A bounty payout is pending.
       * To cancel the bounty, you must unassign and slash the curator.
       **/
      PendingPayout: AugmentedError<ApiType>;
      /**
       * The bounties cannot be claimed/closed because it's still in the countdown period.
       **/
      Premature: AugmentedError<ApiType>;
      /**
       * The reason given is just too big.
       **/
      ReasonTooBig: AugmentedError<ApiType>;
      /**
       * Require bounty curator.
       **/
      RequireCurator: AugmentedError<ApiType>;
      /**
       * Too many approvals are already queued.
       **/
      TooManyQueued: AugmentedError<ApiType>;
      /**
       * The bounty status is unexpected.
       **/
      UnexpectedStatus: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    childBounties: {
      /**
       * The bounty balance is not enough to add new child-bounty.
       **/
      InsufficientBountyBalance: AugmentedError<ApiType>;
      /**
       * The parent bounty is not in active state.
       **/
      ParentBountyNotActive: AugmentedError<ApiType>;
      /**
       * Number of child bounties exceeds limit `MaxActiveChildBountyCount`.
       **/
      TooManyChildBounties: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    claims: {
      /**
       * Invalid Ethereum signature.
       **/
      InvalidEthereumSignature: AugmentedError<ApiType>;
      /**
       * Invalid Native account decoding
       **/
      InvalidNativeAccount: AugmentedError<ApiType>;
      /**
       * Invalid Native (sr25519) signature
       **/
      InvalidNativeSignature: AugmentedError<ApiType>;
      /**
       * A needed statement was not included.
       **/
      InvalidStatement: AugmentedError<ApiType>;
      /**
       * There's not enough in the pot to pay out some unvested amount. Generally implies a
       * logic error.
       **/
      PotUnderflow: AugmentedError<ApiType>;
      /**
       * Account ID sending transaction has no claim.
       **/
      SenderHasNoClaim: AugmentedError<ApiType>;
      /**
       * Ethereum address has no claim.
       **/
      SignerHasNoClaim: AugmentedError<ApiType>;
      /**
       * The account already has a vested balance.
       **/
      VestedBalanceExists: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    council: {
      /**
       * Members are already initialized!
       **/
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       **/
      NotMember: AugmentedError<ApiType>;
      /**
       * Prime account is not a member
       **/
      PrimeAccountNotMember: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       **/
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       **/
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       **/
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       **/
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       **/
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       **/
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    credits: {
      /**
       * Amount specified for burn or claim must be greater than zero.
       **/
      AmountZero: AugmentedError<ApiType>;
      /**
       * No stake tiers configured for this asset.
       **/
      AssetRatesNotConfigured: AugmentedError<ApiType>;
      /**
       * Cannot transfer burned tokens to target account (feature not fully implemented).
       **/
      BurnTransferNotImplemented: AugmentedError<ApiType>;
      /**
       * The requested claim amount exceeds the maximum calculated within the allowed window.
       **/
      ClaimAmountExceedsWindowAllowance: AugmentedError<ApiType>;
      /**
       * There are no stake tiers provided for the update.
       **/
      EmptyStakeTiers: AugmentedError<ApiType>;
      /**
       * Insufficient TNT balance to perform the burn operation.
       **/
      InsufficientTntBalance: AugmentedError<ApiType>;
      /**
       * Invalid claim ID (e.g., too long).
       **/
      InvalidClaimId: AugmentedError<ApiType>;
      /**
       * No stake tiers are configured or the stake amount is below the lowest tier threshold.
       **/
      NoValidTier: AugmentedError<ApiType>;
      /**
       * Amount overflowed.
       **/
      Overflow: AugmentedError<ApiType>;
      /**
       * The stake tiers are not properly sorted by threshold.
       **/
      StakeTiersNotSorted: AugmentedError<ApiType>;
      /**
       * The stake tiers are too large to fit into the storage.
       **/
      StakeTiersOverflow: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    democracy: {
      /**
       * Cannot cancel the same proposal twice
       **/
      AlreadyCanceled: AugmentedError<ApiType>;
      /**
       * The account is already delegating.
       **/
      AlreadyDelegating: AugmentedError<ApiType>;
      /**
       * Identity may not veto a proposal twice
       **/
      AlreadyVetoed: AugmentedError<ApiType>;
      /**
       * Proposal already made
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * The instant referendum origin is currently disallowed.
       **/
      InstantNotAllowed: AugmentedError<ApiType>;
      /**
       * Too high a balance was provided that the account cannot afford.
       **/
      InsufficientFunds: AugmentedError<ApiType>;
      /**
       * Invalid hash
       **/
      InvalidHash: AugmentedError<ApiType>;
      /**
       * Maximum number of votes reached.
       **/
      MaxVotesReached: AugmentedError<ApiType>;
      /**
       * No proposals waiting
       **/
      NoneWaiting: AugmentedError<ApiType>;
      /**
       * Delegation to oneself makes no sense.
       **/
      Nonsense: AugmentedError<ApiType>;
      /**
       * The actor has no permission to conduct the action.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * No external proposal
       **/
      NoProposal: AugmentedError<ApiType>;
      /**
       * The account is not currently delegating.
       **/
      NotDelegating: AugmentedError<ApiType>;
      /**
       * Next external proposal not simple majority
       **/
      NotSimpleMajority: AugmentedError<ApiType>;
      /**
       * The given account did not vote on the referendum.
       **/
      NotVoter: AugmentedError<ApiType>;
      /**
       * The preimage does not exist.
       **/
      PreimageNotExist: AugmentedError<ApiType>;
      /**
       * Proposal still blacklisted
       **/
      ProposalBlacklisted: AugmentedError<ApiType>;
      /**
       * Proposal does not exist
       **/
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * Vote given for invalid referendum
       **/
      ReferendumInvalid: AugmentedError<ApiType>;
      /**
       * Maximum number of items reached.
       **/
      TooMany: AugmentedError<ApiType>;
      /**
       * Value too low
       **/
      ValueLow: AugmentedError<ApiType>;
      /**
       * The account currently has votes attached to it and the operation cannot succeed until
       * these are removed, either through `unvote` or `reap_vote`.
       **/
      VotesExist: AugmentedError<ApiType>;
      /**
       * Voting period too low
       **/
      VotingPeriodLow: AugmentedError<ApiType>;
      /**
       * Invalid upper bound.
       **/
      WrongUpperBound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    electionProviderMultiPhase: {
      /**
       * Some bound not met
       **/
      BoundNotMet: AugmentedError<ApiType>;
      /**
       * The call is not allowed at this point.
       **/
      CallNotAllowed: AugmentedError<ApiType>;
      /**
       * The fallback failed
       **/
      FallbackFailed: AugmentedError<ApiType>;
      /**
       * `Self::insert_submission` returned an invalid index.
       **/
      InvalidSubmissionIndex: AugmentedError<ApiType>;
      /**
       * Snapshot metadata should exist but didn't.
       **/
      MissingSnapshotMetadata: AugmentedError<ApiType>;
      /**
       * OCW submitted solution for wrong round
       **/
      OcwCallWrongEra: AugmentedError<ApiType>;
      /**
       * Submission was prepared for a different round.
       **/
      PreDispatchDifferentRound: AugmentedError<ApiType>;
      /**
       * Submission was too early.
       **/
      PreDispatchEarlySubmission: AugmentedError<ApiType>;
      /**
       * Submission was too weak, score-wise.
       **/
      PreDispatchWeakSubmission: AugmentedError<ApiType>;
      /**
       * Wrong number of winners presented.
       **/
      PreDispatchWrongWinnerCount: AugmentedError<ApiType>;
      /**
       * The origin failed to pay the deposit.
       **/
      SignedCannotPayDeposit: AugmentedError<ApiType>;
      /**
       * Witness data to dispatchable is invalid.
       **/
      SignedInvalidWitness: AugmentedError<ApiType>;
      /**
       * The queue was full, and the solution was not better than any of the existing ones.
       **/
      SignedQueueFull: AugmentedError<ApiType>;
      /**
       * The signed submission consumes too much weight
       **/
      SignedTooMuchWeight: AugmentedError<ApiType>;
      /**
       * Submitted solution has too many winners
       **/
      TooManyWinners: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    elections: {
      /**
       * Duplicated candidate submission.
       **/
      DuplicatedCandidate: AugmentedError<ApiType>;
      /**
       * Candidate does not have enough funds.
       **/
      InsufficientCandidateFunds: AugmentedError<ApiType>;
      /**
       * The renouncing origin presented a wrong `Renouncing` parameter.
       **/
      InvalidRenouncing: AugmentedError<ApiType>;
      /**
       * Prediction regarding replacement after member removal is wrong.
       **/
      InvalidReplacement: AugmentedError<ApiType>;
      /**
       * The provided count of number of votes is incorrect.
       **/
      InvalidVoteCount: AugmentedError<ApiType>;
      /**
       * The provided count of number of candidates is incorrect.
       **/
      InvalidWitnessData: AugmentedError<ApiType>;
      /**
       * Cannot vote with stake less than minimum balance.
       **/
      LowBalance: AugmentedError<ApiType>;
      /**
       * Cannot vote more than maximum allowed.
       **/
      MaximumVotesExceeded: AugmentedError<ApiType>;
      /**
       * Member cannot re-submit candidacy.
       **/
      MemberSubmit: AugmentedError<ApiType>;
      /**
       * Must be a voter.
       **/
      MustBeVoter: AugmentedError<ApiType>;
      /**
       * Not a member.
       **/
      NotMember: AugmentedError<ApiType>;
      /**
       * Must vote for at least one candidate.
       **/
      NoVotes: AugmentedError<ApiType>;
      /**
       * Runner cannot re-submit candidacy.
       **/
      RunnerUpSubmit: AugmentedError<ApiType>;
      /**
       * Too many candidates have been created.
       **/
      TooManyCandidates: AugmentedError<ApiType>;
      /**
       * Cannot vote more than candidates.
       **/
      TooManyVotes: AugmentedError<ApiType>;
      /**
       * Voter can not pay voting bond.
       **/
      UnableToPayBond: AugmentedError<ApiType>;
      /**
       * Cannot vote when no candidates or members exist.
       **/
      UnableToVote: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    ethereum: {
      /**
       * Signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Pre-log is present, therefore transact is not allowed.
       **/
      PreLogExists: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    evm: {
      /**
       * Not enough balance to perform action
       **/
      BalanceLow: AugmentedError<ApiType>;
      /**
       * Calculating total fee overflowed
       **/
      FeeOverflow: AugmentedError<ApiType>;
      /**
       * Gas limit is too high.
       **/
      GasLimitTooHigh: AugmentedError<ApiType>;
      /**
       * Gas limit is too low.
       **/
      GasLimitTooLow: AugmentedError<ApiType>;
      /**
       * Gas price is too low.
       **/
      GasPriceTooLow: AugmentedError<ApiType>;
      /**
       * The chain id is invalid.
       **/
      InvalidChainId: AugmentedError<ApiType>;
      /**
       * Nonce is invalid
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * the signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Calculating total payment overflowed
       **/
      PaymentOverflow: AugmentedError<ApiType>;
      /**
       * EVM reentrancy
       **/
      Reentrancy: AugmentedError<ApiType>;
      /**
       * EIP-3607,
       **/
      TransactionMustComeFromEOA: AugmentedError<ApiType>;
      /**
       * Undefined error.
       **/
      Undefined: AugmentedError<ApiType>;
      /**
       * Withdraw fee failed
       **/
      WithdrawFailed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    grandpa: {
      /**
       * Attempt to signal GRANDPA change with one already pending.
       **/
      ChangePending: AugmentedError<ApiType>;
      /**
       * A given equivocation report is valid but already previously reported.
       **/
      DuplicateOffenceReport: AugmentedError<ApiType>;
      /**
       * An equivocation proof provided as part of an equivocation report is invalid.
       **/
      InvalidEquivocationProof: AugmentedError<ApiType>;
      /**
       * A key ownership proof provided as part of an equivocation report is invalid.
       **/
      InvalidKeyOwnershipProof: AugmentedError<ApiType>;
      /**
       * Attempt to signal GRANDPA pause when the authority set isn't live
       * (either paused or already pending pause).
       **/
      PauseFailed: AugmentedError<ApiType>;
      /**
       * Attempt to signal GRANDPA resume when the authority set isn't paused
       * (either live or already pending resume).
       **/
      ResumeFailed: AugmentedError<ApiType>;
      /**
       * Cannot signal forced change so soon after last.
       **/
      TooSoon: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    hotfixSufficients: {
      /**
       * Maximum address count exceeded
       **/
      MaxAddressCountExceeded: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    hyperbridge: {
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    identity: {
      /**
       * Account ID is already named.
       **/
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Empty index.
       **/
      EmptyIndex: AugmentedError<ApiType>;
      /**
       * Fee is changed.
       **/
      FeeChanged: AugmentedError<ApiType>;
      /**
       * The index is invalid.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Invalid judgement.
       **/
      InvalidJudgement: AugmentedError<ApiType>;
      /**
       * The signature on a username was not valid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * The provided suffix is too long.
       **/
      InvalidSuffix: AugmentedError<ApiType>;
      /**
       * The target is invalid.
       **/
      InvalidTarget: AugmentedError<ApiType>;
      /**
       * The username does not meet the requirements.
       **/
      InvalidUsername: AugmentedError<ApiType>;
      /**
       * The provided judgement was for a different identity.
       **/
      JudgementForDifferentIdentity: AugmentedError<ApiType>;
      /**
       * Judgement given.
       **/
      JudgementGiven: AugmentedError<ApiType>;
      /**
       * Error that occurs when there is an issue paying for judgement.
       **/
      JudgementPaymentFailed: AugmentedError<ApiType>;
      /**
       * The authority cannot allocate any more usernames.
       **/
      NoAllocation: AugmentedError<ApiType>;
      /**
       * No identity found.
       **/
      NoIdentity: AugmentedError<ApiType>;
      /**
       * The username cannot be forcefully removed because it can still be accepted.
       **/
      NotExpired: AugmentedError<ApiType>;
      /**
       * Account isn't found.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Account isn't named.
       **/
      NotNamed: AugmentedError<ApiType>;
      /**
       * Sub-account isn't owned by sender.
       **/
      NotOwned: AugmentedError<ApiType>;
      /**
       * Sender is not a sub-account.
       **/
      NotSub: AugmentedError<ApiType>;
      /**
       * The sender does not have permission to issue a username.
       **/
      NotUsernameAuthority: AugmentedError<ApiType>;
      /**
       * The requested username does not exist.
       **/
      NoUsername: AugmentedError<ApiType>;
      /**
       * Setting this username requires a signature, but none was provided.
       **/
      RequiresSignature: AugmentedError<ApiType>;
      /**
       * Sticky judgement.
       **/
      StickyJudgement: AugmentedError<ApiType>;
      /**
       * Maximum amount of registrars reached. Cannot add any more.
       **/
      TooManyRegistrars: AugmentedError<ApiType>;
      /**
       * Too many subs-accounts.
       **/
      TooManySubAccounts: AugmentedError<ApiType>;
      /**
       * The username is already taken.
       **/
      UsernameTaken: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    imOnline: {
      /**
       * Duplicated heartbeat.
       **/
      DuplicatedHeartbeat: AugmentedError<ApiType>;
      /**
       * Non existent public key.
       **/
      InvalidKey: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    indices: {
      /**
       * The index was not available.
       **/
      InUse: AugmentedError<ApiType>;
      /**
       * The index was not already assigned.
       **/
      NotAssigned: AugmentedError<ApiType>;
      /**
       * The index is assigned to another account.
       **/
      NotOwner: AugmentedError<ApiType>;
      /**
       * The source and destination accounts are identical.
       **/
      NotTransfer: AugmentedError<ApiType>;
      /**
       * The index is permanent and may not be freed/changed.
       **/
      Permanent: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    ismp: {
      /**
       * Couldn't update challenge period
       **/
      ChallengePeriodUpdateFailed: AugmentedError<ApiType>;
      /**
       * Encountered an error while creating the consensus client.
       **/
      ConsensusClientCreationFailed: AugmentedError<ApiType>;
      /**
       * Invalid ISMP message
       **/
      InvalidMessage: AugmentedError<ApiType>;
      /**
       * Requested message was not found
       **/
      MessageNotFound: AugmentedError<ApiType>;
      /**
       * Couldn't update unbonding period
       **/
      UnbondingPeriodUpdateFailed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    lst: {
      /**
       * Bonding extra is restricted to the exact pending reward amount.
       **/
      BondExtraRestricted: AugmentedError<ApiType>;
      /**
       * The pools state cannot be changed.
       **/
      CanNotChangeState: AugmentedError<ApiType>;
      /**
       * None of the funds can be withdrawn yet because the bonding duration has not passed.
       **/
      CannotWithdrawAny: AugmentedError<ApiType>;
      /**
       * The submitted changes to commission change rate are not allowed.
       **/
      CommissionChangeRateNotAllowed: AugmentedError<ApiType>;
      /**
       * Not enough blocks have surpassed since the last commission update.
       **/
      CommissionChangeThrottled: AugmentedError<ApiType>;
      /**
       * The supplied commission exceeds global maximum commission.
       **/
      CommissionExceedsGlobalMaximum: AugmentedError<ApiType>;
      /**
       * The supplied commission exceeds the max allowed commission.
       **/
      CommissionExceedsMaximum: AugmentedError<ApiType>;
      /**
       * Some error occurred that should never happen. This should be reported to the
       * maintainers.
       **/
      Defensive: AugmentedError<ApiType>;
      /**
       * The caller does not have adequate permissions.
       **/
      DoesNotHavePermission: AugmentedError<ApiType>;
      /**
       * The member is fully unbonded (and thus cannot access the bonded and reward pool
       * anymore to, for example, collect rewards).
       **/
      FullyUnbonding: AugmentedError<ApiType>;
      /**
       * Pool id provided is not correct/usable.
       **/
      InvalidPoolId: AugmentedError<ApiType>;
      /**
       * The pool's max commission cannot be set higher than the existing value.
       **/
      MaxCommissionRestricted: AugmentedError<ApiType>;
      /**
       * Too many members in the pool or system.
       **/
      MaxPoolMembers: AugmentedError<ApiType>;
      /**
       * The system is maxed out on pools.
       **/
      MaxPools: AugmentedError<ApiType>;
      /**
       * The member cannot unbond further chunks due to reaching the limit.
       **/
      MaxUnbondingLimit: AugmentedError<ApiType>;
      /**
       * Metadata exceeds [`Config::MaxMetadataLen`]
       **/
      MetadataExceedsMaxLen: AugmentedError<ApiType>;
      /**
       * The amount does not meet the minimum bond to either join or create a pool.
       * 
       * The depositor can never unbond to a value less than `Pallet::depositor_min_bond`. The
       * caller does not have nominating permissions for the pool. Members can never unbond to a
       * value below `MinJoinBond`.
       **/
      MinimumBondNotMet: AugmentedError<ApiType>;
      /**
       * No balance to unbond.
       **/
      NoBalanceToUnbond: AugmentedError<ApiType>;
      /**
       * No commission current has been set.
       **/
      NoCommissionCurrentSet: AugmentedError<ApiType>;
      /**
       * There is no pending commission to claim.
       **/
      NoPendingCommission: AugmentedError<ApiType>;
      /**
       * A pool must be in [`PoolState::Destroying`] in order for the depositor to unbond or for
       * other members to be permissionlessly unbonded.
       **/
      NotDestroying: AugmentedError<ApiType>;
      /**
       * No imbalance in the ED deposit for the pool.
       **/
      NothingToAdjust: AugmentedError<ApiType>;
      /**
       * Either a) the caller cannot make a valid kick or b) the pool is not destroying.
       **/
      NotKickerOrDestroying: AugmentedError<ApiType>;
      /**
       * The caller does not have nominating permissions for the pool.
       **/
      NotNominator: AugmentedError<ApiType>;
      /**
       * The pool is not open to join
       **/
      NotOpen: AugmentedError<ApiType>;
      /**
       * The transaction could not be executed due to overflow risk for the pool.
       **/
      OverflowRisk: AugmentedError<ApiType>;
      /**
       * Partial unbonding now allowed permissionlessly.
       **/
      PartialUnbondNotAllowedPermissionlessly: AugmentedError<ApiType>;
      /**
       * Pool id currently in use.
       **/
      PoolIdInUse: AugmentedError<ApiType>;
      /**
       * An account is not a member.
       **/
      PoolMemberNotFound: AugmentedError<ApiType>;
      /**
       * A (bonded) pool id does not exist.
       **/
      PoolNotFound: AugmentedError<ApiType>;
      /**
       * Pool token creation failed.
       **/
      PoolTokenCreationFailed: AugmentedError<ApiType>;
      /**
       * A reward pool does not exist. In all cases this is a system logic error.
       **/
      RewardPoolNotFound: AugmentedError<ApiType>;
      /**
       * A sub pool does not exist.
       **/
      SubPoolsNotFound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    multiAssetDelegation: {
      /**
       * There are active services using the asset.
       **/
      ActiveServicesUsingAsset: AugmentedError<ApiType>;
      /**
       * The account is already a delegator.
       **/
      AlreadyDelegator: AugmentedError<ApiType>;
      /**
       * The operator is already leaving.
       **/
      AlreadyLeaving: AugmentedError<ApiType>;
      /**
       * The account is already an operator.
       **/
      AlreadyOperator: AugmentedError<ApiType>;
      /**
       * APY exceeds maximum allowed by the extrinsic
       **/
      APYExceedsMaximum: AugmentedError<ApiType>;
      /**
       * Asset already exists in a reward vault
       **/
      AssetAlreadyInVault: AugmentedError<ApiType>;
      /**
       * The asset config is not found
       **/
      AssetConfigNotFound: AugmentedError<ApiType>;
      /**
       * The asset ID is not found
       **/
      AssetNotFound: AugmentedError<ApiType>;
      /**
       * Asset not found in reward vault
       **/
      AssetNotInVault: AugmentedError<ApiType>;
      /**
       * The asset is not whitelisted
       **/
      AssetNotWhitelisted: AugmentedError<ApiType>;
      /**
       * The blueprint ID is already whitelisted
       **/
      BlueprintAlreadyWhitelisted: AugmentedError<ApiType>;
      /**
       * Error returned when trying to remove a blueprint ID that doesn't exist.
       **/
      BlueprintIdNotFound: AugmentedError<ApiType>;
      /**
       * The blueprint is not selected
       **/
      BlueprintNotSelected: AugmentedError<ApiType>;
      /**
       * The unstake request is not ready.
       **/
      BondLessNotReady: AugmentedError<ApiType>;
      /**
       * A unstake request already exists.
       **/
      BondLessRequestAlreadyExists: AugmentedError<ApiType>;
      /**
       * The unstake request is not satisfied.
       **/
      BondLessRequestNotSatisfied: AugmentedError<ApiType>;
      /**
       * The stake amount is too low.
       **/
      BondTooLow: AugmentedError<ApiType>;
      /**
       * The account cannot exit.
       **/
      CannotExit: AugmentedError<ApiType>;
      /**
       * Cannot go offline with active services
       **/
      CannotGoOfflineWithActiveServices: AugmentedError<ApiType>;
      /**
       * Cap cannot be zero
       **/
      CapCannotBeZero: AugmentedError<ApiType>;
      /**
       * Cap exceeds total supply of asset
       **/
      CapExceedsTotalSupply: AugmentedError<ApiType>;
      /**
       * Above deposit caps setup
       **/
      DepositExceedsCapForAsset: AugmentedError<ApiType>;
      /**
       * Deposit amount overflow
       **/
      DepositOverflow: AugmentedError<ApiType>;
      /**
       * Error returned when trying to add a blueprint ID that already exists.
       **/
      DuplicateBlueprintId: AugmentedError<ApiType>;
      /**
       * Erc20 transfer failed
       **/
      ERC20TransferFailed: AugmentedError<ApiType>;
      /**
       * EVM decode error
       **/
      EVMAbiDecode: AugmentedError<ApiType>;
      /**
       * EVM encode error
       **/
      EVMAbiEncode: AugmentedError<ApiType>;
      /**
       * The account has insufficient balance.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Underflow while reducing stake
       **/
      InsufficientStakeRemaining: AugmentedError<ApiType>;
      /**
       * Amount is invalid
       **/
      InvalidAmount: AugmentedError<ApiType>;
      /**
       * Leaving round not reached
       **/
      LeavingRoundNotReached: AugmentedError<ApiType>;
      /**
       * Cannot unstake with locks
       **/
      LockViolation: AugmentedError<ApiType>;
      /**
       * Maximum number of blueprints exceeded
       **/
      MaxBlueprintsExceeded: AugmentedError<ApiType>;
      /**
       * Error returned when the maximum number of delegations is exceeded.
       **/
      MaxDelegationsExceeded: AugmentedError<ApiType>;
      /**
       * Error returned when the maximum number of unstake requests is exceeded.
       **/
      MaxUnstakeRequestsExceeded: AugmentedError<ApiType>;
      /**
       * Error returned when the maximum number of withdraw requests is exceeded.
       **/
      MaxWithdrawRequestsExceeded: AugmentedError<ApiType>;
      /**
       * There is not active delegation
       **/
      NoActiveDelegation: AugmentedError<ApiType>;
      /**
       * There is no unstake request.
       **/
      NoBondLessRequest: AugmentedError<ApiType>;
      /**
       * No matching withdraw reqests found
       **/
      NoMatchingwithdrawRequest: AugmentedError<ApiType>;
      /**
       * There is no scheduled unstake request.
       **/
      NoScheduledBondLess: AugmentedError<ApiType>;
      /**
       * The operator is not active.
       **/
      NotActiveOperator: AugmentedError<ApiType>;
      /**
       * The account is not an operator.
       **/
      NotAnOperator: AugmentedError<ApiType>;
      /**
       * The origin is not authorized to perform this action
       **/
      NotAuthorized: AugmentedError<ApiType>;
      /**
       * The account is not a delegator.
       **/
      NotDelegator: AugmentedError<ApiType>;
      /**
       * Error returned when trying to add/remove blueprint IDs while not in Fixed mode.
       **/
      NotInFixedMode: AugmentedError<ApiType>;
      /**
       * The account is not leaving as an operator.
       **/
      NotLeavingOperator: AugmentedError<ApiType>;
      /**
       * Not a nominator (for native restaking & delegation)
       **/
      NotNominator: AugmentedError<ApiType>;
      /**
       * The operator is not offline.
       **/
      NotOfflineOperator: AugmentedError<ApiType>;
      /**
       * There is no withdraw request.
       **/
      NoWithdrawRequest: AugmentedError<ApiType>;
      /**
       * No withdraw requests found
       **/
      NoWithdrawRequests: AugmentedError<ApiType>;
      /**
       * Overflow from math
       **/
      OverflowRisk: AugmentedError<ApiType>;
      /**
       * An unstake request is already pending
       **/
      PendingUnstakeRequestExists: AugmentedError<ApiType>;
      /**
       * Slash alert failed
       **/
      SlashAlertFailed: AugmentedError<ApiType>;
      /**
       * Overflow while adding stake
       **/
      StakeOverflow: AugmentedError<ApiType>;
      /**
       * Unstake underflow
       **/
      UnstakeAmountTooLarge: AugmentedError<ApiType>;
      /**
       * The reward vault does not exist
       **/
      VaultNotFound: AugmentedError<ApiType>;
      /**
       * A withdraw request already exists.
       **/
      WithdrawRequestAlreadyExists: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    multisig: {
      /**
       * Call is already approved by this signatory.
       **/
      AlreadyApproved: AugmentedError<ApiType>;
      /**
       * The data to be stored is already stored.
       **/
      AlreadyStored: AugmentedError<ApiType>;
      /**
       * The maximum weight information provided was too low.
       **/
      MaxWeightTooLow: AugmentedError<ApiType>;
      /**
       * Threshold must be 2 or greater.
       **/
      MinimumThreshold: AugmentedError<ApiType>;
      /**
       * Call doesn't need any (more) approvals.
       **/
      NoApprovalsNeeded: AugmentedError<ApiType>;
      /**
       * Multisig operation not found when attempting to cancel.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * No timepoint was given, yet the multisig operation is already underway.
       **/
      NoTimepoint: AugmentedError<ApiType>;
      /**
       * Only the account that originally created the multisig is able to cancel it.
       **/
      NotOwner: AugmentedError<ApiType>;
      /**
       * The sender was contained in the other signatories; it shouldn't be.
       **/
      SenderInSignatories: AugmentedError<ApiType>;
      /**
       * The signatories were provided out of order; they should be ordered.
       **/
      SignatoriesOutOfOrder: AugmentedError<ApiType>;
      /**
       * There are too few signatories in the list.
       **/
      TooFewSignatories: AugmentedError<ApiType>;
      /**
       * There are too many signatories in the list.
       **/
      TooManySignatories: AugmentedError<ApiType>;
      /**
       * A timepoint was given, yet no multisig operation is underway.
       **/
      UnexpectedTimepoint: AugmentedError<ApiType>;
      /**
       * A different timepoint was given to the multisig operation that is underway.
       **/
      WrongTimepoint: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    nominationPools: {
      /**
       * An account is already delegating in another pool. An account may only belong to one
       * pool at a time.
       **/
      AccountBelongsToOtherPool: AugmentedError<ApiType>;
      /**
       * The pool or member delegation has already migrated to delegate stake.
       **/
      AlreadyMigrated: AugmentedError<ApiType>;
      /**
       * Bonding extra is restricted to the exact pending reward amount.
       **/
      BondExtraRestricted: AugmentedError<ApiType>;
      /**
       * The pools state cannot be changed.
       **/
      CanNotChangeState: AugmentedError<ApiType>;
      /**
       * None of the funds can be withdrawn yet because the bonding duration has not passed.
       **/
      CannotWithdrawAny: AugmentedError<ApiType>;
      /**
       * The submitted changes to commission change rate are not allowed.
       **/
      CommissionChangeRateNotAllowed: AugmentedError<ApiType>;
      /**
       * Not enough blocks have surpassed since the last commission update.
       **/
      CommissionChangeThrottled: AugmentedError<ApiType>;
      /**
       * The supplied commission exceeds global maximum commission.
       **/
      CommissionExceedsGlobalMaximum: AugmentedError<ApiType>;
      /**
       * The supplied commission exceeds the max allowed commission.
       **/
      CommissionExceedsMaximum: AugmentedError<ApiType>;
      /**
       * Some error occurred that should never happen. This should be reported to the
       * maintainers.
       **/
      Defensive: AugmentedError<ApiType>;
      /**
       * The caller does not have adequate permissions.
       **/
      DoesNotHavePermission: AugmentedError<ApiType>;
      /**
       * The member is fully unbonded (and thus cannot access the bonded and reward pool
       * anymore to, for example, collect rewards).
       **/
      FullyUnbonding: AugmentedError<ApiType>;
      /**
       * Pool id provided is not correct/usable.
       **/
      InvalidPoolId: AugmentedError<ApiType>;
      /**
       * The pool's max commission cannot be set higher than the existing value.
       **/
      MaxCommissionRestricted: AugmentedError<ApiType>;
      /**
       * Too many members in the pool or system.
       **/
      MaxPoolMembers: AugmentedError<ApiType>;
      /**
       * The system is maxed out on pools.
       **/
      MaxPools: AugmentedError<ApiType>;
      /**
       * The member cannot unbond further chunks due to reaching the limit.
       **/
      MaxUnbondingLimit: AugmentedError<ApiType>;
      /**
       * Metadata exceeds [`Config::MaxMetadataLen`]
       **/
      MetadataExceedsMaxLen: AugmentedError<ApiType>;
      /**
       * The amount does not meet the minimum bond to either join or create a pool.
       * 
       * The depositor can never unbond to a value less than `Pallet::depositor_min_bond`. The
       * caller does not have nominating permissions for the pool. Members can never unbond to a
       * value below `MinJoinBond`.
       **/
      MinimumBondNotMet: AugmentedError<ApiType>;
      /**
       * No commission current has been set.
       **/
      NoCommissionCurrentSet: AugmentedError<ApiType>;
      /**
       * There is no pending commission to claim.
       **/
      NoPendingCommission: AugmentedError<ApiType>;
      /**
       * A pool must be in [`PoolState::Destroying`] in order for the depositor to unbond or for
       * other members to be permissionlessly unbonded.
       **/
      NotDestroying: AugmentedError<ApiType>;
      /**
       * No imbalance in the ED deposit for the pool.
       **/
      NothingToAdjust: AugmentedError<ApiType>;
      /**
       * No slash pending that can be applied to the member.
       **/
      NothingToSlash: AugmentedError<ApiType>;
      /**
       * Either a) the caller cannot make a valid kick or b) the pool is not destroying.
       **/
      NotKickerOrDestroying: AugmentedError<ApiType>;
      /**
       * The pool or member delegation has not migrated yet to delegate stake.
       **/
      NotMigrated: AugmentedError<ApiType>;
      /**
       * The caller does not have nominating permissions for the pool.
       **/
      NotNominator: AugmentedError<ApiType>;
      /**
       * The pool is not open to join
       **/
      NotOpen: AugmentedError<ApiType>;
      /**
       * This call is not allowed in the current state of the pallet.
       **/
      NotSupported: AugmentedError<ApiType>;
      /**
       * The transaction could not be executed due to overflow risk for the pool.
       **/
      OverflowRisk: AugmentedError<ApiType>;
      /**
       * Partial unbonding now allowed permissionlessly.
       **/
      PartialUnbondNotAllowedPermissionlessly: AugmentedError<ApiType>;
      /**
       * Pool id currently in use.
       **/
      PoolIdInUse: AugmentedError<ApiType>;
      /**
       * An account is not a member.
       **/
      PoolMemberNotFound: AugmentedError<ApiType>;
      /**
       * A (bonded) pool id does not exist.
       **/
      PoolNotFound: AugmentedError<ApiType>;
      /**
       * A reward pool does not exist. In all cases this is a system logic error.
       **/
      RewardPoolNotFound: AugmentedError<ApiType>;
      /**
       * The slash amount is too low to be applied.
       **/
      SlashTooLow: AugmentedError<ApiType>;
      /**
       * A sub pool does not exist.
       **/
      SubPoolsNotFound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    preimage: {
      /**
       * Preimage has already been noted on-chain.
       **/
      AlreadyNoted: AugmentedError<ApiType>;
      /**
       * No ticket with a cost was returned by [`Config::Consideration`] to store the preimage.
       **/
      NoCost: AugmentedError<ApiType>;
      /**
       * The user is not authorized to perform this action.
       **/
      NotAuthorized: AugmentedError<ApiType>;
      /**
       * The preimage cannot be removed since it has not yet been noted.
       **/
      NotNoted: AugmentedError<ApiType>;
      /**
       * The preimage request cannot be removed since no outstanding requests exist.
       **/
      NotRequested: AugmentedError<ApiType>;
      /**
       * A preimage may not be removed when there are outstanding requests.
       **/
      Requested: AugmentedError<ApiType>;
      /**
       * Preimage is too large to store on-chain.
       **/
      TooBig: AugmentedError<ApiType>;
      /**
       * Too few hashes were requested to be upgraded (i.e. zero).
       **/
      TooFew: AugmentedError<ApiType>;
      /**
       * More than `MAX_HASH_UPGRADE_BULK_COUNT` hashes were requested to be upgraded at once.
       **/
      TooMany: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    proxy: {
      /**
       * Account is already a proxy.
       **/
      Duplicate: AugmentedError<ApiType>;
      /**
       * Call may not be made by proxy because it may escalate its privileges.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * Cannot add self as proxy.
       **/
      NoSelfProxy: AugmentedError<ApiType>;
      /**
       * Proxy registration not found.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Sender is not a proxy of the account to be proxied.
       **/
      NotProxy: AugmentedError<ApiType>;
      /**
       * There are too many proxies registered or too many announcements pending.
       **/
      TooMany: AugmentedError<ApiType>;
      /**
       * Announcement, if made at all, was made too recently.
       **/
      Unannounced: AugmentedError<ApiType>;
      /**
       * A call which is incompatible with the proxy type's filter was attempted.
       **/
      Unproxyable: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    rewards: {
      /**
       * An arithmetic operation resulted in an overflow.
       **/
      ArithmeticOverflow: AugmentedError<ApiType>;
      /**
       * Asset already exists in a reward vault
       **/
      AssetAlreadyInVault: AugmentedError<ApiType>;
      /**
       * Asset is already whitelisted
       **/
      AssetAlreadyWhitelisted: AugmentedError<ApiType>;
      /**
       * Asset not found in reward vault
       **/
      AssetNotInVault: AugmentedError<ApiType>;
      /**
       * Asset is not whitelisted for rewards
       **/
      AssetNotWhitelisted: AugmentedError<ApiType>;
      /**
       * Error returned when trying to remove a blueprint ID that doesn't exist.
       **/
      BlueprintIdNotFound: AugmentedError<ApiType>;
      /**
       * Boost multiplier must be 1
       **/
      BoostMultiplierMustBeOne: AugmentedError<ApiType>;
      /**
       * Arithmetic operation caused an overflow
       **/
      CannotCalculatePropotionalApy: AugmentedError<ApiType>;
      /**
       * Error returned when trying to calculate reward per block
       **/
      CannotCalculateRewardPerBlock: AugmentedError<ApiType>;
      /**
       * Deposit cap is greater than max deposit cap
       **/
      DepositCapGreaterThanMaxDepositCap: AugmentedError<ApiType>;
      /**
       * Deposit cap is less than min deposit cap
       **/
      DepositCapLessThanMinDepositCap: AugmentedError<ApiType>;
      /**
       * Error returned when trying to add a blueprint ID that already exists.
       **/
      DuplicateBlueprintId: AugmentedError<ApiType>;
      /**
       * Incentive cap is greater than deposit cap
       **/
      IncentiveCapGreaterThanDepositCap: AugmentedError<ApiType>;
      /**
       * Incentive cap is greater than max incentive cap
       **/
      IncentiveCapGreaterThanMaxIncentiveCap: AugmentedError<ApiType>;
      /**
       * Incentive cap is less than min incentive cap
       **/
      IncentiveCapLessThanMinIncentiveCap: AugmentedError<ApiType>;
      /**
       * Insufficient rewards balance in pallet account
       **/
      InsufficientRewardsBalance: AugmentedError<ApiType>;
      /**
       * Invalid APY value
       **/
      InvalidAPY: AugmentedError<ApiType>;
      /**
       * Decay rate is too high
       **/
      InvalidDecayRate: AugmentedError<ApiType>;
      /**
       * Vault logo exceeds the maximum allowed length.
       **/
      LogoTooLong: AugmentedError<ApiType>;
      /**
       * Vault name exceeds the maximum allowed length.
       **/
      NameTooLong: AugmentedError<ApiType>;
      /**
       * No rewards available to claim
       **/
      NoRewardsAvailable: AugmentedError<ApiType>;
      /**
       * Operator has no pending rewards to claim.
       **/
      NoRewardsToClaim: AugmentedError<ApiType>;
      /**
       * Pot account not found
       **/
      PotAccountNotFound: AugmentedError<ApiType>;
      /**
       * Pot account not found
       **/
      PotAlreadyExists: AugmentedError<ApiType>;
      /**
       * Error returned when the reward configuration for the vault is not found.
       **/
      RewardConfigNotFound: AugmentedError<ApiType>;
      /**
       * Operator has too many pending rewards.
       **/
      TooManyPendingRewards: AugmentedError<ApiType>;
      /**
       * Total deposit is less than incentive cap
       **/
      TotalDepositLessThanIncentiveCap: AugmentedError<ApiType>;
      /**
       * Failed to transfer funds.
       **/
      TransferFailed: AugmentedError<ApiType>;
      /**
       * Vault already exists
       **/
      VaultAlreadyExists: AugmentedError<ApiType>;
      /**
       * Vault metadata not found for the given vault ID.
       **/
      VaultMetadataNotFound: AugmentedError<ApiType>;
      /**
       * The reward vault does not exist
       **/
      VaultNotFound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    scheduler: {
      /**
       * Failed to schedule a call
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Attempt to use a non-named function on a named task.
       **/
      Named: AugmentedError<ApiType>;
      /**
       * Cannot find the scheduled call.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Reschedule failed because it does not change scheduled time.
       **/
      RescheduleNoChange: AugmentedError<ApiType>;
      /**
       * Given target block number is in the past.
       **/
      TargetBlockNumberInPast: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    services: {
      /**
       * Operator is a member or has already joined the service
       **/
      AlreadyJoined: AugmentedError<ApiType>;
      /**
       * The caller is already registered as a operator.
       **/
      AlreadyRegistered: AugmentedError<ApiType>;
      /**
       * Approval Process is interrupted.
       **/
      ApprovalInterrupted: AugmentedError<ApiType>;
      /**
       * The approval is not requested for the operator (the caller).
       **/
      ApprovalNotRequested: AugmentedError<ApiType>;
      /**
       * Blueprint creation is interrupted.
       **/
      BlueprintCreationInterrupted: AugmentedError<ApiType>;
      /**
       * The service blueprint was not found.
       **/
      BlueprintNotFound: AugmentedError<ApiType>;
      /**
       * Custom asset transfer failed
       **/
      CustomAssetTransferFailed: AugmentedError<ApiType>;
      /**
       * Duplicate assets provided
       **/
      DuplicateAsset: AugmentedError<ApiType>;
      /**
       * Duplicate key used for registration.
       **/
      DuplicateKey: AugmentedError<ApiType>;
      /**
       * Duplicate membership model
       **/
      DuplicateMembershipModel: AugmentedError<ApiType>;
      /**
       * Duplicate operator registration.
       **/
      DuplicateOperator: AugmentedError<ApiType>;
      /**
       * Service does not support dynamic membership
       **/
      DynamicMembershipNotSupported: AugmentedError<ApiType>;
      /**
       * The ERC20 transfer failed.
       **/
      ERC20TransferFailed: AugmentedError<ApiType>;
      /**
       * An error occurred while decoding the EVM ABI.
       **/
      EVMAbiDecode: AugmentedError<ApiType>;
      /**
       * An error occurred while encoding the EVM ABI.
       **/
      EVMAbiEncode: AugmentedError<ApiType>;
      /**
       * Expected the account to be an account ID.
       **/
      ExpectedAccountId: AugmentedError<ApiType>;
      /**
       * Expected the account to be an EVM address.
       **/
      ExpectedEVMAddress: AugmentedError<ApiType>;
      /**
       * Get Heartbeat Interval Failure
       **/
      GetHeartbeatIntervalFailure: AugmentedError<ApiType>;
      /**
       * Get Heartbeat Threshold Failure
       **/
      GetHeartbeatThresholdFailure: AugmentedError<ApiType>;
      /**
       * Get Slashing Window Failure
       **/
      GetSlashingWindowFailure: AugmentedError<ApiType>;
      /**
       * Heartbeat signature verification failed
       **/
      HeartbeatSignatureVerificationFailed: AugmentedError<ApiType>;
      /**
       * Heartbeat too early
       **/
      HeartbeatTooEarly: AugmentedError<ApiType>;
      /**
       * Invalid heartbeat data
       **/
      InvalidHeartbeatData: AugmentedError<ApiType>;
      /**
       * The caller does not have the requirements to call a job.
       **/
      InvalidJobCallInput: AugmentedError<ApiType>;
      /**
       * Invalid Job ID provided
       **/
      InvalidJobId: AugmentedError<ApiType>;
      /**
       * The caller provided an invalid job result.
       **/
      InvalidJobResult: AugmentedError<ApiType>;
      /**
       * Invalid key (zero byte ECDSA key provided)
       **/
      InvalidKey: AugmentedError<ApiType>;
      /**
       * Invalid key for quote
       **/
      InvalidKeyForQuote: AugmentedError<ApiType>;
      /**
       * Invalid quote signature
       **/
      InvalidQuoteSignature: AugmentedError<ApiType>;
      /**
       * The Operator is not allowed to register.
       **/
      InvalidRegistrationInput: AugmentedError<ApiType>;
      /**
       * The caller does not have the requirements to request a service.
       **/
      InvalidRequestInput: AugmentedError<ApiType>;
      /**
       * Invalid security commitments
       **/
      InvalidSecurityCommitments: AugmentedError<ApiType>;
      /**
       * Invalid Security Requirements
       **/
      InvalidSecurityRequirements: AugmentedError<ApiType>;
      /**
       * Invalid signature bytes
       **/
      InvalidSignatureBytes: AugmentedError<ApiType>;
      /**
       * Invalid slash percentage
       **/
      InvalidSlashPercentage: AugmentedError<ApiType>;
      /**
       * The result of the job call was not found.
       **/
      JobCallResultNotFound: AugmentedError<ApiType>;
      /**
       * The requested job definition does not exist.
       * This error is returned when the requested job definition does not exist in the service
       * blueprint.
       **/
      JobDefinitionNotFound: AugmentedError<ApiType>;
      /**
       * Cannot join service - rejected by blueprint
       **/
      JoinRejected: AugmentedError<ApiType>;
      /**
       * Cannot leave service - rejected by blueprint
       **/
      LeaveRejected: AugmentedError<ApiType>;
      /**
       * The Supplied Master Blueprint Service Manager Revision is not found.
       **/
      MasterBlueprintServiceManagerRevisionNotFound: AugmentedError<ApiType>;
      /**
       * The maximum number of assets per service has been exceeded.
       **/
      MaxAssetsPerServiceExceeded: AugmentedError<ApiType>;
      /**
       * Maximum number of blueprints registered by the operator reached.
       **/
      MaxBlueprintsPerOperatorExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of fields per request has been exceeded.
       **/
      MaxFieldsExceeded: AugmentedError<ApiType>;
      /**
       * Maximum number of Master Blueprint Service Manager revisions reached.
       **/
      MaxMasterBlueprintServiceManagerVersionsExceeded: AugmentedError<ApiType>;
      /**
       * Maximum operators reached
       **/
      MaxOperatorsReached: AugmentedError<ApiType>;
      /**
       * The maximum number of permitted callers per service has been exceeded.
       **/
      MaxPermittedCallersExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of operators per service has been exceeded.
       **/
      MaxServiceProvidersExceeded: AugmentedError<ApiType>;
      /**
       * Maximum number of services per operator reached.
       **/
      MaxServicesPerOperatorExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of services per user has been exceeded.
       **/
      MaxServicesPerUserExceeded: AugmentedError<ApiType>;
      /**
       * Missing EVM Origin for the EVM execution.
       **/
      MissingEVMOrigin: AugmentedError<ApiType>;
      /**
       * Missing quote signature
       **/
      MissingQuoteSignature: AugmentedError<ApiType>;
      /**
       * Native asset exposure is too low
       **/
      NativeAssetExposureTooLow: AugmentedError<ApiType>;
      /**
       * No assets provided for the service, at least one asset is required.
       **/
      NoAssetsProvided: AugmentedError<ApiType>;
      /**
       * The Service Blueprint did not return a dispute origin for this service.
       **/
      NoDisputeOrigin: AugmentedError<ApiType>;
      /**
       * Native asset is not found
       **/
      NoNativeAsset: AugmentedError<ApiType>;
      /**
       * The Service Blueprint did not return a slashing origin for this service.
       **/
      NoSlashingOrigin: AugmentedError<ApiType>;
      /**
       * The Operator is not allowed to unregister.
       **/
      NotAllowedToUnregister: AugmentedError<ApiType>;
      /**
       * The Operator is not allowed to update their RPC address.
       **/
      NotAllowedToUpdateRpcAddress: AugmentedError<ApiType>;
      /**
       * Caller is not an operator of the service
       **/
      NotAnOperator: AugmentedError<ApiType>;
      /**
       * The caller is not registered as a operator.
       **/
      NotRegistered: AugmentedError<ApiType>;
      /**
       * Offender is not a registered operator.
       **/
      OffenderNotOperator: AugmentedError<ApiType>;
      /**
       * Approve service request hook failure
       **/
      OnApproveFailure: AugmentedError<ApiType>;
      /**
       * Can join hook failure
       **/
      OnCanJoinFailure: AugmentedError<ApiType>;
      /**
       * Can leave hook failure
       **/
      OnCanLeaveFailure: AugmentedError<ApiType>;
      /**
       * Operator join hook failure
       **/
      OnOperatorJoinFailure: AugmentedError<ApiType>;
      /**
       * Operator leave hook failure
       **/
      OnOperatorLeaveFailure: AugmentedError<ApiType>;
      /**
       * Register hook failure
       **/
      OnRegisterHookFailed: AugmentedError<ApiType>;
      /**
       * Reject service request hook failure
       **/
      OnRejectFailure: AugmentedError<ApiType>;
      /**
       * Request hook failure
       **/
      OnRequestFailure: AugmentedError<ApiType>;
      /**
       * Service init hook
       **/
      OnServiceInitHook: AugmentedError<ApiType>;
      /**
       * The Operator is not active in the delegation system.
       **/
      OperatorNotActive: AugmentedError<ApiType>;
      /**
       * Operator profile not found.
       **/
      OperatorProfileNotFound: AugmentedError<ApiType>;
      /**
       * Payment has already been processed for this call
       **/
      PaymentAlreadyProcessed: AugmentedError<ApiType>;
      /**
       * Payment calculation overflow
       **/
      PaymentCalculationOverflow: AugmentedError<ApiType>;
      /**
       * Rejection Process is interrupted.
       **/
      RejectionInterrupted: AugmentedError<ApiType>;
      /**
       * Service Initialization interrupted.
       **/
      ServiceInitializationInterrupted: AugmentedError<ApiType>;
      /**
       * Service not active
       **/
      ServiceNotActive: AugmentedError<ApiType>;
      /**
       * The service was not found.
       **/
      ServiceNotFound: AugmentedError<ApiType>;
      /**
       * Either the service or the job call was not found.
       **/
      ServiceOrJobCallNotFound: AugmentedError<ApiType>;
      /**
       * The service request was not found.
       **/
      ServiceRequestNotFound: AugmentedError<ApiType>;
      /**
       * Mismatched number of signatures
       **/
      SignatureCountMismatch: AugmentedError<ApiType>;
      /**
       * Signature verification failed
       **/
      SignatureVerificationFailed: AugmentedError<ApiType>;
      /**
       * The termination of the service was interrupted.
       **/
      TerminationInterrupted: AugmentedError<ApiType>;
      /**
       * Too few operators provided for the service's membership model
       **/
      TooFewOperators: AugmentedError<ApiType>;
      /**
       * Too many operators provided for the service's membership model
       **/
      TooManyOperators: AugmentedError<ApiType>;
      /**
       * Too many subscriptions per user
       **/
      TooManySubscriptions: AugmentedError<ApiType>;
      /**
       * An error occurred while type checking the provided input input.
       **/
      TypeCheck: AugmentedError<ApiType>;
      /**
       * The Unapplied Slash are not found.
       **/
      UnappliedSlashNotFound: AugmentedError<ApiType>;
      /**
       * Membership model not supported by blueprint
       **/
      UnsupportedMembershipModel: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    session: {
      /**
       * Registered duplicate key.
       **/
      DuplicatedKey: AugmentedError<ApiType>;
      /**
       * Invalid ownership proof.
       **/
      InvalidProof: AugmentedError<ApiType>;
      /**
       * Key setting account is not live, so it's impossible to associate keys.
       **/
      NoAccount: AugmentedError<ApiType>;
      /**
       * No associated validator ID for account.
       **/
      NoAssociatedValidatorId: AugmentedError<ApiType>;
      /**
       * No keys are associated with this account.
       **/
      NoKeys: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    staking: {
      /**
       * Stash is already bonded.
       **/
      AlreadyBonded: AugmentedError<ApiType>;
      /**
       * Rewards for this era have already been claimed for this validator.
       **/
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Controller is already paired.
       **/
      AlreadyPaired: AugmentedError<ApiType>;
      /**
       * Internal state has become somehow corrupted and the operation cannot continue.
       **/
      BadState: AugmentedError<ApiType>;
      /**
       * A nomination target was supplied that was blocked or otherwise not a validator.
       **/
      BadTarget: AugmentedError<ApiType>;
      /**
       * Some bound is not met.
       **/
      BoundNotMet: AugmentedError<ApiType>;
      /**
       * The user has enough bond and thus cannot be chilled forcefully by an external person.
       **/
      CannotChillOther: AugmentedError<ApiType>;
      /**
       * Cannot reset a ledger.
       **/
      CannotRestoreLedger: AugmentedError<ApiType>;
      /**
       * Commission is too low. Must be at least `MinCommission`.
       **/
      CommissionTooLow: AugmentedError<ApiType>;
      /**
       * Used when attempting to use deprecated controller account logic.
       **/
      ControllerDeprecated: AugmentedError<ApiType>;
      /**
       * Duplicate index.
       **/
      DuplicateIndex: AugmentedError<ApiType>;
      /**
       * Targets cannot be empty.
       **/
      EmptyTargets: AugmentedError<ApiType>;
      /**
       * Attempting to target a stash that still has funds.
       **/
      FundedTarget: AugmentedError<ApiType>;
      /**
       * Incorrect previous history depth input provided.
       **/
      IncorrectHistoryDepth: AugmentedError<ApiType>;
      /**
       * Incorrect number of slashing spans provided.
       **/
      IncorrectSlashingSpans: AugmentedError<ApiType>;
      /**
       * Cannot have a validator or nominator role, with value less than the minimum defined by
       * governance (see `MinValidatorBond` and `MinNominatorBond`). If unbonding is the
       * intention, `chill` first to remove one's role as validator/nominator.
       **/
      InsufficientBond: AugmentedError<ApiType>;
      /**
       * Invalid era to reward.
       **/
      InvalidEraToReward: AugmentedError<ApiType>;
      /**
       * Invalid number of nominations.
       **/
      InvalidNumberOfNominations: AugmentedError<ApiType>;
      /**
       * No nominators exist on this page.
       **/
      InvalidPage: AugmentedError<ApiType>;
      /**
       * Slash record index out of bounds.
       **/
      InvalidSlashIndex: AugmentedError<ApiType>;
      /**
       * Can not schedule more unlock chunks.
       **/
      NoMoreChunks: AugmentedError<ApiType>;
      /**
       * Not a controller account.
       **/
      NotController: AugmentedError<ApiType>;
      /**
       * Not enough funds available to withdraw.
       **/
      NotEnoughFunds: AugmentedError<ApiType>;
      /**
       * Items are not sorted and unique.
       **/
      NotSortedAndUnique: AugmentedError<ApiType>;
      /**
       * Not a stash account.
       **/
      NotStash: AugmentedError<ApiType>;
      /**
       * Can not rebond without unlocking chunks.
       **/
      NoUnlockChunk: AugmentedError<ApiType>;
      /**
       * Provided reward destination is not allowed.
       **/
      RewardDestinationRestricted: AugmentedError<ApiType>;
      /**
       * There are too many nominators in the system. Governance needs to adjust the staking
       * settings to keep things safe for the runtime.
       **/
      TooManyNominators: AugmentedError<ApiType>;
      /**
       * Too many nomination targets supplied.
       **/
      TooManyTargets: AugmentedError<ApiType>;
      /**
       * There are too many validator candidates in the system. Governance needs to adjust the
       * staking settings to keep things safe for the runtime.
       **/
      TooManyValidators: AugmentedError<ApiType>;
      /**
       * Operation not allowed for virtual stakers.
       **/
      VirtualStakerNotAllowed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    sudo: {
      /**
       * Sender must be the Sudo account.
       **/
      RequireSudo: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    system: {
      /**
       * The origin filter prevent the call to be dispatched.
       **/
      CallFiltered: AugmentedError<ApiType>;
      /**
       * Failed to extract the runtime version from the new runtime.
       * 
       * Either calling `Core_version` or decoding `RuntimeVersion` failed.
       **/
      FailedToExtractRuntimeVersion: AugmentedError<ApiType>;
      /**
       * The name of specification does not match between the current runtime
       * and the new runtime.
       **/
      InvalidSpecName: AugmentedError<ApiType>;
      /**
       * A multi-block migration is ongoing and prevents the current code from being replaced.
       **/
      MultiBlockMigrationsOngoing: AugmentedError<ApiType>;
      /**
       * Suicide called when the account has non-default composite data.
       **/
      NonDefaultComposite: AugmentedError<ApiType>;
      /**
       * There is a non-zero reference count preventing the account from being purged.
       **/
      NonZeroRefCount: AugmentedError<ApiType>;
      /**
       * No upgrade authorized.
       **/
      NothingAuthorized: AugmentedError<ApiType>;
      /**
       * The specification version is not allowed to decrease between the current runtime
       * and the new runtime.
       **/
      SpecVersionNeedsToIncrease: AugmentedError<ApiType>;
      /**
       * The submitted code is not authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    tokenGateway: {
      /**
       * Asset Id creation failed
       **/
      AssetCreationError: AugmentedError<ApiType>;
      /**
       * Asset decimals not found
       **/
      AssetDecimalsNotFound: AugmentedError<ApiType>;
      /**
       * Error while teleporting asset
       **/
      AssetTeleportError: AugmentedError<ApiType>;
      /**
       * Coprocessor was not configured in the runtime
       **/
      CoprocessorNotConfigured: AugmentedError<ApiType>;
      /**
       * Asset or update Dispatch Error
       **/
      DispatchError: AugmentedError<ApiType>;
      /**
       * Only root or asset owner can update asset
       **/
      NotAssetOwner: AugmentedError<ApiType>;
      /**
       * Protocol Params have not been initialized
       **/
      NotInitialized: AugmentedError<ApiType>;
      /**
       * Unknown Asset
       **/
      UnknownAsset: AugmentedError<ApiType>;
      /**
       * A asset that has not been registered
       **/
      UnregisteredAsset: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    treasury: {
      /**
       * The payment has already been attempted.
       **/
      AlreadyAttempted: AugmentedError<ApiType>;
      /**
       * The spend is not yet eligible for payout.
       **/
      EarlyPayout: AugmentedError<ApiType>;
      /**
       * The balance of the asset kind is not convertible to the balance of the native asset.
       **/
      FailedToConvertBalance: AugmentedError<ApiType>;
      /**
       * The payment has neither failed nor succeeded yet.
       **/
      Inconclusive: AugmentedError<ApiType>;
      /**
       * The spend origin is valid but the amount it is allowed to spend is lower than the
       * amount to be spent.
       **/
      InsufficientPermission: AugmentedError<ApiType>;
      /**
       * No proposal, bounty or spend at that index.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * The payout was not yet attempted/claimed.
       **/
      NotAttempted: AugmentedError<ApiType>;
      /**
       * There was some issue with the mechanism of payment.
       **/
      PayoutError: AugmentedError<ApiType>;
      /**
       * Proposal has not been approved.
       **/
      ProposalNotApproved: AugmentedError<ApiType>;
      /**
       * The spend has expired and cannot be claimed.
       **/
      SpendExpired: AugmentedError<ApiType>;
      /**
       * Too many approvals in the queue.
       **/
      TooManyApprovals: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    txPause: {
      /**
       * The call is paused.
       **/
      IsPaused: AugmentedError<ApiType>;
      /**
       * The call is unpaused.
       **/
      IsUnpaused: AugmentedError<ApiType>;
      NotFound: AugmentedError<ApiType>;
      /**
       * The call is whitelisted and cannot be paused.
       **/
      Unpausable: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    utility: {
      /**
       * Too many calls batched.
       **/
      TooManyCalls: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    vesting: {
      /**
       * Amount being transferred is too low to create a vesting schedule.
       **/
      AmountLow: AugmentedError<ApiType>;
      /**
       * The account already has `MaxVestingSchedules` count of schedules and thus
       * cannot add another one. Consider merging existing schedules in order to add another.
       **/
      AtMaxVestingSchedules: AugmentedError<ApiType>;
      /**
       * Failed to create a new schedule because some parameter was invalid.
       **/
      InvalidScheduleParams: AugmentedError<ApiType>;
      /**
       * The account given is not vesting.
       **/
      NotVesting: AugmentedError<ApiType>;
      /**
       * An index was out of bounds of the vesting schedules.
       **/
      ScheduleIndexOutOfBounds: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
  } // AugmentedErrors
} // declare module
