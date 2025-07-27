//! Codec for MQTT [v5.0]
//!
//! [v5.0]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html

mod connect;
mod error;
mod packet;
mod poll;
mod publish;
mod subscribe;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use types::{
    decode_properties_async, decode_property_async, encode_properties, encode_properties_len,
    encode_property, encode_property_len, PropertyValue,
};

pub use connect::{
    Auth, AuthProperties, AuthReasonCode, Connack, ConnackProperties, Connect, ConnectProperties,
    ConnectReasonCode, Disconnect, DisconnectProperties, DisconnectReasonCode, LastWill,
    WillProperties,
};
pub use error::ErrorV5;
pub use packet::{Header, Packet, PacketType};
pub use poll::{PollPacket, PollPacketState};
pub use publish::{
    Puback, PubackProperties, PubackReasonCode, Pubcomp, PubcompProperties, PubcompReasonCode,
    Publish, PublishProperties, Pubrec, PubrecProperties, PubrecReasonCode, Pubrel,
    PubrelProperties, PubrelReasonCode,
};
pub use subscribe::{
    RetainHandling, Suback, SubackProperties, Subscribe, SubscribeProperties, SubscribeReasonCode,
    SubscriptionOptions, Unsuback, UnsubackProperties, Unsubscribe, UnsubscribeProperties,
    UnsubscribeReasonCode,
};
pub use types::{PropertyId, UserProperty, VarByteInt};
