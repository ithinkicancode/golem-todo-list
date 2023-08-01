use crate::core::{
    unix_time_from, AppResult,
};
use chrono::Utc;
use getset::{CopyGetters, Getters};
use std::{
    cmp, collections::HashMap,
    num::TryFromIntError,
};
use typed_builder::TypedBuilder;
use uuid::Uuid;

const QUERY_DEFAULT_LIMIT: u32 = 10;

const QUERY_MAX_LIMIT: u32 = 100;

macro_rules! unix_time_now {
    () => {
        Utc::now().timestamp()
    };
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
)]
pub enum Status {
    Backlog,
    InProgress,
    Done,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Clone)]
pub enum QuerySort {
    Deadline,
    Priority,
    Status,
}

#[derive(Clone, TypedBuilder)]
pub struct Query {
    keyword: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
    deadline: Option<String>,
    sort: Option<QuerySort>,
    limit: Option<u32>,
}
impl Query {
    fn validate_limit(
        &self,
    ) -> AppResult<usize> {
        self.limit
            .map(|n| {
                if n > QUERY_MAX_LIMIT {
                    QUERY_MAX_LIMIT
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

    fn check_keyword(
        &self,
        item: &Todo,
    ) -> bool {
        self.keyword
            .as_ref()
            .map(|keyword| {
                item.title
                    .contains(keyword)
            })
            .unwrap_or(true)
    }

    fn check_priority(
        &self,
        item: &Todo,
    ) -> bool {
        self.priority
            .map(|p| p == item.priority)
            .unwrap_or(true)
    }

    fn check_status(
        &self,
        item: &Todo,
    ) -> bool {
        self.status
            .map(|s| s == item.status)
            .unwrap_or(true)
    }
}

#[derive(TypedBuilder)]
pub struct NewTodo {
    title: String,
    priority: Priority,
    deadline: Option<String>,
}
impl NewTodo {
    fn validate_title(
        &self,
    ) -> AppResult<&str> {
        let title = self.title.trim();

        if title.is_empty() {
            return Err(
                "Title cannot be empty"
                    .to_string(),
            );
        }

        Ok(title)
    }
}

#[derive(TypedBuilder)]
pub struct UpdateTodo {
    title: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
    deadline: Option<String>,
}
impl UpdateTodo {
    fn change_is_present(
        &self,
    ) -> bool {
        self.title.is_some()
            || self.priority.is_some()
            || self.status.is_some()
            || self.deadline.is_some()
    }
}

#[derive(
    Clone, Debug, Getters, CopyGetters,
)]
pub struct Todo {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    title: String,
    #[getset(get_copy = "pub")]
    priority: Priority,
    #[getset(get_copy = "pub")]
    status: Status,
    #[getset(get_copy = "pub")]
    created_timestamp: i64,
    #[getset(get_copy = "pub")]
    updated_timestamp: i64,
    #[getset(get_copy = "pub")]
    deadline: Option<i64>,
}

fn item_not_found(id: &str) -> String {
    format!(
        "Item with ID '{}' not found.",
        id
    )
}

fn u64_from(
    n: usize,
) -> AppResult<u64> {
    u64::try_from(n).map_err(
        |e| {
            format!("ERROR converting {} to u64: {}", n, e.to_string())
        }
    )
}

#[derive(Default)]
pub struct TodoList(
    HashMap<String, Todo>,
);
impl TodoList {
    pub fn add(
        &mut self,
        item: NewTodo,
    ) -> AppResult<Todo> {
        let title =
            item.validate_title()?;

        let deadline = unix_time_from(
            &item.deadline,
        )?;

        let id =
            Uuid::new_v4().to_string();

        let now: i64 = unix_time_now!();

        let item = Todo {
            id,
            title: title.to_string(),
            priority: item.priority,
            deadline,
            status: Status::Backlog,
            created_timestamp: now,
            updated_timestamp: now,
        };

        let result = item.clone();

        self.0.insert(
            item.id.clone(),
            item,
        );

        Ok(result)
    }

    pub fn update(
        &mut self,
        id: String,
        change: UpdateTodo,
    ) -> AppResult<Todo> {
        if change.change_is_present() {
            let deadline_update =
                unix_time_from(
                    &change.deadline,
                )?;

            if let Some(todo) =
                self.0.get_mut(&id)
            {
                let mut modified =
                    false;

                if let Some(
                    title_update,
                ) = change.title
                {
                    let title_update =
                        title_update
                            .trim();

                    if !{
                        title_update
                            .is_empty()
                    } && todo.title
                        != title_update
                    {
                        todo.title = title_update.to_string();
                        modified = true;
                    }
                }

                if let Some(
                    priority_update,
                ) = change.priority
                {
                    if todo.priority != priority_update {
                        todo.priority = priority_update;
                        modified = true;
                    }
                }

                if let Some(
                    status_update,
                ) = change.status
                {
                    if todo.status
                        != status_update
                    {
                        todo.status = status_update;
                        modified = true;
                    }
                }

                if todo.deadline
                    != deadline_update
                {
                    todo.deadline =
                        deadline_update;
                    modified = true;
                }

                if modified {
                    todo.updated_timestamp = unix_time_now!();
                }

                Ok(todo.clone())
            } else {
                Err(item_not_found(&id))
            }
        } else {
            Err("At least one change must be present.".to_string())
        }
    }

    pub fn search(
        &self,
        query: Query,
    ) -> AppResult<Vec<Todo>> {
        let deadline = unix_time_from(
            &query.deadline,
        )?;

        let limit: usize =
            query.validate_limit()?;

        let mut result: Vec<_> = self.0
            .values()
            .filter(|t| {
                query.check_keyword(t) &&
                query.check_priority(t) &&
                query.check_status(t) &&
                deadline
                    .map(|deadline| {
                        if let Some(before) = t.deadline {
                            before <= deadline
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        match query.sort {
            Some(
                QuerySort::Priority,
            ) => {
                result.sort_by_key(
                    |t| {
                        cmp::Reverse(
                            t.priority,
                        )
                    },
                );
            }
            Some(QuerySort::Status) => {
                result.sort_by_key(
                    |t| t.status,
                );
            }
            Some(
                QuerySort::Deadline,
            ) => {
                result.sort_by_key(
                    |t| {
                        cmp::Reverse(
                            t.deadline,
                        )
                    },
                );
            }
            None => {
                result.sort_by_key(
                    |t| t.title.clone(),
                );
            }
        };

        Ok(result
            .into_iter()
            .take(limit)
            .collect())
    }

    pub fn get(
        &self,
        id: String,
    ) -> AppResult<Todo> {
        self.0
            .get(&id)
            .cloned()
            .ok_or_else(|| {
                item_not_found(&id)
            })
    }

    pub fn count(
        &self,
    ) -> AppResult<u64> {
        u64_from(self.0.len())
    }

    pub fn delete(
        &mut self,
        id: String,
    ) -> AppResult<()> {
        if self.0.contains_key(&id) {
            self.0.remove(&id);

            Ok(())
        } else {
            Err(item_not_found(&id))
        }
    }

    pub fn delete_done_items(
        &mut self,
    ) -> u64 {
        let mut count: u64 = 0;

        self.0.retain(|_, item| {
            if item.status
                == Status::Done
            {
                count += 1;
                false
            } else {
                true
            }
        });

        count
    }

    pub fn delete_all(
        &mut self,
    ) -> AppResult<u64> {
        let count = self.0.len();

        self.0.clear();

        u64_from(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
