use crate::block_entities::generic::GenericBlockEntity;
use minecraft_protocol::prelude::ProtocolVersion;
use pico_nbt::Value;

/// Lectern block entity with optional book compound (GitHub issue #103).
#[derive(Clone)]
pub struct LecternBlockEntity {
    nbt: Value,
}

impl LecternBlockEntity {
    pub fn from_nbt(entity_nbt: &Value) -> Self {
        Self {
            nbt: GenericBlockEntity::from_nbt(entity_nbt).to_nbt().clone(),
        }
    }

    pub fn to_version_value(&self, _protocol_version: ProtocolVersion) -> Value {
        self.nbt.clone()
    }

    pub fn has_book(&self) -> bool {
        self.book_tag().is_some()
    }

    pub fn book_nbt(&self) -> Option<&Value> {
        self.book_tag()
    }

    fn book_tag(&self) -> Option<&Value> {
        let compound = self.nbt.get_compound()?;
        compound.get("Book").or_else(|| compound.get("book"))
    }

    /// Extract readable page strings from the lectern book for limbo display.
    pub fn page_texts(&self) -> Vec<String> {
        let Some(book) = self.book_tag() else {
            return Vec::new();
        };
        let Some(compound) = book.get_compound() else {
            return Vec::new();
        };

        // Item NBT: tag.pages (legacy) or components / pages list
        let tag = compound
            .get("tag")
            .and_then(|t| t.get_compound())
            .unwrap_or(compound);

        let pages = tag
            .get("pages")
            .or_else(|| tag.get("Pages"))
            .and_then(|p| p.get_list());

        let Some(pages) = pages else {
            return Vec::new();
        };

        pages
            .iter()
            .filter_map(|page| {
                if let Some(s) = page.get_str() {
                    return Some(strip_json_text(s));
                }
                // Component-ish compound with "text"
                page.get_compound()
                    .and_then(|c| c.get("text").and_then(|t| t.get_str()).map(|s| s.to_owned()))
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}

fn strip_json_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        // Best-effort: pull "text":"..." fields
        if let Some(idx) = trimmed.find("\"text\":\"") {
            let rest = &trimmed[idx + 8..];
            if let Some(end) = rest.find('\"') {
                return rest[..end].to_owned();
            }
        }
        if let Some(idx) = trimmed.find("\"text\": \"") {
            let rest = &trimmed[idx + 9..];
            if let Some(end) = rest.find('\"') {
                return rest[..end].to_owned();
            }
        }
    }
    // Plain or quoted string
    trimmed.trim_matches('"').to_owned()
}
