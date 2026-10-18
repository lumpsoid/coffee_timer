use crate::model::ModelError;

/// A non-empty, trimmed recipe name, valid by construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecipeName(String);

impl RecipeName {
    /// The longest name, in characters, that Rust itself makes up for a
    /// recipe ([`RecipeName::copy_of`]). Typed names have no upper bound,
    /// as in the legacy app.
    pub const MAX_GENERATED_CHARS: usize = 60;

    /// What a duplicate is called before the user renames it.
    const COPY_SUFFIX: &'static str = " copy";

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

    /// Reads a stored name leniently: trimmed, and a blank one is
    /// [`RecipeName::untitled`]. Never fails — bad storage never blocks the
    /// app.
    pub fn from_storage(raw: &str) -> Self {
        Self::try_parse(raw).unwrap_or_else(|_| Self::untitled())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The name of a duplicate of this recipe: `V60 copy`. Kept within
    /// [`RecipeName::MAX_GENERATED_CHARS`] by cutting the base name (and any
    /// space the cut leaves at its end), never the suffix.
    pub fn copy_of(&self) -> Self {
        let room = Self::MAX_GENERATED_CHARS.saturating_sub(Self::COPY_SUFFIX.chars().count());
        let base: String = self.0.chars().take(room).collect();
        Self(format!("{}{}", base.trim_end(), Self::COPY_SUFFIX))
    }
}
