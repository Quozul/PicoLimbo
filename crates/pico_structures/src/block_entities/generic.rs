use pico_nbt::Value;

#[derive(Clone)]
pub struct GenericBlockEntity {
    nbt: Value,
}

impl GenericBlockEntity {
    /// Removes the additional fields from the Value stored in the schematic that are used to know
    /// where the block entity is.
    pub fn from_nbt(entity_nbt: &Value) -> Self {
        const KEYS_TO_REMOVE: &[&str] = &["Id", "Pos", "x", "y", "z", "keepPacked"];
        let nbt = if let Some(value) = entity_nbt.get_compound() {
            let mut cloned = value.clone();
            for key in KEYS_TO_REMOVE {
                cloned.swap_remove(*key);
            }
            Value::Compound(cloned)
        } else {
            entity_nbt.clone()
        };
        Self { nbt }
    }

    pub fn to_nbt(&self) -> &Value {
        &self.nbt
    }
}
