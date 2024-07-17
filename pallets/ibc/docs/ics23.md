## Provable store implementation (ICS23)

The IBC protocol's ICS23 specification requires the host machine to be able to commit values into its storage using a set of standard keys and
subsequently be able to provide verifiable proofs of existence or non-existence for those values given these standard keys, this is called the provable store.  
The protocol requires the host to store specific values using the following keys:

- `ConsensusState` - `"clients/{identifier}/consensusStates/{height}"`
- `ClientState` - `"clients/{identifier}/clientState"`
- `ConnectionEnd` - `"connections/{identifier}"`
- `ChanelEnd` - `"channelEnds/ports/{identifier}/channels/{identifier}"`
- `PacketCommitment` - `"commitments/ports/{identifier}/channels/{identifier}/sequences/{sequence}"`
- `PacketReceipt` - `"receipts/ports/{identifier}/channels/{identifier}/sequences/{sequence}"`
- `PacketAcknowledgement` - `"acks/ports/{identifier}/channels/{identifier}/sequences/{sequence}"`
- `NextSequenceSend` - `"nextSequenceSend/ports/{identifier}/channels/{identifier}"`
- `NextSequenceReceive` - `"nextSequenceRecv/ports/{identifier}/channels/{identifier}"`
- `NextSequenceAcknowledge` - `"nextSequenceAck/ports/{identifier}/channels/{identifier}"`

The approach we take to implement this is to make use of the [`child trie API`](https://github.com/paritytech/substrate/blob/master/frame/support/src/storage/child.rs), the child trie API affords us a couple benefits, listed below
- The child trie API is a lower level storage API that allows us to insert values into our runtime storage using
  the custom key paths provided by the IBC protocol.
- it to allows the light client on the counterparty chain use the global state root of the host chain to verify state proofs.

**ICS23 Implementation**

For the [`ics23`](/ics23) implementation,
each member of the provable store is defined as a sub-module.  
A couple methods are implemented for each struct representing a provable store element, each method has a strongly typed interface.
These methods are `insert`, `get` and `remove` in some contexts.  
Notice that we have an `iter` method defined in some provable store implementations, usage of this function should be avoided in on-chain contexts as it increases the POV size of the parachain blocks.
- **Pruning** : Eventually there's going to be an implementation to effectively prune storage of outdated commitments to reduce bloat on our runtime storage.  
  Clients, Connections or channels should not be deleted after they are created.
