// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../hooks/RegisterationHook.sol";
import "../hooks/RequestHook.sol";
import "../JobResultVerifier.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract CGGMP21RegistrationHook is RegistrationHook {
    /// A Simple List of all Participants Ecdsa Public Keys
    struct Participant {
        address addr;
        bytes publicKey;
    }

    Participant[] public participants;

    error InvalidRegistrationInputs();

    function onRegister(bytes calldata participant, bytes calldata registrationInputs)
        public
        payable
        override
        onlyRuntime
    {
        // The inputs are empty, we don't need them.
        if (registrationInputs.length != 0) {
            revert InvalidRegistrationInputs();
        }
        address addr = address(uint160(uint256(keccak256(participant))));
        // add the participant to the list
        participants.push(Participant(addr, participant));
    }
}

contract CGGMP21RequestHook is RequestHook {
    uint8 constant KEYGEN_JOB = 0;
    uint8 constant SIGNING_JOB = 1;

    struct Service {
        /// The id of the service
        uint64 id;
        /// The public keys of participants of the service
        bytes[] participants;
    }

    // Keygens
    // A mapping of serviceId -> JobCallId -> DKG Threshold
    mapping(uint64 => mapping(uint64 => uint8)) public keygens;

    // A mapping of serviceId to Service
    mapping(uint64 => Service) public services;

    error InvalidRequestInputs();
    error InvalidDKGThreshold();
    error KeygenJobNotFound();
    error InvalidJob();

    /// Stores the list of services that are requested
    function onRequest(uint64 serviceId, bytes[] calldata participants, bytes calldata requestInputs)
        public
        payable
        override
        onlyRuntime
    {
        // The requestInputs are empty, we don't need them.
        if (requestInputs.length != 0) {
            revert InvalidRequestInputs();
        }
        // initialize the service
        Service memory service;
        // set the id of the service
        service.id = serviceId;
        // set the participants of the service
        service.participants = participants;
        // store the service
        services[serviceId] = service;
    }

    function onJobCall(uint64 serviceId, uint8 job, uint64 jobCallId, bytes calldata inputs)
        public
        payable
        override
        onlyRuntime
    {
        // Job 0 is the Keygen Job
        if (job == KEYGEN_JOB) {
            // The inputs are the DKG threshold
            (uint8 t) = abi.decode(inputs, (uint8));
            uint256 n = services[serviceId].participants.length;
            // verify the DKG threshold is valid
            if (t == 0 || t > n) {
                revert InvalidDKGThreshold();
            }
            // set the DKG threshold of the service
            keygens[serviceId][jobCallId] = t;
        } else if (job == SIGNING_JOB) {
            // inputs are keygenJobCallId and message hash (32 bytes)
            (uint64 keygenJobCallId, bytes32 message) = abi.decode(inputs, (uint64, bytes32));
            // verify the keygen job exists
            if (keygens[serviceId][keygenJobCallId] == 0) {
                revert KeygenJobNotFound();
            }
        } else {
            revert InvalidJob();
        }
    }
}

contract CGGMP21JobResultVerifier is JobResultVerifier {
    uint8 constant KEYGEN_JOB = 0;
    uint8 constant SIGNING_JOB = 1;

    struct Keygen {
        uint64 jobCallId;
        uint8 t;
        bytes publicKey;
    }

    // A mapping of serviceId & JobCallId to Keygen
    mapping(uint64 => mapping(uint64 => Keygen)) public keygens;

    /// @dev Errors
    /// @dev InvalidJob The job is invalid
    error InvalidJob();
    /// @dev InvalidPublicKey The public key is invalid
    error InvalidPublicKey();
    /// @dev InvalidDKGThreshold The DKG threshold is invalid
    error InvalidDKGThreshold();
    /// @dev InvalidKeygenResult The keygen result is invalid
    error InvalidKeygenResult();
    /// @dev InvalidSignature The signature is invalid
    error InvalidSignature();
    /// @dev InvalidSigner The signer is invalid
    error InvalidSigner();

    function verify(
        uint64 serviceId,
        uint8 jobIndex,
        uint64 jobCallId,
        bytes calldata participant,
        bytes calldata inputs,
        bytes calldata outputs
    ) public override onlyRuntime {
        if (jobIndex == KEYGEN_JOB) {
            verifyKeygen(serviceId, jobCallId, inputs, outputs);
        } else if (jobIndex == SIGNING_JOB) {
            verifySigning(serviceId, jobCallId, inputs, outputs);
        } else {
            revert InvalidJob();
        }
    }

    function verifyKeygen(uint64 serviceId, uint64 jobCallId, bytes calldata inputs, bytes calldata outputs) internal {
        // The inputs are the DKG threshold
        (uint8 t) = abi.decode(inputs, (uint8));
        // The outputs are the public key
        bytes memory publicKey = outputs[0:33];
        // verify the public key is valid Ecdsa public key in the compressed format.
        if (publicKey.length != 33) {
            revert InvalidPublicKey();
        }
        // verify the DKG threshold is valid
        if (t == 0) {
            revert InvalidDKGThreshold();
        }
        // store the keygen
        keygens[serviceId][jobCallId] = Keygen(jobCallId, t, outputs);
    }

    function verifySigning(uint64 serviceId, uint64 jobCallId, bytes calldata inputs, bytes calldata outputs)
        internal
        view
    {
        // The inputs are the keygen result id (which is jobCallId) and the message hash.
        (uint64 keygenJobCallId, bytes32 message) = abi.decode(inputs, (uint64, bytes32));
        // The outputs are the signature
        bytes memory signature = outputs[0:65];
        // verify the signature is valid
        if (signature.length != 65) {
            revert InvalidSignature();
        }

        // get the keygen result
        Keygen memory keygen = keygens[serviceId][keygenJobCallId];
        // verify the keygen result exists
        if (keygen.jobCallId != keygenJobCallId) {
            revert InvalidKeygenResult();
        }
        // recover the public key from the signature
        address signer = ECDSA.recover(message, signature);
        // convert the public key to address format
        address keygenAddr = address(uint160(uint256(keccak256(keygen.publicKey))));
        // verify the public key is valid
        if (signer != keygenAddr) {
            revert InvalidSigner();
        }
    }
}
