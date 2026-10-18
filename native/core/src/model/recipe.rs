use crate::model::{CoffeeDose, ModelError, RecipeId, RecipeName, RecipeStep, WaterAmount};

/// A brew recipe. Every field is a valid-by-construction value: the coffee
/// dose and the total water are stored, the figures derived from them and
/// the steps (ratio, total time) live in `BrewMetrics`, not here.
///
/// Change is expressed as named operations that consume the recipe and
/// return the changed one — there are no setters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    id: RecipeId,
    name: RecipeName,
    description: String,
    coffee: CoffeeDose,
    water: WaterAmount,
    steps: Vec<RecipeStep>,
}

impl Recipe {
    pub fn new(
        id: RecipeId,
        name: RecipeName,
        description: String,
        coffee: CoffeeDose,
        water: WaterAmount,
        steps: Vec<RecipeStep>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            coffee,
            water,
            steps,
        }
    }

    /// A fresh recipe with no description and no steps.
    pub fn blank(id: RecipeId, name: RecipeName, coffee: CoffeeDose, water: WaterAmount) -> Self {
        Self::new(id, name, String::new(), coffee, water, Vec::new())
    }

    pub fn renamed(self, name: RecipeName) -> Self {
        Self { name, ..self }
    }

    pub fn described_as(self, description: String) -> Self {
        Self {
            description,
            ..self
        }
    }

    pub fn with_step_added(mut self, step: RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Drops the step at `index`; an index past the end is an error.
    pub fn with_step_removed_at(mut self, index: usize) -> Result<Self, ModelError> {
        if index >= self.steps.len() {
            return Err(ModelError::StepIndexOutOfRange {
                index,
                len: self.steps.len(),
            });
        }
        self.steps.remove(index);
        Ok(self)
    }

    pub fn id(&self) -> &RecipeId {
        &self.id
    }

    pub fn name(&self) -> &RecipeName {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Ground coffee the recipe uses.
    pub fn coffee(&self) -> CoffeeDose {
        self.coffee
    }

    /// Water the whole recipe uses.
    pub fn water(&self) -> WaterAmount {
        self.water
    }

    pub fn steps(&self) -> &[RecipeStep] {
        &self.steps
    }
}
