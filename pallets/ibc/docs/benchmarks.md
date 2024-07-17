## Benchmarking implementation

For `transfer` and `upgrade_client` extrinsics we have pretty familiar substrate benchmarks, but for the `deliver` extrinsic
we implement a non-trivial benchmark for different light clients.  
To effectively benchmark the `deliver` extrinsic, we need to individually benchmark the processing of eachIBCmessage type using all available light clients,
this is because different light clients have different header and proof verification algorithms that would execute in the runtime with distinct speeds.

Also, all pallets plugged intoIBCare required to benchmark their callbacks and
provide a handler that implements the `CallbackWeight` trait which specifies methods that return the weight of each callback method.

The benchmarking infrastructure for the [`deliver`](/contracts/pallet-ibc/src/weight.rs#L178) extrinsic defines a weight router that collects a batch ofIBCmessages, and calculates the total weight of processing the message batch,
based on the light client needed for proof verification and the specific module callback for handling each message.

#### Writing benchmarks for a light client
The essence of this kind of benchmark is to get an estimate of how much it would cost to verify headers and verify state proofs  
**To benchmark header verification(MsgUpdateClient)**
- Create a valid MsgCreateClient and submit it in the benchmark preparatory code
- Construct a valid light client header
- Construct a MsgUpdateClient from the header and submit it in the actual benchmark code
- Verify that the benchmark was successful by checking if the UpdateClient event was emitted
  Note: the benchmark should be dependent on the number of signatures to be verified in the header
  The pseudocode below describes roughly how the benchmark should look like
```rust
   update_client {
        let i in 0..5;
        let ctx = crate::routing::Context::default();
        let (client_state, consensus_state) = create_initial_client_state_and_consensus_state(); 
        let msg = MsgCreateClient {
            client_state,
            consensus_state,
            signer: Default::default()
        };
        let msg = Any {
            type_url: msg.type_url().to_string(),
            value: msg.encode_vec()
        };
        ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();
        let client_id = get_client_id(); // Get the of the newly created client
        let client_message = create_client_update_message(i); // where i is the number of signatures to be verified in the created header
        let msg = {
            client_id,
            client_message,
            signer: Default::default()
        };

        let msg = Any {
            type_url: msg.type_url().to_string(),
            value: msg.encode_vec()
        };

    }: deliver(RawOrigin::Signed(caller), vec![msg])
        // Assert that UpdateClientEvent was deposited
    }
```
**To benchmark an IBC message type with a light client**
- Create the light client and add it to storage using the context APIs
- Add the necessary values to storage that are required for the benchmark to pass
- Create a mock state tree for the counterparty chain and commit the values needed to prove the message type that is being benchmarked
    - In case of tendermint client the mock state tree will be an AVL tree, for Grandpa client the mock state tree will be a patricia merke trie etc.
- Extract the root for this tree
- Store a consensus state for the light client created above with this extracted root as the commitment root
- Construct theIBCmessage with a proof extracted from the mock state tree
- Assert that the message was processed successfully  
  Pseudocode demonstrating this process
  The following sample is meant to benchmark the channel open ack message type
```rust
   channel_open_ack {
        let mut ctx = routing::Context::<T>::new();
        let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
        pallet_timestamp::Pallet::<T>::set_timestamp(now);
        // Create initial client state and consensus state
        let (mock_client_state, mock_cs_state) = create_initial_client_state_and_consensus_state();
        let client_id = ClientId::new(&mock_client_state.client_type(), 0).unwrap();
        let counterparty_client_id = ClientId::new("11-beefy", 1).unwrap();
        ctx.store_client_type(client_id.clone(), mock_client_state.client_type()).unwrap();
        ctx.store_client_state(client_id.clone(), mock_client_state).unwrap();
        ctx.store_consensus_state(client_id.clone(), Height::new(0, 1), mock_cs_state).unwrap();

        // Successful processing of channel open ack requires an open connection and a channel end to exist with a state of INIT
        let (connection_id, connection_end) = get_open_connection();
        let (port_id, channel_id, channel_end) = get_channel_end_with_init_state(connection_id);
        ctx.store_connection(connection_id, connection_end);
        ctx.store_channel((port_id, channel_id), channel_end);
        
        // Generate a mock state tree of the counterparty chain
        // Insert a channel end that is in TRYOPEN state using theIBCkey path for channels
        // Extract the root and proof for the channel
        // Update the light client consensus state so it can have the required state root required to process
        // the proof that will be submitted
        let (counterparty_channel_id, proof, root): (ChannelId, Vec<u8>, Vec<u8>) = create_and_insert_values_in_mock_state_tree();
        let cs_state = construct_consensus_state_from_root(root);
        ctx.store_consensus_state(client_id, Height::new(0,2), cs_state);
        let msg = MsgChannelOpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version: ChannelVersion::new(pallet_example::VERSION.to_string()),
            proofs: Proofs::new(proof.try_into().unwrap(), None, None, None, Height::new(0, 2)).unwrap(),
            signer: Default::default() // Use a valid value here,
        };

        let msg = Any {
            type_url: msg.type_url().to_string(),
            value: msg.encode_vec()
        };
    }: deliver(RawOrigin::Signed(caller), vec![msg])
        // Assert that channel state is now open
    }
```
