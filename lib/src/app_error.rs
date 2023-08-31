use std::fmt::{
    self, Display, Formatter,
};
use strum_macros::EnumDiscriminants;
use uuid::Uuid;

pub use error_stack::{
    bail, report, Context, Report,
    Result as ErrorStackResult,
    ResultExt,
};

pub type AppResult<T> =
    ErrorStackResult<T, AppError>;

pub trait AppResultExt<T> {
    fn err_as_string(
        self,
    ) -> Result<T, String>;
}
impl<T> AppResultExt<T>
    for AppResult<T>
{
    fn err_as_string(
        self,
    ) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

#[derive(Debug, EnumDiscriminants)]
pub enum AppError {
    CollectionIsEmpty,

    DataConversionU32ToUsize,

    DataConversionUsizeToU64(usize),

    DateTimeParseError {
        input: String,
        expected_format: String,
    },

    EmptyTodoTitle,

    InvalidUuid(String),

    TooLongTodoTitle {
        input: String,
        expected_len: usize,
    },

    TodoNotFound(Uuid),

    UpdateHasNoChanges,
}
impl AppError {
    fn kind(
        &self,
    ) -> AppErrorDiscriminants {
        self.into()
    }
}

impl Display for AppError {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        use AppError as E;

        match self {
            e @ E::CollectionIsEmpty => {
                write!(
                    f,
                    "[{:?}] Dataset cannot be empty.",
                    e.kind()
                )
            },
            e @ E::DataConversionU32ToUsize => {
                write!(
                    f,
                    "[{:?}] Error converting u32 to usize.",
                    e.kind()
                )
            },
            e @ E::DataConversionUsizeToU64(n) => {
                write!(
                    f,
                    "[{:?}] Error converting {} to unsigned-64.",
                    e.kind(),
                    n
                )
            },
            e @ E::DateTimeParseError {
                input,
                expected_format
            } => {
                write!(
                    f,
                    "[{:?}] '{}' is NOT in the required format of '{}'.",
                    e.kind(),
                    input,
                    expected_format
                )
            },
            e @ E::EmptyTodoTitle => {
                write!(
                    f,
                    "[{:?}] Title cannot be empty.",
                    e.kind()
                )
            },
            e @ E::InvalidUuid(s) => {
                write!(
                    f,
                    "[{:?}] Invalid UUID '{}'.",
                    e.kind(),
                    s
                )
            },
            e @ E::TooLongTodoTitle {
                input,
                expected_len
            } => {
                write!(
                    f,
                    "[{:?}] The provided title '{}' exceeds max {} characters.",
                    e.kind(),
                    input,
                    expected_len
                )
            },
            e @ E::TodoNotFound(id) => {
                write!(
                    f,
                    "[{:?}] Item with ID '{}' not found.",
                    e.kind(),
                    id
                )
            },
            e @ E::UpdateHasNoChanges => {
                write!(
                    f,
                    "[{:?}] At least one change must be present.",
                    e.kind()
                )
            },
        }
    }
}
impl Context for AppError {}

#[cfg(test)]
mod tests {

    #[macro_export]
    macro_rules! assert_app_error {
        ($actual:ident, $expected:ident) => {
            pretty_assertions::assert_eq!(
                $actual
                    .unwrap_err()
                    .to_string(),
                $expected.to_string()
            )
        };
    }
}
