// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

/* eslint-disable sort-keys */

export default {
  /**
   * Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>>
   **/
  FrameSystemAccountInfo: {
    nonce: 'u32',
    consumers: 'u32',
    providers: 'u32',
    sufficients: 'u32',
    data: 'PalletBalancesAccountData'
  },
  /**
   * Lookup5: pallet_balances::types::AccountData<Balance>
   **/
  PalletBalancesAccountData: {
    free: 'u128',
    reserved: 'u128',
    frozen: 'u128',
    flags: 'u128'
  },
  /**
   * Lookup8: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
   **/
  FrameSupportDispatchPerDispatchClassWeight: {
    normal: 'SpWeightsWeightV2Weight',
    operational: 'SpWeightsWeightV2Weight',
    mandatory: 'SpWeightsWeightV2Weight'
  },
  /**
   * Lookup9: sp_weights::weight_v2::Weight
   **/
  SpWeightsWeightV2Weight: {
    refTime: 'Compact<u64>',
    proofSize: 'Compact<u64>'
  },
  /**
   * Lookup14: sp_runtime::generic::digest::Digest
   **/
  SpRuntimeDigest: {
    logs: 'Vec<SpRuntimeDigestDigestItem>'
  },
  /**
   * Lookup16: sp_runtime::generic::digest::DigestItem
   **/
  SpRuntimeDigestDigestItem: {
    _enum: {
      Other: 'Bytes',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      Consensus: '([u8;4],Bytes)',
      Seal: '([u8;4],Bytes)',
      PreRuntime: '([u8;4],Bytes)',
      __Unused7: 'Null',
      RuntimeEnvironmentUpdated: 'Null'
    }
  },
  /**
   * Lookup19: frame_system::EventRecord<tangle_testnet_runtime::RuntimeEvent, primitive_types::H256>
   **/
  FrameSystemEventRecord: {
    phase: 'FrameSystemPhase',
    event: 'Event',
    topics: 'Vec<H256>'
  },
  /**
   * Lookup21: frame_system::pallet::Event<T>
   **/
  FrameSystemEvent: {
    _enum: {
      ExtrinsicSuccess: {
        dispatchInfo: 'FrameSupportDispatchDispatchInfo',
      },
      ExtrinsicFailed: {
        dispatchError: 'SpRuntimeDispatchError',
        dispatchInfo: 'FrameSupportDispatchDispatchInfo',
      },
      CodeUpdated: 'Null',
      NewAccount: {
        account: 'AccountId32',
      },
      KilledAccount: {
        account: 'AccountId32',
      },
      Remarked: {
        _alias: {
          hash_: 'hash',
        },
        sender: 'AccountId32',
        hash_: 'H256',
      },
      UpgradeAuthorized: {
        codeHash: 'H256',
        checkVersion: 'bool'
      }
    }
  },
  /**
   * Lookup22: frame_support::dispatch::DispatchInfo
   **/
  FrameSupportDispatchDispatchInfo: {
    weight: 'SpWeightsWeightV2Weight',
    class: 'FrameSupportDispatchDispatchClass',
    paysFee: 'FrameSupportDispatchPays'
  },
  /**
   * Lookup23: frame_support::dispatch::DispatchClass
   **/
  FrameSupportDispatchDispatchClass: {
    _enum: ['Normal', 'Operational', 'Mandatory']
  },
  /**
   * Lookup24: frame_support::dispatch::Pays
   **/
  FrameSupportDispatchPays: {
    _enum: ['Yes', 'No']
  },
  /**
   * Lookup25: sp_runtime::DispatchError
   **/
  SpRuntimeDispatchError: {
    _enum: {
      Other: 'Null',
      CannotLookup: 'Null',
      BadOrigin: 'Null',
      Module: 'SpRuntimeModuleError',
      ConsumerRemaining: 'Null',
      NoProviders: 'Null',
      TooManyConsumers: 'Null',
      Token: 'SpRuntimeTokenError',
      Arithmetic: 'SpArithmeticArithmeticError',
      Transactional: 'SpRuntimeTransactionalError',
      Exhausted: 'Null',
      Corruption: 'Null',
      Unavailable: 'Null',
      RootNotAllowed: 'Null'
    }
  },
  /**
   * Lookup26: sp_runtime::ModuleError
   **/
  SpRuntimeModuleError: {
    index: 'u8',
    error: '[u8;4]'
  },
  /**
   * Lookup27: sp_runtime::TokenError
   **/
  SpRuntimeTokenError: {
    _enum: ['FundsUnavailable', 'OnlyProvider', 'BelowMinimum', 'CannotCreate', 'UnknownAsset', 'Frozen', 'Unsupported', 'CannotCreateHold', 'NotExpendable', 'Blocked']
  },
  /**
   * Lookup28: sp_arithmetic::ArithmeticError
   **/
  SpArithmeticArithmeticError: {
    _enum: ['Underflow', 'Overflow', 'DivisionByZero']
  },
  /**
   * Lookup29: sp_runtime::TransactionalError
   **/
  SpRuntimeTransactionalError: {
    _enum: ['LimitReached', 'NoLayer']
  },
  /**
   * Lookup31: pallet_sudo::pallet::Event<T>
   **/
  PalletSudoEvent: {
    _enum: {
      Sudid: {
        sudoResult: 'Result<Null, SpRuntimeDispatchError>',
      },
      KeyChanged: {
        _alias: {
          new_: 'new',
        },
        old: 'Option<AccountId32>',
        new_: 'AccountId32',
      },
      KeyRemoved: 'Null',
      SudoAsDone: {
        sudoResult: 'Result<Null, SpRuntimeDispatchError>'
      }
    }
  },
  /**
   * Lookup35: pallet_assets::pallet::Event<T, I>
   **/
  PalletAssetsEvent: {
    _enum: {
      Created: {
        assetId: 'u128',
        creator: 'AccountId32',
        owner: 'AccountId32',
      },
      Issued: {
        assetId: 'u128',
        owner: 'AccountId32',
        amount: 'u128',
      },
      Transferred: {
        assetId: 'u128',
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
      },
      Burned: {
        assetId: 'u128',
        owner: 'AccountId32',
        balance: 'u128',
      },
      TeamChanged: {
        assetId: 'u128',
        issuer: 'AccountId32',
        admin: 'AccountId32',
        freezer: 'AccountId32',
      },
      OwnerChanged: {
        assetId: 'u128',
        owner: 'AccountId32',
      },
      Frozen: {
        assetId: 'u128',
        who: 'AccountId32',
      },
      Thawed: {
        assetId: 'u128',
        who: 'AccountId32',
      },
      AssetFrozen: {
        assetId: 'u128',
      },
      AssetThawed: {
        assetId: 'u128',
      },
      AccountsDestroyed: {
        assetId: 'u128',
        accountsDestroyed: 'u32',
        accountsRemaining: 'u32',
      },
      ApprovalsDestroyed: {
        assetId: 'u128',
        approvalsDestroyed: 'u32',
        approvalsRemaining: 'u32',
      },
      DestructionStarted: {
        assetId: 'u128',
      },
      Destroyed: {
        assetId: 'u128',
      },
      ForceCreated: {
        assetId: 'u128',
        owner: 'AccountId32',
      },
      MetadataSet: {
        assetId: 'u128',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
        isFrozen: 'bool',
      },
      MetadataCleared: {
        assetId: 'u128',
      },
      ApprovedTransfer: {
        assetId: 'u128',
        source: 'AccountId32',
        delegate: 'AccountId32',
        amount: 'u128',
      },
      ApprovalCancelled: {
        assetId: 'u128',
        owner: 'AccountId32',
        delegate: 'AccountId32',
      },
      TransferredApproved: {
        assetId: 'u128',
        owner: 'AccountId32',
        delegate: 'AccountId32',
        destination: 'AccountId32',
        amount: 'u128',
      },
      AssetStatusChanged: {
        assetId: 'u128',
      },
      AssetMinBalanceChanged: {
        assetId: 'u128',
        newMinBalance: 'u128',
      },
      Touched: {
        assetId: 'u128',
        who: 'AccountId32',
        depositor: 'AccountId32',
      },
      Blocked: {
        assetId: 'u128',
        who: 'AccountId32'
      }
    }
  },
  /**
   * Lookup36: pallet_balances::pallet::Event<T, I>
   **/
  PalletBalancesEvent: {
    _enum: {
      Endowed: {
        account: 'AccountId32',
        freeBalance: 'u128',
      },
      DustLost: {
        account: 'AccountId32',
        amount: 'u128',
      },
      Transfer: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
      },
      BalanceSet: {
        who: 'AccountId32',
        free: 'u128',
      },
      Reserved: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Unreserved: {
        who: 'AccountId32',
        amount: 'u128',
      },
      ReserveRepatriated: {
        from: 'AccountId32',
        to: 'AccountId32',
        amount: 'u128',
        destinationStatus: 'FrameSupportTokensMiscBalanceStatus',
      },
      Deposit: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Withdraw: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Slashed: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Minted: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Burned: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Suspended: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Restored: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Upgraded: {
        who: 'AccountId32',
      },
      Issued: {
        amount: 'u128',
      },
      Rescinded: {
        amount: 'u128',
      },
      Locked: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Unlocked: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Frozen: {
        who: 'AccountId32',
        amount: 'u128',
      },
      Thawed: {
        who: 'AccountId32',
        amount: 'u128',
      },
      TotalIssuanceForced: {
        _alias: {
          new_: 'new',
        },
        old: 'u128',
        new_: 'u128'
      }
    }
  },
  /**
   * Lookup37: frame_support::traits::tokens::misc::BalanceStatus
   **/
  FrameSupportTokensMiscBalanceStatus: {
    _enum: ['Free', 'Reserved']
  },
  /**
   * Lookup38: pallet_transaction_payment::pallet::Event<T>
   **/
  PalletTransactionPaymentEvent: {
    _enum: {
      TransactionFeePaid: {
        who: 'AccountId32',
        actualFee: 'u128',
        tip: 'u128'
      }
    }
  },
  /**
   * Lookup39: pallet_grandpa::pallet::Event
   **/
  PalletGrandpaEvent: {
    _enum: {
      NewAuthorities: {
        authoritySet: 'Vec<(SpConsensusGrandpaAppPublic,u64)>',
      },
      Paused: 'Null',
      Resumed: 'Null'
    }
  },
  /**
   * Lookup42: sp_consensus_grandpa::app::Public
   **/
  SpConsensusGrandpaAppPublic: 'SpCoreEd25519Public',
  /**
   * Lookup43: sp_core::ed25519::Public
   **/
  SpCoreEd25519Public: '[u8;32]',
  /**
   * Lookup44: pallet_indices::pallet::Event<T>
   **/
  PalletIndicesEvent: {
    _enum: {
      IndexAssigned: {
        who: 'AccountId32',
        index: 'u32',
      },
      IndexFreed: {
        index: 'u32',
      },
      IndexFrozen: {
        index: 'u32',
        who: 'AccountId32'
      }
    }
  },
  /**
   * Lookup45: pallet_democracy::pallet::Event<T>
   **/
  PalletDemocracyEvent: {
    _enum: {
      Proposed: {
        proposalIndex: 'u32',
        deposit: 'u128',
      },
      Tabled: {
        proposalIndex: 'u32',
        deposit: 'u128',
      },
      ExternalTabled: 'Null',
      Started: {
        refIndex: 'u32',
        threshold: 'PalletDemocracyVoteThreshold',
      },
      Passed: {
        refIndex: 'u32',
      },
      NotPassed: {
        refIndex: 'u32',
      },
      Cancelled: {
        refIndex: 'u32',
      },
      Delegated: {
        who: 'AccountId32',
        target: 'AccountId32',
      },
      Undelegated: {
        account: 'AccountId32',
      },
      Vetoed: {
        who: 'AccountId32',
        proposalHash: 'H256',
        until: 'u64',
      },
      Blacklisted: {
        proposalHash: 'H256',
      },
      Voted: {
        voter: 'AccountId32',
        refIndex: 'u32',
        vote: 'PalletDemocracyVoteAccountVote',
      },
      Seconded: {
        seconder: 'AccountId32',
        propIndex: 'u32',
      },
      ProposalCanceled: {
        propIndex: 'u32',
      },
      MetadataSet: {
        _alias: {
          hash_: 'hash',
        },
        owner: 'PalletDemocracyMetadataOwner',
        hash_: 'H256',
      },
      MetadataCleared: {
        _alias: {
          hash_: 'hash',
        },
        owner: 'PalletDemocracyMetadataOwner',
        hash_: 'H256',
      },
      MetadataTransferred: {
        _alias: {
          hash_: 'hash',
        },
        prevOwner: 'PalletDemocracyMetadataOwner',
        owner: 'PalletDemocracyMetadataOwner',
        hash_: 'H256'
      }
    }
  },
  /**
   * Lookup46: pallet_democracy::vote_threshold::VoteThreshold
   **/
  PalletDemocracyVoteThreshold: {
    _enum: ['SuperMajorityApprove', 'SuperMajorityAgainst', 'SimpleMajority']
  },
  /**
   * Lookup47: pallet_democracy::vote::AccountVote<Balance>
   **/
  PalletDemocracyVoteAccountVote: {
    _enum: {
      Standard: {
        vote: 'Vote',
        balance: 'u128',
      },
      Split: {
        aye: 'u128',
        nay: 'u128'
      }
    }
  },
  /**
   * Lookup49: pallet_democracy::types::MetadataOwner
   **/
  PalletDemocracyMetadataOwner: {
    _enum: {
      External: 'Null',
      Proposal: 'u32',
      Referendum: 'u32'
    }
  },
  /**
   * Lookup50: pallet_collective::pallet::Event<T, I>
   **/
  PalletCollectiveEvent: {
    _enum: {
      Proposed: {
        account: 'AccountId32',
        proposalIndex: 'u32',
        proposalHash: 'H256',
        threshold: 'u32',
      },
      Voted: {
        account: 'AccountId32',
        proposalHash: 'H256',
        voted: 'bool',
        yes: 'u32',
        no: 'u32',
      },
      Approved: {
        proposalHash: 'H256',
      },
      Disapproved: {
        proposalHash: 'H256',
      },
      Executed: {
        proposalHash: 'H256',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      MemberExecuted: {
        proposalHash: 'H256',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      Closed: {
        proposalHash: 'H256',
        yes: 'u32',
        no: 'u32'
      }
    }
  },
  /**
   * Lookup51: pallet_vesting::pallet::Event<T>
   **/
  PalletVestingEvent: {
    _enum: {
      VestingUpdated: {
        account: 'AccountId32',
        unvested: 'u128',
      },
      VestingCompleted: {
        account: 'AccountId32'
      }
    }
  },
  /**
   * Lookup52: pallet_elections_phragmen::pallet::Event<T>
   **/
  PalletElectionsPhragmenEvent: {
    _enum: {
      NewTerm: {
        newMembers: 'Vec<(AccountId32,u128)>',
      },
      EmptyTerm: 'Null',
      ElectionError: 'Null',
      MemberKicked: {
        member: 'AccountId32',
      },
      Renounced: {
        candidate: 'AccountId32',
      },
      CandidateSlashed: {
        candidate: 'AccountId32',
        amount: 'u128',
      },
      SeatHolderSlashed: {
        seatHolder: 'AccountId32',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup55: pallet_election_provider_multi_phase::pallet::Event<T>
   **/
  PalletElectionProviderMultiPhaseEvent: {
    _enum: {
      SolutionStored: {
        compute: 'PalletElectionProviderMultiPhaseElectionCompute',
        origin: 'Option<AccountId32>',
        prevEjected: 'bool',
      },
      ElectionFinalized: {
        compute: 'PalletElectionProviderMultiPhaseElectionCompute',
        score: 'SpNposElectionsElectionScore',
      },
      ElectionFailed: 'Null',
      Rewarded: {
        account: 'AccountId32',
        value: 'u128',
      },
      Slashed: {
        account: 'AccountId32',
        value: 'u128',
      },
      PhaseTransitioned: {
        from: 'PalletElectionProviderMultiPhasePhase',
        to: 'PalletElectionProviderMultiPhasePhase',
        round: 'u32'
      }
    }
  },
  /**
   * Lookup56: pallet_election_provider_multi_phase::ElectionCompute
   **/
  PalletElectionProviderMultiPhaseElectionCompute: {
    _enum: ['OnChain', 'Signed', 'Unsigned', 'Fallback', 'Emergency']
  },
  /**
   * Lookup57: sp_npos_elections::ElectionScore
   **/
  SpNposElectionsElectionScore: {
    minimalStake: 'u128',
    sumStake: 'u128',
    sumStakeSquared: 'u128'
  },
  /**
   * Lookup58: pallet_election_provider_multi_phase::Phase<Bn>
   **/
  PalletElectionProviderMultiPhasePhase: {
    _enum: {
      Off: 'Null',
      Signed: 'Null',
      Unsigned: '(bool,u64)',
      Emergency: 'Null'
    }
  },
  /**
   * Lookup60: pallet_staking::pallet::pallet::Event<T>
   **/
  PalletStakingPalletEvent: {
    _enum: {
      EraPaid: {
        eraIndex: 'u32',
        validatorPayout: 'u128',
        remainder: 'u128',
      },
      Rewarded: {
        stash: 'AccountId32',
        dest: 'PalletStakingRewardDestination',
        amount: 'u128',
      },
      Slashed: {
        staker: 'AccountId32',
        amount: 'u128',
      },
      SlashReported: {
        validator: 'AccountId32',
        fraction: 'Perbill',
        slashEra: 'u32',
      },
      OldSlashingReportDiscarded: {
        sessionIndex: 'u32',
      },
      StakersElected: 'Null',
      Bonded: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Unbonded: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Withdrawn: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      Kicked: {
        nominator: 'AccountId32',
        stash: 'AccountId32',
      },
      StakingElectionFailed: 'Null',
      Chilled: {
        stash: 'AccountId32',
      },
      PayoutStarted: {
        eraIndex: 'u32',
        validatorStash: 'AccountId32',
      },
      ValidatorPrefsSet: {
        stash: 'AccountId32',
        prefs: 'PalletStakingValidatorPrefs',
      },
      SnapshotVotersSizeExceeded: {
        _alias: {
          size_: 'size',
        },
        size_: 'u32',
      },
      SnapshotTargetsSizeExceeded: {
        _alias: {
          size_: 'size',
        },
        size_: 'u32',
      },
      ForceEra: {
        mode: 'PalletStakingForcing'
      }
    }
  },
  /**
   * Lookup61: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
   **/
  PalletStakingRewardDestination: {
    _enum: {
      Staked: 'Null',
      Stash: 'Null',
      Controller: 'Null',
      Account: 'AccountId32',
      None: 'Null'
    }
  },
  /**
   * Lookup63: pallet_staking::ValidatorPrefs
   **/
  PalletStakingValidatorPrefs: {
    commission: 'Compact<Perbill>',
    blocked: 'bool'
  },
  /**
   * Lookup65: pallet_staking::Forcing
   **/
  PalletStakingForcing: {
    _enum: ['NotForcing', 'ForceNew', 'ForceNone', 'ForceAlways']
  },
  /**
   * Lookup66: pallet_session::pallet::Event
   **/
  PalletSessionEvent: {
    _enum: {
      NewSession: {
        sessionIndex: 'u32'
      }
    }
  },
  /**
   * Lookup67: pallet_treasury::pallet::Event<T, I>
   **/
  PalletTreasuryEvent: {
    _enum: {
      Proposed: {
        proposalIndex: 'u32',
      },
      Spending: {
        budgetRemaining: 'u128',
      },
      Awarded: {
        proposalIndex: 'u32',
        award: 'u128',
        account: 'AccountId32',
      },
      Rejected: {
        proposalIndex: 'u32',
        slashed: 'u128',
      },
      Burnt: {
        burntFunds: 'u128',
      },
      Rollover: {
        rolloverBalance: 'u128',
      },
      Deposit: {
        value: 'u128',
      },
      SpendApproved: {
        proposalIndex: 'u32',
        amount: 'u128',
        beneficiary: 'AccountId32',
      },
      UpdatedInactive: {
        reactivated: 'u128',
        deactivated: 'u128',
      },
      AssetSpendApproved: {
        index: 'u32',
        assetKind: 'Null',
        amount: 'u128',
        beneficiary: 'AccountId32',
        validFrom: 'u64',
        expireAt: 'u64',
      },
      AssetSpendVoided: {
        index: 'u32',
      },
      Paid: {
        index: 'u32',
        paymentId: 'Null',
      },
      PaymentFailed: {
        index: 'u32',
        paymentId: 'Null',
      },
      SpendProcessed: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup68: pallet_bounties::pallet::Event<T, I>
   **/
  PalletBountiesEvent: {
    _enum: {
      BountyProposed: {
        index: 'u32',
      },
      BountyRejected: {
        index: 'u32',
        bond: 'u128',
      },
      BountyBecameActive: {
        index: 'u32',
      },
      BountyAwarded: {
        index: 'u32',
        beneficiary: 'AccountId32',
      },
      BountyClaimed: {
        index: 'u32',
        payout: 'u128',
        beneficiary: 'AccountId32',
      },
      BountyCanceled: {
        index: 'u32',
      },
      BountyExtended: {
        index: 'u32',
      },
      BountyApproved: {
        index: 'u32',
      },
      CuratorProposed: {
        bountyId: 'u32',
        curator: 'AccountId32',
      },
      CuratorUnassigned: {
        bountyId: 'u32',
      },
      CuratorAccepted: {
        bountyId: 'u32',
        curator: 'AccountId32'
      }
    }
  },
  /**
   * Lookup69: pallet_child_bounties::pallet::Event<T>
   **/
  PalletChildBountiesEvent: {
    _enum: {
      Added: {
        index: 'u32',
        childIndex: 'u32',
      },
      Awarded: {
        index: 'u32',
        childIndex: 'u32',
        beneficiary: 'AccountId32',
      },
      Claimed: {
        index: 'u32',
        childIndex: 'u32',
        payout: 'u128',
        beneficiary: 'AccountId32',
      },
      Canceled: {
        index: 'u32',
        childIndex: 'u32'
      }
    }
  },
  /**
   * Lookup70: pallet_bags_list::pallet::Event<T, I>
   **/
  PalletBagsListEvent: {
    _enum: {
      Rebagged: {
        who: 'AccountId32',
        from: 'u64',
        to: 'u64',
      },
      ScoreUpdated: {
        who: 'AccountId32',
        newScore: 'u64'
      }
    }
  },
  /**
   * Lookup71: pallet_nomination_pools::pallet::Event<T>
   **/
  PalletNominationPoolsEvent: {
    _enum: {
      Created: {
        depositor: 'AccountId32',
        poolId: 'u32',
      },
      Bonded: {
        member: 'AccountId32',
        poolId: 'u32',
        bonded: 'u128',
        joined: 'bool',
      },
      PaidOut: {
        member: 'AccountId32',
        poolId: 'u32',
        payout: 'u128',
      },
      Unbonded: {
        member: 'AccountId32',
        poolId: 'u32',
        balance: 'u128',
        points: 'u128',
        era: 'u32',
      },
      Withdrawn: {
        member: 'AccountId32',
        poolId: 'u32',
        balance: 'u128',
        points: 'u128',
      },
      Destroyed: {
        poolId: 'u32',
      },
      StateChanged: {
        poolId: 'u32',
        newState: 'PalletNominationPoolsPoolState',
      },
      MemberRemoved: {
        poolId: 'u32',
        member: 'AccountId32',
      },
      RolesUpdated: {
        root: 'Option<AccountId32>',
        bouncer: 'Option<AccountId32>',
        nominator: 'Option<AccountId32>',
      },
      PoolSlashed: {
        poolId: 'u32',
        balance: 'u128',
      },
      UnbondingPoolSlashed: {
        poolId: 'u32',
        era: 'u32',
        balance: 'u128',
      },
      PoolCommissionUpdated: {
        poolId: 'u32',
        current: 'Option<(Perbill,AccountId32)>',
      },
      PoolMaxCommissionUpdated: {
        poolId: 'u32',
        maxCommission: 'Perbill',
      },
      PoolCommissionChangeRateUpdated: {
        poolId: 'u32',
        changeRate: 'PalletNominationPoolsCommissionChangeRate',
      },
      PoolCommissionClaimPermissionUpdated: {
        poolId: 'u32',
        permission: 'Option<PalletNominationPoolsCommissionClaimPermission>',
      },
      PoolCommissionClaimed: {
        poolId: 'u32',
        commission: 'u128',
      },
      MinBalanceDeficitAdjusted: {
        poolId: 'u32',
        amount: 'u128',
      },
      MinBalanceExcessAdjusted: {
        poolId: 'u32',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup72: pallet_nomination_pools::PoolState
   **/
  PalletNominationPoolsPoolState: {
    _enum: ['Open', 'Blocked', 'Destroying']
  },
  /**
   * Lookup75: pallet_nomination_pools::CommissionChangeRate<BlockNumber>
   **/
  PalletNominationPoolsCommissionChangeRate: {
    maxIncrease: 'Perbill',
    minDelay: 'u64'
  },
  /**
   * Lookup77: pallet_nomination_pools::CommissionClaimPermission<sp_core::crypto::AccountId32>
   **/
  PalletNominationPoolsCommissionClaimPermission: {
    _enum: {
      Permissionless: 'Null',
      Account: 'AccountId32'
    }
  },
  /**
   * Lookup78: pallet_scheduler::pallet::Event<T>
   **/
  PalletSchedulerEvent: {
    _enum: {
      Scheduled: {
        when: 'u64',
        index: 'u32',
      },
      Canceled: {
        when: 'u64',
        index: 'u32',
      },
      Dispatched: {
        task: '(u64,u32)',
        id: 'Option<[u8;32]>',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      CallUnavailable: {
        task: '(u64,u32)',
        id: 'Option<[u8;32]>',
      },
      PeriodicFailed: {
        task: '(u64,u32)',
        id: 'Option<[u8;32]>',
      },
      PermanentlyOverweight: {
        task: '(u64,u32)',
        id: 'Option<[u8;32]>'
      }
    }
  },
  /**
   * Lookup81: pallet_preimage::pallet::Event<T>
   **/
  PalletPreimageEvent: {
    _enum: {
      Noted: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Requested: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Cleared: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256'
      }
    }
  },
  /**
   * Lookup82: pallet_offences::pallet::Event
   **/
  PalletOffencesEvent: {
    _enum: {
      Offence: {
        kind: '[u8;16]',
        timeslot: 'Bytes'
      }
    }
  },
  /**
   * Lookup84: pallet_tx_pause::pallet::Event<T>
   **/
  PalletTxPauseEvent: {
    _enum: {
      CallPaused: {
        fullName: '(Bytes,Bytes)',
      },
      CallUnpaused: {
        fullName: '(Bytes,Bytes)'
      }
    }
  },
  /**
   * Lookup87: pallet_im_online::pallet::Event<T>
   **/
  PalletImOnlineEvent: {
    _enum: {
      HeartbeatReceived: {
        authorityId: 'PalletImOnlineSr25519AppSr25519Public',
      },
      AllGood: 'Null',
      SomeOffline: {
        offline: 'Vec<(AccountId32,SpStakingExposure)>'
      }
    }
  },
  /**
   * Lookup88: pallet_im_online::sr25519::app_sr25519::Public
   **/
  PalletImOnlineSr25519AppSr25519Public: 'SpCoreSr25519Public',
  /**
   * Lookup89: sp_core::sr25519::Public
   **/
  SpCoreSr25519Public: '[u8;32]',
  /**
   * Lookup92: sp_staking::Exposure<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingExposure: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    others: 'Vec<SpStakingIndividualExposure>'
  },
  /**
   * Lookup95: sp_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingIndividualExposure: {
    who: 'AccountId32',
    value: 'Compact<u128>'
  },
  /**
   * Lookup96: pallet_identity::pallet::Event<T>
   **/
  PalletIdentityEvent: {
    _enum: {
      IdentitySet: {
        who: 'AccountId32',
      },
      IdentityCleared: {
        who: 'AccountId32',
        deposit: 'u128',
      },
      IdentityKilled: {
        who: 'AccountId32',
        deposit: 'u128',
      },
      JudgementRequested: {
        who: 'AccountId32',
        registrarIndex: 'u32',
      },
      JudgementUnrequested: {
        who: 'AccountId32',
        registrarIndex: 'u32',
      },
      JudgementGiven: {
        target: 'AccountId32',
        registrarIndex: 'u32',
      },
      RegistrarAdded: {
        registrarIndex: 'u32',
      },
      SubIdentityAdded: {
        sub: 'AccountId32',
        main: 'AccountId32',
        deposit: 'u128',
      },
      SubIdentityRemoved: {
        sub: 'AccountId32',
        main: 'AccountId32',
        deposit: 'u128',
      },
      SubIdentityRevoked: {
        sub: 'AccountId32',
        main: 'AccountId32',
        deposit: 'u128',
      },
      AuthorityAdded: {
        authority: 'AccountId32',
      },
      AuthorityRemoved: {
        authority: 'AccountId32',
      },
      UsernameSet: {
        who: 'AccountId32',
        username: 'Bytes',
      },
      UsernameQueued: {
        who: 'AccountId32',
        username: 'Bytes',
        expiration: 'u64',
      },
      PreapprovalExpired: {
        whose: 'AccountId32',
      },
      PrimaryUsernameSet: {
        who: 'AccountId32',
        username: 'Bytes',
      },
      DanglingUsernameRemoved: {
        who: 'AccountId32',
        username: 'Bytes'
      }
    }
  },
  /**
   * Lookup98: pallet_utility::pallet::Event
   **/
  PalletUtilityEvent: {
    _enum: {
      BatchInterrupted: {
        index: 'u32',
        error: 'SpRuntimeDispatchError',
      },
      BatchCompleted: 'Null',
      BatchCompletedWithErrors: 'Null',
      ItemCompleted: 'Null',
      ItemFailed: {
        error: 'SpRuntimeDispatchError',
      },
      DispatchedAs: {
        result: 'Result<Null, SpRuntimeDispatchError>'
      }
    }
  },
  /**
   * Lookup99: pallet_multisig::pallet::Event<T>
   **/
  PalletMultisigEvent: {
    _enum: {
      NewMultisig: {
        approving: 'AccountId32',
        multisig: 'AccountId32',
        callHash: '[u8;32]',
      },
      MultisigApproval: {
        approving: 'AccountId32',
        timepoint: 'PalletMultisigTimepoint',
        multisig: 'AccountId32',
        callHash: '[u8;32]',
      },
      MultisigExecuted: {
        approving: 'AccountId32',
        timepoint: 'PalletMultisigTimepoint',
        multisig: 'AccountId32',
        callHash: '[u8;32]',
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      MultisigCancelled: {
        cancelling: 'AccountId32',
        timepoint: 'PalletMultisigTimepoint',
        multisig: 'AccountId32',
        callHash: '[u8;32]'
      }
    }
  },
  /**
   * Lookup100: pallet_multisig::Timepoint<BlockNumber>
   **/
  PalletMultisigTimepoint: {
    height: 'u64',
    index: 'u32'
  },
  /**
   * Lookup101: pallet_ethereum::pallet::Event
   **/
  PalletEthereumEvent: {
    _enum: {
      Executed: {
        from: 'H160',
        to: 'H160',
        transactionHash: 'H256',
        exitReason: 'EvmCoreErrorExitReason',
        extraData: 'Bytes'
      }
    }
  },
  /**
   * Lookup104: evm_core::error::ExitReason
   **/
  EvmCoreErrorExitReason: {
    _enum: {
      Succeed: 'EvmCoreErrorExitSucceed',
      Error: 'EvmCoreErrorExitError',
      Revert: 'EvmCoreErrorExitRevert',
      Fatal: 'EvmCoreErrorExitFatal'
    }
  },
  /**
   * Lookup105: evm_core::error::ExitSucceed
   **/
  EvmCoreErrorExitSucceed: {
    _enum: ['Stopped', 'Returned', 'Suicided']
  },
  /**
   * Lookup106: evm_core::error::ExitError
   **/
  EvmCoreErrorExitError: {
    _enum: {
      StackUnderflow: 'Null',
      StackOverflow: 'Null',
      InvalidJump: 'Null',
      InvalidRange: 'Null',
      DesignatedInvalid: 'Null',
      CallTooDeep: 'Null',
      CreateCollision: 'Null',
      CreateContractLimit: 'Null',
      OutOfOffset: 'Null',
      OutOfGas: 'Null',
      OutOfFund: 'Null',
      PCUnderflow: 'Null',
      CreateEmpty: 'Null',
      Other: 'Text',
      MaxNonce: 'Null',
      InvalidCode: 'u8'
    }
  },
  /**
   * Lookup110: evm_core::error::ExitRevert
   **/
  EvmCoreErrorExitRevert: {
    _enum: ['Reverted']
  },
  /**
   * Lookup111: evm_core::error::ExitFatal
   **/
  EvmCoreErrorExitFatal: {
    _enum: {
      NotSupported: 'Null',
      UnhandledInterrupt: 'Null',
      CallErrorAsFatal: 'EvmCoreErrorExitError',
      Other: 'Text'
    }
  },
  /**
   * Lookup112: pallet_evm::pallet::Event<T>
   **/
  PalletEvmEvent: {
    _enum: {
      Log: {
        log: 'EthereumLog',
      },
      Created: {
        address: 'H160',
      },
      CreatedFailed: {
        address: 'H160',
      },
      Executed: {
        address: 'H160',
      },
      ExecutedFailed: {
        address: 'H160'
      }
    }
  },
  /**
   * Lookup113: ethereum::log::Log
   **/
  EthereumLog: {
    address: 'H160',
    topics: 'Vec<H256>',
    data: 'Bytes'
  },
  /**
   * Lookup115: pallet_base_fee::pallet::Event
   **/
  PalletBaseFeeEvent: {
    _enum: {
      NewBaseFeePerGas: {
        fee: 'U256',
      },
      BaseFeeOverflow: 'Null',
      NewElasticity: {
        elasticity: 'Permill'
      }
    }
  },
  /**
   * Lookup119: pallet_airdrop_claims::pallet::Event<T>
   **/
  PalletAirdropClaimsEvent: {
    _enum: {
      Claimed: {
        recipient: 'AccountId32',
        source: 'PalletAirdropClaimsUtilsMultiAddress',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup120: pallet_airdrop_claims::utils::MultiAddress
   **/
  PalletAirdropClaimsUtilsMultiAddress: {
    _enum: {
      EVM: 'PalletAirdropClaimsUtilsEthereumAddress',
      Native: 'AccountId32'
    }
  },
  /**
   * Lookup121: pallet_airdrop_claims::utils::ethereum_address::EthereumAddress
   **/
  PalletAirdropClaimsUtilsEthereumAddress: '[u8;20]',
  /**
   * Lookup122: pallet_roles::pallet::Event<T>
   **/
  PalletRolesEvent: {
    _enum: {
      RoleAssigned: {
        account: 'AccountId32',
        role: 'TanglePrimitivesRolesRoleType',
      },
      RoleRemoved: {
        account: 'AccountId32',
        role: 'TanglePrimitivesRolesRoleType',
      },
      Slashed: {
        account: 'AccountId32',
        amount: 'u128',
      },
      ProfileCreated: {
        account: 'AccountId32',
        totalProfileRestake: 'u128',
        roles: 'Vec<TanglePrimitivesRolesRoleType>',
      },
      ProfileUpdated: {
        account: 'AccountId32',
        totalProfileRestake: 'u128',
        roles: 'Vec<TanglePrimitivesRolesRoleType>',
      },
      ProfileDeleted: {
        account: 'AccountId32',
      },
      PendingJobs: {
        pendingJobs: 'Vec<(TanglePrimitivesRolesRoleType,u64)>',
      },
      RolesRewardSet: {
        totalRewards: 'u128',
      },
      PayoutStarted: {
        eraIndex: 'u32',
        validatorStash: 'AccountId32',
      },
      Rewarded: {
        stash: 'AccountId32',
        amount: 'u128',
      },
      MinRestakingBondUpdated: {
        value: 'u128'
      }
    }
  },
  /**
   * Lookup123: tangle_primitives::roles::RoleType
   **/
  TanglePrimitivesRolesRoleType: {
    _enum: {
      Tss: 'TanglePrimitivesRolesTssThresholdSignatureRoleType',
      ZkSaaS: 'TanglePrimitivesRolesZksaasZeroKnowledgeRoleType',
      LightClientRelaying: 'Null'
    }
  },
  /**
   * Lookup124: tangle_primitives::roles::tss::ThresholdSignatureRoleType
   **/
  TanglePrimitivesRolesTssThresholdSignatureRoleType: {
    _enum: ['DfnsCGGMP21Secp256k1', 'DfnsCGGMP21Secp256r1', 'DfnsCGGMP21Stark', 'SilentShardDKLS23Secp256k1', 'ZcashFrostP256', 'ZcashFrostP384', 'ZcashFrostSecp256k1', 'ZcashFrostRistretto255', 'ZcashFrostEd25519', 'ZcashFrostEd448', 'GennaroDKGBls381', 'WstsV2']
  },
  /**
   * Lookup125: tangle_primitives::roles::zksaas::ZeroKnowledgeRoleType
   **/
  TanglePrimitivesRolesZksaasZeroKnowledgeRoleType: {
    _enum: ['ZkSaaSGroth16', 'ZkSaaSMarlin']
  },
  /**
   * Lookup129: pallet_jobs::module::Event<T>
   **/
  PalletJobsModuleEvent: {
    _enum: {
      JobSubmitted: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
        details: 'TanglePrimitivesJobsJobSubmission',
      },
      JobResultSubmitted: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
      },
      ValidatorRewarded: {
        id: 'AccountId32',
        reward: 'u128',
      },
      JobRefunded: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
      },
      JobParticipantsUpdated: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
        details: 'TanglePrimitivesJobsJobInfo',
      },
      JobReSubmitted: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
        details: 'TanglePrimitivesJobsJobInfo',
      },
      JobResultExtended: {
        jobId: 'u64',
        roleType: 'TanglePrimitivesRolesRoleType',
        newExpiry: 'u64'
      }
    }
  },
  /**
   * Lookup130: tangle_primitives::jobs::JobSubmission<sp_core::crypto::AccountId32, BlockNumber, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxSubmissionLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsJobSubmission: {
    expiry: 'u64',
    ttl: 'u64',
    jobType: 'TanglePrimitivesJobsJobType',
    fallback: 'TanglePrimitivesJobsFallbackOptions'
  },
  /**
   * Lookup131: tangle_testnet_runtime::MaxParticipants
   **/
  TangleTestnetRuntimeMaxParticipants: 'Null',
  /**
   * Lookup132: tangle_testnet_runtime::MaxSubmissionLen
   **/
  TangleTestnetRuntimeMaxSubmissionLen: 'Null',
  /**
   * Lookup133: tangle_testnet_runtime::MaxAdditionalParamsLen
   **/
  TangleTestnetRuntimeMaxAdditionalParamsLen: 'Null',
  /**
   * Lookup134: tangle_primitives::jobs::JobType<sp_core::crypto::AccountId32, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxSubmissionLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsJobType: {
    _enum: {
      DKGTSSPhaseOne: 'TanglePrimitivesJobsTssDkgtssPhaseOneJobType',
      DKGTSSPhaseTwo: 'TanglePrimitivesJobsTssDkgtssPhaseTwoJobType',
      DKGTSSPhaseThree: 'TanglePrimitivesJobsTssDkgtssPhaseThreeJobType',
      DKGTSSPhaseFour: 'TanglePrimitivesJobsTssDkgtssPhaseFourJobType',
      ZkSaaSPhaseOne: 'TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType',
      ZkSaaSPhaseTwo: 'TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType'
    }
  },
  /**
   * Lookup135: tangle_primitives::jobs::tss::DKGTSSPhaseOneJobType<sp_core::crypto::AccountId32, tangle_testnet_runtime::MaxParticipants>
   **/
  TanglePrimitivesJobsTssDkgtssPhaseOneJobType: {
    participants: 'Vec<AccountId32>',
    threshold: 'u8',
    permittedCaller: 'Option<AccountId32>',
    roleType: 'TanglePrimitivesRolesTssThresholdSignatureRoleType',
    hdWallet: 'bool'
  },
  /**
   * Lookup138: tangle_primitives::jobs::tss::DKGTSSPhaseTwoJobType<tangle_testnet_runtime::MaxSubmissionLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsTssDkgtssPhaseTwoJobType: {
    phaseOneId: 'u64',
    submission: 'Bytes',
    derivationPath: 'Option<Bytes>',
    roleType: 'TanglePrimitivesRolesTssThresholdSignatureRoleType'
  },
  /**
   * Lookup142: tangle_primitives::jobs::tss::DKGTSSPhaseThreeJobType
   **/
  TanglePrimitivesJobsTssDkgtssPhaseThreeJobType: {
    phaseOneId: 'u64',
    roleType: 'TanglePrimitivesRolesTssThresholdSignatureRoleType'
  },
  /**
   * Lookup143: tangle_primitives::jobs::tss::DKGTSSPhaseFourJobType
   **/
  TanglePrimitivesJobsTssDkgtssPhaseFourJobType: {
    phaseOneId: 'u64',
    newPhaseOneId: 'u64',
    roleType: 'TanglePrimitivesRolesTssThresholdSignatureRoleType'
  },
  /**
   * Lookup144: tangle_primitives::jobs::zksaas::ZkSaaSPhaseOneJobType<sp_core::crypto::AccountId32, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType: {
    participants: 'Vec<AccountId32>',
    permittedCaller: 'Option<AccountId32>',
    system: 'TanglePrimitivesJobsZksaasZkSaaSSystem',
    roleType: 'TanglePrimitivesRolesZksaasZeroKnowledgeRoleType'
  },
  /**
   * Lookup145: tangle_primitives::jobs::zksaas::ZkSaaSSystem<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasZkSaaSSystem: {
    _enum: {
      Groth16: 'TanglePrimitivesJobsZksaasGroth16System'
    }
  },
  /**
   * Lookup146: tangle_primitives::jobs::zksaas::Groth16System<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasGroth16System: {
    circuit: 'TanglePrimitivesJobsZksaasHyperData',
    numInputs: 'u64',
    numConstraints: 'u64',
    provingKey: 'TanglePrimitivesJobsZksaasHyperData',
    verifyingKey: 'Bytes',
    wasm: 'TanglePrimitivesJobsZksaasHyperData'
  },
  /**
   * Lookup147: tangle_primitives::jobs::zksaas::HyperData<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasHyperData: {
    _enum: {
      Raw: 'Bytes',
      IPFS: 'Bytes',
      HTTP: 'Bytes'
    }
  },
  /**
   * Lookup148: tangle_primitives::jobs::zksaas::ZkSaaSPhaseTwoJobType<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType: {
    phaseOneId: 'u64',
    request: 'TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest',
    roleType: 'TanglePrimitivesRolesZksaasZeroKnowledgeRoleType'
  },
  /**
   * Lookup149: tangle_primitives::jobs::zksaas::ZkSaaSPhaseTwoRequest<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest: {
    _enum: {
      Groth16: 'TanglePrimitivesJobsZksaasGroth16ProveRequest'
    }
  },
  /**
   * Lookup150: tangle_primitives::jobs::zksaas::Groth16ProveRequest<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasGroth16ProveRequest: {
    publicInput: 'Bytes',
    aShares: 'Vec<TanglePrimitivesJobsZksaasHyperData>',
    axShares: 'Vec<TanglePrimitivesJobsZksaasHyperData>',
    qapShares: 'Vec<TanglePrimitivesJobsZksaasQapShare>'
  },
  /**
   * Lookup154: tangle_primitives::jobs::zksaas::QAPShare<tangle_testnet_runtime::MaxSubmissionLen>
   **/
  TanglePrimitivesJobsZksaasQapShare: {
    a: 'TanglePrimitivesJobsZksaasHyperData',
    b: 'TanglePrimitivesJobsZksaasHyperData',
    c: 'TanglePrimitivesJobsZksaasHyperData'
  },
  /**
   * Lookup156: tangle_primitives::jobs::FallbackOptions
   **/
  TanglePrimitivesJobsFallbackOptions: {
    _enum: {
      Destroy: 'Null',
      RegenerateWithThreshold: 'u8'
    }
  },
  /**
   * Lookup157: tangle_primitives::jobs::JobInfo<sp_core::crypto::AccountId32, BlockNumber, Balance, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxSubmissionLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsJobInfo: {
    owner: 'AccountId32',
    expiry: 'u64',
    ttl: 'u64',
    jobType: 'TanglePrimitivesJobsJobType',
    fee: 'u128',
    fallback: 'TanglePrimitivesJobsFallbackOptions'
  },
  /**
   * Lookup158: pallet_dkg::pallet::Event<T>
   **/
  PalletDkgEvent: {
    _enum: {
      FeeUpdated: 'PalletDkgFeeInfo',
      KeyRotated: {
        fromJobId: 'u64',
        toJobId: 'u64',
        signature: 'Bytes'
      }
    }
  },
  /**
   * Lookup159: pallet_dkg::types::FeeInfo<Balance>
   **/
  PalletDkgFeeInfo: {
    baseFee: 'u128',
    dkgValidatorFee: 'u128',
    sigValidatorFee: 'u128',
    refreshValidatorFee: 'u128',
    storageFeePerByte: 'u128',
    storageFeePerBlock: 'u128'
  },
  /**
   * Lookup160: pallet_zksaas::pallet::Event<T>
   **/
  PalletZksaasEvent: {
    _enum: {
      FeeUpdated: 'PalletZksaasFeeInfo'
    }
  },
  /**
   * Lookup161: pallet_zksaas::types::FeeInfo<Balance>
   **/
  PalletZksaasFeeInfo: {
    baseFee: 'u128',
    circuitFee: 'u128',
    proveFee: 'u128',
    storageFeePerByte: 'u128'
  },
  /**
   * Lookup162: pallet_proxy::pallet::Event<T>
   **/
  PalletProxyEvent: {
    _enum: {
      ProxyExecuted: {
        result: 'Result<Null, SpRuntimeDispatchError>',
      },
      PureCreated: {
        pure: 'AccountId32',
        who: 'AccountId32',
        proxyType: 'TangleTestnetRuntimeProxyType',
        disambiguationIndex: 'u16',
      },
      Announced: {
        real: 'AccountId32',
        proxy: 'AccountId32',
        callHash: 'H256',
      },
      ProxyAdded: {
        delegator: 'AccountId32',
        delegatee: 'AccountId32',
        proxyType: 'TangleTestnetRuntimeProxyType',
        delay: 'u64',
      },
      ProxyRemoved: {
        delegator: 'AccountId32',
        delegatee: 'AccountId32',
        proxyType: 'TangleTestnetRuntimeProxyType',
        delay: 'u64'
      }
    }
  },
  /**
   * Lookup163: tangle_testnet_runtime::ProxyType
   **/
  TangleTestnetRuntimeProxyType: {
    _enum: ['Any', 'NonTransfer', 'Governance', 'Staking']
  },
  /**
   * Lookup165: pallet_multi_asset_delegation::pallet::Event<T>
   **/
  PalletMultiAssetDelegationEvent: {
    _enum: {
      OperatorJoined: {
        who: 'AccountId32',
      },
      OperatorLeavingScheduled: {
        who: 'AccountId32',
      },
      OperatorLeaveCancelled: {
        who: 'AccountId32',
      },
      OperatorLeaveExecuted: {
        who: 'AccountId32',
      },
      OperatorBondMore: {
        who: 'AccountId32',
        additionalBond: 'u128',
      },
      OperatorBondLessScheduled: {
        who: 'AccountId32',
        bondLessAmount: 'u128',
      },
      OperatorBondLessExecuted: {
        who: 'AccountId32',
      },
      OperatorBondLessCancelled: {
        who: 'AccountId32',
      },
      OperatorWentOffline: {
        who: 'AccountId32',
      },
      OperatorWentOnline: {
        who: 'AccountId32',
      },
      Deposited: {
        who: 'AccountId32',
        amount: 'u128',
        assetId: 'Option<u128>',
      },
      ScheduledUnstake: {
        who: 'AccountId32',
        amount: 'u128',
        assetId: 'Option<u128>',
      },
      ExecutedUnstake: {
        who: 'AccountId32',
      },
      CancelledUnstake: {
        who: 'AccountId32',
      },
      Delegated: {
        who: 'AccountId32',
        operator: 'AccountId32',
        amount: 'u128',
        assetId: 'u128',
      },
      ScheduledDelegatorBondLess: {
        who: 'AccountId32',
        operator: 'AccountId32',
        amount: 'u128',
        assetId: 'u128',
      },
      ExecutedDelegatorBondLess: {
        who: 'AccountId32',
      },
      CancelledDelegatorBondLess: {
        who: 'AccountId32',
      },
      WhitelistedAssetsSet: {
        assets: 'Vec<u128>',
      },
      IncentiveAPYAndCapSet: {
        assetId: 'u128',
        apy: 'u128',
        cap: 'u128',
      },
      BlueprintWhitelisted: {
        blueprintId: 'u32'
      }
    }
  },
  /**
   * Lookup168: sygma_access_segregator::pallet::Event<T>
   **/
  SygmaAccessSegregatorEvent: {
    _enum: {
      AccessGranted: {
        palletIndex: 'u8',
        extrinsicName: 'Bytes',
        who: 'AccountId32'
      }
    }
  },
  /**
   * Lookup169: sygma_basic_feehandler::pallet::Event<T>
   **/
  SygmaBasicFeehandlerEvent: {
    _enum: {
      FeeSet: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup170: staging_xcm::v4::asset::AssetId
   **/
  StagingXcmV4AssetAssetId: 'StagingXcmV4Location',
  /**
   * Lookup171: staging_xcm::v4::location::Location
   **/
  StagingXcmV4Location: {
    parents: 'u8',
    interior: 'StagingXcmV4Junctions'
  },
  /**
   * Lookup172: staging_xcm::v4::junctions::Junctions
   **/
  StagingXcmV4Junctions: {
    _enum: {
      Here: 'Null',
      X1: '[Lookup174;1]',
      X2: '[Lookup174;2]',
      X3: '[Lookup174;3]',
      X4: '[Lookup174;4]',
      X5: '[Lookup174;5]',
      X6: '[Lookup174;6]',
      X7: '[Lookup174;7]',
      X8: '[Lookup174;8]'
    }
  },
  /**
   * Lookup174: staging_xcm::v4::junction::Junction
   **/
  StagingXcmV4Junction: {
    _enum: {
      Parachain: 'Compact<u32>',
      AccountId32: {
        network: 'Option<StagingXcmV4JunctionNetworkId>',
        id: '[u8;32]',
      },
      AccountIndex64: {
        network: 'Option<StagingXcmV4JunctionNetworkId>',
        index: 'Compact<u64>',
      },
      AccountKey20: {
        network: 'Option<StagingXcmV4JunctionNetworkId>',
        key: '[u8;20]',
      },
      PalletInstance: 'u8',
      GeneralIndex: 'Compact<u128>',
      GeneralKey: {
        length: 'u8',
        data: '[u8;32]',
      },
      OnlyChild: 'Null',
      Plurality: {
        id: 'XcmV3JunctionBodyId',
        part: 'XcmV3JunctionBodyPart',
      },
      GlobalConsensus: 'StagingXcmV4JunctionNetworkId'
    }
  },
  /**
   * Lookup177: staging_xcm::v4::junction::NetworkId
   **/
  StagingXcmV4JunctionNetworkId: {
    _enum: {
      ByGenesis: '[u8;32]',
      ByFork: {
        blockNumber: 'u64',
        blockHash: '[u8;32]',
      },
      Polkadot: 'Null',
      Kusama: 'Null',
      Westend: 'Null',
      Rococo: 'Null',
      Wococo: 'Null',
      Ethereum: {
        chainId: 'Compact<u64>',
      },
      BitcoinCore: 'Null',
      BitcoinCash: 'Null',
      PolkadotBulletin: 'Null'
    }
  },
  /**
   * Lookup178: xcm::v3::junction::BodyId
   **/
  XcmV3JunctionBodyId: {
    _enum: {
      Unit: 'Null',
      Moniker: '[u8;4]',
      Index: 'Compact<u32>',
      Executive: 'Null',
      Technical: 'Null',
      Legislative: 'Null',
      Judicial: 'Null',
      Defense: 'Null',
      Administration: 'Null',
      Treasury: 'Null'
    }
  },
  /**
   * Lookup179: xcm::v3::junction::BodyPart
   **/
  XcmV3JunctionBodyPart: {
    _enum: {
      Voice: 'Null',
      Members: {
        count: 'Compact<u32>',
      },
      Fraction: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>',
      },
      AtLeastProportion: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>',
      },
      MoreThanProportion: {
        nom: 'Compact<u32>',
        denom: 'Compact<u32>'
      }
    }
  },
  /**
   * Lookup187: sygma_fee_handler_router::pallet::Event<T>
   **/
  SygmaFeeHandlerRouterEvent: {
    _enum: {
      FeeHandlerSet: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        handlerType: 'SygmaFeeHandlerRouterFeeHandlerType'
      }
    }
  },
  /**
   * Lookup188: sygma_fee_handler_router::pallet::FeeHandlerType
   **/
  SygmaFeeHandlerRouterFeeHandlerType: {
    _enum: ['BasicFeeHandler', 'PercentageFeeHandler', 'DynamicFeeHandler']
  },
  /**
   * Lookup189: sygma_percentage_feehandler::pallet::Event<T>
   **/
  SygmaPercentageFeehandlerEvent: {
    _enum: {
      FeeRateSet: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        rateBasisPoint: 'u32',
        feeLowerBound: 'u128',
        feeUpperBound: 'u128'
      }
    }
  },
  /**
   * Lookup190: sygma_bridge::pallet::Event<T>
   **/
  SygmaBridgeEvent: {
    _enum: {
      Deposit: {
        destDomainId: 'u8',
        resourceId: '[u8;32]',
        depositNonce: 'u64',
        sender: 'AccountId32',
        transferType: 'SygmaTraitsTransferType',
        depositData: 'Bytes',
        handlerResponse: 'Bytes',
      },
      ProposalExecution: {
        originDomainId: 'u8',
        depositNonce: 'u64',
        dataHash: '[u8;32]',
      },
      FailedHandlerExecution: {
        error: 'Bytes',
        originDomainId: 'u8',
        depositNonce: 'u64',
      },
      Retry: {
        depositOnBlockHeight: 'u128',
        destDomainId: 'u8',
        sender: 'AccountId32',
      },
      BridgePaused: {
        destDomainId: 'u8',
      },
      BridgeUnpaused: {
        destDomainId: 'u8',
      },
      RegisterDestDomain: {
        sender: 'AccountId32',
        domainId: 'u8',
        chainId: 'U256',
      },
      UnregisterDestDomain: {
        sender: 'AccountId32',
        domainId: 'u8',
        chainId: 'U256',
      },
      FeeCollected: {
        feePayer: 'AccountId32',
        destDomainId: 'u8',
        resourceId: '[u8;32]',
        feeAmount: 'u128',
        feeAssetId: 'StagingXcmV4AssetAssetId',
      },
      AllBridgePaused: {
        sender: 'AccountId32',
      },
      AllBridgeUnpaused: {
        sender: 'AccountId32'
      }
    }
  },
  /**
   * Lookup191: sygma_traits::TransferType
   **/
  SygmaTraitsTransferType: {
    _enum: ['FungibleTransfer', 'NonFungibleTransfer', 'GenericTransfer']
  },
  /**
   * Lookup192: frame_system::Phase
   **/
  FrameSystemPhase: {
    _enum: {
      ApplyExtrinsic: 'u32',
      Finalization: 'Null',
      Initialization: 'Null'
    }
  },
  /**
   * Lookup194: frame_system::LastRuntimeUpgradeInfo
   **/
  FrameSystemLastRuntimeUpgradeInfo: {
    specVersion: 'Compact<u32>',
    specName: 'Text'
  },
  /**
   * Lookup195: frame_system::CodeUpgradeAuthorization<T>
   **/
  FrameSystemCodeUpgradeAuthorization: {
    codeHash: 'H256',
    checkVersion: 'bool'
  },
  /**
   * Lookup196: frame_system::pallet::Call<T>
   **/
  FrameSystemCall: {
    _enum: {
      remark: {
        remark: 'Bytes',
      },
      set_heap_pages: {
        pages: 'u64',
      },
      set_code: {
        code: 'Bytes',
      },
      set_code_without_checks: {
        code: 'Bytes',
      },
      set_storage: {
        items: 'Vec<(Bytes,Bytes)>',
      },
      kill_storage: {
        _alias: {
          keys_: 'keys',
        },
        keys_: 'Vec<Bytes>',
      },
      kill_prefix: {
        prefix: 'Bytes',
        subkeys: 'u32',
      },
      remark_with_event: {
        remark: 'Bytes',
      },
      __Unused8: 'Null',
      authorize_upgrade: {
        codeHash: 'H256',
      },
      authorize_upgrade_without_checks: {
        codeHash: 'H256',
      },
      apply_authorized_upgrade: {
        code: 'Bytes'
      }
    }
  },
  /**
   * Lookup200: frame_system::limits::BlockWeights
   **/
  FrameSystemLimitsBlockWeights: {
    baseBlock: 'SpWeightsWeightV2Weight',
    maxBlock: 'SpWeightsWeightV2Weight',
    perClass: 'FrameSupportDispatchPerDispatchClassWeightsPerClass'
  },
  /**
   * Lookup201: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
   **/
  FrameSupportDispatchPerDispatchClassWeightsPerClass: {
    normal: 'FrameSystemLimitsWeightsPerClass',
    operational: 'FrameSystemLimitsWeightsPerClass',
    mandatory: 'FrameSystemLimitsWeightsPerClass'
  },
  /**
   * Lookup202: frame_system::limits::WeightsPerClass
   **/
  FrameSystemLimitsWeightsPerClass: {
    baseExtrinsic: 'SpWeightsWeightV2Weight',
    maxExtrinsic: 'Option<SpWeightsWeightV2Weight>',
    maxTotal: 'Option<SpWeightsWeightV2Weight>',
    reserved: 'Option<SpWeightsWeightV2Weight>'
  },
  /**
   * Lookup204: frame_system::limits::BlockLength
   **/
  FrameSystemLimitsBlockLength: {
    max: 'FrameSupportDispatchPerDispatchClassU32'
  },
  /**
   * Lookup205: frame_support::dispatch::PerDispatchClass<T>
   **/
  FrameSupportDispatchPerDispatchClassU32: {
    normal: 'u32',
    operational: 'u32',
    mandatory: 'u32'
  },
  /**
   * Lookup206: sp_weights::RuntimeDbWeight
   **/
  SpWeightsRuntimeDbWeight: {
    read: 'u64',
    write: 'u64'
  },
  /**
   * Lookup207: sp_version::RuntimeVersion
   **/
  SpVersionRuntimeVersion: {
    specName: 'Text',
    implName: 'Text',
    authoringVersion: 'u32',
    specVersion: 'u32',
    implVersion: 'u32',
    apis: 'Vec<([u8;8],u32)>',
    transactionVersion: 'u32',
    stateVersion: 'u8'
  },
  /**
   * Lookup212: frame_system::pallet::Error<T>
   **/
  FrameSystemError: {
    _enum: ['InvalidSpecName', 'SpecVersionNeedsToIncrease', 'FailedToExtractRuntimeVersion', 'NonDefaultComposite', 'NonZeroRefCount', 'CallFiltered', 'NothingAuthorized', 'Unauthorized']
  },
  /**
   * Lookup213: pallet_timestamp::pallet::Call<T>
   **/
  PalletTimestampCall: {
    _enum: {
      set: {
        now: 'Compact<u64>'
      }
    }
  },
  /**
   * Lookup214: pallet_sudo::pallet::Call<T>
   **/
  PalletSudoCall: {
    _enum: {
      sudo: {
        call: 'Call',
      },
      sudo_unchecked_weight: {
        call: 'Call',
        weight: 'SpWeightsWeightV2Weight',
      },
      set_key: {
        _alias: {
          new_: 'new',
        },
        new_: 'MultiAddress',
      },
      sudo_as: {
        who: 'MultiAddress',
        call: 'Call',
      },
      remove_key: 'Null'
    }
  },
  /**
   * Lookup216: pallet_assets::pallet::Call<T, I>
   **/
  PalletAssetsCall: {
    _enum: {
      create: {
        id: 'Compact<u128>',
        admin: 'MultiAddress',
        minBalance: 'u128',
      },
      force_create: {
        id: 'Compact<u128>',
        owner: 'MultiAddress',
        isSufficient: 'bool',
        minBalance: 'Compact<u128>',
      },
      start_destroy: {
        id: 'Compact<u128>',
      },
      destroy_accounts: {
        id: 'Compact<u128>',
      },
      destroy_approvals: {
        id: 'Compact<u128>',
      },
      finish_destroy: {
        id: 'Compact<u128>',
      },
      mint: {
        id: 'Compact<u128>',
        beneficiary: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      burn: {
        id: 'Compact<u128>',
        who: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      transfer: {
        id: 'Compact<u128>',
        target: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      transfer_keep_alive: {
        id: 'Compact<u128>',
        target: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      force_transfer: {
        id: 'Compact<u128>',
        source: 'MultiAddress',
        dest: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      freeze: {
        id: 'Compact<u128>',
        who: 'MultiAddress',
      },
      thaw: {
        id: 'Compact<u128>',
        who: 'MultiAddress',
      },
      freeze_asset: {
        id: 'Compact<u128>',
      },
      thaw_asset: {
        id: 'Compact<u128>',
      },
      transfer_ownership: {
        id: 'Compact<u128>',
        owner: 'MultiAddress',
      },
      set_team: {
        id: 'Compact<u128>',
        issuer: 'MultiAddress',
        admin: 'MultiAddress',
        freezer: 'MultiAddress',
      },
      set_metadata: {
        id: 'Compact<u128>',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
      },
      clear_metadata: {
        id: 'Compact<u128>',
      },
      force_set_metadata: {
        id: 'Compact<u128>',
        name: 'Bytes',
        symbol: 'Bytes',
        decimals: 'u8',
        isFrozen: 'bool',
      },
      force_clear_metadata: {
        id: 'Compact<u128>',
      },
      force_asset_status: {
        id: 'Compact<u128>',
        owner: 'MultiAddress',
        issuer: 'MultiAddress',
        admin: 'MultiAddress',
        freezer: 'MultiAddress',
        minBalance: 'Compact<u128>',
        isSufficient: 'bool',
        isFrozen: 'bool',
      },
      approve_transfer: {
        id: 'Compact<u128>',
        delegate: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      cancel_approval: {
        id: 'Compact<u128>',
        delegate: 'MultiAddress',
      },
      force_cancel_approval: {
        id: 'Compact<u128>',
        owner: 'MultiAddress',
        delegate: 'MultiAddress',
      },
      transfer_approved: {
        id: 'Compact<u128>',
        owner: 'MultiAddress',
        destination: 'MultiAddress',
        amount: 'Compact<u128>',
      },
      touch: {
        id: 'Compact<u128>',
      },
      refund: {
        id: 'Compact<u128>',
        allowBurn: 'bool',
      },
      set_min_balance: {
        id: 'Compact<u128>',
        minBalance: 'u128',
      },
      touch_other: {
        id: 'Compact<u128>',
        who: 'MultiAddress',
      },
      refund_other: {
        id: 'Compact<u128>',
        who: 'MultiAddress',
      },
      block: {
        id: 'Compact<u128>',
        who: 'MultiAddress'
      }
    }
  },
  /**
   * Lookup218: pallet_balances::pallet::Call<T, I>
   **/
  PalletBalancesCall: {
    _enum: {
      transfer_allow_death: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      __Unused1: 'Null',
      force_transfer: {
        source: 'MultiAddress',
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      transfer_keep_alive: {
        dest: 'MultiAddress',
        value: 'Compact<u128>',
      },
      transfer_all: {
        dest: 'MultiAddress',
        keepAlive: 'bool',
      },
      force_unreserve: {
        who: 'MultiAddress',
        amount: 'u128',
      },
      upgrade_accounts: {
        who: 'Vec<AccountId32>',
      },
      __Unused7: 'Null',
      force_set_balance: {
        who: 'MultiAddress',
        newFree: 'Compact<u128>',
      },
      force_adjust_total_issuance: {
        direction: 'PalletBalancesAdjustmentDirection',
        delta: 'Compact<u128>'
      }
    }
  },
  /**
   * Lookup219: pallet_balances::types::AdjustmentDirection
   **/
  PalletBalancesAdjustmentDirection: {
    _enum: ['Increase', 'Decrease']
  },
  /**
   * Lookup220: pallet_babe::pallet::Call<T>
   **/
  PalletBabeCall: {
    _enum: {
      report_equivocation: {
        equivocationProof: 'SpConsensusSlotsEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      report_equivocation_unsigned: {
        equivocationProof: 'SpConsensusSlotsEquivocationProof',
        keyOwnerProof: 'SpSessionMembershipProof',
      },
      plan_config_change: {
        config: 'SpConsensusBabeDigestsNextConfigDescriptor'
      }
    }
  },
  /**
   * Lookup221: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_babe::app::Public>
   **/
  SpConsensusSlotsEquivocationProof: {
    offender: 'SpConsensusBabeAppPublic',
    slot: 'u64',
    firstHeader: 'SpRuntimeHeader',
    secondHeader: 'SpRuntimeHeader'
  },
  /**
   * Lookup222: sp_runtime::generic::header::Header<Number, Hash>
   **/
  SpRuntimeHeader: {
    parentHash: 'H256',
    number: 'Compact<u64>',
    stateRoot: 'H256',
    extrinsicsRoot: 'H256',
    digest: 'SpRuntimeDigest'
  },
  /**
   * Lookup223: sp_consensus_babe::app::Public
   **/
  SpConsensusBabeAppPublic: 'SpCoreSr25519Public',
  /**
   * Lookup225: sp_session::MembershipProof
   **/
  SpSessionMembershipProof: {
    session: 'u32',
    trieNodes: 'Vec<Bytes>',
    validatorCount: 'u32'
  },
  /**
   * Lookup226: sp_consensus_babe::digests::NextConfigDescriptor
   **/
  SpConsensusBabeDigestsNextConfigDescriptor: {
    _enum: {
      __Unused0: 'Null',
      V1: {
        c: '(u64,u64)',
        allowedSlots: 'SpConsensusBabeAllowedSlots'
      }
    }
  },
  /**
   * Lookup228: sp_consensus_babe::AllowedSlots
   **/
  SpConsensusBabeAllowedSlots: {
    _enum: ['PrimarySlots', 'PrimaryAndSecondaryPlainSlots', 'PrimaryAndSecondaryVRFSlots']
  },
  /**
   * Lookup229: pallet_grandpa::pallet::Call<T>
   **/
  PalletGrandpaCall: {
    _enum: {
      report_equivocation: {
        equivocationProof: 'SpConsensusGrandpaEquivocationProof',
        keyOwnerProof: 'SpCoreVoid',
      },
      report_equivocation_unsigned: {
        equivocationProof: 'SpConsensusGrandpaEquivocationProof',
        keyOwnerProof: 'SpCoreVoid',
      },
      note_stalled: {
        delay: 'u64',
        bestFinalizedBlockNumber: 'u64'
      }
    }
  },
  /**
   * Lookup230: sp_consensus_grandpa::EquivocationProof<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocationProof: {
    setId: 'u64',
    equivocation: 'SpConsensusGrandpaEquivocation'
  },
  /**
   * Lookup231: sp_consensus_grandpa::Equivocation<primitive_types::H256, N>
   **/
  SpConsensusGrandpaEquivocation: {
    _enum: {
      Prevote: 'FinalityGrandpaEquivocationPrevote',
      Precommit: 'FinalityGrandpaEquivocationPrecommit'
    }
  },
  /**
   * Lookup232: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrevote: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrevote,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup233: finality_grandpa::Prevote<primitive_types::H256, N>
   **/
  FinalityGrandpaPrevote: {
    targetHash: 'H256',
    targetNumber: 'u64'
  },
  /**
   * Lookup234: sp_consensus_grandpa::app::Signature
   **/
  SpConsensusGrandpaAppSignature: 'SpCoreEd25519Signature',
  /**
   * Lookup235: sp_core::ed25519::Signature
   **/
  SpCoreEd25519Signature: '[u8;64]',
  /**
   * Lookup238: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
   **/
  FinalityGrandpaEquivocationPrecommit: {
    roundNumber: 'u64',
    identity: 'SpConsensusGrandpaAppPublic',
    first: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)',
    second: '(FinalityGrandpaPrecommit,SpConsensusGrandpaAppSignature)'
  },
  /**
   * Lookup239: finality_grandpa::Precommit<primitive_types::H256, N>
   **/
  FinalityGrandpaPrecommit: {
    targetHash: 'H256',
    targetNumber: 'u64'
  },
  /**
   * Lookup241: sp_core::Void
   **/
  SpCoreVoid: 'Null',
  /**
   * Lookup242: pallet_indices::pallet::Call<T>
   **/
  PalletIndicesCall: {
    _enum: {
      claim: {
        index: 'u32',
      },
      transfer: {
        _alias: {
          new_: 'new',
        },
        new_: 'MultiAddress',
        index: 'u32',
      },
      free: {
        index: 'u32',
      },
      force_transfer: {
        _alias: {
          new_: 'new',
        },
        new_: 'MultiAddress',
        index: 'u32',
        freeze: 'bool',
      },
      freeze: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup243: pallet_democracy::pallet::Call<T>
   **/
  PalletDemocracyCall: {
    _enum: {
      propose: {
        proposal: 'FrameSupportPreimagesBounded',
        value: 'Compact<u128>',
      },
      second: {
        proposal: 'Compact<u32>',
      },
      vote: {
        refIndex: 'Compact<u32>',
        vote: 'PalletDemocracyVoteAccountVote',
      },
      emergency_cancel: {
        refIndex: 'u32',
      },
      external_propose: {
        proposal: 'FrameSupportPreimagesBounded',
      },
      external_propose_majority: {
        proposal: 'FrameSupportPreimagesBounded',
      },
      external_propose_default: {
        proposal: 'FrameSupportPreimagesBounded',
      },
      fast_track: {
        proposalHash: 'H256',
        votingPeriod: 'u64',
        delay: 'u64',
      },
      veto_external: {
        proposalHash: 'H256',
      },
      cancel_referendum: {
        refIndex: 'Compact<u32>',
      },
      delegate: {
        to: 'MultiAddress',
        conviction: 'PalletDemocracyConviction',
        balance: 'u128',
      },
      undelegate: 'Null',
      clear_public_proposals: 'Null',
      unlock: {
        target: 'MultiAddress',
      },
      remove_vote: {
        index: 'u32',
      },
      remove_other_vote: {
        target: 'MultiAddress',
        index: 'u32',
      },
      blacklist: {
        proposalHash: 'H256',
        maybeRefIndex: 'Option<u32>',
      },
      cancel_proposal: {
        propIndex: 'Compact<u32>',
      },
      set_metadata: {
        owner: 'PalletDemocracyMetadataOwner',
        maybeHash: 'Option<H256>'
      }
    }
  },
  /**
   * Lookup244: frame_support::traits::preimages::Bounded<tangle_testnet_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>
   **/
  FrameSupportPreimagesBounded: {
    _enum: {
      Legacy: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      Inline: 'Bytes',
      Lookup: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
        len: 'u32'
      }
    }
  },
  /**
   * Lookup245: sp_runtime::traits::BlakeTwo256
   **/
  SpRuntimeBlakeTwo256: 'Null',
  /**
   * Lookup247: pallet_democracy::conviction::Conviction
   **/
  PalletDemocracyConviction: {
    _enum: ['None', 'Locked1x', 'Locked2x', 'Locked3x', 'Locked4x', 'Locked5x', 'Locked6x']
  },
  /**
   * Lookup250: pallet_collective::pallet::Call<T, I>
   **/
  PalletCollectiveCall: {
    _enum: {
      set_members: {
        newMembers: 'Vec<AccountId32>',
        prime: 'Option<AccountId32>',
        oldCount: 'u32',
      },
      execute: {
        proposal: 'Call',
        lengthBound: 'Compact<u32>',
      },
      propose: {
        threshold: 'Compact<u32>',
        proposal: 'Call',
        lengthBound: 'Compact<u32>',
      },
      vote: {
        proposal: 'H256',
        index: 'Compact<u32>',
        approve: 'bool',
      },
      __Unused4: 'Null',
      disapprove_proposal: {
        proposalHash: 'H256',
      },
      close: {
        proposalHash: 'H256',
        index: 'Compact<u32>',
        proposalWeightBound: 'SpWeightsWeightV2Weight',
        lengthBound: 'Compact<u32>'
      }
    }
  },
  /**
   * Lookup251: pallet_vesting::pallet::Call<T>
   **/
  PalletVestingCall: {
    _enum: {
      vest: 'Null',
      vest_other: {
        target: 'MultiAddress',
      },
      vested_transfer: {
        target: 'MultiAddress',
        schedule: 'PalletVestingVestingInfo',
      },
      force_vested_transfer: {
        source: 'MultiAddress',
        target: 'MultiAddress',
        schedule: 'PalletVestingVestingInfo',
      },
      merge_schedules: {
        schedule1Index: 'u32',
        schedule2Index: 'u32',
      },
      force_remove_vesting_schedule: {
        target: 'MultiAddress',
        scheduleIndex: 'u32'
      }
    }
  },
  /**
   * Lookup252: pallet_vesting::vesting_info::VestingInfo<Balance, BlockNumber>
   **/
  PalletVestingVestingInfo: {
    locked: 'u128',
    perBlock: 'u128',
    startingBlock: 'u64'
  },
  /**
   * Lookup253: pallet_elections_phragmen::pallet::Call<T>
   **/
  PalletElectionsPhragmenCall: {
    _enum: {
      vote: {
        votes: 'Vec<AccountId32>',
        value: 'Compact<u128>',
      },
      remove_voter: 'Null',
      submit_candidacy: {
        candidateCount: 'Compact<u32>',
      },
      renounce_candidacy: {
        renouncing: 'PalletElectionsPhragmenRenouncing',
      },
      remove_member: {
        who: 'MultiAddress',
        slashBond: 'bool',
        rerunElection: 'bool',
      },
      clean_defunct_voters: {
        numVoters: 'u32',
        numDefunct: 'u32'
      }
    }
  },
  /**
   * Lookup254: pallet_elections_phragmen::Renouncing
   **/
  PalletElectionsPhragmenRenouncing: {
    _enum: {
      Member: 'Null',
      RunnerUp: 'Null',
      Candidate: 'Compact<u32>'
    }
  },
  /**
   * Lookup255: pallet_election_provider_multi_phase::pallet::Call<T>
   **/
  PalletElectionProviderMultiPhaseCall: {
    _enum: {
      submit_unsigned: {
        rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
        witness: 'PalletElectionProviderMultiPhaseSolutionOrSnapshotSize',
      },
      set_minimum_untrusted_score: {
        maybeNextScore: 'Option<SpNposElectionsElectionScore>',
      },
      set_emergency_election_result: {
        supports: 'Vec<(AccountId32,SpNposElectionsSupport)>',
      },
      submit: {
        rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
      },
      governance_fallback: {
        maybeMaxVoters: 'Option<u32>',
        maybeMaxTargets: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup256: pallet_election_provider_multi_phase::RawSolution<tangle_testnet_runtime::NposSolution16>
   **/
  PalletElectionProviderMultiPhaseRawSolution: {
    solution: 'TangleTestnetRuntimeNposSolution16',
    score: 'SpNposElectionsElectionScore',
    round: 'u32'
  },
  /**
   * Lookup257: tangle_testnet_runtime::NposSolution16
   **/
  TangleTestnetRuntimeNposSolution16: {
    votes1: 'Vec<(Compact<u32>,Compact<u16>)>',
    votes2: 'Vec<(Compact<u32>,(Compact<u16>,Compact<PerU16>),Compact<u16>)>',
    votes3: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);2],Compact<u16>)>',
    votes4: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);3],Compact<u16>)>',
    votes5: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);4],Compact<u16>)>',
    votes6: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);5],Compact<u16>)>',
    votes7: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);6],Compact<u16>)>',
    votes8: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);7],Compact<u16>)>',
    votes9: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);8],Compact<u16>)>',
    votes10: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);9],Compact<u16>)>',
    votes11: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);10],Compact<u16>)>',
    votes12: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);11],Compact<u16>)>',
    votes13: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);12],Compact<u16>)>',
    votes14: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);13],Compact<u16>)>',
    votes15: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);14],Compact<u16>)>',
    votes16: 'Vec<(Compact<u32>,[(Compact<u16>,Compact<PerU16>);15],Compact<u16>)>'
  },
  /**
   * Lookup308: pallet_election_provider_multi_phase::SolutionOrSnapshotSize
   **/
  PalletElectionProviderMultiPhaseSolutionOrSnapshotSize: {
    voters: 'Compact<u32>',
    targets: 'Compact<u32>'
  },
  /**
   * Lookup312: sp_npos_elections::Support<sp_core::crypto::AccountId32>
   **/
  SpNposElectionsSupport: {
    total: 'u128',
    voters: 'Vec<(AccountId32,u128)>'
  },
  /**
   * Lookup313: pallet_staking::pallet::pallet::Call<T>
   **/
  PalletStakingPalletCall: {
    _enum: {
      bond: {
        value: 'Compact<u128>',
        payee: 'PalletStakingRewardDestination',
      },
      bond_extra: {
        maxAdditional: 'Compact<u128>',
      },
      unbond: {
        value: 'Compact<u128>',
      },
      withdraw_unbonded: {
        numSlashingSpans: 'u32',
      },
      validate: {
        prefs: 'PalletStakingValidatorPrefs',
      },
      nominate: {
        targets: 'Vec<MultiAddress>',
      },
      chill: 'Null',
      set_payee: {
        payee: 'PalletStakingRewardDestination',
      },
      set_controller: 'Null',
      set_validator_count: {
        _alias: {
          new_: 'new',
        },
        new_: 'Compact<u32>',
      },
      increase_validator_count: {
        additional: 'Compact<u32>',
      },
      scale_validator_count: {
        factor: 'Percent',
      },
      force_no_eras: 'Null',
      force_new_era: 'Null',
      set_invulnerables: {
        invulnerables: 'Vec<AccountId32>',
      },
      force_unstake: {
        stash: 'AccountId32',
        numSlashingSpans: 'u32',
      },
      force_new_era_always: 'Null',
      cancel_deferred_slash: {
        era: 'u32',
        slashIndices: 'Vec<u32>',
      },
      payout_stakers: {
        validatorStash: 'AccountId32',
        era: 'u32',
      },
      rebond: {
        value: 'Compact<u128>',
      },
      reap_stash: {
        stash: 'AccountId32',
        numSlashingSpans: 'u32',
      },
      kick: {
        who: 'Vec<MultiAddress>',
      },
      set_staking_configs: {
        minNominatorBond: 'PalletStakingPalletConfigOpU128',
        minValidatorBond: 'PalletStakingPalletConfigOpU128',
        maxNominatorCount: 'PalletStakingPalletConfigOpU32',
        maxValidatorCount: 'PalletStakingPalletConfigOpU32',
        chillThreshold: 'PalletStakingPalletConfigOpPercent',
        minCommission: 'PalletStakingPalletConfigOpPerbill',
      },
      chill_other: {
        stash: 'AccountId32',
      },
      force_apply_min_commission: {
        validatorStash: 'AccountId32',
      },
      set_min_commission: {
        _alias: {
          new_: 'new',
        },
        new_: 'Perbill',
      },
      payout_stakers_by_page: {
        validatorStash: 'AccountId32',
        era: 'u32',
        page: 'u32',
      },
      update_payee: {
        controller: 'AccountId32',
      },
      deprecate_controller_batch: {
        controllers: 'Vec<AccountId32>'
      }
    }
  },
  /**
   * Lookup317: pallet_staking::pallet::pallet::ConfigOp<T>
   **/
  PalletStakingPalletConfigOpU128: {
    _enum: {
      Noop: 'Null',
      Set: 'u128',
      Remove: 'Null'
    }
  },
  /**
   * Lookup318: pallet_staking::pallet::pallet::ConfigOp<T>
   **/
  PalletStakingPalletConfigOpU32: {
    _enum: {
      Noop: 'Null',
      Set: 'u32',
      Remove: 'Null'
    }
  },
  /**
   * Lookup319: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Percent>
   **/
  PalletStakingPalletConfigOpPercent: {
    _enum: {
      Noop: 'Null',
      Set: 'Percent',
      Remove: 'Null'
    }
  },
  /**
   * Lookup320: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Perbill>
   **/
  PalletStakingPalletConfigOpPerbill: {
    _enum: {
      Noop: 'Null',
      Set: 'Perbill',
      Remove: 'Null'
    }
  },
  /**
   * Lookup322: pallet_session::pallet::Call<T>
   **/
  PalletSessionCall: {
    _enum: {
      set_keys: {
        _alias: {
          keys_: 'keys',
        },
        keys_: 'TangleTestnetRuntimeOpaqueSessionKeys',
        proof: 'Bytes',
      },
      purge_keys: 'Null'
    }
  },
  /**
   * Lookup323: tangle_testnet_runtime::opaque::SessionKeys
   **/
  TangleTestnetRuntimeOpaqueSessionKeys: {
    babe: 'SpConsensusBabeAppPublic',
    grandpa: 'SpConsensusGrandpaAppPublic',
    imOnline: 'PalletImOnlineSr25519AppSr25519Public',
    role: 'TangleCryptoPrimitivesCryptoPublic'
  },
  /**
   * Lookup324: tangle_crypto_primitives::crypto::Public
   **/
  TangleCryptoPrimitivesCryptoPublic: 'SpCoreEcdsaPublic',
  /**
   * Lookup325: sp_core::ecdsa::Public
   **/
  SpCoreEcdsaPublic: '[u8;33]',
  /**
   * Lookup327: pallet_treasury::pallet::Call<T, I>
   **/
  PalletTreasuryCall: {
    _enum: {
      propose_spend: {
        value: 'Compact<u128>',
        beneficiary: 'MultiAddress',
      },
      reject_proposal: {
        proposalId: 'Compact<u32>',
      },
      approve_proposal: {
        proposalId: 'Compact<u32>',
      },
      spend_local: {
        amount: 'Compact<u128>',
        beneficiary: 'MultiAddress',
      },
      remove_approval: {
        proposalId: 'Compact<u32>',
      },
      spend: {
        assetKind: 'Null',
        amount: 'Compact<u128>',
        beneficiary: 'AccountId32',
        validFrom: 'Option<u64>',
      },
      payout: {
        index: 'u32',
      },
      check_status: {
        index: 'u32',
      },
      void_spend: {
        index: 'u32'
      }
    }
  },
  /**
   * Lookup329: pallet_bounties::pallet::Call<T, I>
   **/
  PalletBountiesCall: {
    _enum: {
      propose_bounty: {
        value: 'Compact<u128>',
        description: 'Bytes',
      },
      approve_bounty: {
        bountyId: 'Compact<u32>',
      },
      propose_curator: {
        bountyId: 'Compact<u32>',
        curator: 'MultiAddress',
        fee: 'Compact<u128>',
      },
      unassign_curator: {
        bountyId: 'Compact<u32>',
      },
      accept_curator: {
        bountyId: 'Compact<u32>',
      },
      award_bounty: {
        bountyId: 'Compact<u32>',
        beneficiary: 'MultiAddress',
      },
      claim_bounty: {
        bountyId: 'Compact<u32>',
      },
      close_bounty: {
        bountyId: 'Compact<u32>',
      },
      extend_bounty_expiry: {
        bountyId: 'Compact<u32>',
        remark: 'Bytes'
      }
    }
  },
  /**
   * Lookup330: pallet_child_bounties::pallet::Call<T>
   **/
  PalletChildBountiesCall: {
    _enum: {
      add_child_bounty: {
        parentBountyId: 'Compact<u32>',
        value: 'Compact<u128>',
        description: 'Bytes',
      },
      propose_curator: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>',
        curator: 'MultiAddress',
        fee: 'Compact<u128>',
      },
      accept_curator: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>',
      },
      unassign_curator: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>',
      },
      award_child_bounty: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>',
        beneficiary: 'MultiAddress',
      },
      claim_child_bounty: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>',
      },
      close_child_bounty: {
        parentBountyId: 'Compact<u32>',
        childBountyId: 'Compact<u32>'
      }
    }
  },
  /**
   * Lookup331: pallet_bags_list::pallet::Call<T, I>
   **/
  PalletBagsListCall: {
    _enum: {
      rebag: {
        dislocated: 'MultiAddress',
      },
      put_in_front_of: {
        lighter: 'MultiAddress',
      },
      put_in_front_of_other: {
        heavier: 'MultiAddress',
        lighter: 'MultiAddress'
      }
    }
  },
  /**
   * Lookup332: pallet_nomination_pools::pallet::Call<T>
   **/
  PalletNominationPoolsCall: {
    _enum: {
      join: {
        amount: 'Compact<u128>',
        poolId: 'u32',
      },
      bond_extra: {
        extra: 'PalletNominationPoolsBondExtra',
      },
      claim_payout: 'Null',
      unbond: {
        memberAccount: 'MultiAddress',
        unbondingPoints: 'Compact<u128>',
      },
      pool_withdraw_unbonded: {
        poolId: 'u32',
        numSlashingSpans: 'u32',
      },
      withdraw_unbonded: {
        memberAccount: 'MultiAddress',
        numSlashingSpans: 'u32',
      },
      create: {
        amount: 'Compact<u128>',
        root: 'MultiAddress',
        nominator: 'MultiAddress',
        bouncer: 'MultiAddress',
      },
      create_with_pool_id: {
        amount: 'Compact<u128>',
        root: 'MultiAddress',
        nominator: 'MultiAddress',
        bouncer: 'MultiAddress',
        poolId: 'u32',
      },
      nominate: {
        poolId: 'u32',
        validators: 'Vec<AccountId32>',
      },
      set_state: {
        poolId: 'u32',
        state: 'PalletNominationPoolsPoolState',
      },
      set_metadata: {
        poolId: 'u32',
        metadata: 'Bytes',
      },
      set_configs: {
        minJoinBond: 'PalletNominationPoolsConfigOpU128',
        minCreateBond: 'PalletNominationPoolsConfigOpU128',
        maxPools: 'PalletNominationPoolsConfigOpU32',
        maxMembers: 'PalletNominationPoolsConfigOpU32',
        maxMembersPerPool: 'PalletNominationPoolsConfigOpU32',
        globalMaxCommission: 'PalletNominationPoolsConfigOpPerbill',
      },
      update_roles: {
        poolId: 'u32',
        newRoot: 'PalletNominationPoolsConfigOpAccountId32',
        newNominator: 'PalletNominationPoolsConfigOpAccountId32',
        newBouncer: 'PalletNominationPoolsConfigOpAccountId32',
      },
      chill: {
        poolId: 'u32',
      },
      bond_extra_other: {
        member: 'MultiAddress',
        extra: 'PalletNominationPoolsBondExtra',
      },
      set_claim_permission: {
        permission: 'PalletNominationPoolsClaimPermission',
      },
      claim_payout_other: {
        other: 'AccountId32',
      },
      set_commission: {
        poolId: 'u32',
        newCommission: 'Option<(Perbill,AccountId32)>',
      },
      set_commission_max: {
        poolId: 'u32',
        maxCommission: 'Perbill',
      },
      set_commission_change_rate: {
        poolId: 'u32',
        changeRate: 'PalletNominationPoolsCommissionChangeRate',
      },
      claim_commission: {
        poolId: 'u32',
      },
      adjust_pool_deposit: {
        poolId: 'u32',
      },
      set_commission_claim_permission: {
        poolId: 'u32',
        permission: 'Option<PalletNominationPoolsCommissionClaimPermission>'
      }
    }
  },
  /**
   * Lookup333: pallet_nomination_pools::BondExtra<Balance>
   **/
  PalletNominationPoolsBondExtra: {
    _enum: {
      FreeBalance: 'u128',
      Rewards: 'Null'
    }
  },
  /**
   * Lookup334: pallet_nomination_pools::ConfigOp<T>
   **/
  PalletNominationPoolsConfigOpU128: {
    _enum: {
      Noop: 'Null',
      Set: 'u128',
      Remove: 'Null'
    }
  },
  /**
   * Lookup335: pallet_nomination_pools::ConfigOp<T>
   **/
  PalletNominationPoolsConfigOpU32: {
    _enum: {
      Noop: 'Null',
      Set: 'u32',
      Remove: 'Null'
    }
  },
  /**
   * Lookup336: pallet_nomination_pools::ConfigOp<sp_arithmetic::per_things::Perbill>
   **/
  PalletNominationPoolsConfigOpPerbill: {
    _enum: {
      Noop: 'Null',
      Set: 'Perbill',
      Remove: 'Null'
    }
  },
  /**
   * Lookup337: pallet_nomination_pools::ConfigOp<sp_core::crypto::AccountId32>
   **/
  PalletNominationPoolsConfigOpAccountId32: {
    _enum: {
      Noop: 'Null',
      Set: 'AccountId32',
      Remove: 'Null'
    }
  },
  /**
   * Lookup338: pallet_nomination_pools::ClaimPermission
   **/
  PalletNominationPoolsClaimPermission: {
    _enum: ['Permissioned', 'PermissionlessCompound', 'PermissionlessWithdraw', 'PermissionlessAll']
  },
  /**
   * Lookup339: pallet_scheduler::pallet::Call<T>
   **/
  PalletSchedulerCall: {
    _enum: {
      schedule: {
        when: 'u64',
        maybePeriodic: 'Option<(u64,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel: {
        when: 'u64',
        index: 'u32',
      },
      schedule_named: {
        id: '[u8;32]',
        when: 'u64',
        maybePeriodic: 'Option<(u64,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      cancel_named: {
        id: '[u8;32]',
      },
      schedule_after: {
        after: 'u64',
        maybePeriodic: 'Option<(u64,u32)>',
        priority: 'u8',
        call: 'Call',
      },
      schedule_named_after: {
        id: '[u8;32]',
        after: 'u64',
        maybePeriodic: 'Option<(u64,u32)>',
        priority: 'u8',
        call: 'Call'
      }
    }
  },
  /**
   * Lookup341: pallet_preimage::pallet::Call<T>
   **/
  PalletPreimageCall: {
    _enum: {
      note_preimage: {
        bytes: 'Bytes',
      },
      unnote_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      request_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      unrequest_preimage: {
        _alias: {
          hash_: 'hash',
        },
        hash_: 'H256',
      },
      ensure_updated: {
        hashes: 'Vec<H256>'
      }
    }
  },
  /**
   * Lookup342: pallet_tx_pause::pallet::Call<T>
   **/
  PalletTxPauseCall: {
    _enum: {
      pause: {
        fullName: '(Bytes,Bytes)',
      },
      unpause: {
        ident: '(Bytes,Bytes)'
      }
    }
  },
  /**
   * Lookup343: pallet_im_online::pallet::Call<T>
   **/
  PalletImOnlineCall: {
    _enum: {
      heartbeat: {
        heartbeat: 'PalletImOnlineHeartbeat',
        signature: 'PalletImOnlineSr25519AppSr25519Signature'
      }
    }
  },
  /**
   * Lookup344: pallet_im_online::Heartbeat<BlockNumber>
   **/
  PalletImOnlineHeartbeat: {
    blockNumber: 'u64',
    sessionIndex: 'u32',
    authorityIndex: 'u32',
    validatorsLen: 'u32'
  },
  /**
   * Lookup345: pallet_im_online::sr25519::app_sr25519::Signature
   **/
  PalletImOnlineSr25519AppSr25519Signature: 'SpCoreSr25519Signature',
  /**
   * Lookup346: sp_core::sr25519::Signature
   **/
  SpCoreSr25519Signature: '[u8;64]',
  /**
   * Lookup347: pallet_identity::pallet::Call<T>
   **/
  PalletIdentityCall: {
    _enum: {
      add_registrar: {
        account: 'MultiAddress',
      },
      set_identity: {
        info: 'PalletIdentityLegacyIdentityInfo',
      },
      set_subs: {
        subs: 'Vec<(AccountId32,Data)>',
      },
      clear_identity: 'Null',
      request_judgement: {
        regIndex: 'Compact<u32>',
        maxFee: 'Compact<u128>',
      },
      cancel_request: {
        regIndex: 'u32',
      },
      set_fee: {
        index: 'Compact<u32>',
        fee: 'Compact<u128>',
      },
      set_account_id: {
        _alias: {
          new_: 'new',
        },
        index: 'Compact<u32>',
        new_: 'MultiAddress',
      },
      set_fields: {
        index: 'Compact<u32>',
        fields: 'u64',
      },
      provide_judgement: {
        regIndex: 'Compact<u32>',
        target: 'MultiAddress',
        judgement: 'PalletIdentityJudgement',
        identity: 'H256',
      },
      kill_identity: {
        target: 'MultiAddress',
      },
      add_sub: {
        sub: 'MultiAddress',
        data: 'Data',
      },
      rename_sub: {
        sub: 'MultiAddress',
        data: 'Data',
      },
      remove_sub: {
        sub: 'MultiAddress',
      },
      quit_sub: 'Null',
      add_username_authority: {
        authority: 'MultiAddress',
        suffix: 'Bytes',
        allocation: 'u32',
      },
      remove_username_authority: {
        authority: 'MultiAddress',
      },
      set_username_for: {
        who: 'MultiAddress',
        username: 'Bytes',
        signature: 'Option<SpRuntimeMultiSignature>',
      },
      accept_username: {
        username: 'Bytes',
      },
      remove_expired_approval: {
        username: 'Bytes',
      },
      set_primary_username: {
        username: 'Bytes',
      },
      remove_dangling_username: {
        username: 'Bytes'
      }
    }
  },
  /**
   * Lookup348: pallet_identity::legacy::IdentityInfo<FieldLimit>
   **/
  PalletIdentityLegacyIdentityInfo: {
    additional: 'Vec<(Data,Data)>',
    display: 'Data',
    legal: 'Data',
    web: 'Data',
    riot: 'Data',
    email: 'Data',
    pgpFingerprint: 'Option<[u8;20]>',
    image: 'Data',
    twitter: 'Data'
  },
  /**
   * Lookup384: pallet_identity::types::Judgement<Balance>
   **/
  PalletIdentityJudgement: {
    _enum: {
      Unknown: 'Null',
      FeePaid: 'u128',
      Reasonable: 'Null',
      KnownGood: 'Null',
      OutOfDate: 'Null',
      LowQuality: 'Null',
      Erroneous: 'Null'
    }
  },
  /**
   * Lookup386: sp_runtime::MultiSignature
   **/
  SpRuntimeMultiSignature: {
    _enum: {
      Ed25519: 'SpCoreEd25519Signature',
      Sr25519: 'SpCoreSr25519Signature',
      Ecdsa: 'SpCoreEcdsaSignature'
    }
  },
  /**
   * Lookup387: sp_core::ecdsa::Signature
   **/
  SpCoreEcdsaSignature: '[u8;65]',
  /**
   * Lookup389: pallet_utility::pallet::Call<T>
   **/
  PalletUtilityCall: {
    _enum: {
      batch: {
        calls: 'Vec<Call>',
      },
      as_derivative: {
        index: 'u16',
        call: 'Call',
      },
      batch_all: {
        calls: 'Vec<Call>',
      },
      dispatch_as: {
        asOrigin: 'TangleTestnetRuntimeOriginCaller',
        call: 'Call',
      },
      force_batch: {
        calls: 'Vec<Call>',
      },
      with_weight: {
        call: 'Call',
        weight: 'SpWeightsWeightV2Weight'
      }
    }
  },
  /**
   * Lookup391: tangle_testnet_runtime::OriginCaller
   **/
  TangleTestnetRuntimeOriginCaller: {
    _enum: {
      system: 'FrameSupportDispatchRawOrigin',
      __Unused1: 'Null',
      __Unused2: 'Null',
      Void: 'SpCoreVoid',
      __Unused4: 'Null',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      __Unused9: 'Null',
      __Unused10: 'Null',
      __Unused11: 'Null',
      Council: 'PalletCollectiveRawOrigin',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      __Unused17: 'Null',
      __Unused18: 'Null',
      __Unused19: 'Null',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      __Unused23: 'Null',
      __Unused24: 'Null',
      __Unused25: 'Null',
      __Unused26: 'Null',
      __Unused27: 'Null',
      __Unused28: 'Null',
      __Unused29: 'Null',
      __Unused30: 'Null',
      __Unused31: 'Null',
      Ethereum: 'PalletEthereumRawOrigin'
    }
  },
  /**
   * Lookup392: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
   **/
  FrameSupportDispatchRawOrigin: {
    _enum: {
      Root: 'Null',
      Signed: 'AccountId32',
      None: 'Null'
    }
  },
  /**
   * Lookup393: pallet_collective::RawOrigin<sp_core::crypto::AccountId32, I>
   **/
  PalletCollectiveRawOrigin: {
    _enum: {
      Members: '(u32,u32)',
      Member: 'AccountId32',
      _Phantom: 'Null'
    }
  },
  /**
   * Lookup394: pallet_ethereum::RawOrigin
   **/
  PalletEthereumRawOrigin: {
    _enum: {
      EthereumTransaction: 'H160'
    }
  },
  /**
   * Lookup395: pallet_multisig::pallet::Call<T>
   **/
  PalletMultisigCall: {
    _enum: {
      as_multi_threshold_1: {
        otherSignatories: 'Vec<AccountId32>',
        call: 'Call',
      },
      as_multi: {
        threshold: 'u16',
        otherSignatories: 'Vec<AccountId32>',
        maybeTimepoint: 'Option<PalletMultisigTimepoint>',
        call: 'Call',
        maxWeight: 'SpWeightsWeightV2Weight',
      },
      approve_as_multi: {
        threshold: 'u16',
        otherSignatories: 'Vec<AccountId32>',
        maybeTimepoint: 'Option<PalletMultisigTimepoint>',
        callHash: '[u8;32]',
        maxWeight: 'SpWeightsWeightV2Weight',
      },
      cancel_as_multi: {
        threshold: 'u16',
        otherSignatories: 'Vec<AccountId32>',
        timepoint: 'PalletMultisigTimepoint',
        callHash: '[u8;32]'
      }
    }
  },
  /**
   * Lookup397: pallet_ethereum::pallet::Call<T>
   **/
  PalletEthereumCall: {
    _enum: {
      transact: {
        transaction: 'EthereumTransactionTransactionV2'
      }
    }
  },
  /**
   * Lookup398: ethereum::transaction::TransactionV2
   **/
  EthereumTransactionTransactionV2: {
    _enum: {
      Legacy: 'EthereumTransactionLegacyTransaction',
      EIP2930: 'EthereumTransactionEip2930Transaction',
      EIP1559: 'EthereumTransactionEip1559Transaction'
    }
  },
  /**
   * Lookup399: ethereum::transaction::LegacyTransaction
   **/
  EthereumTransactionLegacyTransaction: {
    nonce: 'U256',
    gasPrice: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    signature: 'EthereumTransactionTransactionSignature'
  },
  /**
   * Lookup400: ethereum::transaction::TransactionAction
   **/
  EthereumTransactionTransactionAction: {
    _enum: {
      Call: 'H160',
      Create: 'Null'
    }
  },
  /**
   * Lookup401: ethereum::transaction::TransactionSignature
   **/
  EthereumTransactionTransactionSignature: {
    v: 'u64',
    r: 'H256',
    s: 'H256'
  },
  /**
   * Lookup403: ethereum::transaction::EIP2930Transaction
   **/
  EthereumTransactionEip2930Transaction: {
    chainId: 'u64',
    nonce: 'U256',
    gasPrice: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    accessList: 'Vec<EthereumTransactionAccessListItem>',
    oddYParity: 'bool',
    r: 'H256',
    s: 'H256'
  },
  /**
   * Lookup405: ethereum::transaction::AccessListItem
   **/
  EthereumTransactionAccessListItem: {
    address: 'H160',
    storageKeys: 'Vec<H256>'
  },
  /**
   * Lookup406: ethereum::transaction::EIP1559Transaction
   **/
  EthereumTransactionEip1559Transaction: {
    chainId: 'u64',
    nonce: 'U256',
    maxPriorityFeePerGas: 'U256',
    maxFeePerGas: 'U256',
    gasLimit: 'U256',
    action: 'EthereumTransactionTransactionAction',
    value: 'U256',
    input: 'Bytes',
    accessList: 'Vec<EthereumTransactionAccessListItem>',
    oddYParity: 'bool',
    r: 'H256',
    s: 'H256'
  },
  /**
   * Lookup407: pallet_evm::pallet::Call<T>
   **/
  PalletEvmCall: {
    _enum: {
      withdraw: {
        address: 'H160',
        value: 'u128',
      },
      call: {
        source: 'H160',
        target: 'H160',
        input: 'Bytes',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>',
      },
      create: {
        source: 'H160',
        init: 'Bytes',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>',
      },
      create2: {
        source: 'H160',
        init: 'Bytes',
        salt: 'H256',
        value: 'U256',
        gasLimit: 'u64',
        maxFeePerGas: 'U256',
        maxPriorityFeePerGas: 'Option<U256>',
        nonce: 'Option<U256>',
        accessList: 'Vec<(H160,Vec<H256>)>'
      }
    }
  },
  /**
   * Lookup411: pallet_dynamic_fee::pallet::Call<T>
   **/
  PalletDynamicFeeCall: {
    _enum: {
      note_min_gas_price_target: {
        target: 'U256'
      }
    }
  },
  /**
   * Lookup412: pallet_base_fee::pallet::Call<T>
   **/
  PalletBaseFeeCall: {
    _enum: {
      set_base_fee_per_gas: {
        fee: 'U256',
      },
      set_elasticity: {
        elasticity: 'Permill'
      }
    }
  },
  /**
   * Lookup413: pallet_hotfix_sufficients::pallet::Call<T>
   **/
  PalletHotfixSufficientsCall: {
    _enum: {
      hotfix_inc_account_sufficients: {
        addresses: 'Vec<H160>'
      }
    }
  },
  /**
   * Lookup415: pallet_airdrop_claims::pallet::Call<T>
   **/
  PalletAirdropClaimsCall: {
    _enum: {
      claim: {
        dest: 'Option<PalletAirdropClaimsUtilsMultiAddress>',
        signer: 'Option<PalletAirdropClaimsUtilsMultiAddress>',
        signature: 'PalletAirdropClaimsUtilsMultiAddressSignature',
      },
      mint_claim: {
        who: 'PalletAirdropClaimsUtilsMultiAddress',
        value: 'u128',
        vestingSchedule: 'Option<Vec<(u128,u128,u64)>>',
        statement: 'Option<PalletAirdropClaimsStatementKind>',
      },
      claim_attest: {
        dest: 'Option<PalletAirdropClaimsUtilsMultiAddress>',
        signer: 'Option<PalletAirdropClaimsUtilsMultiAddress>',
        signature: 'PalletAirdropClaimsUtilsMultiAddressSignature',
        statement: 'Bytes',
      },
      __Unused3: 'Null',
      move_claim: {
        _alias: {
          new_: 'new',
        },
        old: 'PalletAirdropClaimsUtilsMultiAddress',
        new_: 'PalletAirdropClaimsUtilsMultiAddress',
      },
      force_set_expiry_config: {
        expiryBlock: 'u64',
        dest: 'PalletAirdropClaimsUtilsMultiAddress',
      },
      claim_signed: {
        dest: 'Option<PalletAirdropClaimsUtilsMultiAddress>'
      }
    }
  },
  /**
   * Lookup417: pallet_airdrop_claims::utils::MultiAddressSignature
   **/
  PalletAirdropClaimsUtilsMultiAddressSignature: {
    _enum: {
      EVM: 'PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature',
      Native: 'PalletAirdropClaimsUtilsSr25519Signature'
    }
  },
  /**
   * Lookup418: pallet_airdrop_claims::utils::ethereum_address::EcdsaSignature
   **/
  PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature: '[u8;65]',
  /**
   * Lookup419: pallet_airdrop_claims::utils::Sr25519Signature
   **/
  PalletAirdropClaimsUtilsSr25519Signature: 'SpCoreSr25519Signature',
  /**
   * Lookup425: pallet_airdrop_claims::StatementKind
   **/
  PalletAirdropClaimsStatementKind: {
    _enum: ['Regular', 'Safe']
  },
  /**
   * Lookup426: pallet_roles::pallet::Call<T>
   **/
  PalletRolesCall: {
    _enum: {
      create_profile: {
        profile: 'PalletRolesProfile',
        maxActiveServices: 'Option<u32>',
      },
      update_profile: {
        updatedProfile: 'PalletRolesProfile',
      },
      delete_profile: 'Null',
      chill: 'Null',
      unbond_funds: {
        amount: 'Compact<u128>',
      },
      withdraw_unbonded: 'Null',
      payout_stakers: {
        validatorStash: 'AccountId32',
        era: 'u32',
      },
      set_min_restaking_bond: {
        minRestakingBond: 'u128'
      }
    }
  },
  /**
   * Lookup427: pallet_roles::profile::Profile<T>
   **/
  PalletRolesProfile: {
    _enum: {
      Independent: 'PalletRolesProfileIndependentRestakeProfile',
      Shared: 'PalletRolesProfileSharedRestakeProfile'
    }
  },
  /**
   * Lookup428: pallet_roles::profile::IndependentRestakeProfile<T>
   **/
  PalletRolesProfileIndependentRestakeProfile: {
    records: 'Vec<PalletRolesProfileRecord>'
  },
  /**
   * Lookup430: pallet_roles::profile::Record<T>
   **/
  PalletRolesProfileRecord: {
    role: 'TanglePrimitivesRolesRoleType',
    amount: 'Option<u128>'
  },
  /**
   * Lookup432: pallet_roles::profile::SharedRestakeProfile<T>
   **/
  PalletRolesProfileSharedRestakeProfile: {
    records: 'Vec<PalletRolesProfileRecord>',
    amount: 'u128'
  },
  /**
   * Lookup433: pallet_jobs::module::Call<T>
   **/
  PalletJobsModuleCall: {
    _enum: {
      submit_job: {
        job: 'TanglePrimitivesJobsJobSubmission',
      },
      submit_job_result: {
        roleType: 'TanglePrimitivesRolesRoleType',
        jobId: 'u64',
        result: 'TanglePrimitivesJobsJobResult',
      },
      withdraw_rewards: 'Null',
      report_inactive_validator: {
        roleType: 'TanglePrimitivesRolesRoleType',
        jobId: 'u64',
        validator: 'AccountId32',
        offence: 'TanglePrimitivesJobsValidatorOffenceType',
        signatures: 'Vec<Bytes>',
      },
      set_permitted_caller: {
        roleType: 'TanglePrimitivesRolesRoleType',
        jobId: 'u64',
        newPermittedCaller: 'AccountId32',
      },
      set_time_fee: {
        newFee: 'u128',
      },
      submit_misbehavior: {
        misbehavior: 'TanglePrimitivesMisbehaviorMisbehaviorSubmission',
      },
      extend_job_result_ttl: {
        roleType: 'TanglePrimitivesRolesRoleType',
        jobId: 'u64',
        extendBy: 'u64'
      }
    }
  },
  /**
   * Lookup434: tangle_primitives::jobs::JobResult<tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxKeyLen, tangle_testnet_runtime::MaxSignatureLen, tangle_testnet_runtime::MaxDataLen, tangle_testnet_runtime::MaxProofLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsJobResult: {
    _enum: {
      DKGPhaseOne: 'TanglePrimitivesJobsTssDkgtssKeySubmissionResult',
      DKGPhaseTwo: 'TanglePrimitivesJobsTssDkgtssSignatureResult',
      DKGPhaseThree: 'TanglePrimitivesJobsTssDkgtssKeyRefreshResult',
      DKGPhaseFour: 'TanglePrimitivesJobsTssDkgtssKeyRotationResult',
      ZkSaaSPhaseOne: 'TanglePrimitivesJobsZksaasZkSaaSCircuitResult',
      ZkSaaSPhaseTwo: 'TanglePrimitivesJobsZksaasZkSaaSProofResult'
    }
  },
  /**
   * Lookup435: tangle_testnet_runtime::MaxKeyLen
   **/
  TangleTestnetRuntimeMaxKeyLen: 'Null',
  /**
   * Lookup436: tangle_testnet_runtime::MaxSignatureLen
   **/
  TangleTestnetRuntimeMaxSignatureLen: 'Null',
  /**
   * Lookup437: tangle_testnet_runtime::MaxDataLen
   **/
  TangleTestnetRuntimeMaxDataLen: 'Null',
  /**
   * Lookup438: tangle_testnet_runtime::MaxProofLen
   **/
  TangleTestnetRuntimeMaxProofLen: 'Null',
  /**
   * Lookup439: tangle_primitives::jobs::tss::DKGTSSKeySubmissionResult<tangle_testnet_runtime::MaxKeyLen, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxSignatureLen>
   **/
  TanglePrimitivesJobsTssDkgtssKeySubmissionResult: {
    signatureScheme: 'TanglePrimitivesJobsTssDigitalSignatureScheme',
    key: 'Bytes',
    chainCode: 'Option<[u8;32]>',
    participants: 'Vec<Bytes>',
    signatures: 'Vec<Bytes>',
    threshold: 'u8'
  },
  /**
   * Lookup440: tangle_primitives::jobs::tss::DigitalSignatureScheme
   **/
  TanglePrimitivesJobsTssDigitalSignatureScheme: {
    _enum: ['EcdsaSecp256k1', 'EcdsaSecp256r1', 'EcdsaStark', 'SchnorrP256', 'SchnorrP384', 'SchnorrSecp256k1', 'SchnorrSr25519', 'SchnorrRistretto255', 'SchnorrEd25519', 'SchnorrEd448', 'SchnorrTaproot', 'Bls381']
  },
  /**
   * Lookup447: tangle_primitives::jobs::tss::DKGTSSSignatureResult<tangle_testnet_runtime::MaxDataLen, tangle_testnet_runtime::MaxKeyLen, tangle_testnet_runtime::MaxSignatureLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsTssDkgtssSignatureResult: {
    signatureScheme: 'TanglePrimitivesJobsTssDigitalSignatureScheme',
    data: 'Bytes',
    signature: 'Bytes',
    verifyingKey: 'Bytes',
    derivationPath: 'Option<Bytes>',
    chainCode: 'Option<[u8;32]>'
  },
  /**
   * Lookup449: tangle_primitives::jobs::tss::DKGTSSKeyRefreshResult
   **/
  TanglePrimitivesJobsTssDkgtssKeyRefreshResult: {
    signatureScheme: 'TanglePrimitivesJobsTssDigitalSignatureScheme'
  },
  /**
   * Lookup450: tangle_primitives::jobs::tss::DKGTSSKeyRotationResult<tangle_testnet_runtime::MaxKeyLen, tangle_testnet_runtime::MaxSignatureLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsTssDkgtssKeyRotationResult: {
    phaseOneId: 'u64',
    newPhaseOneId: 'u64',
    newKey: 'Bytes',
    key: 'Bytes',
    signature: 'Bytes',
    signatureScheme: 'TanglePrimitivesJobsTssDigitalSignatureScheme',
    derivationPath: 'Option<Bytes>',
    chainCode: 'Option<[u8;32]>'
  },
  /**
   * Lookup451: tangle_primitives::jobs::zksaas::ZkSaaSCircuitResult<tangle_testnet_runtime::MaxParticipants>
   **/
  TanglePrimitivesJobsZksaasZkSaaSCircuitResult: {
    jobId: 'u64',
    participants: 'Vec<SpCoreEcdsaPublic>'
  },
  /**
   * Lookup454: tangle_primitives::jobs::zksaas::ZkSaaSProofResult<tangle_testnet_runtime::MaxProofLen>
   **/
  TanglePrimitivesJobsZksaasZkSaaSProofResult: {
    _enum: {
      Arkworks: 'TanglePrimitivesJobsZksaasArkworksProofResult',
      Circom: 'TanglePrimitivesJobsZksaasCircomProofResult'
    }
  },
  /**
   * Lookup455: tangle_primitives::jobs::zksaas::ArkworksProofResult<tangle_testnet_runtime::MaxProofLen>
   **/
  TanglePrimitivesJobsZksaasArkworksProofResult: {
    proof: 'Bytes'
  },
  /**
   * Lookup457: tangle_primitives::jobs::zksaas::CircomProofResult<tangle_testnet_runtime::MaxProofLen>
   **/
  TanglePrimitivesJobsZksaasCircomProofResult: {
    proof: 'Bytes'
  },
  /**
   * Lookup458: tangle_primitives::jobs::ValidatorOffenceType
   **/
  TanglePrimitivesJobsValidatorOffenceType: {
    _enum: ['Inactivity', 'InvalidSignatureSubmitted', 'RejectedValidAction', 'ApprovedInvalidAction']
  },
  /**
   * Lookup459: tangle_primitives::misbehavior::MisbehaviorSubmission
   **/
  TanglePrimitivesMisbehaviorMisbehaviorSubmission: {
    roleType: 'TanglePrimitivesRolesRoleType',
    offender: '[u8;33]',
    jobId: 'u64',
    justification: 'TanglePrimitivesMisbehaviorMisbehaviorJustification'
  },
  /**
   * Lookup460: tangle_primitives::misbehavior::MisbehaviorJustification
   **/
  TanglePrimitivesMisbehaviorMisbehaviorJustification: {
    _enum: {
      DKGTSS: 'TanglePrimitivesMisbehaviorDkgtssJustification',
      ZkSaaS: 'TanglePrimitivesMisbehaviorZkSaaSJustification'
    }
  },
  /**
   * Lookup461: tangle_primitives::misbehavior::DKGTSSJustification
   **/
  TanglePrimitivesMisbehaviorDkgtssJustification: {
    _enum: {
      DfnsCGGMP21: 'TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification',
      ZCashFrost: 'TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification'
    }
  },
  /**
   * Lookup462: tangle_primitives::misbehavior::dfns_cggmp21::DfnsCGGMP21Justification
   **/
  TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification: {
    _enum: {
      Keygen: {
        participants: 'Vec<[u8;33]>',
        t: 'u16',
        reason: 'TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted',
      },
      KeyRefresh: {
        participants: 'Vec<[u8;33]>',
        t: 'u16',
        reason: 'TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted',
      },
      Signing: {
        participants: 'Vec<[u8;33]>',
        t: 'u16',
        reason: 'TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted'
      }
    }
  },
  /**
   * Lookup464: tangle_primitives::misbehavior::dfns_cggmp21::KeygenAborted
   **/
  TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted: {
    _enum: {
      InvalidDecommitment: {
        round1: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
        round2a: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidSchnorrProof: {
        round2a: 'Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>',
        round3: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      FeldmanVerificationFailed: {
        round2a: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
        round2b: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidDataSize: {
        round2a: 'TanglePrimitivesMisbehaviorSignedRoundMessage'
      }
    }
  },
  /**
   * Lookup465: tangle_primitives::misbehavior::SignedRoundMessage
   **/
  TanglePrimitivesMisbehaviorSignedRoundMessage: {
    sender: 'u16',
    message: 'Bytes',
    signature: 'Bytes'
  },
  /**
   * Lookup467: tangle_primitives::misbehavior::dfns_cggmp21::KeyRefreshAborted
   **/
  TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted: {
    _enum: {
      InvalidDecommitment: {
        round1: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
        round2: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidSchnorrProof: 'Null',
      InvalidModProof: {
        reason: 'TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason',
        round2: 'Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>',
        round3: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidFacProof: 'Null',
      InvalidRingPedersenParameters: {
        round2: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidX: 'Null',
      InvalidXShare: 'Null',
      InvalidDataSize: 'Null',
      PaillierDec: 'Null'
    }
  },
  /**
   * Lookup468: tangle_primitives::misbehavior::dfns_cggmp21::InvalidProofReason
   **/
  TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason: {
    _enum: {
      EqualityCheck: 'u8',
      RangeCheck: 'u8',
      Encryption: 'Null',
      PaillierEnc: 'Null',
      PaillierOp: 'Null',
      ModPow: 'Null',
      ModulusIsPrime: 'Null',
      ModulusIsEven: 'Null',
      IncorrectNthRoot: 'u8',
      IncorrectFourthRoot: 'u8'
    }
  },
  /**
   * Lookup469: tangle_primitives::misbehavior::dfns_cggmp21::SigningAborted
   **/
  TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted: {
    _enum: ['EncProofOfK', 'InvalidPsi', 'InvalidPsiPrimePrime', 'MismatchedDelta']
  },
  /**
   * Lookup470: tangle_primitives::misbehavior::zcash_frost::ZCashFrostJustification
   **/
  TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification: {
    _enum: {
      Keygen: {
        participants: 'Vec<[u8;33]>',
        t: 'u16',
        reason: 'TanglePrimitivesMisbehaviorZcashFrostKeygenAborted',
      },
      Signing: {
        participants: 'Vec<[u8;33]>',
        t: 'u16',
        reason: 'TanglePrimitivesMisbehaviorZcashFrostSigningAborted'
      }
    }
  },
  /**
   * Lookup471: tangle_primitives::misbehavior::zcash_frost::KeygenAborted
   **/
  TanglePrimitivesMisbehaviorZcashFrostKeygenAborted: {
    _enum: {
      InvalidProofOfKnowledge: {
        round1: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
      },
      InvalidSecretShare: {
        round1: 'TanglePrimitivesMisbehaviorSignedRoundMessage',
        round2: 'TanglePrimitivesMisbehaviorSignedRoundMessage'
      }
    }
  },
  /**
   * Lookup472: tangle_primitives::misbehavior::zcash_frost::SigningAborted
   **/
  TanglePrimitivesMisbehaviorZcashFrostSigningAborted: {
    _enum: {
      InvalidSignatureShare: {
        round1: 'Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>',
        round2: 'Vec<TanglePrimitivesMisbehaviorSignedRoundMessage>'
      }
    }
  },
  /**
   * Lookup473: tangle_primitives::misbehavior::ZkSaaSJustification
   **/
  TanglePrimitivesMisbehaviorZkSaaSJustification: 'Null',
  /**
   * Lookup474: pallet_dkg::pallet::Call<T>
   **/
  PalletDkgCall: {
    _enum: {
      set_fee: {
        feeInfo: 'PalletDkgFeeInfo'
      }
    }
  },
  /**
   * Lookup475: pallet_zksaas::pallet::Call<T>
   **/
  PalletZksaasCall: {
    _enum: {
      set_fee: {
        feeInfo: 'PalletZksaasFeeInfo'
      }
    }
  },
  /**
   * Lookup476: pallet_proxy::pallet::Call<T>
   **/
  PalletProxyCall: {
    _enum: {
      proxy: {
        real: 'MultiAddress',
        forceProxyType: 'Option<TangleTestnetRuntimeProxyType>',
        call: 'Call',
      },
      add_proxy: {
        delegate: 'MultiAddress',
        proxyType: 'TangleTestnetRuntimeProxyType',
        delay: 'u64',
      },
      remove_proxy: {
        delegate: 'MultiAddress',
        proxyType: 'TangleTestnetRuntimeProxyType',
        delay: 'u64',
      },
      remove_proxies: 'Null',
      create_pure: {
        proxyType: 'TangleTestnetRuntimeProxyType',
        delay: 'u64',
        index: 'u16',
      },
      kill_pure: {
        spawner: 'MultiAddress',
        proxyType: 'TangleTestnetRuntimeProxyType',
        index: 'u16',
        height: 'Compact<u64>',
        extIndex: 'Compact<u32>',
      },
      announce: {
        real: 'MultiAddress',
        callHash: 'H256',
      },
      remove_announcement: {
        real: 'MultiAddress',
        callHash: 'H256',
      },
      reject_announcement: {
        delegate: 'MultiAddress',
        callHash: 'H256',
      },
      proxy_announced: {
        delegate: 'MultiAddress',
        real: 'MultiAddress',
        forceProxyType: 'Option<TangleTestnetRuntimeProxyType>',
        call: 'Call'
      }
    }
  },
  /**
   * Lookup478: pallet_multi_asset_delegation::pallet::Call<T>
   **/
  PalletMultiAssetDelegationCall: {
    _enum: {
      join_operators: {
        bondAmount: 'u128',
      },
      schedule_leave_operators: 'Null',
      cancel_leave_operators: 'Null',
      execute_leave_operators: 'Null',
      operator_bond_more: {
        additionalBond: 'u128',
      },
      schedule_operator_bond_less: {
        bondLessAmount: 'u128',
      },
      execute_operator_bond_less: 'Null',
      cancel_operator_bond_less: 'Null',
      go_offline: 'Null',
      go_online: 'Null',
      deposit: {
        assetId: 'Option<u128>',
        amount: 'u128',
      },
      schedule_unstake: {
        assetId: 'Option<u128>',
        amount: 'u128',
      },
      execute_unstake: 'Null',
      cancel_unstake: 'Null',
      delegate: {
        operator: 'AccountId32',
        assetId: 'u128',
        amount: 'u128',
      },
      schedule_delegator_bond_less: {
        operator: 'AccountId32',
        assetId: 'u128',
        amount: 'u128',
      },
      execute_delegator_bond_less: 'Null',
      cancel_delegator_bond_less: 'Null',
      set_whitelisted_assets: {
        assets: 'Vec<u128>',
      },
      set_incentive_apy_and_cap: {
        assetId: 'u128',
        apy: 'u128',
        cap: 'u128',
      },
      whitelist_blueprint_for_rewards: {
        blueprintId: 'u32'
      }
    }
  },
  /**
   * Lookup479: sygma_access_segregator::pallet::Call<T>
   **/
  SygmaAccessSegregatorCall: {
    _enum: {
      grant_access: {
        palletIndex: 'u8',
        extrinsicName: 'Bytes',
        who: 'AccountId32'
      }
    }
  },
  /**
   * Lookup480: sygma_basic_feehandler::pallet::Call<T>
   **/
  SygmaBasicFeehandlerCall: {
    _enum: {
      set_fee: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        amount: 'u128'
      }
    }
  },
  /**
   * Lookup481: sygma_fee_handler_router::pallet::Call<T>
   **/
  SygmaFeeHandlerRouterCall: {
    _enum: {
      set_fee_handler: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        handlerType: 'SygmaFeeHandlerRouterFeeHandlerType'
      }
    }
  },
  /**
   * Lookup482: sygma_percentage_feehandler::pallet::Call<T>
   **/
  SygmaPercentageFeehandlerCall: {
    _enum: {
      set_fee_rate: {
        domain: 'u8',
        asset: 'StagingXcmV4AssetAssetId',
        feeRateBasisPoint: 'u32',
        feeLowerBound: 'u128',
        feeUpperBound: 'u128'
      }
    }
  },
  /**
   * Lookup483: sygma_bridge::pallet::Call<T>
   **/
  SygmaBridgeCall: {
    _enum: {
      pause_bridge: {
        destDomainId: 'u8',
      },
      unpause_bridge: {
        destDomainId: 'u8',
      },
      set_mpc_address: {
        addr: 'SygmaTraitsMpcAddress',
      },
      register_domain: {
        destDomainId: 'u8',
        destChainId: 'U256',
      },
      unregister_domain: {
        destDomainId: 'u8',
        destChainId: 'U256',
      },
      deposit: {
        asset: 'StagingXcmV4Asset',
        dest: 'StagingXcmV4Location',
      },
      retry: {
        depositOnBlockHeight: 'u128',
        destDomainId: 'u8',
      },
      execute_proposal: {
        proposals: 'Vec<SygmaBridgeProposal>',
        signature: 'Bytes',
      },
      pause_all_bridges: 'Null',
      unpause_all_bridges: 'Null'
    }
  },
  /**
   * Lookup484: sygma_traits::MpcAddress
   **/
  SygmaTraitsMpcAddress: '[u8;20]',
  /**
   * Lookup485: staging_xcm::v4::asset::Asset
   **/
  StagingXcmV4Asset: {
    id: 'StagingXcmV4AssetAssetId',
    fun: 'StagingXcmV4AssetFungibility'
  },
  /**
   * Lookup486: staging_xcm::v4::asset::Fungibility
   **/
  StagingXcmV4AssetFungibility: {
    _enum: {
      Fungible: 'Compact<u128>',
      NonFungible: 'StagingXcmV4AssetAssetInstance'
    }
  },
  /**
   * Lookup487: staging_xcm::v4::asset::AssetInstance
   **/
  StagingXcmV4AssetAssetInstance: {
    _enum: {
      Undefined: 'Null',
      Index: 'Compact<u128>',
      Array4: '[u8;4]',
      Array8: '[u8;8]',
      Array16: '[u8;16]',
      Array32: '[u8;32]'
    }
  },
  /**
   * Lookup489: sygma_bridge::pallet::Proposal
   **/
  SygmaBridgeProposal: {
    originDomainId: 'u8',
    depositNonce: 'u64',
    resourceId: '[u8;32]',
    data: 'Bytes'
  },
  /**
   * Lookup490: pallet_sudo::pallet::Error<T>
   **/
  PalletSudoError: {
    _enum: ['RequireSudo']
  },
  /**
   * Lookup492: pallet_assets::types::AssetDetails<Balance, sp_core::crypto::AccountId32, DepositBalance>
   **/
  PalletAssetsAssetDetails: {
    owner: 'AccountId32',
    issuer: 'AccountId32',
    admin: 'AccountId32',
    freezer: 'AccountId32',
    supply: 'u128',
    deposit: 'u128',
    minBalance: 'u128',
    isSufficient: 'bool',
    accounts: 'u32',
    sufficients: 'u32',
    approvals: 'u32',
    status: 'PalletAssetsAssetStatus'
  },
  /**
   * Lookup493: pallet_assets::types::AssetStatus
   **/
  PalletAssetsAssetStatus: {
    _enum: ['Live', 'Frozen', 'Destroying']
  },
  /**
   * Lookup495: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra, sp_core::crypto::AccountId32>
   **/
  PalletAssetsAssetAccount: {
    balance: 'u128',
    status: 'PalletAssetsAccountStatus',
    reason: 'PalletAssetsExistenceReason',
    extra: 'Null'
  },
  /**
   * Lookup496: pallet_assets::types::AccountStatus
   **/
  PalletAssetsAccountStatus: {
    _enum: ['Liquid', 'Frozen', 'Blocked']
  },
  /**
   * Lookup497: pallet_assets::types::ExistenceReason<Balance, sp_core::crypto::AccountId32>
   **/
  PalletAssetsExistenceReason: {
    _enum: {
      Consumer: 'Null',
      Sufficient: 'Null',
      DepositHeld: 'u128',
      DepositRefunded: 'Null',
      DepositFrom: '(AccountId32,u128)'
    }
  },
  /**
   * Lookup499: pallet_assets::types::Approval<Balance, DepositBalance>
   **/
  PalletAssetsApproval: {
    amount: 'u128',
    deposit: 'u128'
  },
  /**
   * Lookup500: pallet_assets::types::AssetMetadata<DepositBalance, bounded_collections::bounded_vec::BoundedVec<T, S>>
   **/
  PalletAssetsAssetMetadata: {
    deposit: 'u128',
    name: 'Bytes',
    symbol: 'Bytes',
    decimals: 'u8',
    isFrozen: 'bool'
  },
  /**
   * Lookup502: pallet_assets::pallet::Error<T, I>
   **/
  PalletAssetsError: {
    _enum: ['BalanceLow', 'NoAccount', 'NoPermission', 'Unknown', 'Frozen', 'InUse', 'BadWitness', 'MinBalanceZero', 'UnavailableConsumer', 'BadMetadata', 'Unapproved', 'WouldDie', 'AlreadyExists', 'NoDeposit', 'WouldBurn', 'LiveAsset', 'AssetNotLive', 'IncorrectStatus', 'NotFrozen', 'CallbackFailed']
  },
  /**
   * Lookup504: pallet_balances::types::BalanceLock<Balance>
   **/
  PalletBalancesBalanceLock: {
    id: '[u8;8]',
    amount: 'u128',
    reasons: 'PalletBalancesReasons'
  },
  /**
   * Lookup505: pallet_balances::types::Reasons
   **/
  PalletBalancesReasons: {
    _enum: ['Fee', 'Misc', 'All']
  },
  /**
   * Lookup508: pallet_balances::types::ReserveData<ReserveIdentifier, Balance>
   **/
  PalletBalancesReserveData: {
    id: '[u8;8]',
    amount: 'u128'
  },
  /**
   * Lookup511: pallet_balances::types::IdAmount<tangle_testnet_runtime::RuntimeHoldReason, Balance>
   **/
  PalletBalancesIdAmountRuntimeHoldReason: {
    id: 'TangleTestnetRuntimeRuntimeHoldReason',
    amount: 'u128'
  },
  /**
   * Lookup512: tangle_testnet_runtime::RuntimeHoldReason
   **/
  TangleTestnetRuntimeRuntimeHoldReason: {
    _enum: {
      __Unused0: 'Null',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      __Unused4: 'Null',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      __Unused9: 'Null',
      __Unused10: 'Null',
      __Unused11: 'Null',
      __Unused12: 'Null',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      __Unused17: 'Null',
      __Unused18: 'Null',
      __Unused19: 'Null',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      __Unused23: 'Null',
      __Unused24: 'Null',
      Preimage: 'PalletPreimageHoldReason'
    }
  },
  /**
   * Lookup513: pallet_preimage::pallet::HoldReason
   **/
  PalletPreimageHoldReason: {
    _enum: ['Preimage']
  },
  /**
   * Lookup516: pallet_balances::types::IdAmount<tangle_testnet_runtime::RuntimeFreezeReason, Balance>
   **/
  PalletBalancesIdAmountRuntimeFreezeReason: {
    id: 'TangleTestnetRuntimeRuntimeFreezeReason',
    amount: 'u128'
  },
  /**
   * Lookup517: tangle_testnet_runtime::RuntimeFreezeReason
   **/
  TangleTestnetRuntimeRuntimeFreezeReason: {
    _enum: {
      __Unused0: 'Null',
      __Unused1: 'Null',
      __Unused2: 'Null',
      __Unused3: 'Null',
      __Unused4: 'Null',
      __Unused5: 'Null',
      __Unused6: 'Null',
      __Unused7: 'Null',
      __Unused8: 'Null',
      __Unused9: 'Null',
      __Unused10: 'Null',
      __Unused11: 'Null',
      __Unused12: 'Null',
      __Unused13: 'Null',
      __Unused14: 'Null',
      __Unused15: 'Null',
      __Unused16: 'Null',
      __Unused17: 'Null',
      __Unused18: 'Null',
      __Unused19: 'Null',
      __Unused20: 'Null',
      __Unused21: 'Null',
      __Unused22: 'Null',
      NominationPools: 'PalletNominationPoolsFreezeReason'
    }
  },
  /**
   * Lookup518: pallet_nomination_pools::pallet::FreezeReason
   **/
  PalletNominationPoolsFreezeReason: {
    _enum: ['PoolMinBalance']
  },
  /**
   * Lookup520: pallet_balances::pallet::Error<T, I>
   **/
  PalletBalancesError: {
    _enum: ['VestingBalance', 'LiquidityRestrictions', 'InsufficientBalance', 'ExistentialDeposit', 'Expendability', 'ExistingVestingSchedule', 'DeadAccount', 'TooManyReserves', 'TooManyHolds', 'TooManyFreezes', 'IssuanceDeactivated', 'DeltaZero']
  },
  /**
   * Lookup522: pallet_transaction_payment::Releases
   **/
  PalletTransactionPaymentReleases: {
    _enum: ['V1Ancient', 'V2']
  },
  /**
   * Lookup529: sp_consensus_babe::digests::PreDigest
   **/
  SpConsensusBabeDigestsPreDigest: {
    _enum: {
      __Unused0: 'Null',
      Primary: 'SpConsensusBabeDigestsPrimaryPreDigest',
      SecondaryPlain: 'SpConsensusBabeDigestsSecondaryPlainPreDigest',
      SecondaryVRF: 'SpConsensusBabeDigestsSecondaryVRFPreDigest'
    }
  },
  /**
   * Lookup530: sp_consensus_babe::digests::PrimaryPreDigest
   **/
  SpConsensusBabeDigestsPrimaryPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfSignature: 'SpCoreSr25519VrfVrfSignature'
  },
  /**
   * Lookup531: sp_core::sr25519::vrf::VrfSignature
   **/
  SpCoreSr25519VrfVrfSignature: {
    preOutput: '[u8;32]',
    proof: '[u8;64]'
  },
  /**
   * Lookup532: sp_consensus_babe::digests::SecondaryPlainPreDigest
   **/
  SpConsensusBabeDigestsSecondaryPlainPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64'
  },
  /**
   * Lookup533: sp_consensus_babe::digests::SecondaryVRFPreDigest
   **/
  SpConsensusBabeDigestsSecondaryVRFPreDigest: {
    authorityIndex: 'u32',
    slot: 'u64',
    vrfSignature: 'SpCoreSr25519VrfVrfSignature'
  },
  /**
   * Lookup534: sp_consensus_babe::BabeEpochConfiguration
   **/
  SpConsensusBabeBabeEpochConfiguration: {
    c: '(u64,u64)',
    allowedSlots: 'SpConsensusBabeAllowedSlots'
  },
  /**
   * Lookup536: pallet_babe::pallet::Error<T>
   **/
  PalletBabeError: {
    _enum: ['InvalidEquivocationProof', 'InvalidKeyOwnershipProof', 'DuplicateOffenceReport', 'InvalidConfiguration']
  },
  /**
   * Lookup537: pallet_grandpa::StoredState<N>
   **/
  PalletGrandpaStoredState: {
    _enum: {
      Live: 'Null',
      PendingPause: {
        scheduledAt: 'u64',
        delay: 'u64',
      },
      Paused: 'Null',
      PendingResume: {
        scheduledAt: 'u64',
        delay: 'u64'
      }
    }
  },
  /**
   * Lookup538: pallet_grandpa::StoredPendingChange<N, Limit>
   **/
  PalletGrandpaStoredPendingChange: {
    scheduledAt: 'u64',
    delay: 'u64',
    nextAuthorities: 'Vec<(SpConsensusGrandpaAppPublic,u64)>',
    forced: 'Option<u64>'
  },
  /**
   * Lookup540: pallet_grandpa::pallet::Error<T>
   **/
  PalletGrandpaError: {
    _enum: ['PauseFailed', 'ResumeFailed', 'ChangePending', 'TooSoon', 'InvalidKeyOwnershipProof', 'InvalidEquivocationProof', 'DuplicateOffenceReport']
  },
  /**
   * Lookup542: pallet_indices::pallet::Error<T>
   **/
  PalletIndicesError: {
    _enum: ['NotAssigned', 'NotOwner', 'InUse', 'NotTransfer', 'Permanent']
  },
  /**
   * Lookup547: pallet_democracy::types::ReferendumInfo<BlockNumber, frame_support::traits::preimages::Bounded<tangle_testnet_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
   **/
  PalletDemocracyReferendumInfo: {
    _enum: {
      Ongoing: 'PalletDemocracyReferendumStatus',
      Finished: {
        approved: 'bool',
        end: 'u64'
      }
    }
  },
  /**
   * Lookup548: pallet_democracy::types::ReferendumStatus<BlockNumber, frame_support::traits::preimages::Bounded<tangle_testnet_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
   **/
  PalletDemocracyReferendumStatus: {
    end: 'u64',
    proposal: 'FrameSupportPreimagesBounded',
    threshold: 'PalletDemocracyVoteThreshold',
    delay: 'u64',
    tally: 'PalletDemocracyTally'
  },
  /**
   * Lookup549: pallet_democracy::types::Tally<Balance>
   **/
  PalletDemocracyTally: {
    ayes: 'u128',
    nays: 'u128',
    turnout: 'u128'
  },
  /**
   * Lookup550: pallet_democracy::vote::Voting<Balance, sp_core::crypto::AccountId32, BlockNumber, MaxVotes>
   **/
  PalletDemocracyVoteVoting: {
    _enum: {
      Direct: {
        votes: 'Vec<(u32,PalletDemocracyVoteAccountVote)>',
        delegations: 'PalletDemocracyDelegations',
        prior: 'PalletDemocracyVotePriorLock',
      },
      Delegating: {
        balance: 'u128',
        target: 'AccountId32',
        conviction: 'PalletDemocracyConviction',
        delegations: 'PalletDemocracyDelegations',
        prior: 'PalletDemocracyVotePriorLock'
      }
    }
  },
  /**
   * Lookup554: pallet_democracy::types::Delegations<Balance>
   **/
  PalletDemocracyDelegations: {
    votes: 'u128',
    capital: 'u128'
  },
  /**
   * Lookup555: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
   **/
  PalletDemocracyVotePriorLock: '(u64,u128)',
  /**
   * Lookup558: pallet_democracy::pallet::Error<T>
   **/
  PalletDemocracyError: {
    _enum: ['ValueLow', 'ProposalMissing', 'AlreadyCanceled', 'DuplicateProposal', 'ProposalBlacklisted', 'NotSimpleMajority', 'InvalidHash', 'NoProposal', 'AlreadyVetoed', 'ReferendumInvalid', 'NoneWaiting', 'NotVoter', 'NoPermission', 'AlreadyDelegating', 'InsufficientFunds', 'NotDelegating', 'VotesExist', 'InstantNotAllowed', 'Nonsense', 'WrongUpperBound', 'MaxVotesReached', 'TooMany', 'VotingPeriodLow', 'PreimageNotExist']
  },
  /**
   * Lookup560: pallet_collective::Votes<sp_core::crypto::AccountId32, BlockNumber>
   **/
  PalletCollectiveVotes: {
    index: 'u32',
    threshold: 'u32',
    ayes: 'Vec<AccountId32>',
    nays: 'Vec<AccountId32>',
    end: 'u64'
  },
  /**
   * Lookup561: pallet_collective::pallet::Error<T, I>
   **/
  PalletCollectiveError: {
    _enum: ['NotMember', 'DuplicateProposal', 'ProposalMissing', 'WrongIndex', 'DuplicateVote', 'AlreadyInitialized', 'TooEarly', 'TooManyProposals', 'WrongProposalWeight', 'WrongProposalLength', 'PrimeAccountNotMember']
  },
  /**
   * Lookup564: pallet_vesting::Releases
   **/
  PalletVestingReleases: {
    _enum: ['V0', 'V1']
  },
  /**
   * Lookup565: pallet_vesting::pallet::Error<T>
   **/
  PalletVestingError: {
    _enum: ['NotVesting', 'AtMaxVestingSchedules', 'AmountLow', 'ScheduleIndexOutOfBounds', 'InvalidScheduleParams']
  },
  /**
   * Lookup567: pallet_elections_phragmen::SeatHolder<sp_core::crypto::AccountId32, Balance>
   **/
  PalletElectionsPhragmenSeatHolder: {
    who: 'AccountId32',
    stake: 'u128',
    deposit: 'u128'
  },
  /**
   * Lookup568: pallet_elections_phragmen::Voter<sp_core::crypto::AccountId32, Balance>
   **/
  PalletElectionsPhragmenVoter: {
    votes: 'Vec<AccountId32>',
    stake: 'u128',
    deposit: 'u128'
  },
  /**
   * Lookup569: pallet_elections_phragmen::pallet::Error<T>
   **/
  PalletElectionsPhragmenError: {
    _enum: ['UnableToVote', 'NoVotes', 'TooManyVotes', 'MaximumVotesExceeded', 'LowBalance', 'UnableToPayBond', 'MustBeVoter', 'DuplicatedCandidate', 'TooManyCandidates', 'MemberSubmit', 'RunnerUpSubmit', 'InsufficientCandidateFunds', 'NotMember', 'InvalidWitnessData', 'InvalidVoteCount', 'InvalidRenouncing', 'InvalidReplacement']
  },
  /**
   * Lookup570: pallet_election_provider_multi_phase::ReadySolution<AccountId, MaxWinners>
   **/
  PalletElectionProviderMultiPhaseReadySolution: {
    supports: 'Vec<(AccountId32,SpNposElectionsSupport)>',
    score: 'SpNposElectionsElectionScore',
    compute: 'PalletElectionProviderMultiPhaseElectionCompute'
  },
  /**
   * Lookup572: pallet_election_provider_multi_phase::RoundSnapshot<sp_core::crypto::AccountId32, DataProvider>
   **/
  PalletElectionProviderMultiPhaseRoundSnapshot: {
    voters: 'Vec<(AccountId32,u64,Vec<AccountId32>)>',
    targets: 'Vec<AccountId32>'
  },
  /**
   * Lookup579: pallet_election_provider_multi_phase::signed::SignedSubmission<sp_core::crypto::AccountId32, Balance, tangle_testnet_runtime::NposSolution16>
   **/
  PalletElectionProviderMultiPhaseSignedSignedSubmission: {
    who: 'AccountId32',
    deposit: 'u128',
    rawSolution: 'PalletElectionProviderMultiPhaseRawSolution',
    callFee: 'u128'
  },
  /**
   * Lookup580: pallet_election_provider_multi_phase::pallet::Error<T>
   **/
  PalletElectionProviderMultiPhaseError: {
    _enum: ['PreDispatchEarlySubmission', 'PreDispatchWrongWinnerCount', 'PreDispatchWeakSubmission', 'SignedQueueFull', 'SignedCannotPayDeposit', 'SignedInvalidWitness', 'SignedTooMuchWeight', 'OcwCallWrongEra', 'MissingSnapshotMetadata', 'InvalidSubmissionIndex', 'CallNotAllowed', 'FallbackFailed', 'BoundNotMet', 'TooManyWinners', 'PreDispatchDifferentRound']
  },
  /**
   * Lookup581: pallet_staking::StakingLedger<T>
   **/
  PalletStakingStakingLedger: {
    stash: 'AccountId32',
    total: 'Compact<u128>',
    active: 'Compact<u128>',
    unlocking: 'Vec<PalletStakingUnlockChunk>',
    legacyClaimedRewards: 'Vec<u32>'
  },
  /**
   * Lookup583: pallet_staking::UnlockChunk<Balance>
   **/
  PalletStakingUnlockChunk: {
    value: 'Compact<u128>',
    era: 'Compact<u32>'
  },
  /**
   * Lookup586: pallet_staking::Nominations<T>
   **/
  PalletStakingNominations: {
    targets: 'Vec<AccountId32>',
    submittedIn: 'u32',
    suppressed: 'bool'
  },
  /**
   * Lookup587: pallet_staking::ActiveEraInfo
   **/
  PalletStakingActiveEraInfo: {
    index: 'u32',
    start: 'Option<u64>'
  },
  /**
   * Lookup589: sp_staking::PagedExposureMetadata<Balance>
   **/
  SpStakingPagedExposureMetadata: {
    total: 'Compact<u128>',
    own: 'Compact<u128>',
    nominatorCount: 'u32',
    pageCount: 'u32'
  },
  /**
   * Lookup591: sp_staking::ExposurePage<sp_core::crypto::AccountId32, Balance>
   **/
  SpStakingExposurePage: {
    pageTotal: 'Compact<u128>',
    others: 'Vec<SpStakingIndividualExposure>'
  },
  /**
   * Lookup592: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
   **/
  PalletStakingEraRewardPoints: {
    total: 'u32',
    individual: 'BTreeMap<AccountId32, u32>'
  },
  /**
   * Lookup597: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
   **/
  PalletStakingUnappliedSlash: {
    validator: 'AccountId32',
    own: 'u128',
    others: 'Vec<(AccountId32,u128)>',
    reporters: 'Vec<AccountId32>',
    payout: 'u128'
  },
  /**
   * Lookup601: pallet_staking::slashing::SlashingSpans
   **/
  PalletStakingSlashingSlashingSpans: {
    spanIndex: 'u32',
    lastStart: 'u32',
    lastNonzeroSlash: 'u32',
    prior: 'Vec<u32>'
  },
  /**
   * Lookup602: pallet_staking::slashing::SpanRecord<Balance>
   **/
  PalletStakingSlashingSpanRecord: {
    slashed: 'u128',
    paidOut: 'u128'
  },
  /**
   * Lookup605: pallet_staking::pallet::pallet::Error<T>
   **/
  PalletStakingPalletError: {
    _enum: ['NotController', 'NotStash', 'AlreadyBonded', 'AlreadyPaired', 'EmptyTargets', 'DuplicateIndex', 'InvalidSlashIndex', 'InsufficientBond', 'NoMoreChunks', 'NoUnlockChunk', 'FundedTarget', 'InvalidEraToReward', 'InvalidNumberOfNominations', 'NotSortedAndUnique', 'AlreadyClaimed', 'InvalidPage', 'IncorrectHistoryDepth', 'IncorrectSlashingSpans', 'BadState', 'TooManyTargets', 'BadTarget', 'CannotChillOther', 'TooManyNominators', 'TooManyValidators', 'CommissionTooLow', 'BoundNotMet', 'ControllerDeprecated', 'RestakeActive']
  },
  /**
   * Lookup609: sp_core::crypto::KeyTypeId
   **/
  SpCoreCryptoKeyTypeId: '[u8;4]',
  /**
   * Lookup610: pallet_session::pallet::Error<T>
   **/
  PalletSessionError: {
    _enum: ['InvalidProof', 'NoAssociatedValidatorId', 'DuplicatedKey', 'NoKeys', 'NoAccount']
  },
  /**
   * Lookup612: pallet_treasury::Proposal<sp_core::crypto::AccountId32, Balance>
   **/
  PalletTreasuryProposal: {
    proposer: 'AccountId32',
    value: 'u128',
    beneficiary: 'AccountId32',
    bond: 'u128'
  },
  /**
   * Lookup614: pallet_treasury::SpendStatus<AssetKind, AssetBalance, sp_core::crypto::AccountId32, BlockNumber, PaymentId>
   **/
  PalletTreasurySpendStatus: {
    assetKind: 'Null',
    amount: 'u128',
    beneficiary: 'AccountId32',
    validFrom: 'u64',
    expireAt: 'u64',
    status: 'PalletTreasuryPaymentState'
  },
  /**
   * Lookup615: pallet_treasury::PaymentState<Id>
   **/
  PalletTreasuryPaymentState: {
    _enum: {
      Pending: 'Null',
      Attempted: {
        id: 'Null',
      },
      Failed: 'Null'
    }
  },
  /**
   * Lookup616: frame_support::PalletId
   **/
  FrameSupportPalletId: '[u8;8]',
  /**
   * Lookup617: pallet_treasury::pallet::Error<T, I>
   **/
  PalletTreasuryError: {
    _enum: ['InsufficientProposersBalance', 'InvalidIndex', 'TooManyApprovals', 'InsufficientPermission', 'ProposalNotApproved', 'FailedToConvertBalance', 'SpendExpired', 'EarlyPayout', 'AlreadyAttempted', 'PayoutError', 'NotAttempted', 'Inconclusive']
  },
  /**
   * Lookup618: pallet_bounties::Bounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
   **/
  PalletBountiesBounty: {
    proposer: 'AccountId32',
    value: 'u128',
    fee: 'u128',
    curatorDeposit: 'u128',
    bond: 'u128',
    status: 'PalletBountiesBountyStatus'
  },
  /**
   * Lookup619: pallet_bounties::BountyStatus<sp_core::crypto::AccountId32, BlockNumber>
   **/
  PalletBountiesBountyStatus: {
    _enum: {
      Proposed: 'Null',
      Approved: 'Null',
      Funded: 'Null',
      CuratorProposed: {
        curator: 'AccountId32',
      },
      Active: {
        curator: 'AccountId32',
        updateDue: 'u64',
      },
      PendingPayout: {
        curator: 'AccountId32',
        beneficiary: 'AccountId32',
        unlockAt: 'u64'
      }
    }
  },
  /**
   * Lookup621: pallet_bounties::pallet::Error<T, I>
   **/
  PalletBountiesError: {
    _enum: ['InsufficientProposersBalance', 'InvalidIndex', 'ReasonTooBig', 'UnexpectedStatus', 'RequireCurator', 'InvalidValue', 'InvalidFee', 'PendingPayout', 'Premature', 'HasActiveChildBounty', 'TooManyQueued']
  },
  /**
   * Lookup622: pallet_child_bounties::ChildBounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
   **/
  PalletChildBountiesChildBounty: {
    parentBounty: 'u32',
    value: 'u128',
    fee: 'u128',
    curatorDeposit: 'u128',
    status: 'PalletChildBountiesChildBountyStatus'
  },
  /**
   * Lookup623: pallet_child_bounties::ChildBountyStatus<sp_core::crypto::AccountId32, BlockNumber>
   **/
  PalletChildBountiesChildBountyStatus: {
    _enum: {
      Added: 'Null',
      CuratorProposed: {
        curator: 'AccountId32',
      },
      Active: {
        curator: 'AccountId32',
      },
      PendingPayout: {
        curator: 'AccountId32',
        beneficiary: 'AccountId32',
        unlockAt: 'u64'
      }
    }
  },
  /**
   * Lookup624: pallet_child_bounties::pallet::Error<T>
   **/
  PalletChildBountiesError: {
    _enum: ['ParentBountyNotActive', 'InsufficientBountyBalance', 'TooManyChildBounties']
  },
  /**
   * Lookup625: pallet_bags_list::list::Node<T, I>
   **/
  PalletBagsListListNode: {
    id: 'AccountId32',
    prev: 'Option<AccountId32>',
    next: 'Option<AccountId32>',
    bagUpper: 'u64',
    score: 'u64'
  },
  /**
   * Lookup626: pallet_bags_list::list::Bag<T, I>
   **/
  PalletBagsListListBag: {
    head: 'Option<AccountId32>',
    tail: 'Option<AccountId32>'
  },
  /**
   * Lookup628: pallet_bags_list::pallet::Error<T, I>
   **/
  PalletBagsListError: {
    _enum: {
      List: 'PalletBagsListListListError'
    }
  },
  /**
   * Lookup629: pallet_bags_list::list::ListError
   **/
  PalletBagsListListListError: {
    _enum: ['Duplicate', 'NotHeavier', 'NotInSameBag', 'NodeNotFound']
  },
  /**
   * Lookup630: pallet_nomination_pools::PoolMember<T>
   **/
  PalletNominationPoolsPoolMember: {
    poolId: 'u32',
    points: 'u128',
    lastRecordedRewardCounter: 'u128',
    unbondingEras: 'BTreeMap<u32, u128>'
  },
  /**
   * Lookup635: pallet_nomination_pools::BondedPoolInner<T>
   **/
  PalletNominationPoolsBondedPoolInner: {
    commission: 'PalletNominationPoolsCommission',
    memberCounter: 'u32',
    points: 'u128',
    roles: 'PalletNominationPoolsPoolRoles',
    state: 'PalletNominationPoolsPoolState'
  },
  /**
   * Lookup636: pallet_nomination_pools::Commission<T>
   **/
  PalletNominationPoolsCommission: {
    current: 'Option<(Perbill,AccountId32)>',
    max: 'Option<Perbill>',
    changeRate: 'Option<PalletNominationPoolsCommissionChangeRate>',
    throttleFrom: 'Option<u64>',
    claimPermission: 'Option<PalletNominationPoolsCommissionClaimPermission>'
  },
  /**
   * Lookup639: pallet_nomination_pools::PoolRoles<sp_core::crypto::AccountId32>
   **/
  PalletNominationPoolsPoolRoles: {
    depositor: 'AccountId32',
    root: 'Option<AccountId32>',
    nominator: 'Option<AccountId32>',
    bouncer: 'Option<AccountId32>'
  },
  /**
   * Lookup640: pallet_nomination_pools::RewardPool<T>
   **/
  PalletNominationPoolsRewardPool: {
    lastRecordedRewardCounter: 'u128',
    lastRecordedTotalPayouts: 'u128',
    totalRewardsClaimed: 'u128',
    totalCommissionPending: 'u128',
    totalCommissionClaimed: 'u128'
  },
  /**
   * Lookup641: pallet_nomination_pools::SubPools<T>
   **/
  PalletNominationPoolsSubPools: {
    noEra: 'PalletNominationPoolsUnbondPool',
    withEra: 'BTreeMap<u32, PalletNominationPoolsUnbondPool>'
  },
  /**
   * Lookup642: pallet_nomination_pools::UnbondPool<T>
   **/
  PalletNominationPoolsUnbondPool: {
    points: 'u128',
    balance: 'u128'
  },
  /**
   * Lookup647: pallet_nomination_pools::pallet::Error<T>
   **/
  PalletNominationPoolsError: {
    _enum: {
      PoolNotFound: 'Null',
      PoolMemberNotFound: 'Null',
      RewardPoolNotFound: 'Null',
      SubPoolsNotFound: 'Null',
      AccountBelongsToOtherPool: 'Null',
      FullyUnbonding: 'Null',
      MaxUnbondingLimit: 'Null',
      CannotWithdrawAny: 'Null',
      MinimumBondNotMet: 'Null',
      OverflowRisk: 'Null',
      NotDestroying: 'Null',
      NotNominator: 'Null',
      NotKickerOrDestroying: 'Null',
      NotOpen: 'Null',
      MaxPools: 'Null',
      MaxPoolMembers: 'Null',
      CanNotChangeState: 'Null',
      DoesNotHavePermission: 'Null',
      MetadataExceedsMaxLen: 'Null',
      Defensive: 'PalletNominationPoolsDefensiveError',
      PartialUnbondNotAllowedPermissionlessly: 'Null',
      MaxCommissionRestricted: 'Null',
      CommissionExceedsMaximum: 'Null',
      CommissionExceedsGlobalMaximum: 'Null',
      CommissionChangeThrottled: 'Null',
      CommissionChangeRateNotAllowed: 'Null',
      NoPendingCommission: 'Null',
      NoCommissionCurrentSet: 'Null',
      PoolIdInUse: 'Null',
      InvalidPoolId: 'Null',
      BondExtraRestricted: 'Null',
      NothingToAdjust: 'Null'
    }
  },
  /**
   * Lookup648: pallet_nomination_pools::pallet::DefensiveError
   **/
  PalletNominationPoolsDefensiveError: {
    _enum: ['NotEnoughSpaceInUnbondPool', 'PoolNotFound', 'RewardPoolNotFound', 'SubPoolsNotFound', 'BondedStashKilledPrematurely']
  },
  /**
   * Lookup651: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<tangle_testnet_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, BlockNumber, tangle_testnet_runtime::OriginCaller, sp_core::crypto::AccountId32>
   **/
  PalletSchedulerScheduled: {
    maybeId: 'Option<[u8;32]>',
    priority: 'u8',
    call: 'FrameSupportPreimagesBounded',
    maybePeriodic: 'Option<(u64,u32)>',
    origin: 'TangleTestnetRuntimeOriginCaller'
  },
  /**
   * Lookup653: pallet_scheduler::pallet::Error<T>
   **/
  PalletSchedulerError: {
    _enum: ['FailedToSchedule', 'NotFound', 'TargetBlockNumberInPast', 'RescheduleNoChange', 'Named']
  },
  /**
   * Lookup654: pallet_preimage::OldRequestStatus<sp_core::crypto::AccountId32, Balance>
   **/
  PalletPreimageOldRequestStatus: {
    _enum: {
      Unrequested: {
        deposit: '(AccountId32,u128)',
        len: 'u32',
      },
      Requested: {
        deposit: 'Option<(AccountId32,u128)>',
        count: 'u32',
        len: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup656: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, Ticket>
   **/
  PalletPreimageRequestStatus: {
    _enum: {
      Unrequested: {
        ticket: '(AccountId32,Null)',
        len: 'u32',
      },
      Requested: {
        maybeTicket: 'Option<(AccountId32,Null)>',
        count: 'u32',
        maybeLen: 'Option<u32>'
      }
    }
  },
  /**
   * Lookup660: pallet_preimage::pallet::Error<T>
   **/
  PalletPreimageError: {
    _enum: ['TooBig', 'AlreadyNoted', 'NotAuthorized', 'NotNoted', 'Requested', 'NotRequested', 'TooMany', 'TooFew']
  },
  /**
   * Lookup661: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
   **/
  SpStakingOffenceOffenceDetails: {
    offender: '(AccountId32,SpStakingExposure)',
    reporters: 'Vec<AccountId32>'
  },
  /**
   * Lookup663: pallet_tx_pause::pallet::Error<T>
   **/
  PalletTxPauseError: {
    _enum: ['IsPaused', 'IsUnpaused', 'Unpausable', 'NotFound']
  },
  /**
   * Lookup666: pallet_im_online::pallet::Error<T>
   **/
  PalletImOnlineError: {
    _enum: ['InvalidKey', 'DuplicatedHeartbeat']
  },
  /**
   * Lookup668: pallet_identity::types::Registration<Balance, MaxJudgements, pallet_identity::legacy::IdentityInfo<FieldLimit>>
   **/
  PalletIdentityRegistration: {
    judgements: 'Vec<(u32,PalletIdentityJudgement)>',
    deposit: 'u128',
    info: 'PalletIdentityLegacyIdentityInfo'
  },
  /**
   * Lookup677: pallet_identity::types::RegistrarInfo<Balance, sp_core::crypto::AccountId32, IdField>
   **/
  PalletIdentityRegistrarInfo: {
    account: 'AccountId32',
    fee: 'u128',
    fields: 'u64'
  },
  /**
   * Lookup679: pallet_identity::types::AuthorityProperties<bounded_collections::bounded_vec::BoundedVec<T, S>>
   **/
  PalletIdentityAuthorityProperties: {
    suffix: 'Bytes',
    allocation: 'u32'
  },
  /**
   * Lookup682: pallet_identity::pallet::Error<T>
   **/
  PalletIdentityError: {
    _enum: ['TooManySubAccounts', 'NotFound', 'NotNamed', 'EmptyIndex', 'FeeChanged', 'NoIdentity', 'StickyJudgement', 'JudgementGiven', 'InvalidJudgement', 'InvalidIndex', 'InvalidTarget', 'TooManyRegistrars', 'AlreadyClaimed', 'NotSub', 'NotOwned', 'JudgementForDifferentIdentity', 'JudgementPaymentFailed', 'InvalidSuffix', 'NotUsernameAuthority', 'NoAllocation', 'InvalidSignature', 'RequiresSignature', 'InvalidUsername', 'UsernameTaken', 'NoUsername', 'NotExpired']
  },
  /**
   * Lookup683: pallet_utility::pallet::Error<T>
   **/
  PalletUtilityError: {
    _enum: ['TooManyCalls']
  },
  /**
   * Lookup685: pallet_multisig::Multisig<BlockNumber, Balance, sp_core::crypto::AccountId32, MaxApprovals>
   **/
  PalletMultisigMultisig: {
    when: 'PalletMultisigTimepoint',
    deposit: 'u128',
    depositor: 'AccountId32',
    approvals: 'Vec<AccountId32>'
  },
  /**
   * Lookup686: pallet_multisig::pallet::Error<T>
   **/
  PalletMultisigError: {
    _enum: ['MinimumThreshold', 'AlreadyApproved', 'NoApprovalsNeeded', 'TooFewSignatories', 'TooManySignatories', 'SignatoriesOutOfOrder', 'SenderInSignatories', 'NotFound', 'NotOwner', 'NoTimepoint', 'WrongTimepoint', 'UnexpectedTimepoint', 'MaxWeightTooLow', 'AlreadyStored']
  },
  /**
   * Lookup689: fp_rpc::TransactionStatus
   **/
  FpRpcTransactionStatus: {
    transactionHash: 'H256',
    transactionIndex: 'u32',
    from: 'H160',
    to: 'Option<H160>',
    contractAddress: 'Option<H160>',
    logs: 'Vec<EthereumLog>',
    logsBloom: 'EthbloomBloom'
  },
  /**
   * Lookup692: ethbloom::Bloom
   **/
  EthbloomBloom: '[u8;256]',
  /**
   * Lookup694: ethereum::receipt::ReceiptV3
   **/
  EthereumReceiptReceiptV3: {
    _enum: {
      Legacy: 'EthereumReceiptEip658ReceiptData',
      EIP2930: 'EthereumReceiptEip658ReceiptData',
      EIP1559: 'EthereumReceiptEip658ReceiptData'
    }
  },
  /**
   * Lookup695: ethereum::receipt::EIP658ReceiptData
   **/
  EthereumReceiptEip658ReceiptData: {
    statusCode: 'u8',
    usedGas: 'U256',
    logsBloom: 'EthbloomBloom',
    logs: 'Vec<EthereumLog>'
  },
  /**
   * Lookup696: ethereum::block::Block<ethereum::transaction::TransactionV2>
   **/
  EthereumBlock: {
    header: 'EthereumHeader',
    transactions: 'Vec<EthereumTransactionTransactionV2>',
    ommers: 'Vec<EthereumHeader>'
  },
  /**
   * Lookup697: ethereum::header::Header
   **/
  EthereumHeader: {
    parentHash: 'H256',
    ommersHash: 'H256',
    beneficiary: 'H160',
    stateRoot: 'H256',
    transactionsRoot: 'H256',
    receiptsRoot: 'H256',
    logsBloom: 'EthbloomBloom',
    difficulty: 'U256',
    number: 'U256',
    gasLimit: 'U256',
    gasUsed: 'U256',
    timestamp: 'u64',
    extraData: 'Bytes',
    mixHash: 'H256',
    nonce: 'EthereumTypesHashH64'
  },
  /**
   * Lookup698: ethereum_types::hash::H64
   **/
  EthereumTypesHashH64: '[u8;8]',
  /**
   * Lookup703: pallet_ethereum::pallet::Error<T>
   **/
  PalletEthereumError: {
    _enum: ['InvalidSignature', 'PreLogExists']
  },
  /**
   * Lookup704: pallet_evm::CodeMetadata
   **/
  PalletEvmCodeMetadata: {
    _alias: {
      size_: 'size',
      hash_: 'hash'
    },
    size_: 'u64',
    hash_: 'H256'
  },
  /**
   * Lookup706: pallet_evm::pallet::Error<T>
   **/
  PalletEvmError: {
    _enum: ['BalanceLow', 'FeeOverflow', 'PaymentOverflow', 'WithdrawFailed', 'GasPriceTooLow', 'InvalidNonce', 'GasLimitTooLow', 'GasLimitTooHigh', 'InvalidChainId', 'InvalidSignature', 'Reentrancy', 'TransactionMustComeFromEOA', 'Undefined']
  },
  /**
   * Lookup707: pallet_hotfix_sufficients::pallet::Error<T>
   **/
  PalletHotfixSufficientsError: {
    _enum: ['MaxAddressCountExceeded']
  },
  /**
   * Lookup709: pallet_airdrop_claims::pallet::Error<T>
   **/
  PalletAirdropClaimsError: {
    _enum: ['InvalidEthereumSignature', 'InvalidNativeSignature', 'InvalidNativeAccount', 'SignerHasNoClaim', 'SenderHasNoClaim', 'PotUnderflow', 'InvalidStatement', 'VestedBalanceExists']
  },
  /**
   * Lookup710: pallet_roles::types::RestakingLedger<T>
   **/
  PalletRolesRestakingLedger: {
    stash: 'AccountId32',
    total: 'Compact<u128>',
    profile: 'PalletRolesProfile',
    roles: 'BTreeMap<TanglePrimitivesRolesRoleType, PalletRolesProfileRecord>',
    roleKey: 'Bytes',
    unlocking: 'Vec<PalletRolesUnlockChunk>',
    claimedRewards: 'Vec<u32>',
    maxActiveServices: 'u32'
  },
  /**
   * Lookup716: pallet_roles::types::UnlockChunk<Balance>
   **/
  PalletRolesUnlockChunk: {
    value: 'Compact<u128>',
    era: 'Compact<u32>'
  },
  /**
   * Lookup720: pallet_roles::pallet::Error<T>
   **/
  PalletRolesError: {
    _enum: ['NotValidator', 'HasRoleAssigned', 'RoleNotAssigned', 'MaxRoles', 'RoleCannotBeRemoved', 'RestakingAmountCannotBeUpdated', 'ExceedsMaxRestakeValue', 'InsufficientRestakingBond', 'ProfileUpdateFailed', 'ProfileAlreadyExists', 'NoProfileFound', 'ProfileDeleteRequestFailed', 'SessionKeysNotProvided', 'KeySizeExceeded', 'CannotGetCurrentEra', 'InvalidEraToReward', 'BoundNotMet', 'AlreadyClaimed', 'NoMoreChunks']
  },
  /**
   * Lookup721: tangle_primitives::jobs::PhaseResult<sp_core::crypto::AccountId32, BlockNumber, tangle_testnet_runtime::MaxParticipants, tangle_testnet_runtime::MaxKeyLen, tangle_testnet_runtime::MaxDataLen, tangle_testnet_runtime::MaxSignatureLen, tangle_testnet_runtime::MaxSubmissionLen, tangle_testnet_runtime::MaxProofLen, tangle_testnet_runtime::MaxAdditionalParamsLen>
   **/
  TanglePrimitivesJobsPhaseResult: {
    owner: 'AccountId32',
    result: 'TanglePrimitivesJobsJobResult',
    ttl: 'u64',
    permittedCaller: 'Option<AccountId32>',
    jobType: 'TanglePrimitivesJobsJobType'
  },
  /**
   * Lookup723: pallet_jobs::module::Error<T>
   **/
  PalletJobsModuleError: {
    _enum: ['InvalidJobPhase', 'InvalidValidator', 'InvalidJobParams', 'PreviousResultNotFound', 'ResultExpired', 'JobAlreadyExpired', 'JobNotFound', 'PhaseOneResultNotFound', 'NoRewards', 'NotEnoughValidators', 'EmptyResult', 'EmptyJob', 'ValidatorRoleKeyNotFound', 'ResultNotExpectedType', 'NoPermission', 'TooManyParticipants', 'ExceedsMaxKeySize', 'TooManyJobsForValidator']
  },
  /**
   * Lookup724: pallet_dkg::pallet::Error<T>
   **/
  PalletDkgError: {
    _enum: ['CannotRetreiveSigner', 'NotEnoughSigners', 'InvalidSignatureData', 'NoParticipantsFound', 'NoSignaturesFound', 'InvalidJobType', 'DuplicateSignature', 'InvalidSignature', 'InvalidSignatureScheme', 'InvalidSignatureDeserialization', 'InvalidVerifyingKey', 'InvalidVerifyingKeyDeserialization', 'SigningKeyMismatch', 'InvalidParticipantPublicKey', 'InvalidBlsPublicKey', 'InvalidRoleType', 'InvalidJustification', 'MalformedRoundMessage', 'NotSignedByOffender', 'ValidDecommitment', 'ValidDataSize', 'ValidFeldmanVerification', 'ValidSchnorrProof', 'ValidRingPedersenParameters', 'ValidModProof', 'ValidFrostSignatureShare', 'InvalidFrostMessageSerialization', 'InvalidFrostMessageDeserialization', 'InvalidIdentifierDeserialization', 'ValidFrostSignature', 'UnknownIdentifier', 'DuplicateIdentifier', 'IncorrectNumberOfIdentifiers', 'IdentifierDerivationNotSupported', 'MalformedFrostSignature', 'InvalidFrostSignature', 'InvalidFrostSignatureShare', 'InvalidFrostSignatureScheme', 'MalformedFrostVerifyingKey', 'MalformedFrostSigningKey', 'MissingFrostCommitment', 'IdentityCommitment', 'FrostFieldError', 'FrostGroupError', 'FieldElementError', 'InvalidPublicKey', 'InvalidMessage', 'MalformedStarkSignature']
  },
  /**
   * Lookup725: pallet_zksaas::pallet::Error<T>
   **/
  PalletZksaasError: {
    _enum: ['InvalidJobType', 'InvalidProof', 'MalformedProof']
  },
  /**
   * Lookup728: pallet_proxy::ProxyDefinition<sp_core::crypto::AccountId32, tangle_testnet_runtime::ProxyType, BlockNumber>
   **/
  PalletProxyProxyDefinition: {
    delegate: 'AccountId32',
    proxyType: 'TangleTestnetRuntimeProxyType',
    delay: 'u64'
  },
  /**
   * Lookup732: pallet_proxy::Announcement<sp_core::crypto::AccountId32, primitive_types::H256, BlockNumber>
   **/
  PalletProxyAnnouncement: {
    real: 'AccountId32',
    callHash: 'H256',
    height: 'u64'
  },
  /**
   * Lookup734: pallet_proxy::pallet::Error<T>
   **/
  PalletProxyError: {
    _enum: ['TooMany', 'NotFound', 'NotProxy', 'Unproxyable', 'Duplicate', 'NoPermission', 'Unannounced', 'NoSelfProxy']
  },
  /**
   * Lookup735: pallet_multi_asset_delegation::types::operator::OperatorMetadata<sp_core::crypto::AccountId32, Balance, AssetId>
   **/
  PalletMultiAssetDelegationOperatorOperatorMetadata: {
    bond: 'u128',
    delegationCount: 'u32',
    request: 'Option<PalletMultiAssetDelegationOperatorOperatorBondLessRequest>',
    delegations: 'Vec<PalletMultiAssetDelegationOperatorDelegatorBond>',
    status: 'PalletMultiAssetDelegationOperatorOperatorStatus'
  },
  /**
   * Lookup737: pallet_multi_asset_delegation::types::operator::OperatorBondLessRequest<Balance>
   **/
  PalletMultiAssetDelegationOperatorOperatorBondLessRequest: {
    amount: 'u128',
    requestTime: 'u32'
  },
  /**
   * Lookup739: pallet_multi_asset_delegation::types::operator::DelegatorBond<sp_core::crypto::AccountId32, Balance, AssetId>
   **/
  PalletMultiAssetDelegationOperatorDelegatorBond: {
    delegator: 'AccountId32',
    amount: 'u128',
    assetId: 'u128'
  },
  /**
   * Lookup740: pallet_multi_asset_delegation::types::operator::OperatorStatus
   **/
  PalletMultiAssetDelegationOperatorOperatorStatus: {
    _enum: {
      Active: 'Null',
      Inactive: 'Null',
      Leaving: 'u32'
    }
  },
  /**
   * Lookup741: pallet_multi_asset_delegation::types::operator::OperatorSnapshot<sp_core::crypto::AccountId32, Balance, AssetId>
   **/
  PalletMultiAssetDelegationOperatorOperatorSnapshot: {
    bond: 'u128',
    delegations: 'Vec<PalletMultiAssetDelegationOperatorDelegatorBond>'
  },
  /**
   * Lookup742: pallet_multi_asset_delegation::types::delegator::DelegatorMetadata<sp_core::crypto::AccountId32, Balance, AssetId>
   **/
  PalletMultiAssetDelegationDelegatorDelegatorMetadata: {
    deposits: 'BTreeMap<u128, u128>',
    unstakeRequest: 'Option<PalletMultiAssetDelegationDelegatorUnstakeRequest>',
    delegations: 'Vec<PalletMultiAssetDelegationDelegatorBondInfoDelegator>',
    delegatorBondLessRequest: 'Option<PalletMultiAssetDelegationDelegatorBondLessRequest>',
    status: 'PalletMultiAssetDelegationDelegatorDelegatorStatus'
  },
  /**
   * Lookup747: pallet_multi_asset_delegation::types::delegator::UnstakeRequest<AssetId, Balance>
   **/
  PalletMultiAssetDelegationDelegatorUnstakeRequest: {
    assetId: 'u128',
    amount: 'u128',
    requestedRound: 'u32'
  },
  /**
   * Lookup749: pallet_multi_asset_delegation::types::delegator::BondInfoDelegator<sp_core::crypto::AccountId32, Balance, AssetId>
   **/
  PalletMultiAssetDelegationDelegatorBondInfoDelegator: {
    operator: 'AccountId32',
    amount: 'u128',
    assetId: 'u128'
  },
  /**
   * Lookup751: pallet_multi_asset_delegation::types::delegator::BondLessRequest<AssetId, Balance>
   **/
  PalletMultiAssetDelegationDelegatorBondLessRequest: {
    assetId: 'u128',
    amount: 'u128',
    requestedRound: 'u32'
  },
  /**
   * Lookup752: pallet_multi_asset_delegation::types::delegator::DelegatorStatus
   **/
  PalletMultiAssetDelegationDelegatorDelegatorStatus: {
    _enum: {
      Active: 'Null',
      LeavingScheduled: 'u32'
    }
  },
  /**
   * Lookup753: pallet_multi_asset_delegation::types::rewards::RewardConfig<AssetId, Balance>
   **/
  PalletMultiAssetDelegationRewardsRewardConfig: {
    configs: 'BTreeMap<u128, PalletMultiAssetDelegationRewardsRewardConfigForAsset>',
    whitelistedBlueprintIds: 'Vec<u32>'
  },
  /**
   * Lookup755: pallet_multi_asset_delegation::types::rewards::RewardConfigForAsset<Balance>
   **/
  PalletMultiAssetDelegationRewardsRewardConfigForAsset: {
    apy: 'u128',
    cap: 'u128'
  },
  /**
   * Lookup758: pallet_multi_asset_delegation::pallet::Error<T>
   **/
  PalletMultiAssetDelegationError: {
    _enum: ['AlreadyOperator', 'BondTooLow', 'NotAnOperator', 'CannotExit', 'AlreadyLeaving', 'NotLeavingOperator', 'NotLeavingRound', 'NoScheduledBondLess', 'BondLessRequestNotSatisfied', 'NotActiveOperator', 'NotOfflineOperator', 'AlreadyDelegator', 'NotDelegator', 'WithdrawRequestAlreadyExists', 'InsufficientBalance', 'NoWithdrawRequest', 'UnstakeNotReady', 'NoBondLessRequest', 'BondLessNotReady', 'BondLessRequestAlreadyExists', 'ActiveServicesUsingAsset', 'NoActiveDelegation', 'AssetNotWhitelisted', 'NotAuthorized', 'AssetNotFound', 'BlueprintAlreadyWhitelisted']
  },
  /**
   * Lookup760: sygma_access_segregator::pallet::Error<T>
   **/
  SygmaAccessSegregatorError: {
    _enum: ['Unimplemented', 'GrantAccessFailed']
  },
  /**
   * Lookup762: sygma_basic_feehandler::pallet::Error<T>
   **/
  SygmaBasicFeehandlerError: {
    _enum: ['Unimplemented', 'AccessDenied']
  },
  /**
   * Lookup763: sygma_fee_handler_router::pallet::Error<T>
   **/
  SygmaFeeHandlerRouterError: {
    _enum: ['AccessDenied', 'Unimplemented']
  },
  /**
   * Lookup765: sygma_percentage_feehandler::pallet::Error<T>
   **/
  SygmaPercentageFeehandlerError: {
    _enum: ['Unimplemented', 'AccessDenied', 'FeeRateOutOfRange', 'InvalidFeeBound']
  },
  /**
   * Lookup772: sygma_bridge::pallet::Error<T>
   **/
  SygmaBridgeError: {
    _enum: ['AccessDenied', 'BadMpcSignature', 'InsufficientBalance', 'TransactFailedDeposit', 'TransactFailedWithdraw', 'TransactFailedFeeDeposit', 'TransactFailedHoldInReserved', 'TransactFailedReleaseFromReserved', 'FeeTooExpensive', 'MissingMpcAddress', 'MpcAddrNotUpdatable', 'BridgePaused', 'BridgeUnpaused', 'MissingFeeConfig', 'AssetNotBound', 'ProposalAlreadyComplete', 'EmptyProposalList', 'TransactorFailed', 'InvalidDepositDataInvalidLength', 'InvalidDepositDataInvalidAmount', 'InvalidDepositDataInvalidRecipientLength', 'InvalidDepositDataRecipientLengthNotMatch', 'InvalidDepositDataInvalidRecipient', 'DestDomainNotSupported', 'DestChainIDNotMatch', 'ExtractDestDataFailed', 'DecimalConversionFail', 'DepositNonceOverflow', 'NoLiquidityHolderAccountBound', 'Unimplemented']
  },
  /**
   * Lookup775: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T>
   **/
  FrameSystemExtensionsCheckNonZeroSender: 'Null',
  /**
   * Lookup776: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
   **/
  FrameSystemExtensionsCheckSpecVersion: 'Null',
  /**
   * Lookup777: frame_system::extensions::check_tx_version::CheckTxVersion<T>
   **/
  FrameSystemExtensionsCheckTxVersion: 'Null',
  /**
   * Lookup778: frame_system::extensions::check_genesis::CheckGenesis<T>
   **/
  FrameSystemExtensionsCheckGenesis: 'Null',
  /**
   * Lookup781: frame_system::extensions::check_nonce::CheckNonce<T>
   **/
  FrameSystemExtensionsCheckNonce: 'Compact<u32>',
  /**
   * Lookup782: frame_system::extensions::check_weight::CheckWeight<T>
   **/
  FrameSystemExtensionsCheckWeight: 'Null',
  /**
   * Lookup783: pallet_transaction_payment::ChargeTransactionPayment<T>
   **/
  PalletTransactionPaymentChargeTransactionPayment: 'Compact<u128>',
  /**
   * Lookup784: frame_metadata_hash_extension::CheckMetadataHash<T>
   **/
  FrameMetadataHashExtensionCheckMetadataHash: {
    mode: 'FrameMetadataHashExtensionMode'
  },
  /**
   * Lookup785: frame_metadata_hash_extension::Mode
   **/
  FrameMetadataHashExtensionMode: {
    _enum: ['Disabled', 'Enabled']
  },
  /**
   * Lookup787: tangle_testnet_runtime::Runtime
   **/
  TangleTestnetRuntimeRuntime: 'Null'
};
