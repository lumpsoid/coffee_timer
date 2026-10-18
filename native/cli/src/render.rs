//! Controller state and outputs as text. Mechanical: every label, status and
//! error comes from core; this only lays them out.

use std::io::{self, Write};

use coffee_timer_core::brew::{
    BrewState, BrewView, Effect, RatioSetting, RunningStep, StepRow, StepStatus,
};
use coffee_timer_core::catalog::{CatalogChange, RecipeSummary};
use coffee_timer_core::editor::EditorState;
use coffee_timer_core::model::{AccentOption, Settings, ThemeSelection};

pub fn summary(out: &mut impl Write, summary: &RecipeSummary) -> io::Result<()> {
    let line = format!(
        "{}  {}  Coffee: {}  Water: {}  Time: {}  {}",
        summary.id.as_str(),
        summary.name,
        summary.coffee_label,
        summary.water_label,
        summary.time_label,
        summary.description,
    );
    writeln!(out, "{}", line.trim_end())
}

/// The brew page: recipe header, then the timer and its step list.
pub fn brew_page(out: &mut impl Write, state: &BrewState) -> io::Result<()> {
    let Some(view) = state.view() else {
        return writeln!(out, "{}", BrewState::EMPTY_LABEL);
    };
    writeln!(out, "{}  {}", view.recipe_id().as_str(), view.name())?;
    writeln!(out, "{}", view.description())?;
    let adjusted = match view.ratio() {
        RatioSetting::Adjusted(_) => " (adjusted)",
        RatioSetting::Recipe(_) => "",
    };
    writeln!(
        out,
        "Coffee: {}  Water: {}  Ratio: {}{adjusted}  Time: {}",
        view.coffee_label(),
        view.water_label(),
        view.ratio_label(),
        view.duration_label()
    )?;
    for row in state.step_rows() {
        let pour = row
            .pour_label
            .map_or_else(String::new, |pour| format!("  {pour}"));
        writeln!(
            out,
            "  {}. {}  {}{pour}  {}",
            row.number_label,
            row.name,
            row.duration_label,
            status(row.status)
        )?;
    }
    Ok(())
}

/// One line of the timer: the countdown and step while running or paused,
/// with its "pour to" target if it pours, the total while idle, the time
/// brewed (and the scale's final reading, if any step pours) once done;
/// then every step's status.
pub fn timer_line(state: &BrewState) -> String {
    let pour_to = |view: &BrewView, step: &RunningStep| {
        view.pour_to_label(step.index())
            .map_or_else(String::new, |target| format!(" ({target})"))
    };
    let timer = match state {
        BrewState::Empty => BrewState::EMPTY_LABEL.to_owned(),
        BrewState::Idle(view) => format!("idle {}", view.duration_label()),
        BrewState::Running(view, step) => format!(
            "{} {}{}",
            step.remaining_label(),
            step.name(),
            pour_to(view, step)
        ),
        BrewState::Paused(view, step) => format!(
            "paused {} {}{}",
            step.remaining_label(),
            step.name(),
            pour_to(view, step)
        ),
        BrewState::Finished(view, finished) => format!(
            "done {}{}",
            finished.total_label(),
            view.poured_label()
                .map_or_else(String::new, |poured| format!(" ({poured})"))
        ),
    };
    format!("{timer}  |  {}", statuses(&state.step_rows()))
}

pub fn effects(out: &mut impl Write, effects: &[Effect]) -> io::Result<()> {
    for effect in effects {
        match effect {
            Effect::Vibrate { ms } => writeln!(out, "[vibrate {ms}ms]")?,
            Effect::VibratePattern { pattern_ms } => {
                let pattern: Vec<String> = pattern_ms.iter().map(u32::to_string).collect();
                writeln!(out, "[vibrate pattern {}ms]", pattern.join("/"))?;
            }
            Effect::PlayStepSound => writeln!(out, "[sound]")?,
            Effect::KeepScreenOn(true) => writeln!(out, "[screen on]")?,
            Effect::KeepScreenOn(false) => writeln!(out, "[screen off]")?,
            Effect::ShowBrewNotification(notice) => match &notice.next {
                Some(next) => writeln!(out, "[notify {} | {next}]", notice.title)?,
                None => writeln!(out, "[notify {}]", notice.title)?,
            },
            Effect::HideBrewNotification => writeln!(out, "[notify off]")?,
        }
    }
    Ok(())
}

pub fn change(out: &mut impl Write, change: &CatalogChange) -> io::Result<()> {
    match change {
        CatalogChange::Upserted(id) => writeln!(out, "upserted {}", id.as_str()),
        CatalogChange::Removed(id) => writeln!(out, "removed {}", id.as_str()),
    }
}

pub fn settings(out: &mut impl Write, settings: &Settings) -> io::Result<()> {
    writeln!(out, "Theme: {}", settings.theme().storage_value())?;
    writeln!(out, "Accent: {}", settings.accent().to_hex())?;
    let themes: Vec<&str> = ThemeSelection::ALL
        .into_iter()
        .map(ThemeSelection::storage_value)
        .collect();
    writeln!(out, "Themes: {}", themes.join(", "))?;
    writeln!(out, "Accent options:")?;
    for option in AccentOption::ALL {
        writeln!(out, "  {:<7} {}", option.name(), option.color().to_hex())?;
    }
    Ok(())
}

/// The open draft's field errors, one line each, labelled as the editor
/// labels the field. Empty when no draft is open or no error is shown.
pub fn field_errors(state: &EditorState) -> Vec<String> {
    let Some(draft) = state.draft() else {
        return Vec::new();
    };
    let name = draft
        .name_error()
        .map(|error| format!("Recipe name \"{}\": {error}", draft.name()));
    let coffee = draft
        .coffee_error()
        .map(|error| format!("Coffee \"{}\": {error}", draft.coffee()));
    let water = draft
        .water_error()
        .map(|error| format!("Water \"{}\": {error}", draft.water()));
    let steps = draft.step_rows().into_iter().flat_map(|row| {
        let duration = row.duration_error.map(|error| {
            format!(
                "{} \"{}\", duration \"{}\": {error}",
                row.name_label, row.name, row.duration_text
            )
        });
        let water = row.water_error.map(|error| {
            format!(
                "{} \"{}\", grams \"{}\": {error}",
                row.name_label, row.name, row.water_text
            )
        });
        duration.into_iter().chain(water)
    });
    name.into_iter()
        .chain(coffee)
        .chain(water)
        .chain(steps)
        .collect()
}

fn statuses(rows: &[StepRow]) -> String {
    rows.iter()
        .map(|row| format!("{}:{}", row.number_label, status(row.status)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn status(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Done => "done",
        StepStatus::Current => "current",
        StepStatus::Paused => "paused",
        StepStatus::Pending => "pending",
    }
}
