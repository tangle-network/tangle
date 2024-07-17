## ICS20 - Fungible Token Transfer

Ics20 is an IBC application protocol that facilitates cross chain fungible token transfer.  
Ics20 requires that the host chain has a means to dynamically issue new tokens.  
Ics20 tokens are created on demand based on channel and port combinations, so the host chains must provide a means to  
mint tokens with arbitrary denominations.

### Ics20 Denominations

Denominations, also called denoms for short are a way to represent a unique cross chain token, to allow tracking of a token's flow across multiple chains.   
denoms are created by appending the destination channel and port to the base token denomination. Say a token with a base denomination of `Mars`  
is transferred from chain A on `channelA`, port `transfer` to `channelB` on chain B, port `transfer`, the token denomination  
on receipt at chain b becomes `channelB/transfer/Mars`.  

### Packet Data

Ics20 has a standard format for packet data, ics20 data must be serialized to json before sending.  
The standard format for a sample json serialized ics20 data is as shown below  
`receiver` should be any format that can be deserialized into an AccountID by the destination chain
```json
    {
        "denom": "channelB/transfer/Mars",
        "amount": "22000",
        "sender": "5F3sa2TJAWMqDhXG6jhV4N8ko9SxwGy8TpaNS1repo5EYjQX",
        "receiver": "5F3sa2TJAWMqDhXG6jhV4N8ko9SxwGy8TpaNS1repo5EYjQX"
    }
```
The rust equivalent of the ics20 packet data is defined by [`PacketData`](/applications/transfer/packet.rs#L11)

### Acknowledgement

Ics20 requires an acknowledgement packet to be sent and defines an acknowledgement success type.  
A successful acknowledgement is defined by the string [`AQ==`](/applications/transfer/acknowledgement.rs#L10), any other acknowledgement value is interpreted as an error.

### ICS20 Context

Ics20 context defines the methods required for the ics20 module callbacks to execute correctly.  
Correct execution of the Ics20 protocol hinges on the correct implementation of these methods.  

**BankKeeper**
- **send_coins** - The implementation of this method should send tokens from one account to another.
- **mint_coins** - This method should be implemented to increase the total supply of new token by the specified amount and send that amount to an account.
- **burn_coins** - This method should reduce the token balance of a user and decrease the total supply of such token by the specified amount.  

**Ics20Reader**
- **get_port** - Should return the ics20 port.
- **get_channel_escrow_address** - Should derive an account Id from the channel and port 
- **is_send_enabled** - Should return a boolean that indicates if ics20 send action is allowed.
- **is_receive_enabled** Should return a boolean that indicates if ics20 token receipt is allowed.

### ICS20 Callbacks

Ics20 callbacks contain the logic to handle different stages of the packet flow from send to acknowledgement.
The module callback logic for ICS20 is fully implemented [`here`](/applications/transfer/context.rs#L162).  

To initialize an ics20 transfer, the [`send_transfer`](/applications/transfer/relay/send_transfer.rs) method should be called. 
However, the caller should take care to revert all changes made by this call in case of a failed execution.

**Implementing ICS20 as an on-chain module**
```rust
// Context should have been defined prior to this

impl Ics20Reader for Context {
    type AccountId = AccountId;

    fn get_channel_escrow_address(
        &self,
        port_id: &PortId,
        channel_id: ChannelId,
    ) -> Result<<Self as Ics20Reader>::AccountId, Ics20Error> {
        // Add logic for deriving account Id from port and channel Id
    }

    fn get_port(&self) -> Result<ibc::core::ics24_host::identifier::PortId, Ics20Error> {Ok(PortId::transfer())}

    fn is_receive_enabled(&self) -> bool {
        // Add logic that determines if token receipt is allowed 
        false
    }

    fn is_send_enabled(&self) -> bool {
        // Add logic that determines if token transfer is allowed 
        false
    }
}

impl Ics20Keeper for Context {
    type AccountId = AccountId;
}

impl Ics20Context for Context {
    type AccountId = AccountId;
}

impl BankKeeper for Context {
    type AccountId = AccountId;
    fn mint_coins(
        &mut self,
        account: &Self::AccountId,
        amt: &ibc::applications::transfer::PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        // Add chain specific logic for issuing new tokens
        Ok(())
    }

    fn burn_coins(
        &mut self,
        account: &Self::AccountId,
        amt: &ibc::applications::transfer::PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        // Add chain specific logic for burning tokens
        Ok(())
    }

    fn send_coins(
        &mut self,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amt: &ibc::applications::transfer::PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        // Add chain specific logic for transferring tokens
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Ics20CallbackHandler;

// Implement the Module trait and call the callback functions defined in /ibc/modules/src/applications/transfer/context.rs#L162 appropriately
impl Module for Ics20CallbackHandler {
    fn on_chan_open_init(
        &mut self,
        output: &mut ModuleOutputBuilder,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_chan_open_init(&mut ctx, output, order, connection_hops, port_id, channel_id, counterparty, version)
    }

    fn on_chan_open_try(
        &mut self,
        output: &mut ModuleOutputBuilder,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
        counterparty_version: &Version,
    ) -> Result<Version, Ics04Error> {
        let mut ctx = Context::default();
        on_chan_open_try(&mut ctx, output, order, connection_hops, port_id, channel_id, counterparty, version, counterparty_version)
    }

    fn on_chan_open_ack(
        &mut self,
        output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty_version: &Version,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_chan_open_ack(&mut ctx, output, port_id, channel_id, counterparty_version)
    }

    fn on_chan_open_confirm(
        &mut self,
        output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_chan_open_confirm(&mut ctx, output, port_id, channel_id)
    }

    fn on_chan_close_init(
        &mut self,
        output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_chan_close_init(&mut ctx, output, port_id, channel_id)
    }

    fn on_chan_close_confirm(
        &mut self,
        output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_chan_close_confirm(&mut ctx, output, port_id, channel_id) 
    }

    fn on_recv_packet(
        &self,
        output: &mut ModuleOutputBuilder,
        packet: &Packet,
        relayer: &Signer,
    ) -> OnRecvPacketAck {
        let mut ctx = Context::default();
        on_recv_packet(&mut ctx, output, packet, relayer)
    }

    fn on_acknowledgement_packet(
        &mut self,
        output: &mut ModuleOutputBuilder,
        packet: &Packet,
        acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_acknowledgement_packet(&mut ctx, output, packet, acknowledgement, relayer)
    }

    fn on_timeout_packet(
        &mut self,
        output: &mut ModuleOutputBuilder,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), Ics04Error> {
        let mut ctx = Context::default();
        on_timeout_packet(&mut ctx, output, packet, relayer)
    }
}
    
```
