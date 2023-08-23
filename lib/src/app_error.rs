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

impl Display for AppError {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        use AppError::*;

        match self {
            e @ CollectionIsEmpty => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] Dataset cannot be empty.",
                )
            },
            e @ DataConversionU32ToUsize => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] Error converting u32 to usize.",
                )
            },
            e @ DataConversionUsizeToU64(n) => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] Error converting {} to unsigned-64.",
                    n
                )
            },
            e @ DateTimeParseError {
                input,
                expected_format
            } => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] '{}' is NOT in the required format of '{}'.",
                    input,
                    expected_format
                )
            },
            e @ EmptyTodoTitle => {
                let e: AppErrorDiscriminants = e.into();

                write!(f, "[{e:?}] Title cannot be empty.")
            },
            e @ InvalidUuid(s) => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] Invalid UUID '{}'.",
                    s
                )
            },
            e @ TooLongTodoTitle {
                input,
                expected_len
            } => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] The provided title '{}' exceeds max {} characters.",
                    input,
                    expected_len
                )
            },
            e @ TodoNotFound(id) => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] Item with ID '{}' not found.",
                    id
                )
            },
            e @ UpdateHasNoChanges => {
                let e: AppErrorDiscriminants = e.into();
                write!(
                    f,
                    "[{e:?}] At least one change must be present.",
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
