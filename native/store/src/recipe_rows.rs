use coffee_timer_core::model::{
    CoffeeDose, Pin, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration, StepId, WaterAmount,
};
use coffee_timer_core::ports::{RecipeStore, StoreError};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::sqlite_store::{SqliteStore, db_error};

/// Rows map to core types through the core's constructors, as legacy
/// `sqlite_recipe_repository.dart:103-125` did: a row with a blank id is
/// dropped, a blank name reads as `Untitled`, a duration is clamped. The
/// coffee and water grams are clamped too, and a step's NULL water is no
/// pour.
impl RecipeStore for SqliteStore {
    fn load_all(&self) -> Result<Vec<Recipe>, StoreError> {
        load_all(&*self.conn()?).map_err(db_error)
    }

    fn upsert(&mut self, recipe: &Recipe) -> Result<(), StoreError> {
        upsert(&mut *self.conn()?, recipe).map_err(db_error)
    }

    fn load_pinned(&self) -> Result<Vec<RecipeId>, StoreError> {
        load_pinned(&*self.conn()?).map_err(db_error)
    }

    fn insert_at(&mut self, recipe: &Recipe, index: usize, pin: Pin) -> Result<(), StoreError> {
        insert_at(&mut *self.conn()?, recipe, index, pin).map_err(db_error)
    }

    fn set_pin(&mut self, id: &RecipeId, pin: Pin) -> Result<(), StoreError> {
        self.conn()?
            .execute(
                "UPDATE recipe SET pinned = ?2 WHERE id = ?1",
                params![id.as_str(), pin.is_pinned()],
            )
            .map(drop)
            .map_err(db_error)
    }

    fn remove(&mut self, id: &RecipeId) -> Result<(), StoreError> {
        self.conn()?
            .execute("DELETE FROM recipe WHERE id = ?1", [id.as_str()])
            .map(drop)
            .map_err(db_error)
    }
}

struct RecipeRow {
    id: String,
    name: String,
    description: String,
    coffee_grams: i64,
    water_grams: i64,
}

fn load_all(conn: &Connection) -> rusqlite::Result<Vec<Recipe>> {
    let rows = conn
        .prepare(
            "SELECT id, name, description, coffee_grams, water_grams FROM recipe ORDER BY position",
        )?
        .query_map([], |row| {
            Ok(RecipeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                coffee_grams: row.get(3)?,
                water_grams: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut recipes = Vec::with_capacity(rows.len());
    for row in rows {
        let Ok(id) = RecipeId::try_parse(&row.id) else {
            continue;
        };
        let steps = load_steps(conn, &id)?;
        let name = RecipeName::from_storage(&row.name);
        recipes.push(Recipe::new(
            id,
            name,
            row.description,
            CoffeeDose::clamped(row.coffee_grams),
            WaterAmount::clamped(row.water_grams),
            steps,
        ));
    }
    Ok(recipes)
}

/// The pinned rows' ids, in stored order; a blank id is dropped, as
/// `load_all` drops its row. Any non-zero `pinned` reads as pinned.
fn load_pinned(conn: &Connection) -> rusqlite::Result<Vec<RecipeId>> {
    let ids = conn
        .prepare("SELECT id FROM recipe WHERE pinned <> 0 ORDER BY position")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids
        .iter()
        .filter_map(|id| RecipeId::try_parse(id).ok())
        .collect())
}

fn load_steps(conn: &Connection, recipe: &RecipeId) -> rusqlite::Result<Vec<RecipeStep>> {
    let mut statement = conn.prepare_cached(
        "SELECT id, name, duration_seconds, water_grams FROM step
         WHERE recipe_id = ?1 ORDER BY position",
    )?;
    let rows = statement.query_map([recipe.as_str()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;
    let mut steps = Vec::new();
    for row in rows {
        let (id, name, seconds, water) = row?;
        if let Ok(id) = StepId::try_parse(&id) {
            steps.push(RecipeStep::new(
                id,
                name,
                StepDuration::clamped(seconds),
                water.map(WaterAmount::clamped),
            ));
        }
    }
    Ok(steps)
}

/// One transaction: the recipe row is updated in place (keeping its
/// position and pin) or appended unpinned, and its steps are replaced in
/// order.
fn upsert(conn: &mut Connection, recipe: &Recipe) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    let id = recipe.id().as_str();
    tx.execute(
        "INSERT INTO recipe (id, name, description, coffee_grams, water_grams)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (id) DO UPDATE SET name = excluded.name,
             description = excluded.description,
             coffee_grams = excluded.coffee_grams,
             water_grams = excluded.water_grams",
        params![
            id,
            recipe.name().as_str(),
            recipe.description(),
            recipe.coffee().grams(),
            recipe.water().grams(),
        ],
    )?;
    tx.execute("DELETE FROM step WHERE recipe_id = ?1", [id])?;
    insert_steps(&tx, recipe)?;
    tx.commit()
}

/// One transaction: the rows from the `index`th on move one position down
/// (through negative positions, so no two rows ever share one), the recipe
/// takes the freed position with `pin`, and its steps are inserted in
/// order. An `index` past the end appends.
fn insert_at(
    conn: &mut Connection,
    recipe: &Recipe,
    index: usize,
    pin: Pin,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    let offset = i64::try_from(index).unwrap_or(i64::MAX);
    let at: Option<i64> = tx
        .query_row(
            "SELECT position FROM recipe ORDER BY position LIMIT 1 OFFSET ?1",
            [offset],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(at) = at {
        tx.execute(
            "UPDATE recipe SET position = -position - 1 WHERE position >= ?1",
            [at],
        )?;
        tx.execute(
            "UPDATE recipe SET position = -position WHERE position < 0",
            [],
        )?;
    }
    tx.execute(
        "INSERT INTO recipe (position, id, name, description, coffee_grams, water_grams, pinned)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            at,
            recipe.id().as_str(),
            recipe.name().as_str(),
            recipe.description(),
            recipe.coffee().grams(),
            recipe.water().grams(),
            pin.is_pinned(),
        ],
    )?;
    insert_steps(&tx, recipe)?;
    tx.commit()
}

/// Inserts `recipe`'s steps, in order, for a recipe that has none stored.
fn insert_steps(tx: &Transaction<'_>, recipe: &Recipe) -> rusqlite::Result<()> {
    let mut insert = tx.prepare(
        "INSERT INTO step (id, recipe_id, position, name, duration_seconds, water_grams)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for (position, step) in (0_i64..).zip(recipe.steps()) {
        insert.execute(params![
            step.id().as_str(),
            recipe.id().as_str(),
            position,
            step.name(),
            step.duration().seconds(),
            step.water().map(WaterAmount::grams),
        ])?;
    }
    Ok(())
}
