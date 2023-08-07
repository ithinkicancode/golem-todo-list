pub type AppResult<T> =
    Result<T, String>;

pub fn u64_from(
    n: usize,
) -> AppResult<u64> {
    u64::try_from(n).map_err(
        |e| {
            format!(
                "ERROR converting {} to u64: {:?}",
                n,
                e.to_string()
            )
        }
    )
}
