## Routing (ICS26) and callback handlers

The IBC protocol requires the existence of a router that routes packets to the correct module for processing based on the destination port.  
Each IBC application must define a unique module id and port id to be used in the module router.

### Ibc Handler

This pallet provides a public interface behind the [`ibc_primtives::IbcHandler`](/contracts/pallet-ibc/primitives/src/lib.rs#L373) trait, that allows modules to access the protocol.  
It provides methods for:
- Sending packets, opening and closing channels, performing ics20 transfers - `IbcHandler::handle_message`
- Writing Acknowledgements - `IbcHandler::write_acknowledgemnent`

**Defining an example IBC compliant pallet**
```rust
use ibc_primitives::IbcHandler as IbcHandlerT;
const PORT_ID: &'static str = "example";
const MODULE_ID: &'static str = "pallet_example";
pub trait Config: frame_system::Config {
    IbcHandler: IbcHandlerT<<Self as frame_system::Config>::AccountId>;
    WeightInfo: WeightInfo;
}

#[pallet::call]
impl<T: Config> Pallet<T> { 
    #[pallet::weight(0)]
    pub fn initiate_some_ibc_transaction(origin: OriginFor<T>, params: Params) -> DispatchResult {
        ensure_signed(origin)?;
        T::IbcHandler::handle_message(ibc_primtives::HandlerMessage::SendPacket {
            data: b"ping".to_vec(),
            timeout: ibc_primitives::Timeout::Offset {
                height: Some(params.timeout_height_offset),
                timestamp: Some(params.timeout_timestamp_offset),
            },
            port_id: port_id_from_bytes(PORT_ID.as_bytes().to_vec())
                .expect("Valid port id expected"),
            channel_id,
        })?;
        Ok(())
   }
}
   
#[derive(Clone, Eq, PartialEq)]
pub struct IbcModule<T: Config>(PhantomData<T>);

impl<T: Config> Default for IbcModule<T> {
     fn default() -> Self {
         Self(PhantomData)
     }
}

pub struct PalletExampleAcknowledgement(Vec<u8>);

impl AsRef<[u8]> for PalletExampleAcknowledgement { 
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl GenericAcknowledgement for PalletExampleAcknowledgement {}

impl<T: Config> core::fmt::Debug for IbcModule<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, MODULE_ID)
    }
}

// All these callbacks should be benchmarked
impl<T: Config + Send + Sync> Module for IbcModule<T> {
    /// This is called when a channel init message is processe/// If this callback fails the counterparty will not receive the channel_open_try message
    fn on_chan_open_init(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        _version: &Version,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }

    /// This is called after a channel_open_try message
    /// has been processed successfully, at this point, this module
    /// should verify that the counterparty's channel order, version and port matches what is expected 
    /// If this callback fails the counterparty will not recieve the channel_open_ack message
    fn on_chan_open_try(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        order: Order,
        _connection_hops: &[ConnectionId],
        port_id: &PortId,
        _channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
        counterparty_version: &Version,
    ) -> Result<Version, Ics04Error> {
        if counterparty_version.to_string() != *VERSION || version.to_string() != *VERSION { 
            return Err(Ics04Error::no_common_version())
        }

        if order != Order::Ordered {
	        return Err(Ics04Error::unknown_order_type(order.to_string()))
        }

       let example_port = PortId::from_str(PORT_ID).expect("PORT_ID is static and valid; qed");
       if counterparty.port_id() != &example_port || port_id != &ping_port {
	       return Err(Ics04Error::implementation_specific(format!(
	        "Invalid counterparty port {:?}",
	        counterparty.port_id()
	       )))
       }

        Ok(version.clone())
    }

    /// This is called after channel open acknowledgement is processed
    /// Execute any pallet specific logic that requires channel to be open
    /// If this callback fails the counterparty will not recieve the channel_open_confirm message
    fn on_chan_open_ack(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty_version: &Version,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }

    /// Called after channel open confirm is processed
    fn on_chan_open_confirm(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }

    /// Callled after channel close init messages is processed successfully
    /// If it fails channel close confirm will not be seen on the counterparty
    fn on_chan_close_init(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }

    /// Called when channel close is successfully processed
    /// Execute pallet specific logic that depends on channel closing
    fn on_chan_close_confirm(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }

    /// Called after message receive packet is successfully processed
    /// Execute pallet specific logic on packet data and
    /// write error or success Acknowledgement to storage
    fn on_recv_packet(
        &self,
        _output: &mut ModuleOutputBuilder,
        packet: &Packet,
        _relayer: &Signer,
    ) -> OnRecvPacketAck {
        // Do some custom logic and write acknowledgement
        let success = "success".as_bytes().to_vec();
        let data = String::from_utf8(packet.data.clone()).ok();
        OnRecvPacketAck::Successful(
            Box::new(PalletExampleAcknowledgement(success.clone())),
            Box::new(move |_| {
                T::IbcHandler::write_acknowledgement(&packet, success)
                    .map_err(|e| format!("{:?}", e))
            }),
        )
    }

    /// Called after acknowledgement message is  susccessfully processed  
    /// Decode and handle acknowledgement for both success or error cases  
    fn on_acknowledgement_packet(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        packet: &Packet,
        acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), Ics04Error> {
        // Do some custom logic stuff
        Ok(())
    }

    /// Called on packet timeout message or packet timeout on cose message  
    /// revert changes that were made when packet was sent  
    fn on_timeout_packet(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        packet: &Packet,
        _relayer: &Signer,
    ) -> Result<(), Ics04Error> {
        // Do some stuff
        Ok(())
    }
}

pub struct WeightHandler<T: Config>(PhantomData<T>);
impl<T: Config> Default for WeightHandler<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Config> CallbackWeight for WeightHandler<T> { 
    /// Returns the weight from the benchmark of the `on_chan_open_init` callback
    fn on_chan_open_init(&self) -> Weight {
        T::WeightInfo::on_chan_open_init()
    }
    /// Returns the weight from the benchmark of the `on_chan_open_try` callback
    fn on_chan_open_try(&self) -> Weight {
        T::WeightInfo::on_chan_open_try()
    }

    /// Returns the weight from the benchmark of the `on_chan_open_ack` callback
    fn on_chan_open_ack(&self, port_id: &PortId, channel_id: &ChannelId) -> Weight {
        T::WeightInfo::on_chan_open_ack(port_id, channel_id)
    }
    /// Returns the weight from the benchmark of the `on_chan_open_confirm` callback
    fn on_chan_open_confirm(&self, port_id: &PortId, channel_id: &ChannelId) -> Weight {
        T::WeightInfo::on_chan_open_confirm(port_id, channel_id)
    }
    /// Returns the weight from the benchmark of the `on_chan_close_init` callback
    fn on_chan_close_init(&self, port_id: &PortId, channel_id: &ChannelId) -> Weight {
        T::WeightInfo::on_chan_close_init(port_id, channel_id)
    }
    /// Returns the weight from the benchmark of the `on_chan_close_confirm` callback
    fn on_chan_close_confirm(&self, port_id: &PortId, channel_id: &ChannelId) -> Weight {
        T::WeightInfo::on_chan_close_confirm(port_id, channel_id)
    }  
    /// Returns the weight from the benchmark of the `on_recv_packet` callback  
    /// The weight returned can take the size of the packet data into consideration if necessary  
    fn on_recv_packet(&self, packet: &Packet) -> Weight {
        T::WeightInfo::on_recv_packet(packet)
    }
    /// Returns the weight from the benchmark of the `on_acknowledgement_packet` callback
    /// The weight returned can take the size of the packet data and acknowledgement into consideration if necessary
    fn on_acknowledgement_packet(
		&self,
		packet: &Packet,
		acknowledgement: &Acknowledgement,
    ) -> Weight {
           T::WeightInfo::on_acknowledgement_packet(packet, acknowledgement)
    }

    /// Returns the weight from the benchmark of the `on_timeout_packet` callback
    /// The weight returned can take the size of the packet data into consideration if necessary
    fn on_timeout_packet(&self, packet: &Packet) -> Weight {
        T::WeightInfo::on_timeout_packet(packet)
    }
}

```

### Custom Routes
Pallet IBC provides a means to use a custom router defined as a Runtime Config parameter. This router implements the  
[`ModuleRouter`](/contracts/pallet-ibc/src/routing#L44) trait and provides a way to add IBC support for new modules or overwrite  
existing routes.

The following code snippet shows how a custom router would be configured
```rust
// 1. Define a struct to hold the module callback handlers
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Router {
    // 2. Callback Handlers like this should implement `ibc::core::ics26_routing::context::Module` trait
    pallet_example: pallet_example::IbcModule<Runtime>,
}

// 3. Implement ModuleRouter trait.
impl ModuleRouter for Router {
    // 4. Add support for all custom routes within these function implementations
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        match module_id.as_ref() {
            pallet_example::MODULE_ID => Some(&mut self.pallet_example),
            _ => None,
        }
    }

    fn has_route(module_id: &ModuleId) -> bool {
        matches!(module_id.as_ref(), pallet_example::MODULE_ID)
    }

    fn lookup_module_by_port(port_id: &PortId) -> Option<ModuleId> {
        match port_id.as_str() {
            pallet_example::PORT_ID => ModuleId::from_str(pallet_example::MODULE_ID).ok(),
            _ => None,
        }
    }
}

impl pallet_example::Config for Runtime {
    type IbcHandler = Ibc;
    type WeightInfo = ();
}

impl pallet_ibc::Config for Runtime {
    ...
    type Router = Router;
    
}
```