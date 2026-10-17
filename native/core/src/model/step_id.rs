use crate::model::ModelError;

/// A non-empty step identifier, valid by construction. Distinct from
/// `RecipeId` so the two can never be swapped by accident.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StepId(String);

impl StepId {
    /// Rejects an empty string.
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        if raw.is_empty() {
            return Err(ModelError::EmptyStepId);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
