## Pallet IBC

Pallet IBC is a thin wrapper around [`ibc-rs`](/ibc) that satisfies the runtime requirements for communicating over the IBC protocol from a substrate runtime.

- [`Config`](/contracts/pallet-ibc/src/lib.rs#L207)
- [`Call`](/contracts/pallet-ibc/src/lib.rs#L549)
- [`Pallet`](/contracts/pallet-ibc/src/lib.rs#L258)
- [`ICS23`](/contracts/pallet-ibc/docs/ics23.md)
- [`Routing`](/contracts/pallet-ibc/docs/routing.md)
- [`Benchmarking`](/contracts/pallet-ibc/docs/benchmarks.md)

### Dispatchable functions

- `deliver` - Receives a batch ofIBCtransactions and executes them in the same order as they were sent.
- `transfer` - This initiates an ics20 token transfer from the caller to an account on a connected chain via the ICS20 protocol
- `upgrade_client` - Sets the new consensus state and client state for client upgrades to be executed on connected chains
- `freeze_client` - Freezes a light client at a specified height.

### Adding Ibc to a substrate runtime

Implementing theIBCconfig trait for a substrate runtime
```rust
type AssetId = u128;

parameter_types! {
    pub const ExpectedBlockTime: u64 = 12000;
    pub const SpamProtectionDeposit: Balances = 10000;  
    pub const RelayChainId: light_client_commomn::RelayChain = light_client_commomn::RelayChain::Rococo;
    pub const NativeAssetId: AssetId = 1
    pub const MinimumConnectionDelay: u64 = 300; // 5 minutes
}

impl pallet_ibc::Config for Runtime {
    type TimeProvider = Timestamp;
    type Event = Event;
    type NativeCurrency = Balances;
    type NativeAssetId = NativeAssetId;
    type AssetId = AssetId;
    const PALLET_PREFIX: &'static [u8] = b"ibc/";
    const LIGHT_CLIENT_PROTOCOL: pallet_ibc::LightClientProtocol = pallet_ibc::LightClientProtocol::Grandpa; // Finality protocol this chain will be using
    type ExpectedBlockTime = ExpectedBlockTime; // Expected block time in milliseconds
    type Fungibles = Assets; // Add a module that implements the Transfer, Mutate and Inspect traits defined in frame_support::traits::fungibles
    type AccountIdConversion = ibc_primitives::IbcAccount;
    type IbcDenomToAssetIdConversion = AssetIdProcessor; // Add a module that implements DenomToAssetId
    type WeightInfo = crate::weights::pallet_ibc::WeightInfo<Self>;
    type Router = Router; // A type that implements ModuleRouter trait 
    type MinimumConnectionDelay: MinimumConnectionDelay;
    type ParaId = parachain_info::Pallet<Runtime>;
    type RelayChain = RelayChainId;
    type AdminOrigin = EnsureRoot<AccountId>;
    type SentryOrigin = EnsureRoot<AccountId>;
    type SpamProtectionDeposit = SpamProtectionDeposit;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
{
        // This is the module index hyperspace has been preconfigured to use,
        // if you want to use a different index or custom signed extrinsics
        // you'll have to assemble your own instance of hyperspace.
        Ibc: pallet_ibc = 9944,
}
```

### Terminology

- **ClientState:** This represents a connected chain's light client parameters, required for header verification.

- **ConsensusState:** This represents the timestamp and state root of a connected chain's light client at a particular block height, 
  it is extracted from the block header on successful processing of a ClientUpdate message, its major purpose is for proof verification.

- **PacketCommitment:** The sha256 hash of packet data, sequence,  timeout timestamp and timeout height committed to the runtime storage.

- **PacketReceipt:** An opaque value committed to storage after a message receive packet has been successfully processed.

- **PacketAcknowledgement:** A sha256 hash of packet acknowledgement committed to runtime storage by modules.

- **NextSequenceSend:** A u64 value representing the next packet sequence to be sent on a channel.

- **NextSequenceReceive:** A u64 value representing the next packet sequence to be received on a channel.

- **NextSequenceAck:** A u64 value representing the next acknowledgement sequence to be received on a channel.

### Packet and Acknowledgement Storage

Packets are stored offchain using the indexing API.  
**Note**: This pallet requires the **offchain indexing API** to be enabled when running the node.  
If not enabled the following rpc interfaces will return incomplete data.  
- `query_send_packets`
- `query_recv_packets`
- `query_events`

### ICS20 implementation

The IBC protocol defines an inter-chain token transfer standard that specifies how token transfers should be executed across connected chains.  
ICS20 is anIBCapplication which can be implemented as a standalone pallet nevertheless, it is implemented as a submodule of theIBCpallet [`here`](/contracts/pallet-ibc/src/ics20).  
The core ics20 logic is already implemented in [`ibc-rs`](/ibc/modules/src/applications/transfer), all that's required to integrate this is to implement the callback handlers for ics20 
and implement the [`Ics20Context`](/ibc/modules/src/applications/transfer/context.rs#l118) trait.

`Ics20Context` is dependent on an implementation of `frame_support::traits::fungibles::{Inspect, Mutate, Transfer}` for token registration, minting, transfers and burning.

### Rpc Interface

The [`Rpc interface`](/contracts/pallet-ibc/rpc/src/lib.rs) is designed to allow querying the state of theIBCstore with membership or non-membership proofs for the result.

- `query_send_packets` - Returns send packets for the provided sequences
- `query_recv_packets` - Returns receive packets for the provided sequences
- `query_client_update_time_and_height` - Returns the time and block height at which a client was updated
- `query_proof` - Returns the proof for the given key, it returns a membership proof if a value exists at that location in storage, otherwise a non-membership proof is returned
- `query_balance_with_address` - Returns the native balance of an address
- `query_client_state` - Returns the state of a client with a membership proof
- `query_client_consensus_state` - Returns the consensus state of a client with a membership proof
- `query_upgraded_client` -  Returns the state of an upgraded client with proof
- `query_upgraded_cons_state` - Returns the consensus state of an upgraded client with proof
- `query_clients` -  Returns the states of all clients on chain
- `query_connection` - Returns the connection end for the provided connection Id with a proof
- `query_connections` - Returns all the connection ends on chain
- `query_connection_using_client` - Returns the connections linked with a particular client
- `query_channel`- Returns the chanel end for then provided channel id with a proof
- `query_channel_client` - Returns the client linked to the provided channel id
- `query_connection_channels` -  Returns all channels linked to the provided connection id
- `query_channels` - Returns all channels on chain
- `query_packet_commitments` - Returns all packet commitments for a channel and port combination
- `query_packet_acknowledgements` - Returns all packet acknowledgements for a channel and port combination
- `query_unreceived_packets` - Filters out the sequences for packets that have not been received from a provided list of sequences
- `query_unreceived_acknowledgements` - Filters out the sequences for acknowledgements that have not been received from a provided list of sequences
- `query_next_seq_recv` - Returns the next sequence to be received on a channel with a proof
- `query_packet_commitment` - Returns a packet commitment with a proof
- `query_packet_acknowledgement` - Returns a packet acknowledgement commitment with a proof
- `query_packet_receipt` - Returns a packet receipt with either a membership or a non-membership proof.
- `query_denom_trace` - Query theIBCdenom trace for the provided local asset id
- `query_denom_traces` - Query allIBCdenom traces that exist on chain
- `query_events` - Returns allIBCevents from a block.

#### Runtime API

A set of runtime apis are specified to enable the rpc interface, these are defined here and should be implemented for the runtime for the rpc interface to work correctly.  
The runtime interface is defined [`here`](/contracts/pallet-ibc/runtime-api/src/lib.rs).  
Identical methods are implemented for the pallet to be called in the runtime interface implementation [`here`](/contracts/pallet-ibc/src/impls.rs#L112)

```rust

impl ibc_runtime_api::IbcRuntimeApi<Block> for Runtime {
    fn para_id() -> u32 {
        <Runtime as cumulus_pallet_parachain_system::Config>::SelfParaId::get().into()
    }

    fn child_trie_key() -> Vec<u8> {
        <Runtime as pallet_ibc::Config>::PALLET_PREFIX.to_vec()
    }

    fn query_send_packet_info(channel_id: Vec<u8>, port_id: Vec<u8>, seqs: Vec<u64>) -> Option<Vec<ibc_primitives::PacketInfo>> {
        Ibc::get_send_packet_info(channel_id, port_id, seqs).ok()
    }

    fn client_consensus_state(client_id: Vec<u8>, revision_number: u64, revision_height: u64, latest_cs: bool) -> Option<ibc_primitives::QueryConsensusStateResponse> {
        Ibc::consensus_state(client_id, revision_number, revision_height, latest_cs).ok()
    }

    // Implement remaining methods using theIBCidentical functions in the pallet implementation
  

    fn block_events(extrinsic_index: Option<u32>) -> Vec<pallet_ibc::events::IbcEvent> {
        let mut raw_events = frame_system::Pallet::<Self>::read_events_no_consensus().into_iter();
        if let Some(idx) = extrinsic_index {
            raw_events.find_map(|e| {
                let frame_system::EventRecord{ event, phase, ..} = *e;
                match (event, phase) {
                    (Event::Ibc(pallet_ibc::Event::Events{ events }), frame_system::Phase::ApplyExtrinsic(index)) if index == idx => Some(events),
                     _ => None
                }
            }).unwrap_or_default()
        }
        else { 
            raw_events.filter_map(|e| {
                let frame_system::EventRecord{ event, ..} = *e;

                match event {
                    Event::Ibc(pallet_ibc::Event::Events{ events }) => {
                        Some(events)
                    },
                    _ => None
                }
            }).flatten().collect()
        }
    }
}
```

### IBC Protocol coverage

- [x] ICS02 - Light client implementations  
   **Light clients supported**
  - [x] ICS07 - Tendermint Light Client
  - [x] ICS10 - Grandpa Light Client
  - [x] ICS11 - Beefy Light Client
  - [x] ICS13 - Near Light Client
  - [ ] Ethereum Light Client
- [x] ICS03 - Connections  
- [x] ICS04 - Channels and Ports  
- [x] ICS023 - Vector commitments  
- [x] ICS026 - Routing and callback handlers  
- [x] ICS020 - Fungible token transfer
- [ ] ICS027 - Interchain accounts
- [ ] ICS028 - Cross chain validation
- [ ] ICS029 - Fee payment
- [ ] ICS030 - Middleware
- [ ] ICS031 - Crosschain queries
- [ ] ICS721 - Non-fungible token transfer

### References

Official IBC specification docs [`ibc-spec`](https://github.com/cosmos/ibc)