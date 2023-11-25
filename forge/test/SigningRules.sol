// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/SigningRules.sol";

contract VotableSigningRules is SigningRules {
    function _isVotableProposal(bytes32 phase1JobId, bytes memory phase1JobDetails, bytes memory phase2JobDetails) override pure internal returns (bool) {
        require(phase1JobId == 0x0, "Phase 1 job ID must be 0x0");
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
        bytes memory phase2JobDetails = "test";
        uint8 threshold = 1;
        bool useDemocracy = false;
        address[] memory voters = new address[](0);
        uint40 expiry = 1000;
        bytes32 proposalId = rules.calculateProposalId(phase1JobId, phase1JobDetails);
        bytes32 phase2JobHash = rules.calculatePhase2JobHash(proposalId, phase2JobDetails);

        rules.initialize(phase1JobId, phase1JobDetails, threshold, useDemocracy, voters, expiry);
        assertTrue(rules.initialized());
        assertTrue(rules.threshold(proposalId) == threshold);
        assertTrue(rules.useDemocracy(proposalId) == useDemocracy);
        assertTrue(rules.expiry(proposalId) == expiry);
        assertTrue(rules.useValidators(proposalId) == true);
    }
}
