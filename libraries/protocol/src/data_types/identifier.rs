use crate::data_types::string::StringDecodingError;
use crate::deserialize_packet::DeserializePacketData;
use crate::prelude::SerializePacketData;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
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

impl DeserializePacketData for Identifier {
    type Error = StringDecodingError;

    /// Decodes an identifier.
    /// An identifier is a String with a namespace and a path separated by a colon.
    /// If the namespace is not provided, it defaults to "minecraft".
    fn decode(bytes: &[u8], index: &mut usize) -> Result<Self, Self::Error> {
        let decoded_string = String::decode(bytes, index)?;

        let mut split = decoded_string.split(':');
        let namespace = split.next().unwrap_or("minecraft");
        let thing = split.next().unwrap_or("");
        Ok(Identifier {
            namespace: namespace.to_string(),
            thing: thing.to_string(),
        })
    }
}

impl SerializePacketData for Identifier {
    type Error = std::convert::Infallible;

    fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
        let string = format!("{}:{}", self.namespace, self.thing);
        string.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{DeserializePacketData, SerializePacketData};

    #[test]
    fn test_identifier() {
        let identifier = Identifier::minecraft("overworld");
        let mut bytes = Vec::new();
        identifier.encode(&mut bytes).unwrap();
        let mut index = 0;
        let decoded_identifier = Identifier::decode(&bytes, &mut index).unwrap();
        assert_eq!(identifier, decoded_identifier);
    }
}
