// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/errors';

import type { ApiTypes, AugmentedError } from '@polkadot/api-base/types';

export type __AugmentedError<ApiType extends ApiTypes> = AugmentedError<ApiType>;

declare module '@polkadot/api-base/types/errors' {
  interface AugmentedErrors<ApiType extends ApiTypes> {
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
       * Cannot retreive signers from the signature
       **/
      CannotRetreiveSigner: AugmentedError<ApiType>;
      /**
       * Duplicate identifier
       **/
      DuplicateIdentifier: AugmentedError<ApiType>;
      /**
       * Duplicate signature found in submission
       **/
      DuplicateSignature: AugmentedError<ApiType>;
      /**
       * FROST Field scalar error
       **/
      FrostFieldError: AugmentedError<ApiType>;
      /**
       * FROST Group element error
       **/
      FrostGroupError: AugmentedError<ApiType>;
      /**
       * Identifier derivation not supported
       **/
      IdentifierDerivationNotSupported: AugmentedError<ApiType>;
      /**
       * Invalid FROST commitment
       **/
      IdentityCommitment: AugmentedError<ApiType>;
      /**
       * Incorrect number of identifiers
       **/
      IncorrectNumberOfIdentifiers: AugmentedError<ApiType>;
      /**
       * Invalid BLS public key
       **/
      InvalidBlsPublicKey: AugmentedError<ApiType>;
      /**
       * Invalid FROST message deserialization
       **/
      InvalidFrostMessageDeserialization: AugmentedError<ApiType>;
      /**
       * Invalid FROST message serialization
       **/
      InvalidFrostMessageSerialization: AugmentedError<ApiType>;
      /**
       * Invalid FROST signature
       **/
      InvalidFrostSignature: AugmentedError<ApiType>;
      /**
       * Invalid FROST signature scheme
       **/
      InvalidFrostSignatureScheme: AugmentedError<ApiType>;
      /**
       * Invalid FROST signature share
       **/
      InvalidFrostSignatureShare: AugmentedError<ApiType>;
      /**
       * Invalid identifier deserialization
       **/
      InvalidIdentifierDeserialization: AugmentedError<ApiType>;
      /**
       * Unexpected job type
       **/
      InvalidJobType: AugmentedError<ApiType>;
      /**
       * Invalid Justification type.
       **/
      InvalidJustification: AugmentedError<ApiType>;
      /**
       * Invalid participant public key
       **/
      InvalidParticipantPublicKey: AugmentedError<ApiType>;
      /**
       * Invalid Misbehavior Role type.
       **/
      InvalidRoleType: AugmentedError<ApiType>;
      /**
       * Invalid signature submitted
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * The signers have signed different data
       **/
      InvalidSignatureData: AugmentedError<ApiType>;
      /**
       * Invalid signature deserialization
       **/
      InvalidSignatureDeserialization: AugmentedError<ApiType>;
      /**
       * Invalid signature scheme
       **/
      InvalidSignatureScheme: AugmentedError<ApiType>;
      /**
       * Invalid verifying key submitted
       **/
      InvalidVerifyingKey: AugmentedError<ApiType>;
      /**
       * Invalid verifying key deserialization
       **/
      InvalidVerifyingKeyDeserialization: AugmentedError<ApiType>;
      /**
       * Malformed signature
       **/
      MalformedFrostSignature: AugmentedError<ApiType>;
      /**
       * Malformed FROST signing key
       **/
      MalformedFrostSigningKey: AugmentedError<ApiType>;
      /**
       * Malformed FROST verifying key
       **/
      MalformedFrostVerifyingKey: AugmentedError<ApiType>;
      /**
       * Could not deserialize the round message.
       **/
      MalformedRoundMessage: AugmentedError<ApiType>;
      /**
       * Missing FROST commitment
       **/
      MissingFrostCommitment: AugmentedError<ApiType>;
      /**
       * No participants found
       **/
      NoParticipantsFound: AugmentedError<ApiType>;
      /**
       * No signatures found
       **/
      NoSignaturesFound: AugmentedError<ApiType>;
      /**
       * Not enough signers for threshold
       **/
      NotEnoughSigners: AugmentedError<ApiType>;
      /**
       * Signed Round Message not signed by the offender.
       **/
      NotSignedByOffender: AugmentedError<ApiType>;
      /**
       * Signed with a different key
       **/
      SigningKeyMismatch: AugmentedError<ApiType>;
      /**
       * Unknown identifier
       **/
      UnknownIdentifier: AugmentedError<ApiType>;
      /**
       * The submitted decommitment data size is valid.
       * 
       * This error is returned when the decommitment data size is valid
       * but the caller claims it is invalid!
       **/
      ValidDataSize: AugmentedError<ApiType>;
      /**
       * The submitted decommitment is valid.
       * 
       * This error is returned when the decommitment is valid
       * but the caller claims it is invalid!
       **/
      ValidDecommitment: AugmentedError<ApiType>;
      /**
       * The submitted messages passed Feldman verification.
       * 
       * This error is returned when the messages passed Feldman verification
       * but the caller claims it is invalid!
       **/
      ValidFeldmanVerification: AugmentedError<ApiType>;
      /**
       * Valid FROST signature error for a misbehavior report
       **/
      ValidFrostSignature: AugmentedError<ApiType>;
      /**
       * ------------------------------------------------------------ ///
       * FROST ERRORS                         ///
       * ------------------------------------------------------------ ///
       * Valid FROST signature share
       **/
      ValidFrostSignatureShare: AugmentedError<ApiType>;
      /**
       * The submitted Mod Proof is valid.
       * 
       * This error is returned when the Mod Proof is valid
       * but the caller claims it is invalid.
       **/
      ValidModProof: AugmentedError<ApiType>;
      /**
       * The submitted ring pedersen parameters are valid.
       * 
       * This error is returned when the ring pedersen parameters are valid
       * but the caller claims it is invalid.
       **/
      ValidRingPedersenParameters: AugmentedError<ApiType>;
      /**
       * The submitted Schnorr Proof is valid.
       * 
       * This error is returned when the decommitment and its
       * Schnorr are valid. but the caller
       * claims it is invalid.
       **/
      ValidSchnorrProof: AugmentedError<ApiType>;
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
       * Sumission was prepared for a different round.
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
    jobs: {
      /**
       * empty job
       **/
      EmptyJob: AugmentedError<ApiType>;
      /**
       * empty result
       **/
      EmptyResult: AugmentedError<ApiType>;
      /**
       * Invalid Key size
       **/
      ExceedsMaxKeySize: AugmentedError<ApiType>;
      /**
       * invalid params, cannot execute jobs
       **/
      InvalidJobParams: AugmentedError<ApiType>;
      /**
       * invalid phase provided
       **/
      InvalidJobPhase: AugmentedError<ApiType>;
      /**
       * Given validator not valid for job type
       **/
      InvalidValidator: AugmentedError<ApiType>;
      /**
       * Invalid job expiry input
       **/
      JobAlreadyExpired: AugmentedError<ApiType>;
      /**
       * The requested job was not found
       **/
      JobNotFound: AugmentedError<ApiType>;
      /**
       * No permission to change permitted caller
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * no rewards found for validator
       **/
      NoRewards: AugmentedError<ApiType>;
      /**
       * Not enough validators to exit
       **/
      NotEnoughValidators: AugmentedError<ApiType>;
      /**
       * P1 result not found
       **/
      PhaseOneResultNotFound: AugmentedError<ApiType>;
      /**
       * cannot find phase 1 result
       **/
      PreviousResultNotFound: AugmentedError<ApiType>;
      /**
       * The previous result expired
       **/
      ResultExpired: AugmentedError<ApiType>;
      /**
       * Unexpected result provided
       **/
      ResultNotExpectedType: AugmentedError<ApiType>;
      /**
       * Validator exceeds limit of max active jobs
       **/
      TooManyJobsForValidator: AugmentedError<ApiType>;
      /**
       * Exceeds max participant limits
       **/
      TooManyParticipants: AugmentedError<ApiType>;
      /**
       * Validator role key not found.
       **/
      ValidatorRoleKeyNotFound: AugmentedError<ApiType>;
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
    roles: {
      /**
       * Rewards already claimed
       **/
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Out of bounds input
       **/
      BoundNotMet: AugmentedError<ApiType>;
      /**
       * Cannot find Current era
       **/
      CannotGetCurrentEra: AugmentedError<ApiType>;
      /**
       * Invalid Restaking amount, should not exceed total staked amount.
       **/
      ExceedsMaxRestakeValue: AugmentedError<ApiType>;
      /**
       * Validator has active role assigned.
       **/
      HasRoleAssigned: AugmentedError<ApiType>;
      /**
       * Re staking amount should be greater than minimum Restaking bond requirement.
       **/
      InsufficientRestakingBond: AugmentedError<ApiType>;
      /**
       * Invalid era info
       **/
      InvalidEraToReward: AugmentedError<ApiType>;
      /**
       * Key size exceeded
       **/
      KeySizeExceeded: AugmentedError<ApiType>;
      /**
       * Max role limit reached for the account.
       **/
      MaxRoles: AugmentedError<ApiType>;
      /**
       * Unlock chunks already filled
       **/
      NoMoreChunks: AugmentedError<ApiType>;
      /**
       * Stash controller account not found in Roles Ledger.
       **/
      NoProfileFound: AugmentedError<ApiType>;
      /**
       * Not a validator.
       **/
      NotValidator: AugmentedError<ApiType>;
      /**
       * Profile already exists for given validator account.
       **/
      ProfileAlreadyExists: AugmentedError<ApiType>;
      /**
       * Profile delete request failed due to pending jobs, which can't be opted out at the
       * moment.
       **/
      ProfileDeleteRequestFailed: AugmentedError<ApiType>;
      /**
       * Profile Update failed.
       **/
      ProfileUpdateFailed: AugmentedError<ApiType>;
      /**
       * Restaking amount cannot be lowered if there are any pending jobs. You can only add more
       **/
      RestakingAmountCannotBeUpdated: AugmentedError<ApiType>;
      /**
       * Role cannot due to pending jobs, which can't be opted out at the moment.
       **/
      RoleCannotBeRemoved: AugmentedError<ApiType>;
      /**
       * Given role is not assigned to the validator.
       **/
      RoleNotAssigned: AugmentedError<ApiType>;
      /**
       * SessionKeys not provided
       **/
      SessionKeysNotProvided: AugmentedError<ApiType>;
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
       * Proposer's balance is too low.
       **/
      InsufficientProposersBalance: AugmentedError<ApiType>;
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
    zkSaaS: {
      /**
       * Unexpected job type
       **/
      InvalidJobType: AugmentedError<ApiType>;
      /**
       * Invalid proof
       **/
      InvalidProof: AugmentedError<ApiType>;
      /**
       * Malformed Proof
       * if the proof bytes is not correct.
       **/
      MalformedProof: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
  } // AugmentedErrors
} // declare module
