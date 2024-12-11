use crate::packets::configuration::data::server_link_label::{
    ServerLinkLabel, ServerLinkLabelEncodeError,
};
use protocol::prelude::*;
use thiserror::Error;

#[derive(Debug, PacketOut)]
#[packet_id(0x10)]
pub struct ServerLinksPacket {
    pub links: LengthPaddedVec<ServerLink>,
}

#[derive(Debug, Error)]
#[error("error while encoding a server link")]
pub struct ServerLinkEncodeError;

impl From<std::convert::Infallible> for ServerLinkEncodeError {
    fn from(_: std::convert::Infallible) -> Self {
        ServerLinkEncodeError
    }
}

impl From<ServerLinkLabelEncodeError> for ServerLinkEncodeError {
    fn from(_: ServerLinkLabelEncodeError) -> Self {
        ServerLinkEncodeError
    }
}

#[derive(Debug)]
pub struct ServerLink {
    is_built_in: bool,
    label: ServerLinkLabel,
    url: String,
}

impl ServerLink {
    pub fn built_in(label: ServerLinkLabel, url: impl ToString) -> Self {
        Self {
            is_built_in: true,
            label,
            url: url.to_string(),
        }
    }

    pub fn custom(label: impl ToString, url: impl ToString) -> Self {
        Self {
            is_built_in: false,
            label: ServerLinkLabel::Custom(label.to_string()),
            url: url.to_string(),
        }
    }
}

impl SerializePacketData for ServerLink {
    type Error = ServerLinkEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.is_built_in.encode(bytes)?;
        self.label.encode(bytes)?;
        self.url.encode(bytes)?;
        Ok(())
    }
}
