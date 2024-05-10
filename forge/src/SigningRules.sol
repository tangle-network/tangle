// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {JOBS_CONTRACT} from "./Jobs.sol";

enum ProposalStatus {
    Inactive,
    Active,
    Passed,
    Executed,
    Cancelled
}

struct Proposal {
    ProposalStatus _status;
    uint256 _yesVotes; // bitmap, 256 maximum votes
    uint8 _yesVotesTotal;
    uint40 _proposedBlock; // 1099511627775 maximum block
}

abstract contract SigningRules {
    mapping(uint64 => mapping(address => bool)) public isValidForwarder;
    mapping(uint64 => address) public admins;
    mapping(uint64 => address[]) public voters;
    mapping(uint64 => uint8) public threshold;
    mapping(uint64 => uint64) public expiry;
    mapping(uint64 => uint64) public ttl;
    mapping(uint64 => bool) public useDemocracy;
    mapping(uint64 => bool) public useValidators;

    // keccak256(proposalId, phase2JobHash) => Proposal
    mapping(bytes32 => Proposal) public _proposals;

    bool public initialized;
    // Limit voter number because proposal can fit only so much votes
    uint256 public constant MAX_VOTERS = 256;

    event ProposalEvent(ProposalStatus status, uint64 phase1JobId, bytes32 phase2JobHash);
    event ProposalVote(ProposalStatus status, uint64 phase1JobId, bytes32 phase2JobHash);
    event FailedHandlerExecution(bytes lowLevelData);

    modifier onlyAdmin(uint64 id) {
        require(admins[id] == msg.sender, "Only admin can call this function");
        _;
    }

    function calculatePhase2JobHash(uint64 phase1JobId, bytes memory phase2JobDetails) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(phase1JobId, phase2JobDetails));
    }

    function initialize(
        uint64 phase1JobId,
        uint8 _threshold,
        bool _useDemocracy,
        address[] memory _voters,
        uint64 _expiry,
        uint64 _ttl
    ) external {
        require(_voters.length <= MAX_VOTERS, "Too many voters");
        require(initialized == false, "Already initialized");
        initialized = true;

        // Hash the job data to get the an ID for the job
        threshold[phase1JobId] = _threshold;
        useDemocracy[phase1JobId] = _useDemocracy;
        expiry[phase1JobId] = _expiry;
        ttl[phase1JobId] = _ttl;
        admins[phase1JobId] = msg.sender;

        // If we have voters, add them to the list.
        if (_voters.length > 0) {
            voters[phase1JobId] = _voters;
        } else {
            // Otherwise, use the default list of being all validators ECDSA keys.
            useValidators[phase1JobId] = true;
            _refreshVoters(phase1JobId);
        }
    }

    /// @notice Refresh the list of voters for a proposal w/ validators
    /// @param phase1JobId ID of the proposal to refresh voters for.
    function refreshVoters(uint64 phase1JobId) public onlyAdmin(phase1JobId) {
        _refreshVoters(phase1JobId);
    }

    /// @notice Set a forwarder to be used.
    /// @notice Only callable by an address that currently has the admin role.
    /// @param forwarder Forwarder address to be added.
    /// @param valid Decision for the specific forwarder.
    function adminSetForwarder(uint64 phase1JobId, address forwarder, bool valid) external onlyAdmin(phase1JobId) {
        isValidForwarder[phase1JobId][forwarder] = valid;
    }

    function submitGovernanceProposal(uint64 phase1JobId, bytes memory phase2JobDetails) public {
        // Validate the governance proposal
        bytes32 phase2JobHash = keccak256(abi.encodePacked(phase1JobId, phase2JobDetails));
        require(_proposals[phase2JobHash]._status != ProposalStatus.Executed, "Proposal must have been executed");
        require(useDemocracy[phase1JobId], "Proposal must allow using governance");
        // Submit the proposal to governance pallet
        _submitToDemocracyPallet(phase1JobId, phase2JobDetails);
    }

    function voteProposal(uint64 phase1JobId, bytes memory phase2JobDetails) public {
        // Validate the job/details are AUP
        require(_isVotableProposal(phase1JobId, phase2JobDetails), "Proposal must be votable");
        // Check that we have received enough votes for the anchor update proposal.
        // Execute the proposal happens in `_voteProposal` if this vote tips the balance.
        _voteProposal(phase1JobId, phase2JobDetails);
    }

    /// --------------------------------------------------------------------------------------- ///
    /// ------------------------------------- Internals --------------------------------------- ///
    /// --------------------------------------------------------------------------------------- ///

    /// @notice When called, {_msgSender()} will be marked as voting in favor of proposal.
    /// @notice Proposal must not have already been passed or executed.
    /// @notice {_msgSender()} must not have already voted on proposal.
    /// @notice Emits {ProposalEvent} event with status indicating the proposal status.
    /// @notice Emits {ProposalVote} event.
    function _voteProposal(uint64 phase1JobId, bytes memory phase2JobDetails) internal {
        bytes32 phase2JobHash = keccak256(abi.encodePacked(phase1JobId, phase2JobDetails));
        Proposal storage proposal = _proposals[phase2JobHash];
        if (proposal._status == ProposalStatus.Passed) {
            _executeProposal(phase1JobId, phase2JobHash, phase2JobDetails);
            return;
        }

        address sender = _msgSender(phase1JobId);

        require(uint256(proposal._status) <= 1, "proposal already executed/cancelled");
        require(!_hasVoted(phase1JobId, phase2JobHash, sender), "relayer already voted");

        if (proposal._status == ProposalStatus.Inactive) {
            _proposals[phase2JobHash] = Proposal({
                _status: ProposalStatus.Active,
                _yesVotes: 0,
                _yesVotesTotal: 0,
                _proposedBlock: uint40(block.number) // Overflow is desired.
            });

            emit ProposalEvent(ProposalStatus.Active, phase1JobId, phase2JobHash);
        } else if (uint40(block.number - proposal._proposedBlock) > expiry[phase1JobId]) {
            // if the number of blocks that has passed since this proposal was
            // submitted exceeds the expiry threshold set, cancel the proposal
            proposal._status = ProposalStatus.Cancelled;

            emit ProposalEvent(ProposalStatus.Cancelled, phase1JobId, phase2JobHash);
        }

        if (proposal._status != ProposalStatus.Cancelled) {
            proposal._yesVotes = (proposal._yesVotes | _voterBit(phase1JobId, sender));
            proposal._yesVotesTotal++; // TODO: check if bit counting is cheaper.

            emit ProposalVote(proposal._status, phase1JobId, phase2JobHash);

            // Finalize if _relayerThreshold has been reached
            if (proposal._yesVotesTotal >= threshold[phase1JobId]) {
                proposal._status = ProposalStatus.Passed;
                emit ProposalEvent(ProposalStatus.Passed, phase1JobId, phase2JobHash);
            }
        }
        _proposals[phase2JobHash] = proposal;

        if (proposal._status == ProposalStatus.Passed) {
            _executeProposal(phase1JobId, phase2JobHash, phase2JobDetails);
        }
    }

    /// @notice Execute a proposal.
    /// @param phase1JobId ID of the job that the proposal is associated with.
    /// @notice Proposal must have Passed status.
    /// @notice Emits {ProposalEvent} event with status {Executed}.
    /// @notice Emits {FailedExecution} event with the failed reason.
    function _executeProposal(uint64 phase1JobId, bytes32 phase2JobHash, bytes memory phase2JobDetails) internal {
        Proposal storage proposal = _proposals[phase2JobHash];
        require(proposal._status == ProposalStatus.Passed, "Proposal must have Passed status");

        JOBS_CONTRACT.submitDkgPhaseTwoJob(
            expiry[phase1JobId], ttl[phase1JobId], phase1JobId, phase2JobDetails, bytes("")
        );

        proposal._status = ProposalStatus.Executed;
        emit ProposalEvent(ProposalStatus.Executed, phase1JobId, phase2JobHash);
    }

    function _voterIndex(uint64 phase1JobId, address voter) internal view returns (uint256) {
        for (uint256 i = 0; i < voters[phase1JobId].length; i++) {
            if (voters[phase1JobId][i] == voter) {
                return i + 1;
            }
        }
        return MAX_VOTERS;
    }

    function _voterBit(uint64 phase1JobId, address voter) internal view returns (uint256) {
        return uint256(1) << (_voterIndex(phase1JobId, voter) - 1);
    }

    function _hasVoted(uint64 phase1JobId, bytes32 phase2JobHash, address voter) internal view returns (bool) {
        Proposal storage proposal = _proposals[phase2JobHash];
        return (_voterBit(phase1JobId, voter) & uint256(proposal._yesVotes)) > 0;
    }

    function _msgSender(uint64 phase1JobId) internal view returns (address) {
        address signer = msg.sender;
        if (msg.data.length >= 20 && isValidForwarder[phase1JobId][signer]) {
            assembly {
                signer := shr(96, calldataload(sub(calldatasize(), 20)))
            }
        }
        return signer;
    }

    /// --------------------------------------------------------------------------------------- ///
    /// -------------------------------------- Virtuals --------------------------------------- ///
    /// --------------------------------------------------------------------------------------- ///

    function _isVotableProposal(uint64 phase1JobId, bytes memory phase2JobDetails) internal virtual returns (bool);
    function _refreshVoters(uint64 phase1JobId) internal virtual;
    function _submitToDemocracyPallet(uint64 phase1JobId, bytes memory phase2JobDetails) internal virtual;

    /// --------------------------------------------------------------------------------------- ///
    /// -------------------------------------- Helpers ---------------------------------------- ///
    /// --------------------------------------------------------------------------------------- ///

    function getProposalState(bytes32 phase2JobHash) public view returns (ProposalStatus) {
        return _proposals[phase2JobHash]._status;
    }

    function getProposalYesVotes(bytes32 phase2JobHash) public view returns (uint256) {
        return _proposals[phase2JobHash]._yesVotes;
    }

    function getProposalYesVotesTotal(bytes32 phase2JobHash) public view returns (uint8) {
        return _proposals[phase2JobHash]._yesVotesTotal;
    }
}
