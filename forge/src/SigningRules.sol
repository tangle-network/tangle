// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";

enum ProposalStatus {Inactive, Active, Passed, Executed, Cancelled}

struct Proposal {
    ProposalStatus _status;
    uint256 _yesVotes;      // bitmap, 256 maximum votes
    uint8   _yesVotesTotal;
    uint40  _proposedBlock; // 1099511627775 maximum block
}

abstract contract SigningRules {
    mapping (bytes32 => mapping (address => bool)) public isValidForwarder;
    mapping (bytes32 => address) public admins;
    mapping (bytes32 => address[]) public voters;
    mapping (bytes32 => uint8) public threshold;
    mapping (bytes32 => uint40) public expiry;
    mapping (bytes32 => bool) public useDemocracy;
    mapping (bytes32 => bool) public useValidators;

    // keccak256(proposalId, phase2JobHash) => Proposal
    mapping(bytes32 => Proposal) public _proposals;

    bool public initialized;
    // Limit voter number because proposal can fit only so much votes
    uint256 constant public MAX_VOTERS = 256;

    event ProposalEvent(
        ProposalStatus status,
        bytes32 proposalId,
        bytes32 phase2JobHash
    );
    event ProposalVote(
        ProposalStatus status,
        bytes32 proposalId,
        bytes32 phase2JobHash
    );
    event FailedHandlerExecution(
        bytes lowLevelData
    );

    modifier onlyAdmin(bytes32 id) {
        require(admins[id] == msg.sender, "Only admin can call this function");
        _;
    }

    function calculatePhase1ProposalId(bytes32 phase1JobId, bytes memory phase1JobDetails) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(phase1JobId, phase1JobDetails));
    }

    function calculatePhase2JobHash(bytes32 proposalId, bytes memory phase2JobDetails) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(proposalId, phase2JobDetails));
    }

    function initialize(
        bytes32 phase1JobId,
        bytes memory phase1JobDetails,
        uint8 _threshold,
        bool _useDemocracy,
        address[] memory _voters,
        uint40 _expiry
    ) external {
        require(_voters.length <= MAX_VOTERS, "Too many voters");
        require(initialized == false, "Already initialized");
        initialized = true;

        // Hash the job data to get the an ID for the job
        bytes32 phase1ProposalId = keccak256(abi.encodePacked(phase1JobId, phase1JobDetails));
        threshold[phase1ProposalId] = _threshold;
        useDemocracy[phase1ProposalId] = _useDemocracy;
        expiry[phase1ProposalId] = _expiry;
        admins[phase1ProposalId] = msg.sender;

        // If we have voters, add them to the list.
        if (_voters.length > 0) {
            voters[phase1ProposalId] = _voters;
        } else {
            // Otherwise, use the default list of being all validators ECDSA keys.
            useValidators[phase1ProposalId] = true;
            _refreshVoters(phase1ProposalId);
        }
    }

    /// @notice Refresh the list of voters for a proposal w/ validators
    /// @param phase1ProposalId ID of the proposal to refresh voters for.
    function refreshVoters(bytes32 phase1ProposalId) public onlyAdmin(phase1ProposalId) {
        _refreshVoters(phase1ProposalId);
    }

    /// @notice Set a forwarder to be used.
    /// @notice Only callable by an address that currently has the admin role.
    /// @param forwarder Forwarder address to be added.
    /// @param valid Decision for the specific forwarder.
    function adminSetForwarder(bytes32 proposalId, address forwarder, bool valid) external onlyAdmin(proposalId) {
        isValidForwarder[proposalId][forwarder] = valid;
    }

	function submitGovernanceProposal(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) public {
		// Validate the governance proposal
        bytes32 proposalId = keccak256(abi.encodePacked(phase1JobId, phase1JobDetails));
        bytes32 phase2JobHash = keccak256(abi.encodePacked(proposalId, phase2JobDetails));
        require(_proposals[phase2JobHash]._status != ProposalStatus.Executed, "Proposal must have been executed");
        require(useDemocracy[proposalId], "Proposal must allow using governance");
		// Submit the proposal to governance pallet
        _submitToDemocracyPallet(phase1JobId, phase1JobDetails, phase2JobDetails);
	}

	function voteProposal(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) public {
		// Validate the job/details are AUP
		require(_isVotableProposal(phase1JobId, phase1JobDetails, phase2JobDetails), "Proposal must be votable");
		// Check that we have received enough votes for the anchor update proposal.
        // Execute the proposal happens in `_voteProposal` if this vote tips the balance.
		bytes32 proposalId = keccak256(abi.encodePacked(phase1JobId, phase1JobDetails));
        bytes32 phase2JobHash = keccak256(abi.encodePacked(proposalId, phase2JobDetails));
        _voteProposal(proposalId, phase2JobHash);
	}

    /// --------------------------------------------------------------------------------------- ///
    /// ------------------------------------- Internals --------------------------------------- ///
    /// --------------------------------------------------------------------------------------- ///

    /// @notice When called, {_msgSender()} will be marked as voting in favor of proposal.
    /// @param phase1ProposalId ID of the proposal to vote on.
    /// @notice Proposal must not have already been passed or executed.
    /// @notice {_msgSender()} must not have already voted on proposal.
    /// @notice Emits {ProposalEvent} event with status indicating the proposal status.
    /// @notice Emits {ProposalVote} event.
    function _voteProposal(bytes32 phase1ProposalId, bytes32 phase2JobHash) internal {
        Proposal storage proposal = _proposals[phase2JobHash];
        if (proposal._status == ProposalStatus.Passed) {
            _executeProposal(phase1ProposalId, phase2JobHash);
            return;
        }

        address sender = _msgSender(phase1ProposalId);
        
        require(uint(proposal._status) <= 1, "proposal already executed/cancelled");
        require(!_hasVoted(phase1ProposalId, phase2JobHash, sender), "relayer already voted");

        if (proposal._status == ProposalStatus.Inactive) {
            _proposals[phase2JobHash] = Proposal({
                _status : ProposalStatus.Active,
                _yesVotes : 0,
                _yesVotesTotal : 0,
                _proposedBlock : uint40(block.number) // Overflow is desired.
            });

            emit ProposalEvent(ProposalStatus.Active, phase1ProposalId, phase2JobHash);
        } else if (uint40(block.number - proposal._proposedBlock) > expiry[phase1ProposalId]) {
            // if the number of blocks that has passed since this proposal was
            // submitted exceeds the expiry threshold set, cancel the proposal
            proposal._status = ProposalStatus.Cancelled;

            emit ProposalEvent(ProposalStatus.Cancelled, phase1ProposalId, phase2JobHash);
        }

        if (proposal._status != ProposalStatus.Cancelled) {
            proposal._yesVotes = (proposal._yesVotes | _voterBit(phase1ProposalId, sender));
            proposal._yesVotesTotal++; // TODO: check if bit counting is cheaper.

            emit ProposalVote(proposal._status, phase1ProposalId, phase2JobHash);

            // Finalize if _relayerThreshold has been reached
            if (proposal._yesVotesTotal >= threshold[phase1ProposalId]) {
                proposal._status = ProposalStatus.Passed;
                emit ProposalEvent(ProposalStatus.Passed, phase1ProposalId, phase2JobHash);
            }
        }
        _proposals[phase1ProposalId] = proposal;

        if (proposal._status == ProposalStatus.Passed) {
            _executeProposal(phase1ProposalId, phase2JobHash);
        }
    }

    /// @notice Execute a proposal.
    /// @param phase1ProposalId ID of the proposal to execute.
    /// @notice Proposal must have Passed status.
    /// @notice Emits {ProposalEvent} event with status {Executed}.
    /// @notice Emits {FailedExecution} event with the failed reason.
    function _executeProposal(bytes32 phase1ProposalId, bytes32 phase2JobHash) internal {
        Proposal storage proposal = _proposals[phase2JobHash];
        require(proposal._status == ProposalStatus.Passed, "Proposal must have Passed status");
        proposal._status = ProposalStatus.Executed;                
        emit ProposalEvent(ProposalStatus.Executed, phase1ProposalId, phase2JobHash);
    }

    function _voterIndex(bytes32 phase1ProposalId, address voter) internal view returns (uint) {
        for (uint i = 0; i < voters[phase1ProposalId].length; i++) {
            if (voters[phase1ProposalId][i] == voter) {
                return i + 1;
            }
        }
        return MAX_VOTERS;
    }

    function _voterBit(bytes32 phase1ProposalId, address voter) internal view returns(uint) {
        return uint(1) << (_voterIndex(phase1ProposalId, voter) - 1);
    }

    function _hasVoted(bytes32 phase1ProposalId, bytes32 phase2JobHash, address voter) internal view returns(bool) {
        Proposal storage proposal = _proposals[phase2JobHash];
        return (_voterBit(phase1ProposalId, voter) & uint(proposal._yesVotes)) > 0;
    }

    function _msgSender(bytes32 proposalId) internal view returns (address) {
        address signer = msg.sender;
        if (msg.data.length >= 20 && isValidForwarder[proposalId][signer]) {
            assembly {
                signer := shr(96, calldataload(sub(calldatasize(), 20)))
            }
        }
        return signer;
    }

    /// --------------------------------------------------------------------------------------- ///
    /// -------------------------------------- Virtuals --------------------------------------- ///
    /// --------------------------------------------------------------------------------------- ///

    function _isVotableProposal(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) internal virtual returns (bool);
    function _refreshVoters(bytes32 proposalId) internal virtual;
    function _submitToDemocracyPallet(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) internal virtual;

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