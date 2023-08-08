use crate::app_error::{
    AppError, AppResultExt, IntoReport,
    ResultExt,
};

// Not used in this crate; only used externally
pub type AppResult<T> =
    Result<T, String>;

pub fn u64_from(
    n: usize,
) -> AppResult<u64> {
    u64::try_from(n)
        .into_report()
        .change_context(
            AppError::DataConversionUsizeToU64(n),
        )
        .err_as_string()
}
