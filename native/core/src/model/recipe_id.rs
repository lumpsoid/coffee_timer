use crate::model::ModelError;

/// A non-empty recipe identifier, valid by construction. Build one via
/// [`RecipeId::try_parse`]; once held it is a real id — never re-check it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecipeId(String);

impl RecipeId {
    /// Rejects an empty string.
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        if raw.is_empty() {
            return Err(ModelError::EmptyRecipeId);
        }
        Ok(Self(raw.to_owned()))
    }

    /// `r{n}` — non-empty by construction. Only for the
    /// `SequentialIds` test fake.
    #[cfg(any(test, feature = "test-fakes"))]
    pub(crate) fn sequential(n: u64) -> Self {
        Self(format!("r{n}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
