use chrono::DateTime;

pub type AppResult<T> =
    Result<T, String>;

// TODO: use a simpler format string
pub(crate) const DATE_TIME_FORMAT:
    &str = "%Y-%m-%d %H:%M:%S %z";

pub(crate) fn unix_time_from(
    maybe: &Option<String>,
) -> AppResult<Option<i64>> {
    maybe.as_ref().map(|s| {
        let unix_time =
            DateTime::parse_from_str(s, DATE_TIME_FORMAT)
                .map_err(|e| {
                    format!(
                        "ERROR: '{}' is NOT in the required format of '{}': {:?}.",
                        s,
                        DATE_TIME_FORMAT,
                        e.to_string()
                    )
                })?
                .timestamp();

        Ok(unix_time)
    })
    .transpose()
}
