use error_stack::Context;
use kinded::Kinded;
use std::fmt::{
    self, Display, Formatter,
};

pub type AppResult<T> =
    error_stack::Result<T, AppError>;

pub use error_stack::{
    bail, report, IntoReport, ResultExt,
};

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

#[derive(Debug, Kinded)]
pub enum AppError {
    DataConversionU32ToUsize,
    DataConversionUsizeToU64(usize),
    DateTimeParseError {
        input: String,
        expected_format: String,
    },
    EmptyTodoTitle,
    TooLongTodoTitle {
        input: String,
        expected_len: usize,
    },
    TodoNotFound(String),
    UpdateHasNoChanges,
}

impl Display for AppError {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        use AppError::*;

        match self {
            DataConversionU32ToUsize => {
                write!(
                    f,
                    "[{:?}] Error converting u32 to usize.",
                    DataConversionU32ToUsize,
                )
            },
            DataConversionUsizeToU64(n) => {
                write!(
                    f,
                    "[{:?}] Error converting {} to unsigned-64.",
                    AppErrorKind::DataConversionUsizeToU64,
                    n
                )
            },
            DateTimeParseError {
                input,
                expected_format
            } => {
                write!(
                    f,
                    "[{:?}] '{}' is NOT in the required format of '{}'.",
                    AppErrorKind::DateTimeParseError,
                    input,
                    expected_format
                )
            },
            EmptyTodoTitle => {
                write!(f, "[{:?}] Title cannot be empty.", EmptyTodoTitle)
            },
            TooLongTodoTitle {
                input,
                expected_len
            } => {
                write!(
                    f,
                    "[{:?}] The provided title '{}' exceeds max {} characters.",
                    AppErrorKind::TooLongTodoTitle,
                    input,
                    expected_len
                )
            },
            TodoNotFound(id) => {
                write!(
                    f,
                    "[{:?}] Item with ID '{}' not found.",
                    AppErrorKind::TodoNotFound,
                    id
                )
            },
            UpdateHasNoChanges => {
                write!(
                    f,
                    "[{:?}] At least one change must be present.",
                    UpdateHasNoChanges
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
