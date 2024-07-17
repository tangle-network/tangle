// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde_derive::{Deserialize, Serialize};
use tendermint::abci::Event as AbciEvent;
use tendermint::abci::EventAttribute;

use crate::events::{IbcEvent, IbcEventType};
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::error::Error;
use ibc::core::ics02_client::header::Header;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::prelude::*;

impl TryFrom<IbcEvent> for AbciEvent {
    type Error = Error;

    fn try_from(event: IbcEvent) -> Result<Self, Self::Error> {
        Ok(match event {
            IbcEvent::CreateClient(event) => event.into(),
            IbcEvent::UpdateClient(event) => event.into(),
            IbcEvent::UpgradeClient(event) => event.into(),
            IbcEvent::ClientMisbehaviour(event) => event.into(),
            IbcEvent::OpenInitConnection(event) => event.into(),
            IbcEvent::OpenTryConnection(event) => event.into(),
            IbcEvent::OpenAckConnection(event) => event.into(),
            IbcEvent::OpenConfirmConnection(event) => event.into(),
            IbcEvent::OpenInitChannel(event) => event.into(),
            IbcEvent::OpenTryChannel(event) => event.into(),
            IbcEvent::OpenAckChannel(event) => event.into(),
            IbcEvent::OpenConfirmChannel(event) => event.into(),
            IbcEvent::CloseInitChannel(event) => event.into(),
            IbcEvent::CloseConfirmChannel(event) => event.into(),
            IbcEvent::SendPacket(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::ReceivePacket(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::WriteAcknowledgement(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::AcknowledgePacket(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::TimeoutPacket(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::TimeoutOnClosePacket(event) => event.try_into().map_err(Error::channel)?,
            IbcEvent::AppModule(event) => event.try_into()?,
            IbcEvent::NewBlock(_) | IbcEvent::Empty(_) | IbcEvent::ChainError(_) => {
                return Err(Error::incorrect_event_type(event.to_string()))
            }
        })
    }
}

pub fn try_from_tx(event: &AbciEvent) -> Option<IbcEvent> {
    match event.kind.parse() {
        Ok(IbcEventType::CreateClient) => extract_attributes_from_tx(event)
            .map(CreateClient)
            .map(IbcEvent::CreateClient)
            .ok(),
        Ok(IbcEventType::UpdateClient) => match extract_attributes_from_tx(event) {
            Ok(attributes) => Some(IbcEvent::UpdateClient(UpdateClient {
                common: attributes,
                header: extract_header_from_tx(event).ok(),
            })),
            Err(_) => None,
        },
        Ok(IbcEventType::ClientMisbehaviour) => extract_attributes_from_tx(event)
            .map(ClientMisbehaviour)
            .map(IbcEvent::ClientMisbehaviour)
            .ok(),
        Ok(IbcEventType::UpgradeClient) => extract_attributes_from_tx(event)
            .map(UpgradeClient)
            .map(IbcEvent::UpgradeClient)
            .ok(),
        _ => None,
    }
}

fn extract_attributes_from_tx(event: &AbciEvent) -> Result<Attributes, Error> {
    let mut attr = Attributes::default();

    for tag in &event.attributes {
        let key = tag.key.as_ref();
        let value = tag.value.as_str();
        match key {
            HEIGHT_ATTRIBUTE_KEY => {
                attr.height = value
                    .parse()
                    .map_err(|e| Error::invalid_string_as_height(value.to_string(), e))?
            }
            CLIENT_ID_ATTRIBUTE_KEY => {
                attr.client_id = value.parse().map_err(Error::invalid_client_identifier)?
            }
            CLIENT_TYPE_ATTRIBUTE_KEY => {
                attr.client_type = value
                    .parse()
                    .map_err(|_| Error::unknown_client_type(value.to_string()))?
            }
            CONSENSUS_HEIGHT_ATTRIBUTE_KEY => {
                attr.consensus_height = value
                    .parse()
                    .map_err(|e| Error::invalid_string_as_height(value.to_string(), e))?
            }
            _ => {}
        }
    }

    Ok(attr)
}

pub fn extract_header_from_tx(event: &AbciEvent) -> Result<AnyHeader, Error> {
    for tag in &event.attributes {
        let key = tag.key.as_str();
        let value = tag.value.as_str();
        if key == HEADER_ATTRIBUTE_KEY {
            return AnyHeader::decode_from_string(value);
        }
    }
    Err(Error::missing_raw_header())
}

impl From<CreateClient> for AbciEvent {
    fn from(v: CreateClient) -> Self {
        let attributes = Vec::<EventAttribute>::from(v.0);
        AbciEvent {
            kind: IbcEventType::CreateClient.as_str().to_string(),
            attributes,
        }
    }
}

impl From<UpdateClient> for AbciEvent {
    fn from(v: UpdateClient) -> Self {
        let mut attributes = Vec::<EventAttribute>::from(v.common);
        if let Some(h) = v.header {
            let header = EventAttribute {
                key: HEADER_ATTRIBUTE_KEY.parse().unwrap(),
                value: h.encode_to_string().parse().unwrap(),
                index: false,
            };
            attributes.push(header);
        }
        AbciEvent {
            kind: IbcEventType::UpdateClient.as_str().to_string(),
            attributes,
        }
    }
}

impl From<ClientMisbehaviour> for AbciEvent {
    fn from(v: ClientMisbehaviour) -> Self {
        let attributes = Vec::<EventAttribute>::from(v.0);
        AbciEvent {
            kind: IbcEventType::ClientMisbehaviour.as_str().to_string(),
            attributes,
        }
    }
}

impl From<UpgradeClient> for AbciEvent {
    fn from(v: UpgradeClient) -> Self {
        let attributes = Vec::<EventAttribute>::from(v.0);
        AbciEvent {
            kind: IbcEventType::UpgradeClient.as_str().to_string(),
            attributes,
        }
    }
}

// This is tendermint specific
pub fn from_tx_response_event(height: Height, event: &tendermint::abci::Event) -> Option<IbcEvent> {
    // Return the first hit we find
    if let Some(mut client_res) = ClientEvents::try_from_tx(event) {
        client_res.set_height(height);
        Some(client_res)
    } else if let Some(mut conn_res) = ConnectionEvents::try_from_tx(event) {
        conn_res.set_height(height);
        Some(conn_res)
    } else if let Some(mut chan_res) = ChannelEvents::try_from_tx(event) {
        chan_res.set_height(height);
        Some(chan_res)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::header::MockHeader;
    use ibc::core::ics02_client::header::Header;

    #[test]
    fn client_event_to_abci_event() {
        let height = Height::new(1, 1);
        let attributes = Attributes {
            height,
            client_id: "test_client".parse().unwrap(),
            client_type: ClientState::<()>::client_type(),
            consensus_height: height,
        };
        let mut abci_events = vec![];
        let create_client = CreateClient::from(attributes.clone());
        abci_events.push(AbciEvent::from(create_client.clone()));
        let client_misbehaviour = ClientMisbehaviour::from(attributes.clone());
        abci_events.push(AbciEvent::from(client_misbehaviour.clone()));
        let upgrade_client = UpgradeClient::from(attributes.clone());
        abci_events.push(AbciEvent::from(upgrade_client.clone()));
        let mut update_client = UpdateClient::from(attributes);
        let header = AnyHeader::Mock(MockHeader::new(height));
        update_client.header = Some(header);
        abci_events.push(AbciEvent::from(update_client.clone()));

        for event in abci_events {
            match try_from_tx(&event) {
                Some(e) => match e {
                    IbcEvent::CreateClient(e) => assert_eq!(e.0, create_client.0),
                    IbcEvent::ClientMisbehaviour(e) => assert_eq!(e.0, client_misbehaviour.0),
                    IbcEvent::UpgradeClient(e) => assert_eq!(e.0, upgrade_client.0),
                    IbcEvent::UpdateClient(e) => {
                        assert_eq!(e.common, update_client.common);
                        assert_eq!(e.header, update_client.header);
                    }
                    _ => panic!("unexpected event type"),
                },
                None => panic!("converted event was wrong"),
            }
        }
    }
}
