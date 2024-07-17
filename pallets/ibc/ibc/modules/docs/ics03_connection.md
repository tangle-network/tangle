## ICS03_CONNECTION

IBC connections are direct links between chains, it is recommended to have only one open connection between two  
specific chains, which translates to one connection per light client.  

### Connection Context

The connection context encapsulates all the storage requirements for connections in the context object.    
implement the [`ConnectionReader`](/core/ics03_connection/context.rs#L21)  
and [`ConnectionKeeper`](/core/ics03_connection/context.rs#L51) for the context object

```rust
    impl ConnectionReader for Context { ... }
    
    impl ConnectionKeeper for Context { ... }  
```

### Connection Messages and Events

There are four messages that describe the connection handshake process.  

**MsgConnectionOpenInit**

This message is submitted to start the connection handshake process, there's nothing to prove at this point, so it requires no proof.  
This message contains the connection parameters such as delay period, client_id, optional version parameter and some counterparty parameters

```rust
    pub struct MsgConnectionOpenInit {
	    pub client_id: ClientId,
	    pub counterparty: Counterparty,
	    pub version: Option<Version>,
	    pub delay_period: Duration,
	    pub signer: Signer,
    }
```
This message is processed by the [`conn_open_init`](/core/ics03_connection/handler/conn_open_init.rs) handler.  
The `OpenInitConnection` event is emitted on successful processing of the event.


**MsgConnectionOpenTry**

This message is submitted to a chain after `MsgConnectionOpenInit` has been executed on its counterparty, this message  
requires a connection proof that indicates that the connection was  
initialized on the counterparty and committed to it's state trie, it also requires a consensus proof so the host chain  
can verify that the counterparty has a valid consensus state for its light client committed to the counterparty state trie  
at the time of the handshake.

```rust
    pub struct MsgConnectionOpenTry<C: ClientTypes + Clone + Debug + PartialEq + Eq> {
        pub client_id: ClientId,
        pub client_state: Option<C::AnyClientState>,
        pub counterparty: Counterparty,
        pub counterparty_versions: Vec<Version>,
        // Contains connection and consensus proof
        pub proofs: Proofs,
        pub delay_period: Duration,
        pub signer: Signer,
    }
```
This message is processed by the [`conn_open_try`](/core/ics03_connection/handler/conn_open_try.rs) handler.  
The `OpenTryConnection` event is emitted on successful processing of the event.


**MsgConnectionOpenAck**

This message is submitted to a chain after `MsgConnectionOpenTry` has been executed on its counterparty, this message  
requires a connection proof that indicates that the connection was  
initialized with a state of `TryOpen` on the counterparty and committed to it's state trie, it also requires a consensus  
proof so the host chain can verify that the counterparty has a valid consensus state for its light client committed to the  
counterparty state trie at the time of the handshake.

```rust
    pub struct MsgConnectionOpenAck<C: ClientTypes> {
        pub connection_id: ConnectionId,
        pub counterparty_connection_id: ConnectionId,
        pub client_state: Option<C::AnyClientState>,
        // Contains connection and consensus proof
        pub proofs: Proofs,
        pub version: Version,
        pub signer: Signer,
    }
```
This message is processed by the [`conn_open_ack`](/core/ics03_connection/handler/conn_open_init.rs) handler.  
The `OpenAckConnection` event is emitted on successful processing of the event.  
The connection becomes open if this message is processed successfully.

**MsgConnectionOpenConfirm**

This message is submitted to a chain after `MsgConnectionOpenAck` has been executed on its counterparty, this message  
requires a connection proof that indicates that the connection is `Open` on the counterparty.  
it does not require a consensus proof.

```rust
    pub struct MsgConnectionOpenConfirm {
        pub connection_id: ConnectionId,
        pub proofs: Proofs,
        pub signer: Signer,
    }
```
This message is processed by the [`conn_open_confirm`](/core/ics03_connection/handler/conn_open_init.rs) handler.  
The `OpenConfirmConnection` event is emitted on successful processing of the event.  
The connection becomes open if this message is processed successfully.

**Note**
Connections cannot be closed