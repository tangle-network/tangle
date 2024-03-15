declare const _default: {
    /**
     * Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>>
     **/
    FrameSystemAccountInfo: {
        nonce: string;
        consumers: string;
        providers: string;
        sufficients: string;
        data: string;
    };
    /**
     * Lookup5: pallet_balances::types::AccountData<Balance>
     **/
    PalletBalancesAccountData: {
        free: string;
        reserved: string;
        frozen: string;
        flags: string;
    };
    /**
     * Lookup8: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
     **/
    FrameSupportDispatchPerDispatchClassWeight: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup9: sp_weights::weight_v2::Weight
     **/
    SpWeightsWeightV2Weight: {
        refTime: string;
        proofSize: string;
    };
    /**
     * Lookup14: sp_runtime::generic::digest::Digest
     **/
    SpRuntimeDigest: {
        logs: string;
    };
    /**
     * Lookup16: sp_runtime::generic::digest::DigestItem
     **/
    SpRuntimeDigestDigestItem: {
        _enum: {
            Other: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            Consensus: string;
            Seal: string;
            PreRuntime: string;
            __Unused7: string;
            RuntimeEnvironmentUpdated: string;
        };
    };
    /**
     * Lookup19: frame_system::EventRecord<tangle_runtime::RuntimeEvent, primitive_types::H256>
     **/
    FrameSystemEventRecord: {
        phase: string;
        event: string;
        topics: string;
    };
    /**
     * Lookup21: frame_system::pallet::Event<T>
     **/
    FrameSystemEvent: {
        _enum: {
            ExtrinsicSuccess: {
                dispatchInfo: string;
            };
            ExtrinsicFailed: {
                dispatchError: string;
                dispatchInfo: string;
            };
            CodeUpdated: string;
            NewAccount: {
                account: string;
            };
            KilledAccount: {
                account: string;
            };
            Remarked: {
                _alias: {
                    hash_: string;
                };
                sender: string;
                hash_: string;
            };
            UpgradeAuthorized: {
                codeHash: string;
                checkVersion: string;
            };
        };
    };
    /**
     * Lookup22: frame_support::dispatch::DispatchInfo
     **/
    FrameSupportDispatchDispatchInfo: {
        weight: string;
        class: string;
        paysFee: string;
    };
    /**
     * Lookup23: frame_support::dispatch::DispatchClass
     **/
    FrameSupportDispatchDispatchClass: {
        _enum: string[];
    };
    /**
     * Lookup24: frame_support::dispatch::Pays
     **/
    FrameSupportDispatchPays: {
        _enum: string[];
    };
    /**
     * Lookup25: sp_runtime::DispatchError
     **/
    SpRuntimeDispatchError: {
        _enum: {
            Other: string;
            CannotLookup: string;
            BadOrigin: string;
            Module: string;
            ConsumerRemaining: string;
            NoProviders: string;
            TooManyConsumers: string;
            Token: string;
            Arithmetic: string;
            Transactional: string;
            Exhausted: string;
            Corruption: string;
            Unavailable: string;
            RootNotAllowed: string;
        };
    };
    /**
     * Lookup26: sp_runtime::ModuleError
     **/
    SpRuntimeModuleError: {
        index: string;
        error: string;
    };
    /**
     * Lookup27: sp_runtime::TokenError
     **/
    SpRuntimeTokenError: {
        _enum: string[];
    };
    /**
     * Lookup28: sp_arithmetic::ArithmeticError
     **/
    SpArithmeticArithmeticError: {
        _enum: string[];
    };
    /**
     * Lookup29: sp_runtime::TransactionalError
     **/
    SpRuntimeTransactionalError: {
        _enum: string[];
    };
    /**
     * Lookup31: pallet_sudo::pallet::Event<T>
     **/
    PalletSudoEvent: {
        _enum: {
            Sudid: {
                sudoResult: string;
            };
            KeyChanged: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            KeyRemoved: string;
            SudoAsDone: {
                sudoResult: string;
            };
        };
    };
    /**
     * Lookup35: pallet_balances::pallet::Event<T, I>
     **/
    PalletBalancesEvent: {
        _enum: {
            Endowed: {
                account: string;
                freeBalance: string;
            };
            DustLost: {
                account: string;
                amount: string;
            };
            Transfer: {
                from: string;
                to: string;
                amount: string;
            };
            BalanceSet: {
                who: string;
                free: string;
            };
            Reserved: {
                who: string;
                amount: string;
            };
            Unreserved: {
                who: string;
                amount: string;
            };
            ReserveRepatriated: {
                from: string;
                to: string;
                amount: string;
                destinationStatus: string;
            };
            Deposit: {
                who: string;
                amount: string;
            };
            Withdraw: {
                who: string;
                amount: string;
            };
            Slashed: {
                who: string;
                amount: string;
            };
            Minted: {
                who: string;
                amount: string;
            };
            Burned: {
                who: string;
                amount: string;
            };
            Suspended: {
                who: string;
                amount: string;
            };
            Restored: {
                who: string;
                amount: string;
            };
            Upgraded: {
                who: string;
            };
            Issued: {
                amount: string;
            };
            Rescinded: {
                amount: string;
            };
            Locked: {
                who: string;
                amount: string;
            };
            Unlocked: {
                who: string;
                amount: string;
            };
            Frozen: {
                who: string;
                amount: string;
            };
            Thawed: {
                who: string;
                amount: string;
            };
            TotalIssuanceForced: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
        };
    };
    /**
     * Lookup36: frame_support::traits::tokens::misc::BalanceStatus
     **/
    FrameSupportTokensMiscBalanceStatus: {
        _enum: string[];
    };
    /**
     * Lookup37: pallet_transaction_payment::pallet::Event<T>
     **/
    PalletTransactionPaymentEvent: {
        _enum: {
            TransactionFeePaid: {
                who: string;
                actualFee: string;
                tip: string;
            };
        };
    };
    /**
     * Lookup38: pallet_grandpa::pallet::Event
     **/
    PalletGrandpaEvent: {
        _enum: {
            NewAuthorities: {
                authoritySet: string;
            };
            Paused: string;
            Resumed: string;
        };
    };
    /**
     * Lookup41: sp_consensus_grandpa::app::Public
     **/
    SpConsensusGrandpaAppPublic: string;
    /**
     * Lookup42: sp_core::ed25519::Public
     **/
    SpCoreEd25519Public: string;
    /**
     * Lookup43: pallet_indices::pallet::Event<T>
     **/
    PalletIndicesEvent: {
        _enum: {
            IndexAssigned: {
                who: string;
                index: string;
            };
            IndexFreed: {
                index: string;
            };
            IndexFrozen: {
                index: string;
                who: string;
            };
        };
    };
    /**
     * Lookup44: pallet_democracy::pallet::Event<T>
     **/
    PalletDemocracyEvent: {
        _enum: {
            Proposed: {
                proposalIndex: string;
                deposit: string;
            };
            Tabled: {
                proposalIndex: string;
                deposit: string;
            };
            ExternalTabled: string;
            Started: {
                refIndex: string;
                threshold: string;
            };
            Passed: {
                refIndex: string;
            };
            NotPassed: {
                refIndex: string;
            };
            Cancelled: {
                refIndex: string;
            };
            Delegated: {
                who: string;
                target: string;
            };
            Undelegated: {
                account: string;
            };
            Vetoed: {
                who: string;
                proposalHash: string;
                until: string;
            };
            Blacklisted: {
                proposalHash: string;
            };
            Voted: {
                voter: string;
                refIndex: string;
                vote: string;
            };
            Seconded: {
                seconder: string;
                propIndex: string;
            };
            ProposalCanceled: {
                propIndex: string;
            };
            MetadataSet: {
                _alias: {
                    hash_: string;
                };
                owner: string;
                hash_: string;
            };
            MetadataCleared: {
                _alias: {
                    hash_: string;
                };
                owner: string;
                hash_: string;
            };
            MetadataTransferred: {
                _alias: {
                    hash_: string;
                };
                prevOwner: string;
                owner: string;
                hash_: string;
            };
        };
    };
    /**
     * Lookup45: pallet_democracy::vote_threshold::VoteThreshold
     **/
    PalletDemocracyVoteThreshold: {
        _enum: string[];
    };
    /**
     * Lookup46: pallet_democracy::vote::AccountVote<Balance>
     **/
    PalletDemocracyVoteAccountVote: {
        _enum: {
            Standard: {
                vote: string;
                balance: string;
            };
            Split: {
                aye: string;
                nay: string;
            };
        };
    };
    /**
     * Lookup48: pallet_democracy::types::MetadataOwner
     **/
    PalletDemocracyMetadataOwner: {
        _enum: {
            External: string;
            Proposal: string;
            Referendum: string;
        };
    };
    /**
     * Lookup49: pallet_collective::pallet::Event<T, I>
     **/
    PalletCollectiveEvent: {
        _enum: {
            Proposed: {
                account: string;
                proposalIndex: string;
                proposalHash: string;
                threshold: string;
            };
            Voted: {
                account: string;
                proposalHash: string;
                voted: string;
                yes: string;
                no: string;
            };
            Approved: {
                proposalHash: string;
            };
            Disapproved: {
                proposalHash: string;
            };
            Executed: {
                proposalHash: string;
                result: string;
            };
            MemberExecuted: {
                proposalHash: string;
                result: string;
            };
            Closed: {
                proposalHash: string;
                yes: string;
                no: string;
            };
        };
    };
    /**
     * Lookup50: pallet_vesting::pallet::Event<T>
     **/
    PalletVestingEvent: {
        _enum: {
            VestingUpdated: {
                account: string;
                unvested: string;
            };
            VestingCompleted: {
                account: string;
            };
        };
    };
    /**
     * Lookup51: pallet_elections_phragmen::pallet::Event<T>
     **/
    PalletElectionsPhragmenEvent: {
        _enum: {
            NewTerm: {
                newMembers: string;
            };
            EmptyTerm: string;
            ElectionError: string;
            MemberKicked: {
                member: string;
            };
            Renounced: {
                candidate: string;
            };
            CandidateSlashed: {
                candidate: string;
                amount: string;
            };
            SeatHolderSlashed: {
                seatHolder: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup54: pallet_election_provider_multi_phase::pallet::Event<T>
     **/
    PalletElectionProviderMultiPhaseEvent: {
        _enum: {
            SolutionStored: {
                compute: string;
                origin: string;
                prevEjected: string;
            };
            ElectionFinalized: {
                compute: string;
                score: string;
            };
            ElectionFailed: string;
            Rewarded: {
                account: string;
                value: string;
            };
            Slashed: {
                account: string;
                value: string;
            };
            PhaseTransitioned: {
                from: string;
                to: string;
                round: string;
            };
        };
    };
    /**
     * Lookup55: pallet_election_provider_multi_phase::ElectionCompute
     **/
    PalletElectionProviderMultiPhaseElectionCompute: {
        _enum: string[];
    };
    /**
     * Lookup56: sp_npos_elections::ElectionScore
     **/
    SpNposElectionsElectionScore: {
        minimalStake: string;
        sumStake: string;
        sumStakeSquared: string;
    };
    /**
     * Lookup57: pallet_election_provider_multi_phase::Phase<Bn>
     **/
    PalletElectionProviderMultiPhasePhase: {
        _enum: {
            Off: string;
            Signed: string;
            Unsigned: string;
            Emergency: string;
        };
    };
    /**
     * Lookup59: pallet_staking::pallet::pallet::Event<T>
     **/
    PalletStakingPalletEvent: {
        _enum: {
            EraPaid: {
                eraIndex: string;
                validatorPayout: string;
                remainder: string;
            };
            Rewarded: {
                stash: string;
                dest: string;
                amount: string;
            };
            Slashed: {
                staker: string;
                amount: string;
            };
            SlashReported: {
                validator: string;
                fraction: string;
                slashEra: string;
            };
            OldSlashingReportDiscarded: {
                sessionIndex: string;
            };
            StakersElected: string;
            Bonded: {
                stash: string;
                amount: string;
            };
            Unbonded: {
                stash: string;
                amount: string;
            };
            Withdrawn: {
                stash: string;
                amount: string;
            };
            Kicked: {
                nominator: string;
                stash: string;
            };
            StakingElectionFailed: string;
            Chilled: {
                stash: string;
            };
            PayoutStarted: {
                eraIndex: string;
                validatorStash: string;
            };
            ValidatorPrefsSet: {
                stash: string;
                prefs: string;
            };
            SnapshotVotersSizeExceeded: {
                _alias: {
                    size_: string;
                };
                size_: string;
            };
            SnapshotTargetsSizeExceeded: {
                _alias: {
                    size_: string;
                };
                size_: string;
            };
            ForceEra: {
                mode: string;
            };
        };
    };
    /**
     * Lookup60: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
     **/
    PalletStakingRewardDestination: {
        _enum: {
            Staked: string;
            Stash: string;
            Controller: string;
            Account: string;
            None: string;
        };
    };
    /**
     * Lookup62: pallet_staking::ValidatorPrefs
     **/
    PalletStakingValidatorPrefs: {
        commission: string;
        blocked: string;
    };
    /**
     * Lookup64: pallet_staking::Forcing
     **/
    PalletStakingForcing: {
        _enum: string[];
    };
    /**
     * Lookup65: pallet_session::pallet::Event
     **/
    PalletSessionEvent: {
        _enum: {
            NewSession: {
                sessionIndex: string;
            };
        };
    };
    /**
     * Lookup66: pallet_treasury::pallet::Event<T, I>
     **/
    PalletTreasuryEvent: {
        _enum: {
            Proposed: {
                proposalIndex: string;
            };
            Spending: {
                budgetRemaining: string;
            };
            Awarded: {
                proposalIndex: string;
                award: string;
                account: string;
            };
            Rejected: {
                proposalIndex: string;
                slashed: string;
            };
            Burnt: {
                burntFunds: string;
            };
            Rollover: {
                rolloverBalance: string;
            };
            Deposit: {
                value: string;
            };
            SpendApproved: {
                proposalIndex: string;
                amount: string;
                beneficiary: string;
            };
            UpdatedInactive: {
                reactivated: string;
                deactivated: string;
            };
            AssetSpendApproved: {
                index: string;
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
                expireAt: string;
            };
            AssetSpendVoided: {
                index: string;
            };
            Paid: {
                index: string;
                paymentId: string;
            };
            PaymentFailed: {
                index: string;
                paymentId: string;
            };
            SpendProcessed: {
                index: string;
            };
        };
    };
    /**
     * Lookup67: pallet_bounties::pallet::Event<T, I>
     **/
    PalletBountiesEvent: {
        _enum: {
            BountyProposed: {
                index: string;
            };
            BountyRejected: {
                index: string;
                bond: string;
            };
            BountyBecameActive: {
                index: string;
            };
            BountyAwarded: {
                index: string;
                beneficiary: string;
            };
            BountyClaimed: {
                index: string;
                payout: string;
                beneficiary: string;
            };
            BountyCanceled: {
                index: string;
            };
            BountyExtended: {
                index: string;
            };
            BountyApproved: {
                index: string;
            };
            CuratorProposed: {
                bountyId: string;
                curator: string;
            };
            CuratorUnassigned: {
                bountyId: string;
            };
            CuratorAccepted: {
                bountyId: string;
                curator: string;
            };
        };
    };
    /**
     * Lookup68: pallet_child_bounties::pallet::Event<T>
     **/
    PalletChildBountiesEvent: {
        _enum: {
            Added: {
                index: string;
                childIndex: string;
            };
            Awarded: {
                index: string;
                childIndex: string;
                beneficiary: string;
            };
            Claimed: {
                index: string;
                childIndex: string;
                payout: string;
                beneficiary: string;
            };
            Canceled: {
                index: string;
                childIndex: string;
            };
        };
    };
    /**
     * Lookup69: pallet_bags_list::pallet::Event<T, I>
     **/
    PalletBagsListEvent: {
        _enum: {
            Rebagged: {
                who: string;
                from: string;
                to: string;
            };
            ScoreUpdated: {
                who: string;
                newScore: string;
            };
        };
    };
    /**
     * Lookup70: pallet_nomination_pools::pallet::Event<T>
     **/
    PalletNominationPoolsEvent: {
        _enum: {
            Created: {
                depositor: string;
                poolId: string;
            };
            Bonded: {
                member: string;
                poolId: string;
                bonded: string;
                joined: string;
            };
            PaidOut: {
                member: string;
                poolId: string;
                payout: string;
            };
            Unbonded: {
                member: string;
                poolId: string;
                balance: string;
                points: string;
                era: string;
            };
            Withdrawn: {
                member: string;
                poolId: string;
                balance: string;
                points: string;
            };
            Destroyed: {
                poolId: string;
            };
            StateChanged: {
                poolId: string;
                newState: string;
            };
            MemberRemoved: {
                poolId: string;
                member: string;
            };
            RolesUpdated: {
                root: string;
                bouncer: string;
                nominator: string;
            };
            PoolSlashed: {
                poolId: string;
                balance: string;
            };
            UnbondingPoolSlashed: {
                poolId: string;
                era: string;
                balance: string;
            };
            PoolCommissionUpdated: {
                poolId: string;
                current: string;
            };
            PoolMaxCommissionUpdated: {
                poolId: string;
                maxCommission: string;
            };
            PoolCommissionChangeRateUpdated: {
                poolId: string;
                changeRate: string;
            };
            PoolCommissionClaimPermissionUpdated: {
                poolId: string;
                permission: string;
            };
            PoolCommissionClaimed: {
                poolId: string;
                commission: string;
            };
            MinBalanceDeficitAdjusted: {
                poolId: string;
                amount: string;
            };
            MinBalanceExcessAdjusted: {
                poolId: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup71: pallet_nomination_pools::PoolState
     **/
    PalletNominationPoolsPoolState: {
        _enum: string[];
    };
    /**
     * Lookup74: pallet_nomination_pools::CommissionChangeRate<BlockNumber>
     **/
    PalletNominationPoolsCommissionChangeRate: {
        maxIncrease: string;
        minDelay: string;
    };
    /**
     * Lookup76: pallet_nomination_pools::CommissionClaimPermission<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsCommissionClaimPermission: {
        _enum: {
            Permissionless: string;
            Account: string;
        };
    };
    /**
     * Lookup77: pallet_scheduler::pallet::Event<T>
     **/
    PalletSchedulerEvent: {
        _enum: {
            Scheduled: {
                when: string;
                index: string;
            };
            Canceled: {
                when: string;
                index: string;
            };
            Dispatched: {
                task: string;
                id: string;
                result: string;
            };
            CallUnavailable: {
                task: string;
                id: string;
            };
            PeriodicFailed: {
                task: string;
                id: string;
            };
            PermanentlyOverweight: {
                task: string;
                id: string;
            };
        };
    };
    /**
     * Lookup80: pallet_preimage::pallet::Event<T>
     **/
    PalletPreimageEvent: {
        _enum: {
            Noted: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Requested: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Cleared: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
        };
    };
    /**
     * Lookup81: pallet_offences::pallet::Event
     **/
    PalletOffencesEvent: {
        _enum: {
            Offence: {
                kind: string;
                timeslot: string;
            };
        };
    };
    /**
     * Lookup83: pallet_tx_pause::pallet::Event<T>
     **/
    PalletTxPauseEvent: {
        _enum: {
            CallPaused: {
                fullName: string;
            };
            CallUnpaused: {
                fullName: string;
            };
        };
    };
    /**
     * Lookup86: pallet_im_online::pallet::Event<T>
     **/
    PalletImOnlineEvent: {
        _enum: {
            HeartbeatReceived: {
                authorityId: string;
            };
            AllGood: string;
            SomeOffline: {
                offline: string;
            };
        };
    };
    /**
     * Lookup87: pallet_im_online::sr25519::app_sr25519::Public
     **/
    PalletImOnlineSr25519AppSr25519Public: string;
    /**
     * Lookup88: sp_core::sr25519::Public
     **/
    SpCoreSr25519Public: string;
    /**
     * Lookup91: sp_staking::Exposure<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingExposure: {
        total: string;
        own: string;
        others: string;
    };
    /**
     * Lookup94: sp_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingIndividualExposure: {
        who: string;
        value: string;
    };
    /**
     * Lookup95: pallet_identity::pallet::Event<T>
     **/
    PalletIdentityEvent: {
        _enum: {
            IdentitySet: {
                who: string;
            };
            IdentityCleared: {
                who: string;
                deposit: string;
            };
            IdentityKilled: {
                who: string;
                deposit: string;
            };
            JudgementRequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementUnrequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementGiven: {
                target: string;
                registrarIndex: string;
            };
            RegistrarAdded: {
                registrarIndex: string;
            };
            SubIdentityAdded: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentityRemoved: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentityRevoked: {
                sub: string;
                main: string;
                deposit: string;
            };
            AuthorityAdded: {
                authority: string;
            };
            AuthorityRemoved: {
                authority: string;
            };
            UsernameSet: {
                who: string;
                username: string;
            };
            UsernameQueued: {
                who: string;
                username: string;
                expiration: string;
            };
            PreapprovalExpired: {
                whose: string;
            };
            PrimaryUsernameSet: {
                who: string;
                username: string;
            };
            DanglingUsernameRemoved: {
                who: string;
                username: string;
            };
        };
    };
    /**
     * Lookup97: pallet_utility::pallet::Event
     **/
    PalletUtilityEvent: {
        _enum: {
            BatchInterrupted: {
                index: string;
                error: string;
            };
            BatchCompleted: string;
            BatchCompletedWithErrors: string;
            ItemCompleted: string;
            ItemFailed: {
                error: string;
            };
            DispatchedAs: {
                result: string;
            };
        };
    };
    /**
     * Lookup98: pallet_multisig::pallet::Event<T>
     **/
    PalletMultisigEvent: {
        _enum: {
            NewMultisig: {
                approving: string;
                multisig: string;
                callHash: string;
            };
            MultisigApproval: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
            MultisigExecuted: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
                result: string;
            };
            MultisigCancelled: {
                cancelling: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
        };
    };
    /**
     * Lookup99: pallet_multisig::Timepoint<BlockNumber>
     **/
    PalletMultisigTimepoint: {
        height: string;
        index: string;
    };
    /**
     * Lookup100: pallet_ethereum::pallet::Event
     **/
    PalletEthereumEvent: {
        _enum: {
            Executed: {
                from: string;
                to: string;
                transactionHash: string;
                exitReason: string;
                extraData: string;
            };
        };
    };
    /**
     * Lookup103: evm_core::error::ExitReason
     **/
    EvmCoreErrorExitReason: {
        _enum: {
            Succeed: string;
            Error: string;
            Revert: string;
            Fatal: string;
        };
    };
    /**
     * Lookup104: evm_core::error::ExitSucceed
     **/
    EvmCoreErrorExitSucceed: {
        _enum: string[];
    };
    /**
     * Lookup105: evm_core::error::ExitError
     **/
    EvmCoreErrorExitError: {
        _enum: {
            StackUnderflow: string;
            StackOverflow: string;
            InvalidJump: string;
            InvalidRange: string;
            DesignatedInvalid: string;
            CallTooDeep: string;
            CreateCollision: string;
            CreateContractLimit: string;
            OutOfOffset: string;
            OutOfGas: string;
            OutOfFund: string;
            PCUnderflow: string;
            CreateEmpty: string;
            Other: string;
            MaxNonce: string;
            InvalidCode: string;
        };
    };
    /**
     * Lookup109: evm_core::error::ExitRevert
     **/
    EvmCoreErrorExitRevert: {
        _enum: string[];
    };
    /**
     * Lookup110: evm_core::error::ExitFatal
     **/
    EvmCoreErrorExitFatal: {
        _enum: {
            NotSupported: string;
            UnhandledInterrupt: string;
            CallErrorAsFatal: string;
            Other: string;
        };
    };
    /**
     * Lookup111: pallet_evm::pallet::Event<T>
     **/
    PalletEvmEvent: {
        _enum: {
            Log: {
                log: string;
            };
            Created: {
                address: string;
            };
            CreatedFailed: {
                address: string;
            };
            Executed: {
                address: string;
            };
            ExecutedFailed: {
                address: string;
            };
        };
    };
    /**
     * Lookup112: ethereum::log::Log
     **/
    EthereumLog: {
        address: string;
        topics: string;
        data: string;
    };
    /**
     * Lookup114: pallet_base_fee::pallet::Event
     **/
    PalletBaseFeeEvent: {
        _enum: {
            NewBaseFeePerGas: {
                fee: string;
            };
            BaseFeeOverflow: string;
            NewElasticity: {
                elasticity: string;
            };
        };
    };
    /**
     * Lookup118: pallet_airdrop_claims::pallet::Event<T>
     **/
    PalletAirdropClaimsEvent: {
        _enum: {
            Claimed: {
                recipient: string;
                source: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup119: pallet_airdrop_claims::utils::MultiAddress
     **/
    PalletAirdropClaimsUtilsMultiAddress: {
        _enum: {
            EVM: string;
            Native: string;
        };
    };
    /**
     * Lookup120: pallet_airdrop_claims::utils::ethereum_address::EthereumAddress
     **/
    PalletAirdropClaimsUtilsEthereumAddress: string;
    /**
     * Lookup121: pallet_roles::pallet::Event<T>
     **/
    PalletRolesEvent: {
        _enum: {
            RoleAssigned: {
                account: string;
                role: string;
            };
            RoleRemoved: {
                account: string;
                role: string;
            };
            Slashed: {
                account: string;
                amount: string;
            };
            ProfileCreated: {
                account: string;
                totalProfileRestake: string;
                roles: string;
            };
            ProfileUpdated: {
                account: string;
                totalProfileRestake: string;
                roles: string;
            };
            ProfileDeleted: {
                account: string;
            };
            PendingJobs: {
                pendingJobs: string;
            };
            RolesRewardSet: {
                totalRewards: string;
            };
            PayoutStarted: {
                eraIndex: string;
                validatorStash: string;
            };
            Rewarded: {
                stash: string;
                amount: string;
            };
            MinRestakingBondUpdated: {
                value: string;
            };
        };
    };
    /**
     * Lookup122: tangle_primitives::roles::RoleType
     **/
    TanglePrimitivesRolesRoleType: {
        _enum: {
            Tss: string;
            ZkSaaS: string;
            LightClientRelaying: string;
        };
    };
    /**
     * Lookup123: tangle_primitives::roles::tss::ThresholdSignatureRoleType
     **/
    TanglePrimitivesRolesTssThresholdSignatureRoleType: {
        _enum: string[];
    };
    /**
     * Lookup124: tangle_primitives::roles::zksaas::ZeroKnowledgeRoleType
     **/
    TanglePrimitivesRolesZksaasZeroKnowledgeRoleType: {
        _enum: string[];
    };
    /**
     * Lookup128: pallet_jobs::module::Event<T>
     **/
    PalletJobsModuleEvent: {
        _enum: {
            JobSubmitted: {
                jobId: string;
                roleType: string;
                details: string;
            };
            JobResultSubmitted: {
                jobId: string;
                roleType: string;
            };
            ValidatorRewarded: {
                id: string;
                reward: string;
            };
            JobRefunded: {
                jobId: string;
                roleType: string;
            };
            JobParticipantsUpdated: {
                jobId: string;
                roleType: string;
                details: string;
            };
            JobReSubmitted: {
                jobId: string;
                roleType: string;
                details: string;
            };
            JobResultExtended: {
                jobId: string;
                roleType: string;
                newExpiry: string;
            };
        };
    };
    /**
     * Lookup129: tangle_primitives::jobs::JobSubmission<sp_core::crypto::AccountId32, BlockNumber, tangle_runtime::MaxParticipants, tangle_runtime::MaxSubmissionLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsJobSubmission: {
        expiry: string;
        ttl: string;
        jobType: string;
        fallback: string;
    };
    /**
     * Lookup130: tangle_runtime::MaxParticipants
     **/
    TangleRuntimeMaxParticipants: string;
    /**
     * Lookup131: tangle_runtime::MaxSubmissionLen
     **/
    TangleRuntimeMaxSubmissionLen: string;
    /**
     * Lookup132: tangle_runtime::MaxAdditionalParamsLen
     **/
    TangleRuntimeMaxAdditionalParamsLen: string;
    /**
     * Lookup133: tangle_primitives::jobs::JobType<sp_core::crypto::AccountId32, tangle_runtime::MaxParticipants, tangle_runtime::MaxSubmissionLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsJobType: {
        _enum: {
            DKGTSSPhaseOne: string;
            DKGTSSPhaseTwo: string;
            DKGTSSPhaseThree: string;
            DKGTSSPhaseFour: string;
            ZkSaaSPhaseOne: string;
            ZkSaaSPhaseTwo: string;
        };
    };
    /**
     * Lookup134: tangle_primitives::jobs::tss::DKGTSSPhaseOneJobType<sp_core::crypto::AccountId32, tangle_runtime::MaxParticipants>
     **/
    TanglePrimitivesJobsTssDkgtssPhaseOneJobType: {
        participants: string;
        threshold: string;
        permittedCaller: string;
        roleType: string;
    };
    /**
     * Lookup137: tangle_primitives::jobs::tss::DKGTSSPhaseTwoJobType<tangle_runtime::MaxSubmissionLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsTssDkgtssPhaseTwoJobType: {
        phaseOneId: string;
        submission: string;
        derivationPath: string;
        roleType: string;
    };
    /**
     * Lookup141: tangle_primitives::jobs::tss::DKGTSSPhaseThreeJobType
     **/
    TanglePrimitivesJobsTssDkgtssPhaseThreeJobType: {
        phaseOneId: string;
        roleType: string;
    };
    /**
     * Lookup142: tangle_primitives::jobs::tss::DKGTSSPhaseFourJobType
     **/
    TanglePrimitivesJobsTssDkgtssPhaseFourJobType: {
        phaseOneId: string;
        newPhaseOneId: string;
        roleType: string;
    };
    /**
     * Lookup143: tangle_primitives::jobs::zksaas::ZkSaaSPhaseOneJobType<sp_core::crypto::AccountId32, tangle_runtime::MaxParticipants, tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasZkSaaSPhaseOneJobType: {
        participants: string;
        permittedCaller: string;
        system: string;
        roleType: string;
    };
    /**
     * Lookup144: tangle_primitives::jobs::zksaas::ZkSaaSSystem<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasZkSaaSSystem: {
        _enum: {
            Groth16: string;
        };
    };
    /**
     * Lookup145: tangle_primitives::jobs::zksaas::Groth16System<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasGroth16System: {
        circuit: string;
        numInputs: string;
        numConstraints: string;
        provingKey: string;
        verifyingKey: string;
        wasm: string;
    };
    /**
     * Lookup146: tangle_primitives::jobs::zksaas::HyperData<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasHyperData: {
        _enum: {
            Raw: string;
            IPFS: string;
            HTTP: string;
        };
    };
    /**
     * Lookup147: tangle_primitives::jobs::zksaas::ZkSaaSPhaseTwoJobType<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasZkSaaSPhaseTwoJobType: {
        phaseOneId: string;
        request: string;
        roleType: string;
    };
    /**
     * Lookup148: tangle_primitives::jobs::zksaas::ZkSaaSPhaseTwoRequest<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasZkSaaSPhaseTwoRequest: {
        _enum: {
            Groth16: string;
        };
    };
    /**
     * Lookup149: tangle_primitives::jobs::zksaas::Groth16ProveRequest<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasGroth16ProveRequest: {
        publicInput: string;
        aShares: string;
        axShares: string;
        qapShares: string;
    };
    /**
     * Lookup153: tangle_primitives::jobs::zksaas::QAPShare<tangle_runtime::MaxSubmissionLen>
     **/
    TanglePrimitivesJobsZksaasQapShare: {
        a: string;
        b: string;
        c: string;
    };
    /**
     * Lookup155: tangle_primitives::jobs::FallbackOptions
     **/
    TanglePrimitivesJobsFallbackOptions: {
        _enum: {
            Destroy: string;
            RegenerateWithThreshold: string;
        };
    };
    /**
     * Lookup156: tangle_primitives::jobs::JobInfo<sp_core::crypto::AccountId32, BlockNumber, Balance, tangle_runtime::MaxParticipants, tangle_runtime::MaxSubmissionLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsJobInfo: {
        owner: string;
        expiry: string;
        ttl: string;
        jobType: string;
        fee: string;
        fallback: string;
    };
    /**
     * Lookup157: pallet_dkg::pallet::Event<T>
     **/
    PalletDkgEvent: {
        _enum: {
            FeeUpdated: string;
            KeyRotated: {
                fromJobId: string;
                toJobId: string;
                signature: string;
            };
        };
    };
    /**
     * Lookup158: pallet_dkg::types::FeeInfo<Balance>
     **/
    PalletDkgFeeInfo: {
        baseFee: string;
        dkgValidatorFee: string;
        sigValidatorFee: string;
        refreshValidatorFee: string;
        storageFeePerByte: string;
        storageFeePerBlock: string;
    };
    /**
     * Lookup159: pallet_zksaas::pallet::Event<T>
     **/
    PalletZksaasEvent: {
        _enum: {
            FeeUpdated: string;
        };
    };
    /**
     * Lookup160: pallet_zksaas::types::FeeInfo<Balance>
     **/
    PalletZksaasFeeInfo: {
        baseFee: string;
        circuitFee: string;
        proveFee: string;
        storageFeePerByte: string;
    };
    /**
     * Lookup161: frame_system::Phase
     **/
    FrameSystemPhase: {
        _enum: {
            ApplyExtrinsic: string;
            Finalization: string;
            Initialization: string;
        };
    };
    /**
     * Lookup163: frame_system::LastRuntimeUpgradeInfo
     **/
    FrameSystemLastRuntimeUpgradeInfo: {
        specVersion: string;
        specName: string;
    };
    /**
     * Lookup165: frame_system::CodeUpgradeAuthorization<T>
     **/
    FrameSystemCodeUpgradeAuthorization: {
        codeHash: string;
        checkVersion: string;
    };
    /**
     * Lookup166: frame_system::pallet::Call<T>
     **/
    FrameSystemCall: {
        _enum: {
            remark: {
                remark: string;
            };
            set_heap_pages: {
                pages: string;
            };
            set_code: {
                code: string;
            };
            set_code_without_checks: {
                code: string;
            };
            set_storage: {
                items: string;
            };
            kill_storage: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
            };
            kill_prefix: {
                prefix: string;
                subkeys: string;
            };
            remark_with_event: {
                remark: string;
            };
            __Unused8: string;
            authorize_upgrade: {
                codeHash: string;
            };
            authorize_upgrade_without_checks: {
                codeHash: string;
            };
            apply_authorized_upgrade: {
                code: string;
            };
        };
    };
    /**
     * Lookup170: frame_system::limits::BlockWeights
     **/
    FrameSystemLimitsBlockWeights: {
        baseBlock: string;
        maxBlock: string;
        perClass: string;
    };
    /**
     * Lookup171: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
     **/
    FrameSupportDispatchPerDispatchClassWeightsPerClass: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup172: frame_system::limits::WeightsPerClass
     **/
    FrameSystemLimitsWeightsPerClass: {
        baseExtrinsic: string;
        maxExtrinsic: string;
        maxTotal: string;
        reserved: string;
    };
    /**
     * Lookup174: frame_system::limits::BlockLength
     **/
    FrameSystemLimitsBlockLength: {
        max: string;
    };
    /**
     * Lookup175: frame_support::dispatch::PerDispatchClass<T>
     **/
    FrameSupportDispatchPerDispatchClassU32: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup176: sp_weights::RuntimeDbWeight
     **/
    SpWeightsRuntimeDbWeight: {
        read: string;
        write: string;
    };
    /**
     * Lookup177: sp_version::RuntimeVersion
     **/
    SpVersionRuntimeVersion: {
        specName: string;
        implName: string;
        authoringVersion: string;
        specVersion: string;
        implVersion: string;
        apis: string;
        transactionVersion: string;
        stateVersion: string;
    };
    /**
     * Lookup183: frame_system::pallet::Error<T>
     **/
    FrameSystemError: {
        _enum: string[];
    };
    /**
     * Lookup184: pallet_timestamp::pallet::Call<T>
     **/
    PalletTimestampCall: {
        _enum: {
            set: {
                now: string;
            };
        };
    };
    /**
     * Lookup185: pallet_sudo::pallet::Call<T>
     **/
    PalletSudoCall: {
        _enum: {
            sudo: {
                call: string;
            };
            sudo_unchecked_weight: {
                call: string;
                weight: string;
            };
            set_key: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            sudo_as: {
                who: string;
                call: string;
            };
            remove_key: string;
        };
    };
    /**
     * Lookup187: pallet_balances::pallet::Call<T, I>
     **/
    PalletBalancesCall: {
        _enum: {
            transfer_allow_death: {
                dest: string;
                value: string;
            };
            __Unused1: string;
            force_transfer: {
                source: string;
                dest: string;
                value: string;
            };
            transfer_keep_alive: {
                dest: string;
                value: string;
            };
            transfer_all: {
                dest: string;
                keepAlive: string;
            };
            force_unreserve: {
                who: string;
                amount: string;
            };
            upgrade_accounts: {
                who: string;
            };
            __Unused7: string;
            force_set_balance: {
                who: string;
                newFree: string;
            };
            force_adjust_total_issuance: {
                direction: string;
                delta: string;
            };
        };
    };
    /**
     * Lookup189: pallet_balances::types::AdjustmentDirection
     **/
    PalletBalancesAdjustmentDirection: {
        _enum: string[];
    };
    /**
     * Lookup190: pallet_babe::pallet::Call<T>
     **/
    PalletBabeCall: {
        _enum: {
            report_equivocation: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_equivocation_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            plan_config_change: {
                config: string;
            };
        };
    };
    /**
     * Lookup191: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_babe::app::Public>
     **/
    SpConsensusSlotsEquivocationProof: {
        offender: string;
        slot: string;
        firstHeader: string;
        secondHeader: string;
    };
    /**
     * Lookup192: sp_runtime::generic::header::Header<Number, Hash>
     **/
    SpRuntimeHeader: {
        parentHash: string;
        number: string;
        stateRoot: string;
        extrinsicsRoot: string;
        digest: string;
    };
    /**
     * Lookup193: sp_consensus_babe::app::Public
     **/
    SpConsensusBabeAppPublic: string;
    /**
     * Lookup195: sp_session::MembershipProof
     **/
    SpSessionMembershipProof: {
        session: string;
        trieNodes: string;
        validatorCount: string;
    };
    /**
     * Lookup196: sp_consensus_babe::digests::NextConfigDescriptor
     **/
    SpConsensusBabeDigestsNextConfigDescriptor: {
        _enum: {
            __Unused0: string;
            V1: {
                c: string;
                allowedSlots: string;
            };
        };
    };
    /**
     * Lookup198: sp_consensus_babe::AllowedSlots
     **/
    SpConsensusBabeAllowedSlots: {
        _enum: string[];
    };
    /**
     * Lookup199: pallet_grandpa::pallet::Call<T>
     **/
    PalletGrandpaCall: {
        _enum: {
            report_equivocation: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_equivocation_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            note_stalled: {
                delay: string;
                bestFinalizedBlockNumber: string;
            };
        };
    };
    /**
     * Lookup200: sp_consensus_grandpa::EquivocationProof<primitive_types::H256, N>
     **/
    SpConsensusGrandpaEquivocationProof: {
        setId: string;
        equivocation: string;
    };
    /**
     * Lookup201: sp_consensus_grandpa::Equivocation<primitive_types::H256, N>
     **/
    SpConsensusGrandpaEquivocation: {
        _enum: {
            Prevote: string;
            Precommit: string;
        };
    };
    /**
     * Lookup202: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
     **/
    FinalityGrandpaEquivocationPrevote: {
        roundNumber: string;
        identity: string;
        first: string;
        second: string;
    };
    /**
     * Lookup203: finality_grandpa::Prevote<primitive_types::H256, N>
     **/
    FinalityGrandpaPrevote: {
        targetHash: string;
        targetNumber: string;
    };
    /**
     * Lookup204: sp_consensus_grandpa::app::Signature
     **/
    SpConsensusGrandpaAppSignature: string;
    /**
     * Lookup205: sp_core::ed25519::Signature
     **/
    SpCoreEd25519Signature: string;
    /**
     * Lookup208: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
     **/
    FinalityGrandpaEquivocationPrecommit: {
        roundNumber: string;
        identity: string;
        first: string;
        second: string;
    };
    /**
     * Lookup209: finality_grandpa::Precommit<primitive_types::H256, N>
     **/
    FinalityGrandpaPrecommit: {
        targetHash: string;
        targetNumber: string;
    };
    /**
     * Lookup211: sp_core::Void
     **/
    SpCoreVoid: string;
    /**
     * Lookup212: pallet_indices::pallet::Call<T>
     **/
    PalletIndicesCall: {
        _enum: {
            claim: {
                index: string;
            };
            transfer: {
                _alias: {
                    new_: string;
                };
                new_: string;
                index: string;
            };
            free: {
                index: string;
            };
            force_transfer: {
                _alias: {
                    new_: string;
                };
                new_: string;
                index: string;
                freeze: string;
            };
            freeze: {
                index: string;
            };
        };
    };
    /**
     * Lookup213: pallet_democracy::pallet::Call<T>
     **/
    PalletDemocracyCall: {
        _enum: {
            propose: {
                proposal: string;
                value: string;
            };
            second: {
                proposal: string;
            };
            vote: {
                refIndex: string;
                vote: string;
            };
            emergency_cancel: {
                refIndex: string;
            };
            external_propose: {
                proposal: string;
            };
            external_propose_majority: {
                proposal: string;
            };
            external_propose_default: {
                proposal: string;
            };
            fast_track: {
                proposalHash: string;
                votingPeriod: string;
                delay: string;
            };
            veto_external: {
                proposalHash: string;
            };
            cancel_referendum: {
                refIndex: string;
            };
            delegate: {
                to: string;
                conviction: string;
                balance: string;
            };
            undelegate: string;
            clear_public_proposals: string;
            unlock: {
                target: string;
            };
            remove_vote: {
                index: string;
            };
            remove_other_vote: {
                target: string;
                index: string;
            };
            blacklist: {
                proposalHash: string;
                maybeRefIndex: string;
            };
            cancel_proposal: {
                propIndex: string;
            };
            set_metadata: {
                owner: string;
                maybeHash: string;
            };
        };
    };
    /**
     * Lookup214: frame_support::traits::preimages::Bounded<tangle_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>
     **/
    FrameSupportPreimagesBounded: {
        _enum: {
            Legacy: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Inline: string;
            Lookup: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                len: string;
            };
        };
    };
    /**
     * Lookup215: sp_runtime::traits::BlakeTwo256
     **/
    SpRuntimeBlakeTwo256: string;
    /**
     * Lookup217: pallet_democracy::conviction::Conviction
     **/
    PalletDemocracyConviction: {
        _enum: string[];
    };
    /**
     * Lookup220: pallet_collective::pallet::Call<T, I>
     **/
    PalletCollectiveCall: {
        _enum: {
            set_members: {
                newMembers: string;
                prime: string;
                oldCount: string;
            };
            execute: {
                proposal: string;
                lengthBound: string;
            };
            propose: {
                threshold: string;
                proposal: string;
                lengthBound: string;
            };
            vote: {
                proposal: string;
                index: string;
                approve: string;
            };
            __Unused4: string;
            disapprove_proposal: {
                proposalHash: string;
            };
            close: {
                proposalHash: string;
                index: string;
                proposalWeightBound: string;
                lengthBound: string;
            };
        };
    };
    /**
     * Lookup221: pallet_vesting::pallet::Call<T>
     **/
    PalletVestingCall: {
        _enum: {
            vest: string;
            vest_other: {
                target: string;
            };
            vested_transfer: {
                target: string;
                schedule: string;
            };
            force_vested_transfer: {
                source: string;
                target: string;
                schedule: string;
            };
            merge_schedules: {
                schedule1Index: string;
                schedule2Index: string;
            };
            force_remove_vesting_schedule: {
                target: string;
                scheduleIndex: string;
            };
        };
    };
    /**
     * Lookup222: pallet_vesting::vesting_info::VestingInfo<Balance, BlockNumber>
     **/
    PalletVestingVestingInfo: {
        locked: string;
        perBlock: string;
        startingBlock: string;
    };
    /**
     * Lookup223: pallet_elections_phragmen::pallet::Call<T>
     **/
    PalletElectionsPhragmenCall: {
        _enum: {
            vote: {
                votes: string;
                value: string;
            };
            remove_voter: string;
            submit_candidacy: {
                candidateCount: string;
            };
            renounce_candidacy: {
                renouncing: string;
            };
            remove_member: {
                who: string;
                slashBond: string;
                rerunElection: string;
            };
            clean_defunct_voters: {
                numVoters: string;
                numDefunct: string;
            };
        };
    };
    /**
     * Lookup224: pallet_elections_phragmen::Renouncing
     **/
    PalletElectionsPhragmenRenouncing: {
        _enum: {
            Member: string;
            RunnerUp: string;
            Candidate: string;
        };
    };
    /**
     * Lookup225: pallet_election_provider_multi_phase::pallet::Call<T>
     **/
    PalletElectionProviderMultiPhaseCall: {
        _enum: {
            submit_unsigned: {
                rawSolution: string;
                witness: string;
            };
            set_minimum_untrusted_score: {
                maybeNextScore: string;
            };
            set_emergency_election_result: {
                supports: string;
            };
            submit: {
                rawSolution: string;
            };
            governance_fallback: {
                maybeMaxVoters: string;
                maybeMaxTargets: string;
            };
        };
    };
    /**
     * Lookup226: pallet_election_provider_multi_phase::RawSolution<tangle_runtime::NposSolution16>
     **/
    PalletElectionProviderMultiPhaseRawSolution: {
        solution: string;
        score: string;
        round: string;
    };
    /**
     * Lookup227: tangle_runtime::NposSolution16
     **/
    TangleRuntimeNposSolution16: {
        votes1: string;
        votes2: string;
        votes3: string;
        votes4: string;
        votes5: string;
        votes6: string;
        votes7: string;
        votes8: string;
        votes9: string;
        votes10: string;
        votes11: string;
        votes12: string;
        votes13: string;
        votes14: string;
        votes15: string;
        votes16: string;
    };
    /**
     * Lookup278: pallet_election_provider_multi_phase::SolutionOrSnapshotSize
     **/
    PalletElectionProviderMultiPhaseSolutionOrSnapshotSize: {
        voters: string;
        targets: string;
    };
    /**
     * Lookup282: sp_npos_elections::Support<sp_core::crypto::AccountId32>
     **/
    SpNposElectionsSupport: {
        total: string;
        voters: string;
    };
    /**
     * Lookup283: pallet_staking::pallet::pallet::Call<T>
     **/
    PalletStakingPalletCall: {
        _enum: {
            bond: {
                value: string;
                payee: string;
            };
            bond_extra: {
                maxAdditional: string;
            };
            unbond: {
                value: string;
            };
            withdraw_unbonded: {
                numSlashingSpans: string;
            };
            validate: {
                prefs: string;
            };
            nominate: {
                targets: string;
            };
            chill: string;
            set_payee: {
                payee: string;
            };
            set_controller: string;
            set_validator_count: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            increase_validator_count: {
                additional: string;
            };
            scale_validator_count: {
                factor: string;
            };
            force_no_eras: string;
            force_new_era: string;
            set_invulnerables: {
                invulnerables: string;
            };
            force_unstake: {
                stash: string;
                numSlashingSpans: string;
            };
            force_new_era_always: string;
            cancel_deferred_slash: {
                era: string;
                slashIndices: string;
            };
            payout_stakers: {
                validatorStash: string;
                era: string;
            };
            rebond: {
                value: string;
            };
            reap_stash: {
                stash: string;
                numSlashingSpans: string;
            };
            kick: {
                who: string;
            };
            set_staking_configs: {
                minNominatorBond: string;
                minValidatorBond: string;
                maxNominatorCount: string;
                maxValidatorCount: string;
                chillThreshold: string;
                minCommission: string;
            };
            chill_other: {
                stash: string;
            };
            force_apply_min_commission: {
                validatorStash: string;
            };
            set_min_commission: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            payout_stakers_by_page: {
                validatorStash: string;
                era: string;
                page: string;
            };
            update_payee: {
                controller: string;
            };
            deprecate_controller_batch: {
                controllers: string;
            };
        };
    };
    /**
     * Lookup287: pallet_staking::pallet::pallet::ConfigOp<T>
     **/
    PalletStakingPalletConfigOpU128: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup288: pallet_staking::pallet::pallet::ConfigOp<T>
     **/
    PalletStakingPalletConfigOpU32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup289: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Percent>
     **/
    PalletStakingPalletConfigOpPercent: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup290: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Perbill>
     **/
    PalletStakingPalletConfigOpPerbill: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup292: pallet_session::pallet::Call<T>
     **/
    PalletSessionCall: {
        _enum: {
            set_keys: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
                proof: string;
            };
            purge_keys: string;
        };
    };
    /**
     * Lookup293: tangle_runtime::opaque::SessionKeys
     **/
    TangleRuntimeOpaqueSessionKeys: {
        babe: string;
        grandpa: string;
        imOnline: string;
        role: string;
    };
    /**
     * Lookup294: tangle_crypto_primitives::crypto::Public
     **/
    TangleCryptoPrimitivesCryptoPublic: string;
    /**
     * Lookup295: sp_core::ecdsa::Public
     **/
    SpCoreEcdsaPublic: string;
    /**
     * Lookup297: pallet_treasury::pallet::Call<T, I>
     **/
    PalletTreasuryCall: {
        _enum: {
            propose_spend: {
                value: string;
                beneficiary: string;
            };
            reject_proposal: {
                proposalId: string;
            };
            approve_proposal: {
                proposalId: string;
            };
            spend_local: {
                amount: string;
                beneficiary: string;
            };
            remove_approval: {
                proposalId: string;
            };
            spend: {
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
            };
            payout: {
                index: string;
            };
            check_status: {
                index: string;
            };
            void_spend: {
                index: string;
            };
        };
    };
    /**
     * Lookup299: pallet_bounties::pallet::Call<T, I>
     **/
    PalletBountiesCall: {
        _enum: {
            propose_bounty: {
                value: string;
                description: string;
            };
            approve_bounty: {
                bountyId: string;
            };
            propose_curator: {
                bountyId: string;
                curator: string;
                fee: string;
            };
            unassign_curator: {
                bountyId: string;
            };
            accept_curator: {
                bountyId: string;
            };
            award_bounty: {
                bountyId: string;
                beneficiary: string;
            };
            claim_bounty: {
                bountyId: string;
            };
            close_bounty: {
                bountyId: string;
            };
            extend_bounty_expiry: {
                bountyId: string;
                remark: string;
            };
        };
    };
    /**
     * Lookup300: pallet_child_bounties::pallet::Call<T>
     **/
    PalletChildBountiesCall: {
        _enum: {
            add_child_bounty: {
                parentBountyId: string;
                value: string;
                description: string;
            };
            propose_curator: {
                parentBountyId: string;
                childBountyId: string;
                curator: string;
                fee: string;
            };
            accept_curator: {
                parentBountyId: string;
                childBountyId: string;
            };
            unassign_curator: {
                parentBountyId: string;
                childBountyId: string;
            };
            award_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
                beneficiary: string;
            };
            claim_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
            };
            close_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
            };
        };
    };
    /**
     * Lookup301: pallet_bags_list::pallet::Call<T, I>
     **/
    PalletBagsListCall: {
        _enum: {
            rebag: {
                dislocated: string;
            };
            put_in_front_of: {
                lighter: string;
            };
            put_in_front_of_other: {
                heavier: string;
                lighter: string;
            };
        };
    };
    /**
     * Lookup302: pallet_nomination_pools::pallet::Call<T>
     **/
    PalletNominationPoolsCall: {
        _enum: {
            join: {
                amount: string;
                poolId: string;
            };
            bond_extra: {
                extra: string;
            };
            claim_payout: string;
            unbond: {
                memberAccount: string;
                unbondingPoints: string;
            };
            pool_withdraw_unbonded: {
                poolId: string;
                numSlashingSpans: string;
            };
            withdraw_unbonded: {
                memberAccount: string;
                numSlashingSpans: string;
            };
            create: {
                amount: string;
                root: string;
                nominator: string;
                bouncer: string;
            };
            create_with_pool_id: {
                amount: string;
                root: string;
                nominator: string;
                bouncer: string;
                poolId: string;
            };
            nominate: {
                poolId: string;
                validators: string;
            };
            set_state: {
                poolId: string;
                state: string;
            };
            set_metadata: {
                poolId: string;
                metadata: string;
            };
            set_configs: {
                minJoinBond: string;
                minCreateBond: string;
                maxPools: string;
                maxMembers: string;
                maxMembersPerPool: string;
                globalMaxCommission: string;
            };
            update_roles: {
                poolId: string;
                newRoot: string;
                newNominator: string;
                newBouncer: string;
            };
            chill: {
                poolId: string;
            };
            bond_extra_other: {
                member: string;
                extra: string;
            };
            set_claim_permission: {
                permission: string;
            };
            claim_payout_other: {
                other: string;
            };
            set_commission: {
                poolId: string;
                newCommission: string;
            };
            set_commission_max: {
                poolId: string;
                maxCommission: string;
            };
            set_commission_change_rate: {
                poolId: string;
                changeRate: string;
            };
            claim_commission: {
                poolId: string;
            };
            adjust_pool_deposit: {
                poolId: string;
            };
            set_commission_claim_permission: {
                poolId: string;
                permission: string;
            };
        };
    };
    /**
     * Lookup303: pallet_nomination_pools::BondExtra<Balance>
     **/
    PalletNominationPoolsBondExtra: {
        _enum: {
            FreeBalance: string;
            Rewards: string;
        };
    };
    /**
     * Lookup304: pallet_nomination_pools::ConfigOp<T>
     **/
    PalletNominationPoolsConfigOpU128: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup305: pallet_nomination_pools::ConfigOp<T>
     **/
    PalletNominationPoolsConfigOpU32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup306: pallet_nomination_pools::ConfigOp<sp_arithmetic::per_things::Perbill>
     **/
    PalletNominationPoolsConfigOpPerbill: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup307: pallet_nomination_pools::ConfigOp<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsConfigOpAccountId32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup308: pallet_nomination_pools::ClaimPermission
     **/
    PalletNominationPoolsClaimPermission: {
        _enum: string[];
    };
    /**
     * Lookup309: pallet_scheduler::pallet::Call<T>
     **/
    PalletSchedulerCall: {
        _enum: {
            schedule: {
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel: {
                when: string;
                index: string;
            };
            schedule_named: {
                id: string;
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel_named: {
                id: string;
            };
            schedule_after: {
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            schedule_named_after: {
                id: string;
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
        };
    };
    /**
     * Lookup311: pallet_preimage::pallet::Call<T>
     **/
    PalletPreimageCall: {
        _enum: {
            note_preimage: {
                bytes: string;
            };
            unnote_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            request_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            unrequest_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            ensure_updated: {
                hashes: string;
            };
        };
    };
    /**
     * Lookup312: pallet_tx_pause::pallet::Call<T>
     **/
    PalletTxPauseCall: {
        _enum: {
            pause: {
                fullName: string;
            };
            unpause: {
                ident: string;
            };
        };
    };
    /**
     * Lookup313: pallet_im_online::pallet::Call<T>
     **/
    PalletImOnlineCall: {
        _enum: {
            heartbeat: {
                heartbeat: string;
                signature: string;
            };
        };
    };
    /**
     * Lookup314: pallet_im_online::Heartbeat<BlockNumber>
     **/
    PalletImOnlineHeartbeat: {
        blockNumber: string;
        sessionIndex: string;
        authorityIndex: string;
        validatorsLen: string;
    };
    /**
     * Lookup315: pallet_im_online::sr25519::app_sr25519::Signature
     **/
    PalletImOnlineSr25519AppSr25519Signature: string;
    /**
     * Lookup316: sp_core::sr25519::Signature
     **/
    SpCoreSr25519Signature: string;
    /**
     * Lookup317: pallet_identity::pallet::Call<T>
     **/
    PalletIdentityCall: {
        _enum: {
            add_registrar: {
                account: string;
            };
            set_identity: {
                info: string;
            };
            set_subs: {
                subs: string;
            };
            clear_identity: string;
            request_judgement: {
                regIndex: string;
                maxFee: string;
            };
            cancel_request: {
                regIndex: string;
            };
            set_fee: {
                index: string;
                fee: string;
            };
            set_account_id: {
                _alias: {
                    new_: string;
                };
                index: string;
                new_: string;
            };
            set_fields: {
                index: string;
                fields: string;
            };
            provide_judgement: {
                regIndex: string;
                target: string;
                judgement: string;
                identity: string;
            };
            kill_identity: {
                target: string;
            };
            add_sub: {
                sub: string;
                data: string;
            };
            rename_sub: {
                sub: string;
                data: string;
            };
            remove_sub: {
                sub: string;
            };
            quit_sub: string;
            add_username_authority: {
                authority: string;
                suffix: string;
                allocation: string;
            };
            remove_username_authority: {
                authority: string;
            };
            set_username_for: {
                who: string;
                username: string;
                signature: string;
            };
            accept_username: {
                username: string;
            };
            remove_expired_approval: {
                username: string;
            };
            set_primary_username: {
                username: string;
            };
            remove_dangling_username: {
                username: string;
            };
        };
    };
    /**
     * Lookup318: pallet_identity::legacy::IdentityInfo<FieldLimit>
     **/
    PalletIdentityLegacyIdentityInfo: {
        additional: string;
        display: string;
        legal: string;
        web: string;
        riot: string;
        email: string;
        pgpFingerprint: string;
        image: string;
        twitter: string;
    };
    /**
     * Lookup354: pallet_identity::types::Judgement<Balance>
     **/
    PalletIdentityJudgement: {
        _enum: {
            Unknown: string;
            FeePaid: string;
            Reasonable: string;
            KnownGood: string;
            OutOfDate: string;
            LowQuality: string;
            Erroneous: string;
        };
    };
    /**
     * Lookup356: sp_runtime::MultiSignature
     **/
    SpRuntimeMultiSignature: {
        _enum: {
            Ed25519: string;
            Sr25519: string;
            Ecdsa: string;
        };
    };
    /**
     * Lookup357: sp_core::ecdsa::Signature
     **/
    SpCoreEcdsaSignature: string;
    /**
     * Lookup359: pallet_utility::pallet::Call<T>
     **/
    PalletUtilityCall: {
        _enum: {
            batch: {
                calls: string;
            };
            as_derivative: {
                index: string;
                call: string;
            };
            batch_all: {
                calls: string;
            };
            dispatch_as: {
                asOrigin: string;
                call: string;
            };
            force_batch: {
                calls: string;
            };
            with_weight: {
                call: string;
                weight: string;
            };
        };
    };
    /**
     * Lookup361: tangle_runtime::OriginCaller
     **/
    TangleRuntimeOriginCaller: {
        _enum: {
            system: string;
            __Unused1: string;
            __Unused2: string;
            Void: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            Council: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            Ethereum: string;
        };
    };
    /**
     * Lookup362: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
     **/
    FrameSupportDispatchRawOrigin: {
        _enum: {
            Root: string;
            Signed: string;
            None: string;
        };
    };
    /**
     * Lookup363: pallet_collective::RawOrigin<sp_core::crypto::AccountId32, I>
     **/
    PalletCollectiveRawOrigin: {
        _enum: {
            Members: string;
            Member: string;
            _Phantom: string;
        };
    };
    /**
     * Lookup364: pallet_ethereum::RawOrigin
     **/
    PalletEthereumRawOrigin: {
        _enum: {
            EthereumTransaction: string;
        };
    };
    /**
     * Lookup365: pallet_multisig::pallet::Call<T>
     **/
    PalletMultisigCall: {
        _enum: {
            as_multi_threshold_1: {
                otherSignatories: string;
                call: string;
            };
            as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                call: string;
                maxWeight: string;
            };
            approve_as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                callHash: string;
                maxWeight: string;
            };
            cancel_as_multi: {
                threshold: string;
                otherSignatories: string;
                timepoint: string;
                callHash: string;
            };
        };
    };
    /**
     * Lookup367: pallet_ethereum::pallet::Call<T>
     **/
    PalletEthereumCall: {
        _enum: {
            transact: {
                transaction: string;
            };
        };
    };
    /**
     * Lookup368: ethereum::transaction::TransactionV2
     **/
    EthereumTransactionTransactionV2: {
        _enum: {
            Legacy: string;
            EIP2930: string;
            EIP1559: string;
        };
    };
    /**
     * Lookup369: ethereum::transaction::LegacyTransaction
     **/
    EthereumTransactionLegacyTransaction: {
        nonce: string;
        gasPrice: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        signature: string;
    };
    /**
     * Lookup370: ethereum::transaction::TransactionAction
     **/
    EthereumTransactionTransactionAction: {
        _enum: {
            Call: string;
            Create: string;
        };
    };
    /**
     * Lookup371: ethereum::transaction::TransactionSignature
     **/
    EthereumTransactionTransactionSignature: {
        v: string;
        r: string;
        s: string;
    };
    /**
     * Lookup373: ethereum::transaction::EIP2930Transaction
     **/
    EthereumTransactionEip2930Transaction: {
        chainId: string;
        nonce: string;
        gasPrice: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
        oddYParity: string;
        r: string;
        s: string;
    };
    /**
     * Lookup375: ethereum::transaction::AccessListItem
     **/
    EthereumTransactionAccessListItem: {
        address: string;
        storageKeys: string;
    };
    /**
     * Lookup376: ethereum::transaction::EIP1559Transaction
     **/
    EthereumTransactionEip1559Transaction: {
        chainId: string;
        nonce: string;
        maxPriorityFeePerGas: string;
        maxFeePerGas: string;
        gasLimit: string;
        action: string;
        value: string;
        input: string;
        accessList: string;
        oddYParity: string;
        r: string;
        s: string;
    };
    /**
     * Lookup377: pallet_evm::pallet::Call<T>
     **/
    PalletEvmCall: {
        _enum: {
            withdraw: {
                address: string;
                value: string;
            };
            call: {
                source: string;
                target: string;
                input: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
            create: {
                source: string;
                init: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
            create2: {
                source: string;
                init: string;
                salt: string;
                value: string;
                gasLimit: string;
                maxFeePerGas: string;
                maxPriorityFeePerGas: string;
                nonce: string;
                accessList: string;
            };
        };
    };
    /**
     * Lookup381: pallet_dynamic_fee::pallet::Call<T>
     **/
    PalletDynamicFeeCall: {
        _enum: {
            note_min_gas_price_target: {
                target: string;
            };
        };
    };
    /**
     * Lookup382: pallet_base_fee::pallet::Call<T>
     **/
    PalletBaseFeeCall: {
        _enum: {
            set_base_fee_per_gas: {
                fee: string;
            };
            set_elasticity: {
                elasticity: string;
            };
        };
    };
    /**
     * Lookup383: pallet_hotfix_sufficients::pallet::Call<T>
     **/
    PalletHotfixSufficientsCall: {
        _enum: {
            hotfix_inc_account_sufficients: {
                addresses: string;
            };
        };
    };
    /**
     * Lookup385: pallet_airdrop_claims::pallet::Call<T>
     **/
    PalletAirdropClaimsCall: {
        _enum: {
            claim: {
                dest: string;
                signer: string;
                signature: string;
            };
            mint_claim: {
                who: string;
                value: string;
                vestingSchedule: string;
                statement: string;
            };
            claim_attest: {
                dest: string;
                signer: string;
                signature: string;
                statement: string;
            };
            __Unused3: string;
            move_claim: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            force_set_expiry_config: {
                expiryBlock: string;
                dest: string;
            };
        };
    };
    /**
     * Lookup387: pallet_airdrop_claims::utils::MultiAddressSignature
     **/
    PalletAirdropClaimsUtilsMultiAddressSignature: {
        _enum: {
            EVM: string;
            Native: string;
        };
    };
    /**
     * Lookup388: pallet_airdrop_claims::utils::ethereum_address::EcdsaSignature
     **/
    PalletAirdropClaimsUtilsEthereumAddressEcdsaSignature: string;
    /**
     * Lookup389: pallet_airdrop_claims::utils::Sr25519Signature
     **/
    PalletAirdropClaimsUtilsSr25519Signature: string;
    /**
     * Lookup395: pallet_airdrop_claims::StatementKind
     **/
    PalletAirdropClaimsStatementKind: {
        _enum: string[];
    };
    /**
     * Lookup396: pallet_roles::pallet::Call<T>
     **/
    PalletRolesCall: {
        _enum: {
            create_profile: {
                profile: string;
                maxActiveServices: string;
            };
            update_profile: {
                updatedProfile: string;
            };
            delete_profile: string;
            chill: string;
            unbond_funds: {
                amount: string;
            };
            withdraw_unbonded: string;
            payout_stakers: {
                validatorStash: string;
                era: string;
            };
            set_min_restaking_bond: {
                minRestakingBond: string;
            };
        };
    };
    /**
     * Lookup397: pallet_roles::profile::Profile<T>
     **/
    PalletRolesProfile: {
        _enum: {
            Independent: string;
            Shared: string;
        };
    };
    /**
     * Lookup398: pallet_roles::profile::IndependentRestakeProfile<T>
     **/
    PalletRolesProfileIndependentRestakeProfile: {
        records: string;
    };
    /**
     * Lookup400: pallet_roles::profile::Record<T>
     **/
    PalletRolesProfileRecord: {
        role: string;
        amount: string;
    };
    /**
     * Lookup403: pallet_roles::profile::SharedRestakeProfile<T>
     **/
    PalletRolesProfileSharedRestakeProfile: {
        records: string;
        amount: string;
    };
    /**
     * Lookup404: pallet_jobs::module::Call<T>
     **/
    PalletJobsModuleCall: {
        _enum: {
            submit_job: {
                job: string;
            };
            submit_job_result: {
                roleType: string;
                jobId: string;
                result: string;
            };
            withdraw_rewards: string;
            report_inactive_validator: {
                roleType: string;
                jobId: string;
                validator: string;
                offence: string;
                signatures: string;
            };
            set_permitted_caller: {
                roleType: string;
                jobId: string;
                newPermittedCaller: string;
            };
            set_time_fee: {
                newFee: string;
            };
            submit_misbehavior: {
                misbehavior: string;
            };
            extend_job_result_ttl: {
                roleType: string;
                jobId: string;
                extendBy: string;
            };
        };
    };
    /**
     * Lookup405: tangle_primitives::jobs::JobResult<tangle_runtime::MaxParticipants, tangle_runtime::MaxKeyLen, tangle_runtime::MaxSignatureLen, tangle_runtime::MaxDataLen, tangle_runtime::MaxProofLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsJobResult: {
        _enum: {
            DKGPhaseOne: string;
            DKGPhaseTwo: string;
            DKGPhaseThree: string;
            DKGPhaseFour: string;
            ZkSaaSPhaseOne: string;
            ZkSaaSPhaseTwo: string;
        };
    };
    /**
     * Lookup406: tangle_runtime::MaxKeyLen
     **/
    TangleRuntimeMaxKeyLen: string;
    /**
     * Lookup407: tangle_runtime::MaxSignatureLen
     **/
    TangleRuntimeMaxSignatureLen: string;
    /**
     * Lookup408: tangle_runtime::MaxDataLen
     **/
    TangleRuntimeMaxDataLen: string;
    /**
     * Lookup409: tangle_runtime::MaxProofLen
     **/
    TangleRuntimeMaxProofLen: string;
    /**
     * Lookup410: tangle_primitives::jobs::tss::DKGTSSKeySubmissionResult<tangle_runtime::MaxKeyLen, tangle_runtime::MaxParticipants, tangle_runtime::MaxSignatureLen>
     **/
    TanglePrimitivesJobsTssDkgtssKeySubmissionResult: {
        signatureScheme: string;
        key: string;
        participants: string;
        signatures: string;
        threshold: string;
    };
    /**
     * Lookup411: tangle_primitives::jobs::tss::DigitalSignatureScheme
     **/
    TanglePrimitivesJobsTssDigitalSignatureScheme: {
        _enum: string[];
    };
    /**
     * Lookup418: tangle_primitives::jobs::tss::DKGTSSSignatureResult<tangle_runtime::MaxDataLen, tangle_runtime::MaxKeyLen, tangle_runtime::MaxSignatureLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsTssDkgtssSignatureResult: {
        signatureScheme: string;
        derivationPath: string;
        data: string;
        signature: string;
        verifyingKey: string;
    };
    /**
     * Lookup420: tangle_primitives::jobs::tss::DKGTSSKeyRefreshResult
     **/
    TanglePrimitivesJobsTssDkgtssKeyRefreshResult: {
        signatureScheme: string;
    };
    /**
     * Lookup421: tangle_primitives::jobs::tss::DKGTSSKeyRotationResult<tangle_runtime::MaxKeyLen, tangle_runtime::MaxSignatureLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsTssDkgtssKeyRotationResult: {
        phaseOneId: string;
        newPhaseOneId: string;
        newKey: string;
        key: string;
        signature: string;
        signatureScheme: string;
        derivationPath: string;
    };
    /**
     * Lookup422: tangle_primitives::jobs::zksaas::ZkSaaSCircuitResult<tangle_runtime::MaxParticipants>
     **/
    TanglePrimitivesJobsZksaasZkSaaSCircuitResult: {
        jobId: string;
        participants: string;
    };
    /**
     * Lookup425: tangle_primitives::jobs::zksaas::ZkSaaSProofResult<tangle_runtime::MaxProofLen>
     **/
    TanglePrimitivesJobsZksaasZkSaaSProofResult: {
        _enum: {
            Arkworks: string;
            Circom: string;
        };
    };
    /**
     * Lookup426: tangle_primitives::jobs::zksaas::ArkworksProofResult<tangle_runtime::MaxProofLen>
     **/
    TanglePrimitivesJobsZksaasArkworksProofResult: {
        proof: string;
    };
    /**
     * Lookup428: tangle_primitives::jobs::zksaas::CircomProofResult<tangle_runtime::MaxProofLen>
     **/
    TanglePrimitivesJobsZksaasCircomProofResult: {
        proof: string;
    };
    /**
     * Lookup429: tangle_primitives::jobs::ValidatorOffenceType
     **/
    TanglePrimitivesJobsValidatorOffenceType: {
        _enum: string[];
    };
    /**
     * Lookup430: tangle_primitives::misbehavior::MisbehaviorSubmission
     **/
    TanglePrimitivesMisbehaviorMisbehaviorSubmission: {
        roleType: string;
        offender: string;
        jobId: string;
        justification: string;
    };
    /**
     * Lookup431: tangle_primitives::misbehavior::MisbehaviorJustification
     **/
    TanglePrimitivesMisbehaviorMisbehaviorJustification: {
        _enum: {
            DKGTSS: string;
            ZkSaaS: string;
        };
    };
    /**
     * Lookup432: tangle_primitives::misbehavior::DKGTSSJustification
     **/
    TanglePrimitivesMisbehaviorDkgtssJustification: {
        _enum: {
            DfnsCGGMP21: string;
            ZCashFrost: string;
        };
    };
    /**
     * Lookup433: tangle_primitives::misbehavior::dfns_cggmp21::DfnsCGGMP21Justification
     **/
    TanglePrimitivesMisbehaviorDfnsCggmp21DfnsCGGMP21Justification: {
        _enum: {
            Keygen: {
                participants: string;
                t: string;
                reason: string;
            };
            KeyRefresh: {
                participants: string;
                t: string;
                reason: string;
            };
            Signing: {
                participants: string;
                t: string;
                reason: string;
            };
        };
    };
    /**
     * Lookup435: tangle_primitives::misbehavior::dfns_cggmp21::KeygenAborted
     **/
    TanglePrimitivesMisbehaviorDfnsCggmp21KeygenAborted: {
        _enum: {
            InvalidDecommitment: {
                round1: string;
                round2a: string;
            };
            InvalidSchnorrProof: {
                round2a: string;
                round3: string;
            };
            FeldmanVerificationFailed: {
                round2a: string;
                round2b: string;
            };
            InvalidDataSize: {
                round2a: string;
            };
        };
    };
    /**
     * Lookup436: tangle_primitives::misbehavior::SignedRoundMessage
     **/
    TanglePrimitivesMisbehaviorSignedRoundMessage: {
        sender: string;
        message: string;
        signature: string;
    };
    /**
     * Lookup438: tangle_primitives::misbehavior::dfns_cggmp21::KeyRefreshAborted
     **/
    TanglePrimitivesMisbehaviorDfnsCggmp21KeyRefreshAborted: {
        _enum: {
            InvalidDecommitment: {
                round1: string;
                round2: string;
            };
            InvalidSchnorrProof: string;
            InvalidModProof: {
                reason: string;
                round2: string;
                round3: string;
            };
            InvalidFacProof: string;
            InvalidRingPedersenParameters: {
                round2: string;
            };
            InvalidX: string;
            InvalidXShare: string;
            InvalidDataSize: string;
            PaillierDec: string;
        };
    };
    /**
     * Lookup439: tangle_primitives::misbehavior::dfns_cggmp21::InvalidProofReason
     **/
    TanglePrimitivesMisbehaviorDfnsCggmp21InvalidProofReason: {
        _enum: {
            EqualityCheck: string;
            RangeCheck: string;
            Encryption: string;
            PaillierEnc: string;
            PaillierOp: string;
            ModPow: string;
            ModulusIsPrime: string;
            ModulusIsEven: string;
            IncorrectNthRoot: string;
            IncorrectFourthRoot: string;
        };
    };
    /**
     * Lookup440: tangle_primitives::misbehavior::dfns_cggmp21::SigningAborted
     **/
    TanglePrimitivesMisbehaviorDfnsCggmp21SigningAborted: {
        _enum: string[];
    };
    /**
     * Lookup441: tangle_primitives::misbehavior::zcash_frost::ZCashFrostJustification
     **/
    TanglePrimitivesMisbehaviorZcashFrostZCashFrostJustification: {
        _enum: {
            Keygen: {
                participants: string;
                t: string;
                reason: string;
            };
            Signing: {
                participants: string;
                t: string;
                reason: string;
            };
        };
    };
    /**
     * Lookup442: tangle_primitives::misbehavior::zcash_frost::KeygenAborted
     **/
    TanglePrimitivesMisbehaviorZcashFrostKeygenAborted: {
        _enum: {
            InvalidProofOfKnowledge: {
                round1: string;
            };
            InvalidSecretShare: {
                round1: string;
                round2: string;
            };
        };
    };
    /**
     * Lookup443: tangle_primitives::misbehavior::zcash_frost::SigningAborted
     **/
    TanglePrimitivesMisbehaviorZcashFrostSigningAborted: {
        _enum: {
            InvalidSignatureShare: {
                round1: string;
                round2: string;
            };
        };
    };
    /**
     * Lookup444: tangle_primitives::misbehavior::ZkSaaSJustification
     **/
    TanglePrimitivesMisbehaviorZkSaaSJustification: string;
    /**
     * Lookup445: pallet_dkg::pallet::Call<T>
     **/
    PalletDkgCall: {
        _enum: {
            set_fee: {
                feeInfo: string;
            };
        };
    };
    /**
     * Lookup446: pallet_zksaas::pallet::Call<T>
     **/
    PalletZksaasCall: {
        _enum: {
            set_fee: {
                feeInfo: string;
            };
        };
    };
    /**
     * Lookup447: pallet_sudo::pallet::Error<T>
     **/
    PalletSudoError: {
        _enum: string[];
    };
    /**
     * Lookup450: pallet_balances::types::BalanceLock<Balance>
     **/
    PalletBalancesBalanceLock: {
        id: string;
        amount: string;
        reasons: string;
    };
    /**
     * Lookup451: pallet_balances::types::Reasons
     **/
    PalletBalancesReasons: {
        _enum: string[];
    };
    /**
     * Lookup454: pallet_balances::types::ReserveData<ReserveIdentifier, Balance>
     **/
    PalletBalancesReserveData: {
        id: string;
        amount: string;
    };
    /**
     * Lookup457: pallet_balances::types::IdAmount<tangle_runtime::RuntimeHoldReason, Balance>
     **/
    PalletBalancesIdAmountRuntimeHoldReason: {
        id: string;
        amount: string;
    };
    /**
     * Lookup458: tangle_runtime::RuntimeHoldReason
     **/
    TangleRuntimeRuntimeHoldReason: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            Preimage: string;
        };
    };
    /**
     * Lookup459: pallet_preimage::pallet::HoldReason
     **/
    PalletPreimageHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup462: pallet_balances::types::IdAmount<tangle_runtime::RuntimeFreezeReason, Balance>
     **/
    PalletBalancesIdAmountRuntimeFreezeReason: {
        id: string;
        amount: string;
    };
    /**
     * Lookup463: tangle_runtime::RuntimeFreezeReason
     **/
    TangleRuntimeRuntimeFreezeReason: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            NominationPools: string;
        };
    };
    /**
     * Lookup464: pallet_nomination_pools::pallet::FreezeReason
     **/
    PalletNominationPoolsFreezeReason: {
        _enum: string[];
    };
    /**
     * Lookup466: pallet_balances::pallet::Error<T, I>
     **/
    PalletBalancesError: {
        _enum: string[];
    };
    /**
     * Lookup468: pallet_transaction_payment::Releases
     **/
    PalletTransactionPaymentReleases: {
        _enum: string[];
    };
    /**
     * Lookup475: sp_consensus_babe::digests::PreDigest
     **/
    SpConsensusBabeDigestsPreDigest: {
        _enum: {
            __Unused0: string;
            Primary: string;
            SecondaryPlain: string;
            SecondaryVRF: string;
        };
    };
    /**
     * Lookup476: sp_consensus_babe::digests::PrimaryPreDigest
     **/
    SpConsensusBabeDigestsPrimaryPreDigest: {
        authorityIndex: string;
        slot: string;
        vrfSignature: string;
    };
    /**
     * Lookup477: sp_core::sr25519::vrf::VrfSignature
     **/
    SpCoreSr25519VrfVrfSignature: {
        preOutput: string;
        proof: string;
    };
    /**
     * Lookup478: sp_consensus_babe::digests::SecondaryPlainPreDigest
     **/
    SpConsensusBabeDigestsSecondaryPlainPreDigest: {
        authorityIndex: string;
        slot: string;
    };
    /**
     * Lookup479: sp_consensus_babe::digests::SecondaryVRFPreDigest
     **/
    SpConsensusBabeDigestsSecondaryVRFPreDigest: {
        authorityIndex: string;
        slot: string;
        vrfSignature: string;
    };
    /**
     * Lookup480: sp_consensus_babe::BabeEpochConfiguration
     **/
    SpConsensusBabeBabeEpochConfiguration: {
        c: string;
        allowedSlots: string;
    };
    /**
     * Lookup482: pallet_babe::pallet::Error<T>
     **/
    PalletBabeError: {
        _enum: string[];
    };
    /**
     * Lookup483: pallet_grandpa::StoredState<N>
     **/
    PalletGrandpaStoredState: {
        _enum: {
            Live: string;
            PendingPause: {
                scheduledAt: string;
                delay: string;
            };
            Paused: string;
            PendingResume: {
                scheduledAt: string;
                delay: string;
            };
        };
    };
    /**
     * Lookup484: pallet_grandpa::StoredPendingChange<N, Limit>
     **/
    PalletGrandpaStoredPendingChange: {
        scheduledAt: string;
        delay: string;
        nextAuthorities: string;
        forced: string;
    };
    /**
     * Lookup486: pallet_grandpa::pallet::Error<T>
     **/
    PalletGrandpaError: {
        _enum: string[];
    };
    /**
     * Lookup488: pallet_indices::pallet::Error<T>
     **/
    PalletIndicesError: {
        _enum: string[];
    };
    /**
     * Lookup494: pallet_democracy::types::ReferendumInfo<BlockNumber, frame_support::traits::preimages::Bounded<tangle_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
     **/
    PalletDemocracyReferendumInfo: {
        _enum: {
            Ongoing: string;
            Finished: {
                approved: string;
                end: string;
            };
        };
    };
    /**
     * Lookup495: pallet_democracy::types::ReferendumStatus<BlockNumber, frame_support::traits::preimages::Bounded<tangle_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
     **/
    PalletDemocracyReferendumStatus: {
        end: string;
        proposal: string;
        threshold: string;
        delay: string;
        tally: string;
    };
    /**
     * Lookup496: pallet_democracy::types::Tally<Balance>
     **/
    PalletDemocracyTally: {
        ayes: string;
        nays: string;
        turnout: string;
    };
    /**
     * Lookup497: pallet_democracy::vote::Voting<Balance, sp_core::crypto::AccountId32, BlockNumber, MaxVotes>
     **/
    PalletDemocracyVoteVoting: {
        _enum: {
            Direct: {
                votes: string;
                delegations: string;
                prior: string;
            };
            Delegating: {
                balance: string;
                target: string;
                conviction: string;
                delegations: string;
                prior: string;
            };
        };
    };
    /**
     * Lookup501: pallet_democracy::types::Delegations<Balance>
     **/
    PalletDemocracyDelegations: {
        votes: string;
        capital: string;
    };
    /**
     * Lookup502: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
     **/
    PalletDemocracyVotePriorLock: string;
    /**
     * Lookup505: pallet_democracy::pallet::Error<T>
     **/
    PalletDemocracyError: {
        _enum: string[];
    };
    /**
     * Lookup507: pallet_collective::Votes<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletCollectiveVotes: {
        index: string;
        threshold: string;
        ayes: string;
        nays: string;
        end: string;
    };
    /**
     * Lookup508: pallet_collective::pallet::Error<T, I>
     **/
    PalletCollectiveError: {
        _enum: string[];
    };
    /**
     * Lookup511: pallet_vesting::Releases
     **/
    PalletVestingReleases: {
        _enum: string[];
    };
    /**
     * Lookup512: pallet_vesting::pallet::Error<T>
     **/
    PalletVestingError: {
        _enum: string[];
    };
    /**
     * Lookup514: pallet_elections_phragmen::SeatHolder<sp_core::crypto::AccountId32, Balance>
     **/
    PalletElectionsPhragmenSeatHolder: {
        who: string;
        stake: string;
        deposit: string;
    };
    /**
     * Lookup515: pallet_elections_phragmen::Voter<sp_core::crypto::AccountId32, Balance>
     **/
    PalletElectionsPhragmenVoter: {
        votes: string;
        stake: string;
        deposit: string;
    };
    /**
     * Lookup516: pallet_elections_phragmen::pallet::Error<T>
     **/
    PalletElectionsPhragmenError: {
        _enum: string[];
    };
    /**
     * Lookup517: pallet_election_provider_multi_phase::ReadySolution<AccountId, MaxWinners>
     **/
    PalletElectionProviderMultiPhaseReadySolution: {
        supports: string;
        score: string;
        compute: string;
    };
    /**
     * Lookup519: pallet_election_provider_multi_phase::RoundSnapshot<sp_core::crypto::AccountId32, DataProvider>
     **/
    PalletElectionProviderMultiPhaseRoundSnapshot: {
        voters: string;
        targets: string;
    };
    /**
     * Lookup526: pallet_election_provider_multi_phase::signed::SignedSubmission<sp_core::crypto::AccountId32, Balance, tangle_runtime::NposSolution16>
     **/
    PalletElectionProviderMultiPhaseSignedSignedSubmission: {
        who: string;
        deposit: string;
        rawSolution: string;
        callFee: string;
    };
    /**
     * Lookup527: pallet_election_provider_multi_phase::pallet::Error<T>
     **/
    PalletElectionProviderMultiPhaseError: {
        _enum: string[];
    };
    /**
     * Lookup528: pallet_staking::StakingLedger<T>
     **/
    PalletStakingStakingLedger: {
        stash: string;
        total: string;
        active: string;
        unlocking: string;
        legacyClaimedRewards: string;
    };
    /**
     * Lookup530: pallet_staking::UnlockChunk<Balance>
     **/
    PalletStakingUnlockChunk: {
        value: string;
        era: string;
    };
    /**
     * Lookup533: pallet_staking::Nominations<T>
     **/
    PalletStakingNominations: {
        targets: string;
        submittedIn: string;
        suppressed: string;
    };
    /**
     * Lookup534: pallet_staking::ActiveEraInfo
     **/
    PalletStakingActiveEraInfo: {
        index: string;
        start: string;
    };
    /**
     * Lookup536: sp_staking::PagedExposureMetadata<Balance>
     **/
    SpStakingPagedExposureMetadata: {
        total: string;
        own: string;
        nominatorCount: string;
        pageCount: string;
    };
    /**
     * Lookup538: sp_staking::ExposurePage<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingExposurePage: {
        pageTotal: string;
        others: string;
    };
    /**
     * Lookup539: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
     **/
    PalletStakingEraRewardPoints: {
        total: string;
        individual: string;
    };
    /**
     * Lookup544: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
     **/
    PalletStakingUnappliedSlash: {
        validator: string;
        own: string;
        others: string;
        reporters: string;
        payout: string;
    };
    /**
     * Lookup548: pallet_staking::slashing::SlashingSpans
     **/
    PalletStakingSlashingSlashingSpans: {
        spanIndex: string;
        lastStart: string;
        lastNonzeroSlash: string;
        prior: string;
    };
    /**
     * Lookup549: pallet_staking::slashing::SpanRecord<Balance>
     **/
    PalletStakingSlashingSpanRecord: {
        slashed: string;
        paidOut: string;
    };
    /**
     * Lookup552: pallet_staking::pallet::pallet::Error<T>
     **/
    PalletStakingPalletError: {
        _enum: string[];
    };
    /**
     * Lookup556: sp_core::crypto::KeyTypeId
     **/
    SpCoreCryptoKeyTypeId: string;
    /**
     * Lookup557: pallet_session::pallet::Error<T>
     **/
    PalletSessionError: {
        _enum: string[];
    };
    /**
     * Lookup559: pallet_treasury::Proposal<sp_core::crypto::AccountId32, Balance>
     **/
    PalletTreasuryProposal: {
        proposer: string;
        value: string;
        beneficiary: string;
        bond: string;
    };
    /**
     * Lookup561: pallet_treasury::SpendStatus<AssetKind, AssetBalance, sp_core::crypto::AccountId32, BlockNumber, PaymentId>
     **/
    PalletTreasurySpendStatus: {
        assetKind: string;
        amount: string;
        beneficiary: string;
        validFrom: string;
        expireAt: string;
        status: string;
    };
    /**
     * Lookup562: pallet_treasury::PaymentState<Id>
     **/
    PalletTreasuryPaymentState: {
        _enum: {
            Pending: string;
            Attempted: {
                id: string;
            };
            Failed: string;
        };
    };
    /**
     * Lookup563: frame_support::PalletId
     **/
    FrameSupportPalletId: string;
    /**
     * Lookup564: pallet_treasury::pallet::Error<T, I>
     **/
    PalletTreasuryError: {
        _enum: string[];
    };
    /**
     * Lookup565: pallet_bounties::Bounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
     **/
    PalletBountiesBounty: {
        proposer: string;
        value: string;
        fee: string;
        curatorDeposit: string;
        bond: string;
        status: string;
    };
    /**
     * Lookup566: pallet_bounties::BountyStatus<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletBountiesBountyStatus: {
        _enum: {
            Proposed: string;
            Approved: string;
            Funded: string;
            CuratorProposed: {
                curator: string;
            };
            Active: {
                curator: string;
                updateDue: string;
            };
            PendingPayout: {
                curator: string;
                beneficiary: string;
                unlockAt: string;
            };
        };
    };
    /**
     * Lookup568: pallet_bounties::pallet::Error<T, I>
     **/
    PalletBountiesError: {
        _enum: string[];
    };
    /**
     * Lookup569: pallet_child_bounties::ChildBounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
     **/
    PalletChildBountiesChildBounty: {
        parentBounty: string;
        value: string;
        fee: string;
        curatorDeposit: string;
        status: string;
    };
    /**
     * Lookup570: pallet_child_bounties::ChildBountyStatus<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletChildBountiesChildBountyStatus: {
        _enum: {
            Added: string;
            CuratorProposed: {
                curator: string;
            };
            Active: {
                curator: string;
            };
            PendingPayout: {
                curator: string;
                beneficiary: string;
                unlockAt: string;
            };
        };
    };
    /**
     * Lookup571: pallet_child_bounties::pallet::Error<T>
     **/
    PalletChildBountiesError: {
        _enum: string[];
    };
    /**
     * Lookup572: pallet_bags_list::list::Node<T, I>
     **/
    PalletBagsListListNode: {
        id: string;
        prev: string;
        next: string;
        bagUpper: string;
        score: string;
    };
    /**
     * Lookup573: pallet_bags_list::list::Bag<T, I>
     **/
    PalletBagsListListBag: {
        head: string;
        tail: string;
    };
    /**
     * Lookup575: pallet_bags_list::pallet::Error<T, I>
     **/
    PalletBagsListError: {
        _enum: {
            List: string;
        };
    };
    /**
     * Lookup576: pallet_bags_list::list::ListError
     **/
    PalletBagsListListListError: {
        _enum: string[];
    };
    /**
     * Lookup577: pallet_nomination_pools::PoolMember<T>
     **/
    PalletNominationPoolsPoolMember: {
        poolId: string;
        points: string;
        lastRecordedRewardCounter: string;
        unbondingEras: string;
    };
    /**
     * Lookup582: pallet_nomination_pools::BondedPoolInner<T>
     **/
    PalletNominationPoolsBondedPoolInner: {
        commission: string;
        memberCounter: string;
        points: string;
        roles: string;
        state: string;
    };
    /**
     * Lookup583: pallet_nomination_pools::Commission<T>
     **/
    PalletNominationPoolsCommission: {
        current: string;
        max: string;
        changeRate: string;
        throttleFrom: string;
        claimPermission: string;
    };
    /**
     * Lookup586: pallet_nomination_pools::PoolRoles<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsPoolRoles: {
        depositor: string;
        root: string;
        nominator: string;
        bouncer: string;
    };
    /**
     * Lookup587: pallet_nomination_pools::RewardPool<T>
     **/
    PalletNominationPoolsRewardPool: {
        lastRecordedRewardCounter: string;
        lastRecordedTotalPayouts: string;
        totalRewardsClaimed: string;
        totalCommissionPending: string;
        totalCommissionClaimed: string;
    };
    /**
     * Lookup588: pallet_nomination_pools::SubPools<T>
     **/
    PalletNominationPoolsSubPools: {
        noEra: string;
        withEra: string;
    };
    /**
     * Lookup589: pallet_nomination_pools::UnbondPool<T>
     **/
    PalletNominationPoolsUnbondPool: {
        points: string;
        balance: string;
    };
    /**
     * Lookup594: pallet_nomination_pools::pallet::Error<T>
     **/
    PalletNominationPoolsError: {
        _enum: {
            PoolNotFound: string;
            PoolMemberNotFound: string;
            RewardPoolNotFound: string;
            SubPoolsNotFound: string;
            AccountBelongsToOtherPool: string;
            FullyUnbonding: string;
            MaxUnbondingLimit: string;
            CannotWithdrawAny: string;
            MinimumBondNotMet: string;
            OverflowRisk: string;
            NotDestroying: string;
            NotNominator: string;
            NotKickerOrDestroying: string;
            NotOpen: string;
            MaxPools: string;
            MaxPoolMembers: string;
            CanNotChangeState: string;
            DoesNotHavePermission: string;
            MetadataExceedsMaxLen: string;
            Defensive: string;
            PartialUnbondNotAllowedPermissionlessly: string;
            MaxCommissionRestricted: string;
            CommissionExceedsMaximum: string;
            CommissionExceedsGlobalMaximum: string;
            CommissionChangeThrottled: string;
            CommissionChangeRateNotAllowed: string;
            NoPendingCommission: string;
            NoCommissionCurrentSet: string;
            PoolIdInUse: string;
            InvalidPoolId: string;
            BondExtraRestricted: string;
            NothingToAdjust: string;
        };
    };
    /**
     * Lookup595: pallet_nomination_pools::pallet::DefensiveError
     **/
    PalletNominationPoolsDefensiveError: {
        _enum: string[];
    };
    /**
     * Lookup598: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<tangle_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, BlockNumber, tangle_runtime::OriginCaller, sp_core::crypto::AccountId32>
     **/
    PalletSchedulerScheduled: {
        maybeId: string;
        priority: string;
        call: string;
        maybePeriodic: string;
        origin: string;
    };
    /**
     * Lookup600: pallet_scheduler::pallet::Error<T>
     **/
    PalletSchedulerError: {
        _enum: string[];
    };
    /**
     * Lookup601: pallet_preimage::OldRequestStatus<sp_core::crypto::AccountId32, Balance>
     **/
    PalletPreimageOldRequestStatus: {
        _enum: {
            Unrequested: {
                deposit: string;
                len: string;
            };
            Requested: {
                deposit: string;
                count: string;
                len: string;
            };
        };
    };
    /**
     * Lookup603: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, Ticket>
     **/
    PalletPreimageRequestStatus: {
        _enum: {
            Unrequested: {
                ticket: string;
                len: string;
            };
            Requested: {
                maybeTicket: string;
                count: string;
                maybeLen: string;
            };
        };
    };
    /**
     * Lookup607: pallet_preimage::pallet::Error<T>
     **/
    PalletPreimageError: {
        _enum: string[];
    };
    /**
     * Lookup608: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
     **/
    SpStakingOffenceOffenceDetails: {
        offender: string;
        reporters: string;
    };
    /**
     * Lookup610: pallet_tx_pause::pallet::Error<T>
     **/
    PalletTxPauseError: {
        _enum: string[];
    };
    /**
     * Lookup613: pallet_im_online::pallet::Error<T>
     **/
    PalletImOnlineError: {
        _enum: string[];
    };
    /**
     * Lookup615: pallet_identity::types::Registration<Balance, MaxJudgements, pallet_identity::legacy::IdentityInfo<FieldLimit>>
     **/
    PalletIdentityRegistration: {
        judgements: string;
        deposit: string;
        info: string;
    };
    /**
     * Lookup624: pallet_identity::types::RegistrarInfo<Balance, sp_core::crypto::AccountId32, IdField>
     **/
    PalletIdentityRegistrarInfo: {
        account: string;
        fee: string;
        fields: string;
    };
    /**
     * Lookup626: pallet_identity::types::AuthorityProperties<bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletIdentityAuthorityProperties: {
        suffix: string;
        allocation: string;
    };
    /**
     * Lookup629: pallet_identity::pallet::Error<T>
     **/
    PalletIdentityError: {
        _enum: string[];
    };
    /**
     * Lookup630: pallet_utility::pallet::Error<T>
     **/
    PalletUtilityError: {
        _enum: string[];
    };
    /**
     * Lookup632: pallet_multisig::Multisig<BlockNumber, Balance, sp_core::crypto::AccountId32, MaxApprovals>
     **/
    PalletMultisigMultisig: {
        when: string;
        deposit: string;
        depositor: string;
        approvals: string;
    };
    /**
     * Lookup633: pallet_multisig::pallet::Error<T>
     **/
    PalletMultisigError: {
        _enum: string[];
    };
    /**
     * Lookup636: fp_rpc::TransactionStatus
     **/
    FpRpcTransactionStatus: {
        transactionHash: string;
        transactionIndex: string;
        from: string;
        to: string;
        contractAddress: string;
        logs: string;
        logsBloom: string;
    };
    /**
     * Lookup639: ethbloom::Bloom
     **/
    EthbloomBloom: string;
    /**
     * Lookup641: ethereum::receipt::ReceiptV3
     **/
    EthereumReceiptReceiptV3: {
        _enum: {
            Legacy: string;
            EIP2930: string;
            EIP1559: string;
        };
    };
    /**
     * Lookup642: ethereum::receipt::EIP658ReceiptData
     **/
    EthereumReceiptEip658ReceiptData: {
        statusCode: string;
        usedGas: string;
        logsBloom: string;
        logs: string;
    };
    /**
     * Lookup643: ethereum::block::Block<ethereum::transaction::TransactionV2>
     **/
    EthereumBlock: {
        header: string;
        transactions: string;
        ommers: string;
    };
    /**
     * Lookup644: ethereum::header::Header
     **/
    EthereumHeader: {
        parentHash: string;
        ommersHash: string;
        beneficiary: string;
        stateRoot: string;
        transactionsRoot: string;
        receiptsRoot: string;
        logsBloom: string;
        difficulty: string;
        number: string;
        gasLimit: string;
        gasUsed: string;
        timestamp: string;
        extraData: string;
        mixHash: string;
        nonce: string;
    };
    /**
     * Lookup645: ethereum_types::hash::H64
     **/
    EthereumTypesHashH64: string;
    /**
     * Lookup650: pallet_ethereum::pallet::Error<T>
     **/
    PalletEthereumError: {
        _enum: string[];
    };
    /**
     * Lookup651: pallet_evm::CodeMetadata
     **/
    PalletEvmCodeMetadata: {
        _alias: {
            size_: string;
            hash_: string;
        };
        size_: string;
        hash_: string;
    };
    /**
     * Lookup653: pallet_evm::pallet::Error<T>
     **/
    PalletEvmError: {
        _enum: string[];
    };
    /**
     * Lookup654: pallet_hotfix_sufficients::pallet::Error<T>
     **/
    PalletHotfixSufficientsError: {
        _enum: string[];
    };
    /**
     * Lookup656: pallet_airdrop_claims::pallet::Error<T>
     **/
    PalletAirdropClaimsError: {
        _enum: string[];
    };
    /**
     * Lookup657: pallet_roles::types::RestakingLedger<T>
     **/
    PalletRolesRestakingLedger: {
        stash: string;
        total: string;
        profile: string;
        roles: string;
        roleKey: string;
        unlocking: string;
        claimedRewards: string;
        maxActiveServices: string;
    };
    /**
     * Lookup663: pallet_roles::types::UnlockChunk<Balance>
     **/
    PalletRolesUnlockChunk: {
        value: string;
        era: string;
    };
    /**
     * Lookup667: pallet_roles::pallet::Error<T>
     **/
    PalletRolesError: {
        _enum: string[];
    };
    /**
     * Lookup668: tangle_primitives::jobs::PhaseResult<sp_core::crypto::AccountId32, BlockNumber, tangle_runtime::MaxParticipants, tangle_runtime::MaxKeyLen, tangle_runtime::MaxDataLen, tangle_runtime::MaxSignatureLen, tangle_runtime::MaxSubmissionLen, tangle_runtime::MaxProofLen, tangle_runtime::MaxAdditionalParamsLen>
     **/
    TanglePrimitivesJobsPhaseResult: {
        owner: string;
        result: string;
        ttl: string;
        permittedCaller: string;
        jobType: string;
    };
    /**
     * Lookup670: pallet_jobs::module::Error<T>
     **/
    PalletJobsModuleError: {
        _enum: string[];
    };
    /**
     * Lookup671: pallet_dkg::pallet::Error<T>
     **/
    PalletDkgError: {
        _enum: string[];
    };
    /**
     * Lookup672: pallet_zksaas::pallet::Error<T>
     **/
    PalletZksaasError: {
        _enum: string[];
    };
    /**
     * Lookup675: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T>
     **/
    FrameSystemExtensionsCheckNonZeroSender: string;
    /**
     * Lookup676: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
     **/
    FrameSystemExtensionsCheckSpecVersion: string;
    /**
     * Lookup677: frame_system::extensions::check_tx_version::CheckTxVersion<T>
     **/
    FrameSystemExtensionsCheckTxVersion: string;
    /**
     * Lookup678: frame_system::extensions::check_genesis::CheckGenesis<T>
     **/
    FrameSystemExtensionsCheckGenesis: string;
    /**
     * Lookup681: frame_system::extensions::check_nonce::CheckNonce<T>
     **/
    FrameSystemExtensionsCheckNonce: string;
    /**
     * Lookup682: frame_system::extensions::check_weight::CheckWeight<T>
     **/
    FrameSystemExtensionsCheckWeight: string;
    /**
     * Lookup683: pallet_transaction_payment::ChargeTransactionPayment<T>
     **/
    PalletTransactionPaymentChargeTransactionPayment: string;
    /**
     * Lookup685: tangle_runtime::Runtime
     **/
    TangleRuntimeRuntime: string;
};
export default _default;
