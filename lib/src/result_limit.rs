use crate::app_error::{
    AppError, AppResult,
};
use error_stack::{
    IntoReport, ResultExt,
};

type Limit = u32;

const QUERY_DEFAULT_LIMIT: Limit = 10;

const QUERY_MAX_LIMIT: Limit = 100;

#[derive(Default)]
pub struct OptionalResultLimit(
    Option<Limit>,
);

impl OptionalResultLimit {
    pub fn new(
        value: Option<Limit>,
    ) -> Self {
        Self(value)
    }

    pub(crate) fn validated(
        &self,
    ) -> AppResult<usize> {
        self.0
            .map(|n| {
                if n > QUERY_MAX_LIMIT {
                    QUERY_MAX_LIMIT
                } else if n < 1 {
                    QUERY_DEFAULT_LIMIT
                } else {
                    n
                }
            })
            .unwrap_or(
                QUERY_DEFAULT_LIMIT,
            )
            .try_into()
            .into_report()
            .change_context(
                AppError::DataConversionU32ToUsize,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl OptionalResultLimit {
        pub(crate) fn some(
            n: Limit,
        ) -> Self {
            Self(Some(n))
        }
    }
}
