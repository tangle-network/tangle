# GRANDPA Finality Algorithm

This sub crate consists of a GRANDPA finality proof verifier and prover.


###     [grandpa-primitives](primitives/src/lib.rs)

A `no_std` compatible crate which contains primitive types which are shared by the prover and verifier crates.


###     [grandpa-light-client-verifier](verifier/src/lib.rs)

A `no_std` compatible crate that exports a verification function for GRANDPA commitments, and parachain headers which have been finalized by the GRANDPA protocol.
<br />
The intention is for the verifier to be used in an IBC light client, but can be used as well in other trustless bridging protocols.

###     [grandpa-prover](prover/src/lib.rs)
This contains utility functions for assembling Grandpa proofs as well as parachain proofs from a running node, that can then be verified by the light-client crate.

## License

Apache-2.0 
