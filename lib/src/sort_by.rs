use crate::{
    deadline::UnixTime,
    query::QuerySort,
    todos::{Priority, Status, Todo},
};
use std::cmp;

#[derive(
    Eq, PartialEq, Ord, PartialOrd,
)]
pub(crate) enum SortBy {
    Deadline(Option<UnixTime>),

    Priority(cmp::Reverse<Priority>),

    Status(Status),

    Title(String),
}

impl SortBy {
    pub(crate) fn from(
        query_sort: &Option<QuerySort>,
    ) -> impl Fn(&Todo) -> Self + '_
    {
        move |t: &Todo| match query_sort
        {
            Some(
                QuerySort::Priority,
            ) => SortBy::Priority(
                cmp::Reverse(
                    t.priority(),
                ),
            ),
            Some(QuerySort::Status) => {
                SortBy::Status(
                    t.status(),
                )
            }
            Some(
                QuerySort::Deadline,
            ) => SortBy::Deadline(
                t.deadline(),
            ),
            None => SortBy::Title(
                t.title().into(),
            ),
        }
    }
}
