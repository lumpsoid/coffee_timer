//! Catalog ↔ recipe list signals.

use coffee_timer_core::catalog::{self, CatalogChange, RecipeCatalog, RemovedRecipe, RestoreError};
use coffee_timer_core::editor::DeleteError;
use coffee_timer_core::model::{self, BrewMetrics, LabelStyle, Recipe};

use crate::signals::{
    LabelShade, LabelTexture, LabelTone, RecipeLabel, RecipeListUpdate, RecipeSummary, StainCorner,
};

impl RecipeListUpdate {
    /// Every recipe the catalog holds, in order.
    pub fn snapshot(catalog: &RecipeCatalog) -> Self {
        Self::Snapshot {
            recipes: catalog.state().iter().map(summary).collect(),
        }
    }

    /// The diff for a persisted change: the changed id's row as `catalog`
    /// now holds it, at its place, or its removal when the catalog holds no
    /// such recipe, with the catalog's undo offer for it if any.
    /// Read from the catalog like `BrewController::on_catalog_change`, so
    /// an `Upserted` id the catalog lacks (which it never returns) cannot
    /// send a stale row.
    pub fn of(change: &CatalogChange, catalog: &RecipeCatalog) -> Self {
        let (CatalogChange::Upserted(id) | CatalogChange::Removed(id)) = change;
        match catalog.get(id).zip(catalog.index_of(id)) {
            Some((recipe, index)) => Self::Upserted {
                recipe: summary(recipe),
                index: u32::try_from(index).unwrap_or(u32::MAX),
            },
            None => Self::Removed {
                id: id.as_str().to_owned(),
                undo_notice: catalog.restorable(id).map(RemovedRecipe::notice),
            },
        }
    }
}

impl From<RestoreError> for RecipeListUpdate {
    fn from(error: RestoreError) -> Self {
        Self::Failed {
            message: error.to_string(),
        }
    }
}

impl From<DeleteError> for RecipeListUpdate {
    fn from(error: DeleteError) -> Self {
        Self::Failed {
            message: error.to_string(),
        }
    }
}

fn summary(recipe: &Recipe) -> RecipeSummary {
    catalog::RecipeSummary::of(recipe, &BrewMetrics::default()).into()
}

impl From<catalog::RecipeSummary> for RecipeSummary {
    fn from(s: catalog::RecipeSummary) -> Self {
        Self {
            id: s.id.as_str().to_owned(),
            name: s.name,
            description: s.description,
            coffee_label: s.coffee_label,
            water_label: s.water_label,
            time_label: s.time_label,
            label: s.style.into(),
        }
    }
}

impl From<LabelStyle> for RecipeLabel {
    fn from(style: LabelStyle) -> Self {
        Self {
            tone: match style.tone() {
                model::LabelTone::Roast => LabelTone::Roast,
                model::LabelTone::Caramel => LabelTone::Caramel,
                model::LabelTone::Crema => LabelTone::Crema,
            },
            texture: match style.texture() {
                model::LabelTexture::Kraft => LabelTexture::Kraft,
                model::LabelTexture::Linen => LabelTexture::Linen,
                model::LabelTexture::Halftone => LabelTexture::Halftone,
                model::LabelTexture::Pinstripe => LabelTexture::Pinstripe,
                model::LabelTexture::Contour => LabelTexture::Contour,
                model::LabelTexture::Burlap => LabelTexture::Burlap,
            },
            shade: match style.shade() {
                model::LabelShade::Base => LabelShade::Base,
                model::LabelShade::Darker => LabelShade::Darker,
                model::LabelShade::Lighter => LabelShade::Lighter,
            },
            stain: match style.stain() {
                model::StainCorner::BottomRight => StainCorner::BottomRight,
                model::StainCorner::TopLeft => StainCorner::TopLeft,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use coffee_timer_core::fakes::FakeRecipeStore;
    use coffee_timer_core::model::{
        CoffeeDose, RecipeId, RecipeName, RecipeStep, StepDuration, StepId, WaterAmount,
    };

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn v60() -> Result<Recipe, Box<dyn std::error::Error>> {
        Ok(Recipe::new(
            RecipeId::try_parse("r1")?,
            RecipeName::try_parse("V60")?,
            "clean".to_owned(),
            CoffeeDose::try_parse(20)?,
            WaterAmount::try_parse(100)?,
            vec![RecipeStep::new(
                StepId::try_parse("s1")?,
                "Add coffee".to_owned(),
                StepDuration::try_parse(95)?,
                None,
            )],
        ))
    }

    fn catalog_with(recipes: Vec<Recipe>) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
        let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::with_recipes(recipes)));
        catalog.load()?;
        Ok(catalog)
    }

    fn assert_v60_row(row: &RecipeSummary) {
        assert_eq!(row.id, "r1");
        assert_eq!(row.name, "V60");
        assert_eq!(row.description, "clean");
        assert_eq!(row.time_label, "1:35");
        assert_eq!(row.coffee_label, "20 g");
        assert_eq!(row.water_label, "100 g");
        assert!(matches!(row.label.tone, LabelTone::Roast));
    }

    #[test]
    fn snapshot_lists_every_recipe() -> TestResult {
        let catalog = catalog_with(vec![v60()?])?;
        let RecipeListUpdate::Snapshot { recipes } = RecipeListUpdate::snapshot(&catalog) else {
            return Err("not a snapshot".into());
        };
        assert_eq!(recipes.len(), 1);
        recipes.iter().for_each(assert_v60_row);
        Ok(())
    }

    #[test]
    fn upserted_sends_the_stored_row() -> TestResult {
        let mut catalog = catalog_with(vec![])?;
        let change = catalog.upsert(v60()?)?;
        let RecipeListUpdate::Upserted { recipe, index } = RecipeListUpdate::of(&change, &catalog)
        else {
            return Err("not upserted".into());
        };
        assert_v60_row(&recipe);
        assert_eq!(index, 0);
        Ok(())
    }

    #[test]
    fn removed_sends_the_id() -> TestResult {
        let mut catalog = catalog_with(vec![v60()?])?;
        let change = catalog.remove(RecipeId::try_parse("r1")?)?;
        let RecipeListUpdate::Removed { id, undo_notice } = RecipeListUpdate::of(&change, &catalog)
        else {
            return Err("not removed".into());
        };
        assert_eq!(id, "r1");
        assert_eq!(undo_notice.as_deref(), Some("Deleted V60"));
        Ok(())
    }

    #[test]
    fn a_restore_sends_the_row_at_its_old_place() -> TestResult {
        let other = Recipe::blank(
            RecipeId::try_parse("r2")?,
            model::RecipeName::try_parse("Aero")?,
            CoffeeDose::try_parse(15)?,
            WaterAmount::try_parse(250)?,
        );
        let mut catalog = catalog_with(vec![v60()?, other])?;
        catalog.remove(RecipeId::try_parse("r1")?)?;
        let change = catalog.restore(&RecipeId::try_parse("r1")?)?;
        let RecipeListUpdate::Upserted { recipe, index } = RecipeListUpdate::of(&change, &catalog)
        else {
            return Err("not upserted".into());
        };
        assert_v60_row(&recipe);
        assert_eq!(index, 0);
        Ok(())
    }

    #[test]
    fn removing_an_unknown_id_offers_no_undo() -> TestResult {
        let mut catalog = catalog_with(vec![v60()?])?;
        let change = catalog.remove(RecipeId::try_parse("r9")?)?;
        let RecipeListUpdate::Removed { undo_notice, .. } = RecipeListUpdate::of(&change, &catalog)
        else {
            return Err("not removed".into());
        };
        assert_eq!(undo_notice, None);
        Ok(())
    }

    #[test]
    fn an_upserted_id_the_catalog_lacks_is_removed() -> TestResult {
        let catalog = catalog_with(vec![])?;
        let change = CatalogChange::Upserted(RecipeId::try_parse("r9")?);
        let RecipeListUpdate::Removed { id, undo_notice } = RecipeListUpdate::of(&change, &catalog)
        else {
            return Err("not removed".into());
        };
        assert_eq!(id, "r9");
        assert_eq!(undo_notice, None);
        Ok(())
    }

    #[test]
    fn a_store_failure_is_cores_delete_message() -> TestResult {
        let update = RecipeListUpdate::from(DeleteError::Store(
            coffee_timer_core::ports::StoreError::new("disk full"),
        ));
        let RecipeListUpdate::Failed { message } = update else {
            return Err("not a failure".into());
        };
        assert_eq!(
            message,
            "Failed to delete recipe: Storage failed: disk full"
        );
        Ok(())
    }
}
