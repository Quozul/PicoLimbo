use pico_nbt2::Value;

#[derive(Clone)]
pub struct GenericBlockEntity {
    nbt: Value,
}

impl GenericBlockEntity {
    /// Removes the additional fields from the Value stored in the schematic that are used to know
    /// where the block entity is.
    pub fn from_nbt(_entity_nbt: &Value) -> Self {
        unimplemented!();
        /*let nbt = match entity_nbt {
            Nbt::Compound { value, .. } => {
                let filtered: Vec<Nbt> = value
                    .iter()
                    .filter(|tag| {
                        !matches!(
                            tag.get_name().as_deref(),
                            Some("Id" | "Pos" | "x" | "y" | "z" | "keepPacked")
                        )
                    })
                    .cloned()
                    .collect();
                Nbt::Compound {
                    name: None,
                    value: filtered,
                }
            }
            _ => entity_nbt.clone(),
        };
        Self { nbt }*/
    }

    pub fn to_nbt(&self) -> &Value {
        &self.nbt
    }
}
