use serde::de::{Error as DeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::Identifier;

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(r#"a string of the form "namespace:thing""#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Identifier, E>
    where
        E: DeError,
    {
        Identifier::try_from(v).map_err(E::custom)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Identifier, E>
    where
        E: DeError,
    {
        self.visit_str(v)
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdentifierVisitor)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
