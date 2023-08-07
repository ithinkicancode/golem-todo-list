use crate::core::AppResult;
use chrono::naive::NaiveDateTime;
use once_cell::sync::Lazy;

pub(crate) const INVALID_DATE_TIME_FORMAT: &str =
    "is NOT in the required format of";

const USER_DATE_TIME_FORMAT: &str =
    "%Y-%m-%d %H";

static DATE_TIME_FORMAT: Lazy<String> =
    Lazy::new(|| {
        format!(
            "{}:%M:%S",
            USER_DATE_TIME_FORMAT
        )
    });

#[derive(Clone, Default)]
pub struct OptionalDeadlineInput(
    Option<String>,
);

impl OptionalDeadlineInput {
    pub fn new(
        value: Option<String>,
    ) -> Self {
        Self(value)
    }

    pub(crate) fn is_some(
        &self,
    ) -> bool {
        self.0.is_some()
    }

    pub(crate) fn unix_time(
        &self,
    ) -> AppResult<Option<i64>> {
        self.0.as_ref().map(|s| {
            let unix_time =
                NaiveDateTime::parse_from_str(
                    &format!("{}:00:00", s),
                    &DATE_TIME_FORMAT
                )
                .map_err(|e| {
                    format!(
                        "ERROR: '{}' {} '{}': {:?}.",
                        s,
                        INVALID_DATE_TIME_FORMAT,
                        USER_DATE_TIME_FORMAT,
                        e.to_string()
                    )
                })?
                .timestamp();

            Ok(unix_time)
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    impl OptionalDeadlineInput {
        pub(crate) fn some(
            s: &str,
        ) -> Self {
            Self(Some(s.to_string()))
        }

        pub(crate) fn none() -> Self {
            Self(None)
        }
    }

    #[test_case(
        "2022-01-01 09",
        1641027600 ;
        "epoch of 2022-01-01 09 hour should be 1641027600"
    )]
    #[test_case(
        "1970-01-01 00",
        0 ;
        "epoch of 1970-01-01 00 hour should be 0"
    )]
    fn unix_time_should_succeed_with_expected_unix_time(
        input: &str,
        expected: i64,
    ) {
        let deadline =
            OptionalDeadlineInput(
                Some(input.to_string()),
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
    fn unix_time_should_fail_when_input_does_not_match_expected_date_time_format(
        input: &str,
    ) {
        let deadline =
            OptionalDeadlineInput(
                Some(input.to_string()),
            );
        let actual = deadline
            .unix_time()
            .unwrap_err();

        assert!(actual.contains(
            INVALID_DATE_TIME_FORMAT
        ))
    }
}