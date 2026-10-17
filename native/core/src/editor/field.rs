use crate::model::ModelError;

/// One text field as typed, with what it parses to. Kept together so the
/// form shows the user's text and its error, and the two never disagree:
/// both are set only by [`Field::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field<T> {
    raw: String,
    parsed: Result<T, ModelError>,
}

impl<T> Field<T> {
    pub(crate) fn parse(raw: &str, parse: fn(&str) -> Result<T, ModelError>) -> Self {
        Self {
            raw: raw.to_owned(),
            parsed: parse(raw),
        }
    }

    pub(crate) fn raw(&self) -> &str {
        &self.raw
    }

    pub(crate) fn value(&self) -> Option<&T> {
        self.parsed.as_ref().ok()
    }

    pub(crate) fn error(&self) -> Option<&ModelError> {
        self.parsed.as_ref().err()
    }
}
