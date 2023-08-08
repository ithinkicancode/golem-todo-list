use crate::app_error::{
    bail, AppError, AppResult,
};
use nutype::nutype;

#[nutype(sanitize(trim))]
#[derive(Clone)]
pub struct Title(String);

impl Title {
    pub(crate) const MAX_LEN: usize =
        20;

    pub(crate) fn validated(
        &self,
    ) -> AppResult<String> {
        let title =
            self.clone().into_inner();

        let size = title.len();

        if size < 1 {
            bail!(
                AppError::EmptyTodoTitle
            )
        } else if size > Self::MAX_LEN {
            bail!(
                AppError::TooLongTodoTitle {
                    input: title,
                    expected_len: Self::MAX_LEN
                }
            )
        } else {
            Ok(title)
        }
    }
}
