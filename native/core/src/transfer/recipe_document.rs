use crate::model::{
    CoffeeDose, GrindClicks, LabelTexture, ModelError, PourSpeed, Recipe, RecipeName, RecipeStep,
    StepDuration, WaterAmount,
};
use crate::transfer::wire::{Document, Header, RecipeWire, StepWire};
use crate::transfer::{DocumentError, ExportError, ImportedRecipe, ImportedStep, RecipeExport};

/// The recipe document, a versioned JSON text:
///
/// ```json
/// {"format": "coffee-timer-recipes", "version": 1, "recipes": [
///   {"name": "V60", "description": "", "coffee_grams": 15, "water_grams": 250,
///    "pour_grams_per_second": 5.0, "grind_clicks": 22,
///    "steps": [{"name": "Bloom", "seconds": 30, "water_grams": 50}]}]}
/// ```
///
/// It carries what a recipe brews, not where the user keeps it: no recipe
/// or step ids and no pin. A step that pours nothing has no `water_grams`;
/// a recipe without `pour_grams_per_second` pours at the default speed,
/// and one without `grind_clicks` has no grind setting.
pub struct RecipeDocument;

impl RecipeDocument {
    /// The `format` every document carries.
    pub const FORMAT: &'static str = "coffee-timer-recipes";
    /// The one `version` this app writes and reads.
    pub const VERSION: u64 = 1;

    /// `recipes`, in the given order, as a document.
    pub fn encode(recipes: &[&Recipe]) -> Result<RecipeExport, ExportError> {
        let document = Document {
            format: Self::FORMAT.to_owned(),
            version: Self::VERSION,
            recipes: recipes.iter().map(|recipe| recipe_wire(recipe)).collect(),
        };
        let json = serde_json::to_string_pretty(&document)
            .map_err(|e| ExportError::Encode(e.to_string()))?;
        Ok(RecipeExport::new(json, recipes.len()))
    }

    /// Reads every recipe in `text`, or none: the first problem refuses the
    /// whole document. Each value goes through its model type's strict
    /// parsing, as typed input does; a blank step name is `Step N`.
    pub fn decode(text: &str) -> Result<Vec<ImportedRecipe>, DocumentError> {
        if text.trim().is_empty() {
            return Err(DocumentError::Empty);
        }
        let header: Header = serde_json::from_str(text).map_err(malformed)?;
        if header.format.as_deref() != Some(Self::FORMAT) {
            return Err(DocumentError::WrongFormat);
        }
        match header.version {
            None => return Err(DocumentError::MissingVersion),
            Some(Self::VERSION) => {}
            Some(other) => return Err(DocumentError::UnsupportedVersion(other)),
        }
        let document: Document = serde_json::from_str(text).map_err(malformed)?;
        if document.recipes.is_empty() {
            return Err(DocumentError::NoRecipes);
        }
        (1..)
            .zip(document.recipes)
            .map(|(position, recipe)| imported_recipe(position, recipe))
            .collect()
    }
}

fn malformed(error: serde_json::Error) -> DocumentError {
    DocumentError::Malformed(error.to_string())
}

fn recipe_wire(recipe: &Recipe) -> RecipeWire {
    RecipeWire {
        name: recipe.name().as_str().to_owned(),
        description: recipe.description().to_owned(),
        coffee_grams: i64::from(recipe.coffee().grams()),
        water_grams: i64::from(recipe.water().grams()),
        pour_grams_per_second: Some(recipe.pour_speed().grams_per_second()),
        grind_clicks: recipe.grind().map(|grind| i64::from(grind.clicks())),
        label_texture: recipe
            .label_texture()
            .map(|texture| texture.storage_value().to_owned()),
        steps: recipe.steps().iter().map(step_wire).collect(),
    }
}

fn step_wire(step: &RecipeStep) -> StepWire {
    StepWire {
        name: step.name().to_owned(),
        seconds: i64::from(step.duration().seconds()),
        water_grams: step.water().map(|water| i64::from(water.grams())),
    }
}

/// The recipe at `position` (1-based).
fn imported_recipe(position: usize, recipe: RecipeWire) -> Result<ImportedRecipe, DocumentError> {
    let invalid = |field: &'static str| {
        move |error: ModelError| DocumentError::Invalid {
            recipe: position,
            step: None,
            field,
            error,
        }
    };
    let name = RecipeName::try_parse(&recipe.name).map_err(invalid("name"))?;
    let coffee = CoffeeDose::try_parse(recipe.coffee_grams).map_err(invalid("coffee_grams"))?;
    let water = WaterAmount::try_parse(recipe.water_grams).map_err(invalid("water_grams"))?;
    let pour_speed = recipe
        .pour_grams_per_second
        .map(PourSpeed::try_parse)
        .transpose()
        .map_err(invalid("pour_grams_per_second"))?
        .unwrap_or_default();
    let grind = recipe
        .grind_clicks
        .map(GrindClicks::try_parse)
        .transpose()
        .map_err(invalid("grind_clicks"))?;
    let label_texture = recipe
        .label_texture
        .as_deref()
        .map(LabelTexture::try_parse)
        .transpose()
        .map_err(invalid("label_texture"))?;
    if recipe.steps.is_empty() {
        return Err(DocumentError::NoSteps { recipe: position });
    }
    let steps = (1..)
        .zip(recipe.steps)
        .map(|(step, wire)| imported_step(position, step, wire))
        .collect::<Result<_, _>>()?;
    Ok(ImportedRecipe::new(
        name,
        recipe.description,
        coffee,
        water,
        pour_speed,
        grind,
        steps,
    )
    .with_label_texture(label_texture))
}

/// The step at `position` (1-based) of the recipe at `recipe`.
fn imported_step(
    recipe: usize,
    position: usize,
    step: StepWire,
) -> Result<ImportedStep, DocumentError> {
    let invalid = |field: &'static str| {
        move |error: ModelError| DocumentError::Invalid {
            recipe,
            step: Some(position),
            field,
            error,
        }
    };
    let duration = StepDuration::try_parse(step.seconds).map_err(invalid("seconds"))?;
    let water = step
        .water_grams
        .map(WaterAmount::try_parse)
        .transpose()
        .map_err(invalid("water_grams"))?;
    Ok(ImportedStep::new(
        RecipeStep::label_or_default(&step.name, position),
        duration,
        water,
    ))
}
