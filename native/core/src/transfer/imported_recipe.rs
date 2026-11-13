use crate::model::{
    CoffeeDose, GrindClicks, LabelTexture, PourSpeed, Recipe, RecipeName, WaterAmount,
};
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
    pour_speed: PourSpeed,
    grind: Option<GrindClicks>,
    label_texture: Option<LabelTexture>,
    steps: Vec<ImportedStep>,
}

impl ImportedRecipe {
    pub(crate) fn new(
        name: RecipeName,
        description: String,
        coffee: CoffeeDose,
        water: WaterAmount,
        pour_speed: PourSpeed,
        grind: Option<GrindClicks>,
        steps: Vec<ImportedStep>,
    ) -> Self {
        Self {
            name,
            description,
            coffee,
            water,
            pour_speed,
            grind,
            label_texture: None,
            steps,
        }
    }

    /// The paper the recipe carries, if the document chose one.
    pub(crate) fn with_label_texture(self, label_texture: Option<LabelTexture>) -> Self {
        Self {
            label_texture,
            ..self
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
        .with_pour_speed(self.pour_speed)
        .with_grind(self.grind)
        .with_label_texture(self.label_texture)
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

    pub fn pour_speed(&self) -> PourSpeed {
        self.pour_speed
    }

    pub fn grind(&self) -> Option<GrindClicks> {
        self.grind
    }

    pub fn label_texture(&self) -> Option<LabelTexture> {
        self.label_texture
    }

    pub fn steps(&self) -> &[ImportedStep] {
        &self.steps
    }
}
