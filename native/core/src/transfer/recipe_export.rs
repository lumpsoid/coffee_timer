/// Recipes written as a document, ready for a shell to copy or save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeExport {
    json: String,
    count: usize,
}

impl RecipeExport {
    pub(crate) fn new(json: String, count: usize) -> Self {
        Self { json, count }
    }

    /// The document's text.
    pub fn json(&self) -> &str {
        &self.json
    }

    /// How many recipes it holds.
    pub fn count(&self) -> usize {
        self.count
    }

    /// `2 recipes`.
    pub fn count_label(&self) -> String {
        super::recipes_label(self.count)
    }

    /// What the GUI shows once the document is on the clipboard:
    /// `Copied 2 recipes`.
    pub fn copied_notice(&self) -> String {
        format!("Copied {}", self.count_label())
    }
}
