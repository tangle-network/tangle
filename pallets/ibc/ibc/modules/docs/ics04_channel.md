## ICS04_CHANNEL

IBC channels are direct links between identical deployments of an application on connected chains.  
Channels are built on top of open connections.

### Channel Context

The channel context encapsulates all the storage requirements for channels in the context object.    
implement the [`ChannelReader`](/core/ics04_channel/context.rs#L26) and [`ChannelKeeper`](/core/ics04_channel/context.rs#L107) for the context object

```text
    impl ChannelReader for Context { ... }
    
    impl ChannelKeeper for Context { ... }  
```

### Channel Messages and Events

There are four messages that describe the channel open handshake process and two for the channel close process.

**MsgChannelOpenInit**

This message is submitted to start the channel handshake process, there's nothing to prove at this point, so it requires no proof.  
This message contains the proposed channel_end, port_id and signer.

```rust
    pub struct MsgChannelOpenInit {
        pub port_id: PortId,
        pub channel: ChannelEnd,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_open_init`](/core/ics04_channel/handler/chan_open_init.rs) handler.  
The `OpenInitChannel` event is emitted on successful processing of the event.


**MsgChannelOpenTry**

This message is submitted to a chain after `MsgChannelOpenInit` has been executed on its counterparty, this message  
requires a proof that indicates that the channel was initialized on the counterparty and committed to it's state trie.

```rust
    pub struct MsgChannelOpenTry {
        pub port_id: PortId,
        pub channel: ChannelEnd,
        pub counterparty_version: Version,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_open_try`](/core/ics04_channel/handler/chan_open_try.rs) handler.  
The `OpenTryChannel` event is emitted on successful processing of the event.


**MsgChannelOpenAck**

This message is submitted to a chain after `MsgChannelOpenTry` has been executed on its counterparty, this message  
requires a proof that indicates that the channel was initialized with a state of `TryOpen` on the counterparty and committed to it's state trie

```rust
    pub struct MsgChannelOpenAck {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub counterparty_channel_id: ChannelId,
        pub counterparty_version: Version,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_open_ack`](/core/ics04_channel/handler/chan_open_ack.rs) handler.  
The `OpenAckChannel` event is emitted on successful processing of the event.  
The channel becomes open if this message is processed successfully.

**MsgChannelOpenConfirm**

This message is submitted to a chain after `MsgChannelOpenAck` has been executed on its counterparty, this message  
requires a proof that indicates that the channel is `Open` on the counterparty.

```rust
    pub struct MsgChannelOpenConfirm {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_open_confirm`](/core/ics04_channel/handler/chan_open_confirm.rs) handler.  
The `OpenConfirmChannel` event is emitted on successful processing of the event.  
The channel becomes open if this message is processed successfully.

**MsgChannelCloseInit**

This message is submitted to start the channel close process, there's nothing to prove at this point, so it requires no proof.

```rust
    pub struct MsgChannelCloseInit {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_close_init`](/core/ics04_channel/handler/chan_close_init.rs) handler.  
The `CloseInitChannel` event is emitted on successful processing of the event.

**MsgChannelOpenConfirm**

This message is submitted to a chain after `MsgChannelCloseInit` has been executed on its counterparty, this message  
requires a proof that indicates that the channel state is `Close` on the counterparty.

```rust
    pub struct MsgChannelOpenConfirm {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```
This message is processed by the [`chan_close_confirm`](/core/ics04_channel/handler/chan_close_confirm.rs) handler.  
The `CloseConfirmChannel` event is emitted on successful processing of the event.  
The channel becomes closed if this message is processed successfully.

### Packets

Packets are means through which protocols built on top of core IBC exchange data, IBC application protocols determine  
how this data is serialized and deserialized.  
The packet data remains opaque to the core IBC protocol which facilitates its transmission across connections through channels.

**SendPacket**

This is not like other messages, it just involves creating a packet and depositing the packet commitment in the state  
and emitting the `SendPacketEvent`.
To handle packet creation call the [`send_packet`](/core/ics04_channel/handler/send_packet.rs) method.

**MsgReceivePacket**

This message is submitted to process a `SendPacket` from the counterparty chain, it is accompanied by a commitment  
membership proof, that proves the packet was committed to the counterparty state.  
This packet is handled by the function defined here [`here`](/core/ics04_channel/handler/recv_packet.rs#L41).  

```rust
    pub struct MsgRecvPacket {
	    pub packet: Packet,
	    pub proofs: Proofs,
	    pub signer: Signer,
    }
```

After verifying the validity of this packet, the top level dispatch function calls the `on_recv_packet`  
module callback through the router using the destination port id defined in the packet.  
A `ReceivePacket` event is emitted after a successful execution.

**MsgAcknowledgement**

This message is submitted to process a packet acknowledgement from the counterparty, it is accompanied by a commitment  
membership proof, that proves the acknowledgement was committed to the counterparty state.  
This packet is handled by the function defined here [`here`](/core/ics04_channel/handler/acknowledgement.rs#L29).  

```rust
    pub struct MsgAcknowledgement {
	    pub packet: Packet,
	    pub acknowledgement: Acknowledgement,
	    pub proofs: Proofs,
	    pub signer: Signer,
    }
```

After verifying the validity of this packet, the top level dispatch function calls the `on_acknowledgement_packet`  
module callback through the router using the source port id defined in the packet.  
An `AcknowledgePacket` event is emitted after a successful execution.

**MsgTimeout**

This message is submitted to the chain where a `SendPacket` originated if the said packet has not been acknowledged before the timeout height or timestamp  
specified in the packet have elapsed. It is accompanied by a non membership proof of receipt. This proof validates the statement that the packet was never  
received on the counterparty.  

```rust
    pub struct MsgTimeout {
	    pub packet: Packet,
	    pub next_sequence_recv: Sequence,
	    pub proofs: Proofs,
	    pub signer: Signer,
    }
```

This message serves as a way to ensure packets are not left hanging unattended for long periods of time.
This packet is handled by the function defined here [`here`](/core/ics04_channel/handler/timeout.rs#L29)    
After verifying the validity of this packet, the top level dispatch function calls the `on_timeout_packet` module callback through the router using the source port id defined in the packet.  
A `Timeout` event is emitted after a successful execution.

**MsgTimeoutOnClose**
This message is submitted to the chain where a `SendPacket` originated if the said packet has not been acknowledged before the channel on the counterparty is closed.  
It is accompanied by a non membership proof of receipt alongside a membership proof of the closed channel. This proof validates the statement that the packet was never  
received on the counterparty and that the channel has been closed.

```rust
    pub struct MsgTimeoutOnClose {
        pub packet: Packet,
        pub next_sequence_recv: Sequence,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```

This packet is handled by the function defined here [`here`](/core/ics04_channel/handler/timeout_on_close.rs#L29)    
After verifying the validity of this packet, the top level dispatch function calls the `on_timeout_packet` module callback through the router using the source port id defined in the packet.  
A `TimeoutOnClose` event is emitted after a successful execution.

### Ports

PortIds and ModuleIds are unique strings used to identify modules, the former is only used locally within the framework,  
while the latter is part of the ICS specification.  
The port requirements are defined by the [`PortReader`](/core/ics05_port/context.rs#L10)  
trait to be implemented on the `Context` object and provide a way for ports to map to ModuleIds.

Ports and ModuleIds should be statically defined.  