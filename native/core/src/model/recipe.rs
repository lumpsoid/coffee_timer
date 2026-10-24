use crate::model::{
    CoffeeDose, GrindClicks, ModelError, PourScale, PourSpeed, RecipeId, RecipeName, RecipeStep,
    WaterAmount, WaterScaleError,
};

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
    pour_speed: PourSpeed,
    grind: Option<GrindClicks>,
    steps: Vec<RecipeStep>,
}

impl Recipe {
    /// Pours at [`PourSpeed::DEFAULT`] until given its own speed
    /// ([`Recipe::with_pour_speed`]), and has no grind setting until given
    /// one ([`Recipe::with_grind`]).
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
            pour_speed: PourSpeed::DEFAULT,
            grind: None,
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

    pub fn with_pour_speed(self, pour_speed: PourSpeed) -> Self {
        Self { pour_speed, ..self }
    }

    pub fn with_grind(self, grind: Option<GrindClicks>) -> Self {
        Self { grind, ..self }
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

    /// The recipe at `water_grams` of total water, the coffee kept: what a
    /// brew at an adjusted ratio shows, made the recipe's own. Every step's
    /// pour is scaled by [`PourScale`], the rule the brew page shows them
    /// by, so the stored pours are the ones shown. The total water and each
    /// scaled pour must be a [`WaterAmount`]; a step that pours nothing
    /// still pours nothing.
    pub fn with_water_scaled_to(self, water_grams: u32) -> Result<Self, WaterScaleError> {
        let water = WaterAmount::try_parse(i64::from(water_grams))
            .map_err(|_| WaterScaleError::Water { grams: water_grams })?;
        let pours = PourScale::new(self.water, water_grams).pours(&self.steps);
        let steps = self
            .steps
            .into_iter()
            .zip(pours)
            .enumerate()
            .map(|(index, (step, pour))| {
                let Some(pour) = pour else {
                    return Ok(step);
                };
                let grams = pour.grams();
                let amount = WaterAmount::try_parse(i64::from(grams)).map_err(|_| {
                    WaterScaleError::Pour {
                        position: index + 1,
                        grams,
                    }
                })?;
                Ok(step.pouring(Some(amount)))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            water,
            steps,
            ..self
        })
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

    /// How fast the recipe pours: what the editor times a step's pour by.
    pub fn pour_speed(&self) -> PourSpeed {
        self.pour_speed
    }

    /// Where to set the grinder; none when the recipe does not say.
    pub fn grind(&self) -> Option<GrindClicks> {
        self.grind
    }

    pub fn steps(&self) -> &[RecipeStep] {
        &self.steps
    }
}
