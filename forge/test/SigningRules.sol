// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/SigningRules.sol";
import { Proposal, ProposalStatus } from "../src/SigningRules.sol";

contract VotableSigningRules is SigningRules {
    function _isVotableProposal(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) override pure internal returns (bool) {
        require(phase1JobId != 0x0, "Phase 1 job ID must be 0x0");
        require(phase1JobDetails.length != 0, "Job details must be non-empty");
        require(phase2JobDetails.length != 0, "Job details must be non-empty");
        return true;
    }

    function _refreshVoters(bytes32 proposalId) override internal {
        // Do nothing
    }

    function _submitToDemocracyPallet(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) override internal {
        // Do nothing
    }
}

contract SigningRulesTest is Test {
    VotableSigningRules public rules;

    function setUp() public {
        rules = new VotableSigningRules();
    }

    function test_setup() public {
        bytes32 phase1JobId = "1";
        bytes memory phase1JobDetails = "test";
        uint8 threshold = 1;
        bool useDemocracy = false;
        address[] memory voters = new address[](0);
        uint40 expiry = 1000;
        bytes32 proposalId = rules.calculatePhase1ProposalId(phase1JobId, phase1JobDetails);
        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        assertTrue(rules.initialized());
        assertTrue(rules.threshold(proposalId) == threshold);
        assertTrue(rules.useDemocracy(proposalId) == useDemocracy);
        assertTrue(rules.expiry(proposalId) == expiry);
        assertTrue(rules.useValidators(proposalId) == true);
        assertTrue(rules.admins(proposalId) == address(this));
    }

    function test_submitAndVoteOnProposal() public {
        bytes32 phase1JobId = "1";
        bytes memory phase1JobDetails = "test";
        bytes memory phase2JobDetails = "test";
        uint8 threshold = 2;
        bool useDemocracy = false;
        address[] memory voters = new address[](2);
        voters[0] = vm.addr(1);
        voters[1] = vm.addr(2);
        uint40 expiry = 1000;
        bytes32 phase1ProposalId = rules.calculatePhase1ProposalId(phase1JobId, phase1JobDetails);
        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        vm.prank(vm.addr(1));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);
        assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Active);

        vm.expectRevert("relayer already voted");
        vm.prank(vm.addr(1));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);

        vm.prank(vm.addr(2));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);
        assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Passed);
    }

    function test_submitAndVote255Participants() public {
        bytes32 phase1JobId = "1";
        bytes memory phase1JobDetails = "test";
        bytes memory phase2JobDetails = "test";
        uint8 threshold = 255;
        bool useDemocracy = false;
        address[] memory voters = new address[](255);
        for (uint8 i = 0; i < 255; i++) {
            voters[i] = vm.addr(i + 1);
        }
        uint40 expiry = 1000;
        bytes32 phase1ProposalId = rules.calculatePhase1ProposalId(phase1JobId, phase1JobDetails);
        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        for (uint8 i = 0; i < 255; i++) {
            vm.prank(vm.addr(i + 1));
            rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);

            if (i < 254) {
                assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Active);
            }
        }
        assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Passed);
    }

    function test_submitVoteAndExpireProposal() public {
        bytes32 phase1JobId = "1";
        bytes memory phase1JobDetails = "test";
        bytes memory phase2JobDetails = "test";
        uint8 threshold = 2;
        bool useDemocracy = false;
        address[] memory voters = new address[](2);
        voters[0] = vm.addr(1);
        voters[1] = vm.addr(2);
        uint40 expiry = 10;
        uint nowBlockNumber = block.number;
        bytes32 phase1ProposalId = rules.calculatePhase1ProposalId(phase1JobId, phase1JobDetails);
        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        vm.prank(vm.addr(1));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);
        assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Active);

        vm.roll(nowBlockNumber + expiry + 1);
        vm.prank(vm.addr(2));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);
        assertTrue(rules.getProposalState(phase1ProposalId) == ProposalStatus.Cancelled);
        vm.expectRevert("proposal already executed/cancelled");
        vm.prank(vm.addr(2));
        rules.voteProposal(phase1JobId, phase1JobDetails, phase2JobDetails);
    }

    function test_adminFunctions() public {
        bytes32 phase1JobId = "1";
        bytes memory phase1JobDetails = "test";
        uint8 threshold = 2;
        bool useDemocracy = false;
        address[] memory voters = new address[](2);
        voters[0] = vm.addr(1);
        voters[1] = vm.addr(2);
        uint40 expiry = 1000;
        bytes32 phase1ProposalId = rules.calculatePhase1ProposalId(phase1JobId, phase1JobDetails);
        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        
        rules.adminSetForwarder(phase1ProposalId, vm.addr(100), true);
        assertTrue(rules.isValidForwarder(phase1ProposalId, vm.addr(100)));
        assertFalse(rules.isValidForwarder(phase1ProposalId, vm.addr(101)));

        vm.expectRevert("Only admin can call this function");
        vm.prank(vm.addr(1));
        rules.adminSetForwarder(phase1ProposalId, vm.addr(100), false);
    }
}
