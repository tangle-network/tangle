## ICS_026 Routing

The ICS routing specification enablesIBCapplication protocols to receive packets after the coreIBCprotocol has verified  
the validity of the packet.

The [`Ics26Context`](/core/ics26_routing/context.rs#L32) trait encapsulates the requirements for the router.  

As per previous context traits, the `Ics26Context` trait should be implemented for the Context object.  
`Ics26Context` has an associated type `Router` that accepts a type that implements the [`Router`](code/centauri/ibc/modules/src/core/ics26_routing/context.rs#L215) trait.

It is recommended that the router use statically defined port and module Ids.  

**A snippet showing an example of how the router would be implemented**
```rust
    pub struct Context {
        router: IbcRouter,
    }

    pub struct IbcRouter {
        ics_20: ics20::IbcModule,
    }

    impl Default for IbcRouter {
        fn default() -> Self {
            Self {
                ics_20: ics20::IbcModule::default(),
            }
        }
    }

    impl Router for IbcRouter {
        fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
            match module_id.as_ref() {
                ICS20_MODULE_ID => Some(&mut self.ics_20),
                &_ => None,
            }
        }

        fn has_route(&self, module_id: &ModuleId) -> bool {
            matches!(
                module_id.as_ref(), ICS20_MODULE_ID
		    )
        }
    }

    impl Ics26Context for Context {
        type Router = IbcRouter;

        fn router(&self) -> &Self::Router {
            &self.router
        }

        fn router_mut(&mut self) -> &mut Self::Router {
            &mut self.router
        }
    }
```

### Message Handling

Handling IBC messages is as simple as calling the [`deliver`](/core/ics26_routing/handler.rs#L40) function with the context and message.  
The internal dispatch mechanism will decode the protobuf message and route it to the correct message handler for processing.  
```rust
    deliver(&mut ctx, message)?;
```

**Ics26Envelope**

The [`Ics26Envelope`](/core/ics26_routing/msgs.rs#L33) enum implements the `TryFrom<Any>`  
and it is what helps us convert the protobuf `Any` into a concrete IBC message.  
Ibc messages have a unique `type_url`, which is a string that helps us identify what concrete message type to decode the raw protobuf bytes into.