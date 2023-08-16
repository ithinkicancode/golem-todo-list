use crate::app_error::{
    AppError, AppResult, IntoReport,
    ResultExt,
};
use derive_more::From;

type Limit = u32;

const QUERY_DEFAULT_LIMIT: Limit = 10;

const QUERY_MAX_LIMIT: Limit = 100;

#[derive(Default, From)]
pub struct OptionalResultLimit(
    Option<Limit>,
);

impl OptionalResultLimit {
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

    fn assert_limit(
        actual: Option<Limit>,
        expected: Limit,
    ) {
        let actual: OptionalResultLimit = actual.into();

        assert_eq!(
            actual.validated().unwrap(),
            expected as usize,
        )
    }

    #[test]
    fn validated_should_return_the_same_value_when_it_is_within_range(
    ) {
        let n = 20;

        assert_limit(Some(n), n)
    }

    #[test]
    fn validated_should_return_default_when_it_is_not_provided(
    ) {
        assert_limit(
            None,
            QUERY_DEFAULT_LIMIT,
        )
    }

    #[test]
    fn validated_should_return_default_when_it_is_zero(
    ) {
        assert_limit(
            Some(0),
            QUERY_DEFAULT_LIMIT,
        )
    }

    #[test]
    fn validated_should_return_max_when_a_greater_value_is_provided(
    ) {
        assert_limit(
            Some(QUERY_MAX_LIMIT + 1),
            QUERY_MAX_LIMIT,
        )
    }
}
