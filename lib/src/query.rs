use crate::{
    core::AppResult,
    deadline::OptionalDeadlineInput,
    todos::{Priority, Status, Todo},
};
use getset::Getters;
use std::num::TryFromIntError;
use typed_builder::TypedBuilder;

type ResultCap = u32;

const QUERY_DEFAULT_LIMIT: ResultCap =
    10;

const QUERY_MAX_LIMIT: ResultCap = 100;

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

    limit: Option<ResultCap>,
}

impl Query {
    pub(crate) fn validate_limit(
        &self,
    ) -> AppResult<usize> {
        self.limit
            .map(|n| {
                if n > QUERY_MAX_LIMIT {
                    QUERY_MAX_LIMIT
                } else if n < 1 {
                    QUERY_DEFAULT_LIMIT
                } else {
                    n
                }
            })
            .unwrap_or(
                QUERY_DEFAULT_LIMIT,
            )
            .try_into()
            .map_err(
                |e: TryFromIntError| {
                    e.to_string()
                },
            )
    }

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
