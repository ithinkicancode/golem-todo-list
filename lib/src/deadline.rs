use crate::{
    app_error::{
        AppError, AppResult,
        IntoReport, ResultExt,
    },
    core::UnixTime,
};
use chrono::naive::NaiveDateTime;
use derive_more::From;
use once_cell::sync::Lazy;

pub(crate) const USER_DATE_TIME_FORMAT: &str =
    "%Y-%m-%d %H";

static DATE_TIME_FORMAT: Lazy<String> =
    Lazy::new(|| {
        format!(
            "{}:%M:%S",
            USER_DATE_TIME_FORMAT
        )
    });

#[derive(Clone, Default, From)]
pub struct OptionalDeadlineInput(
    Option<String>,
);

impl OptionalDeadlineInput {
    pub(crate) fn is_some(
        &self,
    ) -> bool {
        self.0.is_some()
    }

    pub(crate) fn unix_time(
        &self,
    ) -> AppResult<Option<UnixTime>>
    {
        self.0.as_ref().map(|s| {
            let unix_time =
                NaiveDateTime::parse_from_str(
                    &format!("{}:00:00", s.trim()),
                    &DATE_TIME_FORMAT
                )
                .into_report()
                .change_context(
                    AppError::DateTimeParseError {
                        input: s.into(),
                        expected_format: USER_DATE_TIME_FORMAT.into(),
                    },
                )?
                .timestamp();

            Ok(unix_time)
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_app_error;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    impl OptionalDeadlineInput {
        pub(crate) fn some(
            s: &str,
        ) -> Self {
            Self(Some(s.into()))
        }

        pub(crate) fn none() -> Self {
            Self(None)
        }
    }

    #[test_case(
        "2022-01-01 09",
        1_641_027_600 ;
        "epoch of 2022-01-01 09 hour should be 1641027600"
    )]
    #[test_case(
        "1970-01-01 00",
        0 ;
        "epoch of 1970-01-01 00 hour should be 0"
    )]
    #[test_case(
        "  2222-12-31 23  ",
        7_983_874_800 ;
        "epoch of 2222-12-31 23 hour should be 1641027600"
    )]
    fn unix_time_should_succeed_with_expected_unix_time(
        input: &str,
        expected: i64,
    ) {
        let deadline =
            OptionalDeadlineInput::some(
                input,
            );
        let actual = deadline
            .unix_time()
            .unwrap()
            .unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn unix_time_should_succeed_with_none_when_input_is_none(
    ) {
        let deadline =
            OptionalDeadlineInput(None);

        let actual = deadline
            .unix_time()
            .unwrap();

        assert_eq!(actual, None)
    }

    #[test_case("2022-01-01")]
    #[test_case("abc")]
    #[test_case("2021-02-29 01")]
    fn unix_time_should_fail_when_input_does_not_match_expected_date_time_format_or_the_date_is_invalid(
        input: &str,
    ) {
        let deadline =
            OptionalDeadlineInput::some(
                input,
            );
        let actual =
            deadline.unix_time();

        let expected = AppError::DateTimeParseError {
                input: input.into(),
                expected_format: USER_DATE_TIME_FORMAT.into()
            };

        assert_app_error!(
            actual, expected
        )
    }
}
