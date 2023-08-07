use crate::app_error::{
    AppError, AppResult,
};
use error_stack::bail;
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
                    input: title.clone(),
                    expected_len: Self::MAX_LEN
                }
            )
        } else {
            Ok(title)
        }
    }
}
