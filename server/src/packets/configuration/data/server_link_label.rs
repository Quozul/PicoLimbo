use protocol::prelude::{Nbt, NbtEncodeError, SerializePacketData};
use thiserror::Error;

#[derive(Debug)]
pub enum ServerLinkLabel {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
    Custom(String),
}

#[derive(Debug, Error)]
pub enum ServerLinkLabelEncodeError {
    #[error("invalid label")]
    InvalidLabel,
    #[error("invalid custom label")]
    Infallible,
}

impl From<NbtEncodeError> for ServerLinkLabelEncodeError {
    fn from(_: NbtEncodeError) -> Self {
        ServerLinkLabelEncodeError::InvalidLabel
    }
}

impl From<std::convert::Infallible> for ServerLinkLabelEncodeError {
    fn from(_: std::convert::Infallible) -> Self {
        ServerLinkLabelEncodeError::Infallible
    }
}

impl SerializePacketData for ServerLinkLabel {
    type Error = ServerLinkLabelEncodeError;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        match self {
            ServerLinkLabel::BugReport => 0i32.encode(bytes)?,
            ServerLinkLabel::CommunityGuidelines => 1i32.encode(bytes)?,
            ServerLinkLabel::Support => 2i32.encode(bytes)?,
            ServerLinkLabel::Status => 3i32.encode(bytes)?,
            ServerLinkLabel::Feedback => 4i32.encode(bytes)?,
            ServerLinkLabel::Community => 5i32.encode(bytes)?,
            ServerLinkLabel::Website => 6i32.encode(bytes)?,
            ServerLinkLabel::Forums => 7i32.encode(bytes)?,
            ServerLinkLabel::News => 8i32.encode(bytes)?,
            ServerLinkLabel::Announcements => 9i32.encode(bytes)?,
            ServerLinkLabel::Custom(label) => {
                let nbt = Nbt::NamelessString {
                    value: label.clone(),
                };
                nbt.encode(bytes)?;
            }
        };

        Ok(())
    }
}
