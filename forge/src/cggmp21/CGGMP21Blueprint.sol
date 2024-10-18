// SPDX-License-Identifier: GPL-3.0-only
// DO NOT USE THIS IN PRODUCTION, IT IS JUST FOR TESTING.
pragma solidity >=0.8.3;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "core/BlueprintServiceManagerBase.sol";

contract CGGMP21Blueprint is BlueprintServiceManagerBase {
    /// A Simple List of all Operator Ecdsa Public Keys
    struct Operator {
        address addr;
        bytes publicKey;
    }

    uint8 constant KEYGEN_JOB = 0;
    uint8 constant SIGNING_JOB = 1;

    Operator[] public blueprintOperators;

    struct Service {
        /// The id of the service
        uint64 id;
        /// The list of participants of the service
        Operator[] operators;
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

    function onRegister(bytes calldata operator, bytes calldata registrationInputs)
        public
        payable
        override
        onlyFromRootChain
    {
        address addr = operatorAddressFromPublicKey(operator);
        // add the participant to the list
        blueprintOperators.push(Operator(addr, operator));
    }

    function onRequest(uint64 serviceId, bytes[] calldata operators, bytes calldata requestInputs)
        public
        payable
        override
        onlyFromRootChain
    {
        // The requestInputs are empty, we don't need them.
        if (requestInputs.length != 0) {
            revert InvalidRequestInputs();
        }
        // Create the service
        Service storage service = services[serviceId];
        service.id = serviceId;

        for (uint256 i = 0; i < operators.length; i++) {
            address addr = operatorAddressFromPublicKey(operators[i]);
            service.operators.push(Operator(addr, operators[i]));
        }
    }

    function onJobCall(uint64 serviceId, uint8 job, uint64 jobCallId, bytes calldata inputs)
        public
        payable
        override
        onlyFromRootChain
    {
        // Job 0 is the Keygen Job
        if (job == KEYGEN_JOB) {
            // The inputs are the DKG threshold
            uint8 t = abi.decode(inputs, (uint8));
            uint256 n = services[serviceId].operators.length;
            // verify the DKG threshold is valid
            if (t == 0 || t > n) {
                revert InvalidDKGThreshold();
            }
            // set the DKG threshold of the service
            keygens[serviceId][jobCallId] = t;
        } else if (job == SIGNING_JOB) {
            // inputs are keygenJobCallId and message hash (32 bytes)
            (uint64 keygenJobCallId, bytes32 _message) = abi.decode(inputs, (uint64, bytes32));
            // verify the keygen job exists
            if (keygens[serviceId][keygenJobCallId] == 0) {
                revert KeygenJobNotFound();
            }
        } else {
            revert InvalidJob();
        }
    }

    function operatorAddressFromPublicKey(bytes calldata publicKey) public pure returns (address) {
        return address(uint160(uint256(keccak256(publicKey))));
    }
}
