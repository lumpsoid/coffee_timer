use std::{fmt, io};

use coffee_timer_core::brew::{BrewError, SaveRatioError};
use coffee_timer_core::catalog::PinError;
use coffee_timer_core::editor::{DeleteError, EditError, SaveError};
use coffee_timer_core::model::ModelError;
use coffee_timer_core::ports::StoreError;
use coffee_timer_core::transfer::{ExportError, ImportError};

/// Why a command failed. Displays the core's own error text; the CLI adds
/// none of its own except for I/O.
#[derive(Debug)]
pub enum CliError {
    Store(StoreError),
    Model(ModelError),
    Brew(BrewError),
    SaveRatio(SaveRatioError),
    Edit(EditError),
    /// A rejected save, with the draft's field errors (none unless a field
    /// is invalid).
    Save {
        error: SaveError,
        fields: Vec<String>,
    },
    Delete(DeleteError),
    Pin(PinError),
    Export(ExportError),
    Import(ImportError),
    Io(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Model(e) => e.fmt(f),
            Self::Brew(e) => e.fmt(f),
            Self::SaveRatio(e) => e.fmt(f),
            Self::Edit(e) => e.fmt(f),
            Self::Save { error, fields } => {
                write!(f, "{error}")?;
                for field in fields {
                    write!(f, "\n  {field}")?;
                }
                Ok(())
            }
            Self::Delete(e) => e.fmt(f),
            Self::Pin(e) => e.fmt(f),
            Self::Export(e) => e.fmt(f),
            Self::Import(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for CliError {}

impl From<StoreError> for CliError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl From<ModelError> for CliError {
    fn from(e: ModelError) -> Self {
        Self::Model(e)
    }
}

impl From<BrewError> for CliError {
    fn from(e: BrewError) -> Self {
        Self::Brew(e)
    }
}

impl From<SaveRatioError> for CliError {
    fn from(e: SaveRatioError) -> Self {
        Self::SaveRatio(e)
    }
}

impl From<EditError> for CliError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}

impl From<DeleteError> for CliError {
    fn from(e: DeleteError) -> Self {
        Self::Delete(e)
    }
}

impl From<PinError> for CliError {
    fn from(e: PinError) -> Self {
        Self::Pin(e)
    }
}

impl From<ExportError> for CliError {
    fn from(e: ExportError) -> Self {
        Self::Export(e)
    }
}

impl From<ImportError> for CliError {
    fn from(e: ImportError) -> Self {
        Self::Import(e)
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
