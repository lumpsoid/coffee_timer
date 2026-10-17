use crate::model::ModelError;
use crate::model::hyphenated::hyphenated;

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

    /// `value` as a hyphenated lowercase UUID
    /// (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`). Infallible: the text is
    /// never empty. For an id generator that mints UUIDs.
    pub fn from_u128(value: u128) -> Self {
        Self(hyphenated(value))
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
