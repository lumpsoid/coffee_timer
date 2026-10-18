use crate::model::{CoffeeDose, Recipe, RecipeName, WaterAmount};
use crate::ports::IdGen;
use crate::transfer::ImportedStep;

/// One recipe read from a document: every value valid, at least one step,
/// but no ids yet. They are minted only once the whole document is valid,
/// so a refused document uses up none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedRecipe {
    name: RecipeName,
    description: String,
    coffee: CoffeeDose,
    water: WaterAmount,
    steps: Vec<ImportedStep>,
}

impl ImportedRecipe {
    pub(crate) fn new(
        name: RecipeName,
        description: String,
        coffee: CoffeeDose,
        water: WaterAmount,
        steps: Vec<ImportedStep>,
    ) -> Self {
        Self {
            name,
            description,
            coffee,
            water,
            steps,
        }
    }

    /// A new recipe: a fresh recipe id and a fresh id for every step.
    pub fn into_recipe(self, ids: &mut dyn IdGen) -> Recipe {
        let id = ids.recipe_id();
        let steps = self
            .steps
            .into_iter()
            .map(|step| step.into_step(ids))
            .collect();
        Recipe::new(
            id,
            self.name,
            self.description,
            self.coffee,
            self.water,
            steps,
        )
    }

    pub fn name(&self) -> &RecipeName {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn coffee(&self) -> CoffeeDose {
        self.coffee
    }

    pub fn water(&self) -> WaterAmount {
        self.water
    }

    pub fn steps(&self) -> &[ImportedStep] {
        &self.steps
    }
}
