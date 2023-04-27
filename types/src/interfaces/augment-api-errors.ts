// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/errors';

import type { ApiTypes, AugmentedError } from '@polkadot/api-base/types';

export type __AugmentedError<ApiType extends ApiTypes> = AugmentedError<ApiType>;

declare module '@polkadot/api-base/types/errors' {
  interface AugmentedErrors<ApiType extends ApiTypes> {
    assetRegistry: {
      /**
       * Asset is already registered.
       **/
      AssetAlreadyRegistered: AugmentedError<ApiType>;
      /**
       * Asset exists in to pool
       **/
      AssetExistsInPool: AugmentedError<ApiType>;
      /**
       * Invalid asset name or symbol.
       **/
      AssetNotFound: AugmentedError<ApiType>;
      /**
       * Asset not found in pool
       **/
      AssetNotFoundInPool: AugmentedError<ApiType>;
      /**
       * Asset ID is not registered in the asset-registry.
       **/
      AssetNotRegistered: AugmentedError<ApiType>;
      /**
       * Incorrect number of assets provided to create shared asset.
       **/
      InvalidSharedAssetLen: AugmentedError<ApiType>;
      /**
       * Max number of assets in pool is reached
       **/
      MaxAssetIdInPoolReached: AugmentedError<ApiType>;
      /**
       * Asset Id is not available. This only happens when it reaches the MAX
       * value of given id type.
       **/
      NoIdAvailable: AugmentedError<ApiType>;
      /**
       * Invalid asset name or symbol.
       **/
      TooLong: AugmentedError<ApiType>;
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
       * Beneficiary account must pre-exist
       **/
      DeadAccount: AugmentedError<ApiType>;
      /**
       * Value too low to create account due to existential deposit
       **/
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * A vesting schedule already exists for this account
       **/
      ExistingVestingSchedule: AugmentedError<ApiType>;
      /**
       * Balance too low to send value.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Transfer/payment would kill account
       **/
      KeepAlive: AugmentedError<ApiType>;
      /**
       * Account liquidity restrictions prevent withdrawal
       **/
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Number of named reserves exceed MaxReserves
       **/
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * Vesting balance too high to send value
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
    bridgeRegistry: {
      /**
       * Resources map to different bridge indices
       **/
      BridgeIndexError: AugmentedError<ApiType>;
      /**
       * Bridge does not exist.
       **/
      BridgeNotFound: AugmentedError<ApiType>;
      /**
       * Parameters haven't been initialized
       **/
      ParametersNotInitialized: AugmentedError<ApiType>;
      /**
       * Proposal is not signed and should not be processed
       **/
      ProposalNotSigned: AugmentedError<ApiType>;
      /**
       * Too many additional fields.
       **/
      TooManyFields: AugmentedError<ApiType>;
      /**
       * Too many resources.
       **/
      TooManyResources: AugmentedError<ApiType>;
      /**
       * Error during verification
       **/
      VerifyError: AugmentedError<ApiType>;
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
    currencies: {
      /**
       * Unable to convert the Amount type into Balance.
       **/
      AmountIntoBalanceFailed: AugmentedError<ApiType>;
      /**
       * Balance is too low.
       **/
      BalanceTooLow: AugmentedError<ApiType>;
      /**
       * Deposit result is not expected
       **/
      DepositFailed: AugmentedError<ApiType>;
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
    dkg: {
      /**
       * Already submitted a public key
       **/
      AlreadySubmittedPublicKey: AugmentedError<ApiType>;
      /**
       * Already submitted a public key signature
       **/
      AlreadySubmittedSignature: AugmentedError<ApiType>;
      /**
       * Must be calling from the controller account
       **/
      InvalidControllerAccount: AugmentedError<ApiType>;
      /**
       * Invalid misbehaviour reports
       **/
      InvalidMisbehaviourReports: AugmentedError<ApiType>;
      /**
       * Invalid Nonece used, must be greater than [`refresh_nonce`].
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Invalid public key submission
       **/
      InvalidPublicKeys: AugmentedError<ApiType>;
      /**
       * Refresh delay should be in the range of 0% - 100%
       **/
      InvalidRefreshDelay: AugmentedError<ApiType>;
      /**
       * Invalid public key signature submission
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Invalid threshold
       **/
      InvalidThreshold: AugmentedError<ApiType>;
      /**
       * Must be an an authority
       **/
      MustBeAnActiveAuthority: AugmentedError<ApiType>;
      /**
       * Must be queued  to become an authority
       **/
      MustBeAQueuedAuthority: AugmentedError<ApiType>;
      /**
       * No mapped account to authority
       **/
      NoMappedAccount: AugmentedError<ApiType>;
      /**
       * No NextPublicKey stored on-chain.
       **/
      NoNextPublicKey: AugmentedError<ApiType>;
      /**
       * Input is out of bounds
       **/
      OutOfBounds: AugmentedError<ApiType>;
      /**
       * DKG Refresh is already in progress.
       **/
      RefreshInProgress: AugmentedError<ApiType>;
      /**
       * Used signature from past sessions
       **/
      UsedSignature: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    dkgProposalHandler: {
      /**
       * Chain id is invalid
       **/
      ChainIdInvalid: AugmentedError<ApiType>;
      /**
       * Proposal bytes length is invalid
       **/
      InvalidProposalBytesLength: AugmentedError<ApiType>;
      /**
       * Error names should be descriptive.
       **/
      NoneValue: AugmentedError<ApiType>;
      /**
       * Proposal with the ID has already been submitted
       **/
      ProposalAlreadyExists: AugmentedError<ApiType>;
      /**
       * No proposal with the ID was found
       **/
      ProposalDoesNotExists: AugmentedError<ApiType>;
      /**
       * Proposal format is invalid
       **/
      ProposalFormatInvalid: AugmentedError<ApiType>;
      /**
       * Proposal must be unsigned
       **/
      ProposalMustBeUnsigned: AugmentedError<ApiType>;
      /**
       * Proposal out of bounds
       **/
      ProposalOutOfBounds: AugmentedError<ApiType>;
      /**
       * Proposal signature is invalid
       **/
      ProposalSignatureInvalid: AugmentedError<ApiType>;
      /**
       * Proposal length exceeds max allowed per batch
       **/
      ProposalsLengthOverflow: AugmentedError<ApiType>;
      /**
       * Errors should have helpful documentation associated with them.
       **/
      StorageOverflow: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    dkgProposals: {
      /**
       * Chain has already been enabled
       **/
      ChainAlreadyWhitelisted: AugmentedError<ApiType>;
      /**
       * Interactions with this chain is not permitted
       **/
      ChainNotWhitelisted: AugmentedError<ApiType>;
      /**
       * Provided chain Id is not valid
       **/
      InvalidChainId: AugmentedError<ApiType>;
      /**
       * Account does not have correct permissions
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      /**
       * Proposer threshold cannot be 0
       **/
      InvalidThreshold: AugmentedError<ApiType>;
      /**
       * Protected operation, must be performed by proposer
       **/
      MustBeProposer: AugmentedError<ApiType>;
      /**
       * Input is out of bounds
       **/
      OutOfBounds: AugmentedError<ApiType>;
      /**
       * Proposal has either failed or succeeded
       **/
      ProposalAlreadyComplete: AugmentedError<ApiType>;
      /**
       * A proposal with these parameters has already been submitted
       **/
      ProposalAlreadyExists: AugmentedError<ApiType>;
      /**
       * No proposal with the ID was found
       **/
      ProposalDoesNotExist: AugmentedError<ApiType>;
      /**
       * Lifetime of proposal has been exceeded
       **/
      ProposalExpired: AugmentedError<ApiType>;
      /**
       * Cannot complete proposal, needs more votes
       **/
      ProposalNotComplete: AugmentedError<ApiType>;
      /**
       * Proposer already in set
       **/
      ProposerAlreadyExists: AugmentedError<ApiType>;
      /**
       * Proposer has already submitted some vote for this proposal
       **/
      ProposerAlreadyVoted: AugmentedError<ApiType>;
      /**
       * Proposer Count is Zero
       **/
      ProposerCountIsZero: AugmentedError<ApiType>;
      /**
       * Provided accountId is not a proposer
       **/
      ProposerInvalid: AugmentedError<ApiType>;
      /**
       * Resource ID provided isn't mapped to anything
       **/
      ResourceDoesNotExist: AugmentedError<ApiType>;
      /**
       * Proposer threshold not set
       **/
      ThresholdNotSet: AugmentedError<ApiType>;
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
    eth2Client: {
      ActiveHeaderSlotNumberLessThanFinalizedSlot: AugmentedError<ApiType>;
      /**
       * The light client is already initialized for the typed chain ID
       **/
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Block already submitted
       **/
      BlockAlreadySubmitted: AugmentedError<ApiType>;
      BlockHashesDoNotMatch: AugmentedError<ApiType>;
      CurrentSyncCommitteeNotSet: AugmentedError<ApiType>;
      FinalizedBeaconHeaderNotPresent: AugmentedError<ApiType>;
      FinalizedExecutionHeaderNotPresent: AugmentedError<ApiType>;
      ForkEpochNotFound: AugmentedError<ApiType>;
      ForkVersionNotFound: AugmentedError<ApiType>;
      GenesisValidatorsRootNotFound: AugmentedError<ApiType>;
      HeaderHashDoesNotExist: AugmentedError<ApiType>;
      InvalidBlsSignature: AugmentedError<ApiType>;
      InvalidExecutionBlock: AugmentedError<ApiType>;
      InvalidExecutionBlockHashProof: AugmentedError<ApiType>;
      InvalidFinalityProof: AugmentedError<ApiType>;
      InvalidNextSyncCommitteeProof: AugmentedError<ApiType>;
      InvalidSignaturePeriod: AugmentedError<ApiType>;
      InvalidSyncCommitteeBitsSum: AugmentedError<ApiType>;
      InvalidUpdatePeriod: AugmentedError<ApiType>;
      /**
       * For attempting to update the light client
       **/
      LightClientUpdateNotAllowed: AugmentedError<ApiType>;
      NextSyncCommitteeNotPresent: AugmentedError<ApiType>;
      NextSyncCommitteeNotSet: AugmentedError<ApiType>;
      /**
       * Self-explanatory
       **/
      NotTrustedSigner: AugmentedError<ApiType>;
      /**
       * For attempting to register
       **/
      SubmitterAlreadyRegistered: AugmentedError<ApiType>;
      SubmitterExhaustedLimit: AugmentedError<ApiType>;
      /**
       * For attempting to unregister
       **/
      SubmitterHasUsedStorage: AugmentedError<ApiType>;
      /**
       * For attempting to unregister
       **/
      SubmitterNotRegistered: AugmentedError<ApiType>;
      SyncCommitteeBitsSumLessThanThreshold: AugmentedError<ApiType>;
      SyncCommitteeUpdateNotPresent: AugmentedError<ApiType>;
      /**
       * The client can't be executed in the trustless mode without BLS sigs verification on
       * Mainnet
       **/
      TrustlessModeError: AugmentedError<ApiType>;
      UnfinalizedHeaderNotPresent: AugmentedError<ApiType>;
      /**
       * Unknown parent block header hash
       **/
      UnknownParentHeader: AugmentedError<ApiType>;
      /**
       * The updates validation can't be disabled for mainnet
       **/
      ValidateUpdatesParameterError: AugmentedError<ApiType>;
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
    hasherBn254: {
      /**
       * Error during hashing
       **/
      HashError: AugmentedError<ApiType>;
      /**
       * Parameters haven't been initialized
       **/
      HashingParametersNotInitialized: AugmentedError<ApiType>;
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
       * The target is invalid.
       **/
      InvalidTarget: AugmentedError<ApiType>;
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
       * No identity found.
       **/
      NoIdentity: AugmentedError<ApiType>;
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
       * Sticky judgement.
       **/
      StickyJudgement: AugmentedError<ApiType>;
      /**
       * Too many additional fields.
       **/
      TooManyFields: AugmentedError<ApiType>;
      /**
       * Maximum amount of registrars reached. Cannot add any more.
       **/
      TooManyRegistrars: AugmentedError<ApiType>;
      /**
       * Too many subs-accounts.
       **/
      TooManySubAccounts: AugmentedError<ApiType>;
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
    keyStorage: {
      MaxPubkeyOwnersExceeded: AugmentedError<ApiType>;
      PubKeyOutOfBounds: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    linkableTreeBn254: {
      /**
       * Edge already exists
       **/
      EdgeAlreadyExists: AugmentedError<ApiType>;
      /**
       * Edge does not exist
       **/
      EdgeDoesntExists: AugmentedError<ApiType>;
      /**
       * Invalid latest leaf index
       **/
      InvalidLatestLeafIndex: AugmentedError<ApiType>;
      /**
       * Invalid Merkle Roots
       **/
      InvalidMerkleRoots: AugmentedError<ApiType>;
      /**
       * Invalid neighbor root passed in withdrawal
       * (neighbor root is not in neighbor history)
       **/
      InvalidNeighborWithdrawRoot: AugmentedError<ApiType>;
      /**
       * Anchor is at maximum number of edges for the given tree
       **/
      TooManyEdges: AugmentedError<ApiType>;
      UnknownRoot: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    merkleTreeBn254: {
      /**
       * Invalid length for default hashes
       **/
      ExceedsMaxDefaultHashes: AugmentedError<ApiType>;
      /**
       * Invalid length for edges
       **/
      ExceedsMaxEdges: AugmentedError<ApiType>;
      /**
       * Tree is full
       **/
      ExceedsMaxLeaves: AugmentedError<ApiType>;
      /**
       * Invalid  leaf index,  either taken or too large
       **/
      InvalidLeafIndex: AugmentedError<ApiType>;
      /**
       * Account does not have correct permissions
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      /**
       * Invalid depth of the tree specified
       **/
      InvalidTreeDepth: AugmentedError<ApiType>;
      /**
       * Tree doesnt exist
       **/
      TreeDoesntExist: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    mixerBn254: {
      /**
       * Invalid nullifier that is already used
       * (this error is returned when a nullifier is used twice)
       **/
      AlreadyRevealedNullifier: AugmentedError<ApiType>;
      InvalidArbitraryData: AugmentedError<ApiType>;
      /**
       * Account does not have correct permissions
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      /**
       * Invalid withdraw proof
       **/
      InvalidWithdrawProof: AugmentedError<ApiType>;
      /**
       * No mixer found
       **/
      NoMixerFound: AugmentedError<ApiType>;
      /**
       * Invalid root
       **/
      UnknownRoot: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    mixerVerifierBn254: {
      /**
       * Error during verification
       **/
      VerifyError: AugmentedError<ApiType>;
      /**
       * Parameters haven't been initialized
       **/
      VerifyingParametersNotInitialized: AugmentedError<ApiType>;
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
       * The depositor can never unbond to a value less than
       * `Pallet::depositor_min_bond`. The caller does not have nominating
       * permissions for the pool. Members can never unbond to a value below `MinJoinBond`.
       **/
      MinimumBondNotMet: AugmentedError<ApiType>;
      /**
       * A pool must be in [`PoolState::Destroying`] in order for the depositor to unbond or for
       * other members to be permissionlessly unbonded.
       **/
      NotDestroying: AugmentedError<ApiType>;
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
    preimage: {
      /**
       * Preimage has already been noted on-chain.
       **/
      AlreadyNoted: AugmentedError<ApiType>;
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
    signatureBridge: {
      /**
       * Call does not match resource id according to resources mapping
       **/
      CallDoesNotMatchResourceId: AugmentedError<ApiType>;
      /**
       * Call does not match parsed call from proposal data
       **/
      CallNotConsistentWithProposalData: AugmentedError<ApiType>;
      /**
       * Chain has already been enabled
       **/
      ChainAlreadyWhitelisted: AugmentedError<ApiType>;
      /**
       * Interactions with this chain is not permitted
       **/
      ChainNotWhitelisted: AugmentedError<ApiType>;
      /**
       * Chain Id Type from the r_id does not match this chain
       **/
      IncorrectExecutionChainIdType: AugmentedError<ApiType>;
      /**
       * Invalid call - calls must be delegated to handler pallets
       **/
      InvalidCall: AugmentedError<ApiType>;
      /**
       * Provided chain Id is not valid
       **/
      InvalidChainId: AugmentedError<ApiType>;
      /**
       * Invalid nonce
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Account does not have correct permissions
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      /**
       * Invalid proposal data
       **/
      InvalidProposalData: AugmentedError<ApiType>;
      /**
       * Protected operation, must be performed by relayer
       **/
      MustBeMaintainer: AugmentedError<ApiType>;
      /**
       * A proposal with these parameters has already been submitted
       **/
      ProposalAlreadyExists: AugmentedError<ApiType>;
      /**
       * Resource ID provided is already mapped to anchor
       **/
      ResourceAlreadyExists: AugmentedError<ApiType>;
      /**
       * Resource ID provided isn't mapped to anything
       **/
      ResourceDoesNotExist: AugmentedError<ApiType>;
      /**
       * Provided signature is not from the active maintainer
       **/
      SignatureInvalid: AugmentedError<ApiType>;
      /**
       * The max limit for string is exceeded
       **/
      StringLimitExceeded: AugmentedError<ApiType>;
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
       * Commission is too low. Must be at least `MinCommission`.
       **/
      CommissionTooLow: AugmentedError<ApiType>;
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
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    sudo: {
      /**
       * Sender must be the Sudo account
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
       * Suicide called when the account has non-default composite data.
       **/
      NonDefaultComposite: AugmentedError<ApiType>;
      /**
       * There is a non-zero reference count preventing the account from being purged.
       **/
      NonZeroRefCount: AugmentedError<ApiType>;
      /**
       * The specification version is not allowed to decrease between the current runtime
       * and the new runtime.
       **/
      SpecVersionNeedsToIncrease: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    tokens: {
      /**
       * Cannot convert Amount into Balance type
       **/
      AmountIntoBalanceFailed: AugmentedError<ApiType>;
      /**
       * The balance is too low
       **/
      BalanceTooLow: AugmentedError<ApiType>;
      /**
       * Beneficiary account must pre-exist
       **/
      DeadAccount: AugmentedError<ApiType>;
      /**
       * Value too low to create account due to existential deposit
       **/
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * Transfer/payment would kill account
       **/
      KeepAlive: AugmentedError<ApiType>;
      /**
       * Failed because liquidity restrictions due to locking
       **/
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Failed because the maximum locks was exceeded
       **/
      MaxLocksExceeded: AugmentedError<ApiType>;
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    tokenWrapper: {
      /**
       * Insufficient Balance for an asset
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Invalid transaction amount
       **/
      InvalidAmount: AugmentedError<ApiType>;
      /**
       * Invalid nonce
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Name exceeds maximum limit
       **/
      NameExceedsMaximumLimit: AugmentedError<ApiType>;
      /**
       * Assets not found in selected pool
       **/
      NotFoundInPool: AugmentedError<ApiType>;
      NoWrappingFeePercentFound: AugmentedError<ApiType>;
      /**
       * AssetId not found in selected pool share
       **/
      UnregisteredAssetId: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    tokenWrapperHandler: {
      /**
       * Access violation.
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      ResourceIsAlreadyAnchored: AugmentedError<ApiType>;
      /**
       * Storage overflowed.
       **/
      StorageOverflow: AugmentedError<ApiType>;
      TokenWrapperHandlerNotFound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    transactionPause: {
      /**
       * can not pause
       **/
      CannotPause: AugmentedError<ApiType>;
      /**
       * invalid character encoding
       **/
      InvalidCharacter: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    treasury: {
      /**
       * The spend origin is valid but the amount it is allowed to spend is lower than the
       * amount to be spent.
       **/
      InsufficientPermission: AugmentedError<ApiType>;
      /**
       * Proposer's balance is too low.
       **/
      InsufficientProposersBalance: AugmentedError<ApiType>;
      /**
       * No proposal or bounty at that index.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Proposal has not been approved.
       **/
      ProposalNotApproved: AugmentedError<ApiType>;
      /**
       * Too many approvals in the queue.
       **/
      TooManyApprovals: AugmentedError<ApiType>;
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
    vAnchorBn254: {
      /**
       * Invalid nullifier that is already used
       * (this error is returned when a nullifier is used twice)
       **/
      AlreadyRevealedNullifier: AugmentedError<ApiType>;
      InvalidDepositAmount: AugmentedError<ApiType>;
      InvalidExtAmount: AugmentedError<ApiType>;
      InvalidExtData: AugmentedError<ApiType>;
      InvalidFee: AugmentedError<ApiType>;
      InvalidInputNullifiers: AugmentedError<ApiType>;
      /**
       * Invalid nonce
       **/
      InvalidNonce: AugmentedError<ApiType>;
      InvalidPublicAmount: AugmentedError<ApiType>;
      /**
       * Invalid transaction proof
       **/
      InvalidTransactionProof: AugmentedError<ApiType>;
      InvalidWithdrawAmount: AugmentedError<ApiType>;
      /**
       * Variable Anchor not found.
       **/
      NoVAnchorFound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    vAnchorHandlerBn254: {
      AnchorHandlerNotFound: AugmentedError<ApiType>;
      /**
       * Invalid nonce
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Access violation.
       **/
      InvalidPermissions: AugmentedError<ApiType>;
      /**
       * Invalid resource ID
       **/
      InvalidResourceId: AugmentedError<ApiType>;
      ResourceIsAlreadyAnchored: AugmentedError<ApiType>;
      SourceChainIdNotFound: AugmentedError<ApiType>;
      /**
       * Storage overflowed.
       **/
      StorageOverflow: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    vAnchorVerifier: {
      /**
       * Error during verification
       **/
      VerifyError: AugmentedError<ApiType>;
      /**
       * Parameters haven't been initialized
       **/
      VerifyingParametersNotInitialized: AugmentedError<ApiType>;
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
