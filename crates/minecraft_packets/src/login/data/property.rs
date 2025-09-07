use minecraft_protocol::prelude::*;

#[derive(PacketOut, PacketIn, Clone)]
pub struct Property {
    name: String,
    value: String,
    signature: Optional<String>,
}

impl Property {
    const TEXTURES_NAMES: &'static str = "textures";

    pub fn new(name: String, value: String, signature: Option<String>) -> Self {
        Self {
            name,
            value,
            signature: signature.into(),
        }
    }

    pub fn textures<S>(value: &S, signature: Option<&S>) -> Self
    where
        S: ToString + ?Sized,
    {
        let signature = signature.map(|t| t.to_string());

        Self {
            name: Self::TEXTURES_NAMES.to_string(),
            value: value.to_string(),
            signature: signature.into(),
        }
    }

    pub fn is_textures(&self) -> bool {
        self.name == Self::TEXTURES_NAMES
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn signature(&self) -> Option<String> {
        self.signature.clone().into()
    }
}
