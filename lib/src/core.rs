use chrono::naive::NaiveDateTime;
use once_cell::sync::Lazy;

pub type AppResult<T> =
    Result<T, String>;

const USER_DATE_TIME_FORMAT: &str =
    "%Y-%m-%d %H";

static DATE_TIME_FORMAT: Lazy<String> =
    Lazy::new(|| {
        format!(
            "{}:%M:%S",
            USER_DATE_TIME_FORMAT
        )
    });

pub fn u64_from(
    n: usize,
) -> AppResult<u64> {
    u64::try_from(n).map_err(
        |e| {
            format!("ERROR converting {} to u64: {}", n, e.to_string())
        }
    )
}

pub(crate) fn unix_time_from(
    maybe: &Option<String>,
) -> AppResult<Option<i64>> {
    maybe.as_ref().map(|s| {
        let unix_time =
            NaiveDateTime::parse_from_str(
                &format!("{}:00:00", s),
                &DATE_TIME_FORMAT
            )
            .map_err(|e| {
                format!(
                    "ERROR: '{}' is NOT in the required format of '{}': {:?}.",
                    s,
                    USER_DATE_TIME_FORMAT,
                    e.to_string()
                )
            })?
            .timestamp();

        Ok(unix_time)
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

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
    fn unix_time_from_should_succeed_with_expected_unix_time(
        input: &str,
        expected: i64,
    ) {
        let actual = unix_time_from(
            &Some(input.to_string()),
        )
        .unwrap()
        .unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn unix_time_from_should_succeed_with_none_when_input_is_none(
    ) {
        let actual =
            unix_time_from(&None)
                .unwrap();

        assert_eq!(actual, None)
    }

    #[test_case("2022-01-01")]
    #[test_case("abc")]
    fn unix_time_from_should_fail_when_input_does_not_match_expected_date_time_format(
        input: &str,
    ) {
        let actual = unix_time_from(
            &Some(input.to_string()),
        )
        .unwrap_err();

        assert!(
            actual.contains("is NOT in the required format of"),
        )
    }
}
