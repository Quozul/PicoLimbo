use crate::data_types::string::StringDecodingError;
use crate::prelude::EncodePacketField;
use crate::traits::decode_packet_field::DecodePacketField;
use std::fmt::Display;
use std::str::FromStr;
use tokio::io::AsyncRead;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Identifier {
    pub namespace: String,
    pub thing: String,
}

impl Identifier {
    pub fn new(namespace: &str, thing: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            thing: thing.to_string(),
        }
    }

    pub fn minecraft(thing: &str) -> Self {
        Self::new("minecraft", thing)
    }
}

impl FromStr for Identifier {
    type Err = std::convert::Infallible;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split = string.split(':');
        let namespace = split.next().unwrap_or("minecraft");
        let thing = split
            .next()
            .unwrap_or_else(|| panic!("Invalid identifier string: {}", string));
        Ok(Self {
            namespace: namespace.to_string(),
            thing: thing.to_string(),
        })
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.thing)
    }
}

#[async_trait::async_trait]
impl DecodePacketField for Identifier {
    type Error = StringDecodingError;

    /// Decodes an identifier.
    /// An identifier is a String with a namespace and a path separated by a colon.
    /// If the namespace is not provided, it defaults to "minecraft".
    async fn decode<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: AsyncRead + Unpin + Send,
    {
        let decoded_string = String::decode(reader).await?;

        let mut split = decoded_string.split(':');
        let namespace = split.next().unwrap_or("minecraft");
        let thing = split.next().unwrap_or("");
        Ok(Identifier {
            namespace: namespace.to_string(),
            thing: thing.to_string(),
        })
    }
}

impl EncodePacketField for Identifier {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let string = format!("{}:{}", self.namespace, self.thing);
        string.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{DecodePacketField, EncodePacketField};

    #[tokio::test]
    async fn test_identifier() {
        // Given
        let identifier = Identifier::minecraft("overworld");
        // TODO: Make the expected_bytes definition static
        let mut expected_bytes = Vec::new();
        identifier.encode(&mut expected_bytes).unwrap();
        let mut reader = tokio_test::io::Builder::new().read(&expected_bytes).build();

        // When
        let decoded_identifier = Identifier::decode(&mut reader).await.unwrap();

        // Then
        assert_eq!(identifier, decoded_identifier);
    }
}
