use crate::core::AppResult;
use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(Clone)]
pub struct Title(String);

impl Title {
    pub(crate) const EMPTY_TITLE_ERROR: &str =
        "Title cannot be empty.";

    pub(crate) const EXCEEDING_MAX_LEN_ERROR: &str =
      "Title cannot exceed 20 characters.";

    pub(crate) fn validated(
        &self,
    ) -> AppResult<String> {
        let title =
            self.clone().into_inner();

        if title.is_empty() {
            Err(Self::EMPTY_TITLE_ERROR
                .to_string())
        } else if title.len() > 20 {
            Err(Self::EXCEEDING_MAX_LEN_ERROR
              .to_string())
        } else {
            Ok(title)
        }
    }
}
