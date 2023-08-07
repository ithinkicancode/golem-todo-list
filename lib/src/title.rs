use crate::core::AppResult;
use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(Clone)]
pub struct Title(String);

impl Title {
    const MAX_LEN: usize = 20;

    pub(crate) const EXCEEDING_MAX_LEN_ERROR: &str =
        "Title cannot exceed 20 characters.";

    pub(crate) const EMPTY_TITLE_ERROR: &str =
        "Title cannot be empty.";

    pub(crate) fn validated(
        &self,
    ) -> AppResult<String> {
        let title =
            self.clone().into_inner();

        let size = title.len();

        if size < 1 {
            Err(Self::EMPTY_TITLE_ERROR
                .to_string())
        } else if size > Self::MAX_LEN {
            Err(Self::EXCEEDING_MAX_LEN_ERROR
                .to_string())
        } else {
            Ok(title)
        }
    }
}
