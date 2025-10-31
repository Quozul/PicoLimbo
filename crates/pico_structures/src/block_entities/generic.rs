use pico_nbt::prelude::Nbt;

#[derive(Clone)]
pub struct GenericBlockEntity {
    nbt: Nbt,
}

impl GenericBlockEntity {
    pub fn from_nbt(entity_nbt: &Nbt) -> Self {
        let nbt = match entity_nbt {
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
        Self { nbt }
    }

    pub fn to_nbt(&self) -> Nbt {
        self.nbt.clone()
    }
}
