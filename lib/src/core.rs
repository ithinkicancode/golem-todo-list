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

pub(crate) fn unix_time_from(
    maybe: &Option<String>,
) -> AppResult<Option<i64>> {
    maybe.as_ref().map(|s| {
        let s = format!("{}:00:00", s);
        let unix_time =
            NaiveDateTime::parse_from_str(&s, &DATE_TIME_FORMAT)
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

    #[test]
    fn unix_time_from_should_succeed_with_correct_unix_time(
    ) {
        let input = "2022-01-01 09";
        let actual = unix_time_from(
            &Some(input.to_string()),
        )
        .unwrap()
        .unwrap();

        assert_eq!(actual, 1641027600)
    }
}
