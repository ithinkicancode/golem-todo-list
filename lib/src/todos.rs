use crate::core::{
    unix_time_from, AppResult,
};
use chrono::Utc;
use once_cell::sync::Lazy;
use std::{
    cmp, collections::HashMap,
    num::TryFromIntError,
};
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

// TODO: use builder instead of pub fields
#[derive(Clone)]
pub struct Query {
    pub keyword: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<Status>,
    pub deadline: Option<String>,
    pub sort: Option<QuerySort>,
    pub limit: Option<u32>,
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

// TODO: use builder instead of pub fields
pub struct NewTodo {
    pub title: String,
    pub priority: Priority,
    pub deadline: Option<String>,
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

// TODO: use builder instead of pub fields
pub struct UpdateTodo {
    pub title: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<Status>,
    pub deadline: Option<String>,
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

#[derive(Clone, Debug)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub priority: Priority,
    pub status: Status,
    pub created_timestamp: i64,
    pub updated_timestamp: i64,
    pub deadline: Option<i64>,
}

struct AppState {
    items: HashMap<String, Todo>,
}

static mut APP_STATE: Lazy<AppState> =
    Lazy::new(|| AppState {
        items: HashMap::new(),
    });

fn with_app_state<T>(
    f: impl FnOnce(&mut AppState) -> T,
) -> T {
    unsafe { f(&mut APP_STATE) }
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

pub fn add(
    item: NewTodo,
) -> AppResult<Todo> {
    let title =
        item.validate_title()?;

    let deadline =
        unix_time_from(&item.deadline)?;

    let id = Uuid::new_v4().to_string();

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

    with_app_state(
        |AppState { items }| {
            items.insert(
                item.id.clone(),
                item,
            );
        },
    );

    Ok(result)
}

pub fn update(
    id: String,
    change: UpdateTodo,
) -> AppResult<Todo> {
    if change.change_is_present() {
        let deadline_update =
            unix_time_from(
                &change.deadline,
            )?;

        with_app_state(
            |AppState { items }| {
                if let Some(todo) =
                    items.get_mut(&id)
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

                        if !{ title_update.is_empty() } &&
                            todo.title != title_update
                        {
                            todo.title = title_update.to_string();
                            modified = true;
                        }
                    }

                    if let Some(
                        priority_update,
                    ) =
                        change.priority
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
                        if todo.status != status_update {
                            todo.status = status_update;
                            modified = true;
                        }
                    }

                    if todo.deadline != deadline_update {
                        todo.deadline =
                            deadline_update;
                        modified = true;
                    }

                    if modified {
                        todo.updated_timestamp = unix_time_now!();
                    }

                    Ok(todo.clone())
                } else {
                    Err(item_not_found(
                        &id,
                    ))
                }
            },
        )
    } else {
        Err("At least one change must be present.".to_string())
    }
}

pub fn search(
    query: Query,
) -> AppResult<Vec<Todo>> {
    let deadline = unix_time_from(
        &query.deadline,
    )?;

    let limit: usize =
        query.validate_limit()?;

    with_app_state(
        |AppState { items }| {
            let mut result: Vec<_> =
                items
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
                        |t: &Todo| cmp::Reverse(t.priority)
                    );
                }
                Some(
                    QuerySort::Status,
                ) => {
                    result.sort_by_key(
                        |t| t.status,
                    );
                }
                Some(
                    QuerySort::Deadline,
                ) => {
                    result.sort_by_key(
                        |t| cmp::Reverse(t.deadline)
                    );
                }
                None => {
                    result.sort_by_key(
                        |t| {
                            t.title
                                .clone()
                        },
                    );
                }
            };

            Ok(result
                .into_iter()
                .take(limit)
                .collect())
        },
    )
}

pub fn get(
    id: String,
) -> AppResult<Todo> {
    with_app_state(
        |AppState { items }| {
            if let Some(item) =
                items.get(&id)
            {
                Ok(item.clone())
            } else {
                Err(item_not_found(&id))
            }
        },
    )
}

pub fn count() -> AppResult<u64> {
    with_app_state(
        |AppState { items }| {
            u64_from(items.len())
        },
    )
}

pub fn delete(
    id: String,
) -> AppResult<()> {
    with_app_state(
        |AppState { items }| {
            if items.contains_key(&id) {
                items.remove(&id);

                Ok(())
            } else {
                Err(item_not_found(&id))
            }
        },
    )
}

pub fn delete_done_items() -> u64 {
    with_app_state(
        |AppState { items }| {
            let mut count: u64 = 0;

            items.retain(|_, item| {
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
        },
    )
}

pub fn delete_all() -> AppResult<u64> {
    with_app_state(
        |AppState { items }| {
            let count = items.len();

            items.clear();

            u64_from(count)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
