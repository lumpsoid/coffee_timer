use crate::model::ModelError;

/// A non-empty, trimmed recipe name, valid by construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecipeName(String);

impl RecipeName {
    /// Trims `raw` and rejects a blank result.
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ModelError::BlankRecipeName);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Fallback for stored rows with a blank name.
    pub fn untitled() -> Self {
        Self("Untitled".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
