//! Brew state and effects → brew signals.

use coffee_timer_core::brew::{
    BrewNotice, BrewState, BrewView, Effect, FinishedBrew, RatioChange, RatioSetting, RunningStep,
    StepRow, StepStatus,
};
use coffee_timer_core::model::StepDuration;

use crate::signals::{
    BrewFinished, BrewRatioChange, BrewRatioSetting, BrewRecipe, BrewStep, BrewStepStatus,
    BrewTimer, BrewUpdate, PlatformEffect,
};

impl From<&BrewState> for BrewUpdate {
    fn from(state: &BrewState) -> Self {
        let steps = || state.step_rows().into_iter().map(BrewStep::from).collect();
        match state {
            BrewState::Empty => Self::Empty {
                label: BrewState::EMPTY_LABEL.to_owned(),
            },
            BrewState::Idle(view) => Self::Idle {
                recipe: recipe(view, steps()),
            },
            BrewState::Running(view, step) => Self::Running {
                recipe: recipe(view, steps()),
                timer: timer(view, step),
            },
            BrewState::Paused(view, step) => Self::Paused {
                recipe: recipe(view, steps()),
                timer: timer(view, step),
            },
            BrewState::Finished(view, done) => Self::Finished {
                recipe: recipe(view, steps()),
                finished: finished(view, done),
            },
        }
    }
}

fn finished(view: &BrewView, finished: &FinishedBrew) -> BrewFinished {
    BrewFinished {
        total_label: finished.total_label().to_owned(),
        poured_label: view.poured_label(),
    }
}

fn recipe(view: &BrewView, steps: Vec<BrewStep>) -> BrewRecipe {
    BrewRecipe {
        id: view.recipe_id().as_str().to_owned(),
        name: view.name().to_owned(),
        description: view.description().to_owned(),
        coffee_label: view.coffee_label().to_owned(),
        water_label: view.water_label().to_owned(),
        ratio_label: view.ratio_label().to_owned(),
        ratio: view.ratio().into(),
        grind_label: view.grind_label(),
        duration_label: view.duration_label().to_owned(),
        label: view.style().into(),
        steps,
    }
}

impl From<RatioSetting> for BrewRatioSetting {
    fn from(setting: RatioSetting) -> Self {
        match setting {
            RatioSetting::Recipe(_) => Self::Recipe,
            RatioSetting::Adjusted(_) => Self::Adjusted,
        }
    }
}

impl From<BrewRatioChange> for RatioChange {
    fn from(change: BrewRatioChange) -> Self {
        match change {
            BrewRatioChange::MoreWater => Self::MoreWater,
            BrewRatioChange::LessWater => Self::LessWater,
            BrewRatioChange::Recipe => Self::Recipe,
        }
    }
}

impl From<StepRow> for BrewStep {
    fn from(row: StepRow) -> Self {
        Self {
            number_label: row.number_label,
            name: row.name,
            duration_label: row.duration_label,
            pour_label: row.pour_label,
            status: row.status.into(),
            track_start: row.track_start,
            track_end: row.track_end,
        }
    }
}

impl From<StepStatus> for BrewStepStatus {
    fn from(status: StepStatus) -> Self {
        match status {
            StepStatus::Done => Self::Done,
            StepStatus::Current => Self::Current,
            StepStatus::Paused => Self::Paused,
            StepStatus::Pending => Self::Pending,
        }
    }
}

fn timer(view: &BrewView, step: &RunningStep) -> BrewTimer {
    BrewTimer {
        run: u32::try_from(step.run()).unwrap_or(u32::MAX),
        // A step index beyond `u32` cannot occur (steps are typed in by
        // hand); saturate rather than panic.
        step_index: u32::try_from(step.index()).unwrap_or(u32::MAX),
        step_name: step.name().to_owned(),
        position_label: view.position_label(step.index()),
        remaining_label: step.remaining_label(),
        pour_to_label: view.pour_to_label(step.index()),
        next_label: view.next_label(step.index()),
        sweep_seconds: step.sweep().map(StepDuration::seconds),
    }
}

impl From<Effect> for PlatformEffect {
    fn from(effect: Effect) -> Self {
        match effect {
            Effect::PlayStepSound => Self::PlayStepSound,
            Effect::Vibrate { ms } => Self::Vibrate { ms },
            Effect::VibratePattern { pattern_ms } => Self::VibratePattern { pattern_ms },
            Effect::KeepScreenOn(on) => Self::KeepScreenOn { on },
            Effect::ShowBrewNotification(BrewNotice { title, next }) => {
                Self::ShowBrewNotification { title, next }
            }
            Effect::HideBrewNotification => Self::HideBrewNotification,
        }
    }
}

#[cfg(test)]
mod tests {
    use coffee_timer_core::brew::BrewController;
    use coffee_timer_core::catalog::RecipeCatalog;
    use coffee_timer_core::fakes::FakeRecipeStore;
    use coffee_timer_core::model::{
        CoffeeDose, Recipe, RecipeId, RecipeName, RecipeStep, Settings, StepId, WaterAmount,
    };

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn step(id: &str, name: &str, seconds: i64) -> Result<RecipeStep, Box<dyn std::error::Error>> {
        Ok(RecipeStep::new(
            StepId::try_parse(id)?,
            name.to_owned(),
            StepDuration::try_parse(seconds)?,
            None,
        ))
    }

    /// A brew on `r1`: "Bloom" 30 s, then "Pour" 0 s.
    fn opened() -> Result<BrewController, Box<dyn std::error::Error>> {
        let recipe = Recipe::new(
            RecipeId::try_parse("r1")?,
            RecipeName::try_parse("V60")?,
            "clean".to_owned(),
            CoffeeDose::try_parse(20)?,
            WaterAmount::try_parse(100)?,
            vec![step("s1", "Bloom", 30)?, step("s2", "Pour", 0)?],
        );
        let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::with_recipes(vec![recipe])));
        catalog.load()?;
        let mut brew = BrewController::new();
        let _effects = brew.open(&RecipeId::try_parse("r1")?, &catalog);
        Ok(brew)
    }

    fn statuses(recipe: &BrewRecipe) -> Vec<&'static str> {
        recipe
            .steps
            .iter()
            .map(|s| match s.status {
                BrewStepStatus::Done => "done",
                BrewStepStatus::Current => "current",
                BrewStepStatus::Paused => "paused",
                BrewStepStatus::Pending => "pending",
            })
            .collect()
    }

    #[test]
    fn empty_carries_its_label() -> TestResult {
        let BrewUpdate::Empty { label } = BrewUpdate::from(&BrewState::Empty) else {
            return Err("not empty".into());
        };
        assert_eq!(label, "Empty recipe");
        Ok(())
    }

    #[test]
    fn idle_shows_the_recipe_with_every_step_pending() -> TestResult {
        let brew = opened()?;
        let BrewUpdate::Idle { recipe } = BrewUpdate::from(brew.state()) else {
            return Err("not idle".into());
        };
        assert_eq!(recipe.id, "r1");
        assert_eq!(recipe.name, "V60");
        assert_eq!(recipe.description, "clean");
        assert_eq!(recipe.duration_label, "0:30");
        assert_eq!(recipe.ratio_label, "1:5");
        assert!(matches!(recipe.ratio, BrewRatioSetting::Recipe));
        let track: Vec<_> = recipe
            .steps
            .iter()
            .map(|s| (s.track_start, s.track_end))
            .collect();
        assert_eq!(track, [(0.0, 1.0), (1.0, 1.0)]);
        assert_eq!(statuses(&recipe), ["pending", "pending"]);
        assert_eq!(recipe.steps[0].number_label, "1");
        assert_eq!(recipe.steps[0].name, "Bloom");
        assert_eq!(recipe.steps[0].duration_label, "30s");
        Ok(())
    }

    #[test]
    fn an_adjusted_ratio_shows_its_water_and_label() -> TestResult {
        let mut brew = opened()?;
        brew.adjust_ratio(BrewRatioChange::MoreWater.into())?;
        let BrewUpdate::Idle { recipe } = BrewUpdate::from(brew.state()) else {
            return Err("not idle".into());
        };
        assert_eq!(recipe.coffee_label, "20g");
        assert_eq!(recipe.water_label, "110g");
        assert_eq!(recipe.ratio_label, "1:5.5");
        assert!(matches!(recipe.ratio, BrewRatioSetting::Adjusted));
        Ok(())
    }

    #[test]
    fn running_carries_the_step_index_and_its_sweep() -> TestResult {
        let mut brew = opened()?;
        let _effects = brew.start(0)?;
        let _effects = brew.tick(&Settings::FALLBACK);
        let BrewUpdate::Running { recipe, timer } = BrewUpdate::from(brew.state()) else {
            return Err("not running".into());
        };
        assert_eq!(statuses(&recipe), ["current", "pending"]);
        assert_eq!(timer.run, 1);
        assert_eq!(timer.step_index, 0);
        assert_eq!(timer.step_name, "Bloom");
        assert_eq!(timer.position_label, "Step 1 of 2");
        assert_eq!(timer.remaining_label, "00:29");
        assert_eq!(timer.sweep_seconds, Some(30));
        assert_eq!(timer.next_label, "Next: Pour · 0:00");
        Ok(())
    }

    #[test]
    fn a_zero_second_step_has_no_sweep() -> TestResult {
        let mut brew = opened()?;
        let _effects = brew.start(1)?;
        let BrewUpdate::Running { recipe, timer } = BrewUpdate::from(brew.state()) else {
            return Err("not running".into());
        };
        assert_eq!(statuses(&recipe), ["done", "current"]);
        assert_eq!(timer.step_index, 1);
        assert_eq!(timer.sweep_seconds, None);
        assert_eq!(timer.next_label, "Last step");
        Ok(())
    }

    #[test]
    fn paused_carries_the_held_step_and_its_status() -> TestResult {
        let mut brew = opened()?;
        let _effects = brew.start(0)?;
        let _effects = brew.tick(&Settings::FALLBACK);
        let _effects = brew.pause()?;
        let BrewUpdate::Paused { recipe, timer } = BrewUpdate::from(brew.state()) else {
            return Err("not paused".into());
        };
        assert_eq!(statuses(&recipe), ["paused", "pending"]);
        assert_eq!(timer.run, 1);
        assert_eq!(timer.step_index, 0);
        assert_eq!(timer.remaining_label, "00:29");
        assert_eq!(timer.next_label, "Next: Pour · 0:00");
        Ok(())
    }

    #[test]
    fn finished_marks_every_step_done_and_carries_the_total() -> TestResult {
        let mut brew = opened()?;
        let _effects = brew.start(0)?;
        for _ in 0..31 {
            let _effects = brew.tick(&Settings::FALLBACK);
        }
        let BrewUpdate::Finished { recipe, finished } = BrewUpdate::from(brew.state()) else {
            return Err("not finished".into());
        };
        assert_eq!(statuses(&recipe), ["done", "done"]);
        assert_eq!(finished.total_label, "0:30");
        assert_eq!(finished.poured_label, None, "no step pours");
        Ok(())
    }
}
