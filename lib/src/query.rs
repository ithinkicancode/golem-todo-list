use crate::{
    deadline::OptionalDeadlineInput,
    result_limit::OptionalResultLimit,
    todos::{Priority, Status, Todo},
};
use getset::Getters;
use typed_builder::TypedBuilder;

#[derive(Clone)]
pub enum QuerySort {
    Deadline,
    Priority,
    Status,
}

#[derive(
    Default, Getters, TypedBuilder,
)]
#[builder(field_defaults(default))]
pub struct Query {
    keyword: Option<String>,

    priority: Option<Priority>,

    status: Option<Status>,

    #[getset(get = "pub")]
    deadline: OptionalDeadlineInput,

    #[getset(get = "pub")]
    sort: Option<QuerySort>,

    #[getset(get = "pub")]
    limit: OptionalResultLimit,
}

impl Query {
    pub(crate) fn match_keyword(
        &self,
        todo: &Todo,
    ) -> bool {
        self.keyword
            .as_ref()
            .map(|keyword| {
                todo.title()
                    .contains(keyword)
            })
            .unwrap_or(true)
    }

    pub(crate) fn match_priority(
        &self,
        todo: &Todo,
    ) -> bool {
        self.priority
            .map(|p| {
                p == todo.priority()
            })
            .unwrap_or(true)
    }

    pub(crate) fn match_status(
        &self,
        todo: &Todo,
    ) -> bool {
        self.status
            .map(|s| s == todo.status())
            .unwrap_or(true)
    }

    pub(crate) fn match_deadline(
        deadline: &Option<i64>,
        todo: &Todo,
    ) -> bool {
        deadline
            .map(|deadline| {
                if let Some(before) =
                    todo.deadline()
                {
                    before <= deadline
                } else {
                    true
                }
            })
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Query {
        pub(crate) fn empty() -> Self {
            Self::default()
        }
    }
}
