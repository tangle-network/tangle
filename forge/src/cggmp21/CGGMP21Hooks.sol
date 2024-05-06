// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../hooks/RegisterationHook.sol";
import "../hooks/RequestHook.sol";

contract CGGMP21RegistrationHook is RegistrationHook {
    /// A Simple List of all Participants Ecdsa Public Keys
    struct Participant {
      address addr;
      bytes publicKey;
    }

    Participant[] public participants;

    function onRegister(
      bytes calldata participant,
      bytes calldata registrationInputs
    ) public override payable onlyRuntime {
        // The inputs are empty, we don't need them.
        require(registrationInputs.length == 0, "CGGMP21RegistrationHook: Invalid registrationInputs");
        address addr = address(uint160(uint256(keccak256(participant))));
        // add the participant to the list
        participants.push(Participant(addr, participant));
    }
}

contract CGGMP21RequestHook is RequestHook {
    struct Service {
      /// The id of the service
      uint64 id;
      /// The public keys of participants of the service
      bytes[] participants;
      /// The DKG threshold.
      uint8 t;
    }

    mapping(uint64 => Service) public services;

    /// Stores the list of services that are requested
    function onRequest(
      uint64 serviceId,
      bytes[] calldata participants,
      bytes calldata requestInputs
    ) public override payable onlyRuntime {
      // The requestInputs are empty, we don't need them.
      require(requestInputs.length == 0, "CGGMP21RequestHook: Invalid requestInputs");
      // initialize the service
      Service memory service;
      // set the id of the service
      service.id = serviceId;
      // set the participants of the service
      service.participants = participants;
      // set the DKG threshold of the service
      service.t = 0;
      // store the service
      services[serviceId] = service;
    }

    function onJobCall(
      uint64 serviceId,
      uint8 job,
      uint64 jobCallId,
      bytes calldata inputs
    ) public override payable onlyRuntime {
      // Job 0 is the Keygen Job
      if (job == 0) {
        // The inputs are the DKG threshold
        (uint8 t) = abi.decode(inputs, (uint8));
        uint n = services[serviceId].participants.length;
        // verify the DKG threshold is valid
        require(t > 0 && t <= n, "CGGMP21RequestHook: Invalid DKG threshold");
        // set the DKG threshold of the service
        services[serviceId].t = t;
      }
      // Job 1 is the Signing Job
      else if (job == 1) {
        require(inputs.length == 32, "CGGMP21RequestHook: Invalid inputs must be 32 bytes");
      } else {
        revert("CGGMP21RequestHook: Invalid job");
      }
    }

}

