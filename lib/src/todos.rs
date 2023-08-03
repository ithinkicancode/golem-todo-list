use crate::core::{
    unix_time_from, AppResult,
};
use binary_heap_plus::BinaryHeap;
use chrono::Utc;
use getset::{CopyGetters, Getters};
use std::{
    cmp, collections::HashMap,
    num::TryFromIntError,
};
use typed_builder::TypedBuilder;
use uuid::Uuid;

type ResultCap = u32;

const QUERY_DEFAULT_LIMIT: ResultCap =
    10;

const QUERY_MAX_LIMIT: ResultCap = 100;

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

#[derive(
    Ord, Eq, PartialEq, PartialOrd,
)]
enum SortBy {
    Deadline(cmp::Reverse<Option<i64>>),
    Priority(cmp::Reverse<Priority>),
    Status(Status),
    Title(String),
}
impl SortBy {
    fn from(
        query_sort: &Option<QuerySort>,
    ) -> impl Fn(&Todo) -> Self + '_
    {
        move |t: &Todo| match query_sort
        {
            Some(
                QuerySort::Priority,
            ) => SortBy::Priority(
                cmp::Reverse(
                    t.priority,
                ),
            ),
            Some(QuerySort::Status) => {
                SortBy::Status(t.status)
            }
            Some(
                QuerySort::Deadline,
            ) => SortBy::Deadline(
                cmp::Reverse(
                    t.deadline,
                ),
            ),
            None => SortBy::Title(
                t.title.clone(),
            ),
        }
    }
}

#[derive(
    Clone, Default, TypedBuilder,
)]
pub struct Query {
    keyword: Option<String>,
    priority: Option<Priority>,
    status: Option<Status>,
    deadline: Option<String>,
    sort: Option<QuerySort>,
    limit: Option<ResultCap>,
}
impl Query {
    fn validate_limit(
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

    fn match_keyword(
        &self,
        todo: &Todo,
    ) -> bool {
        self.keyword
            .as_ref()
            .map(|keyword| {
                todo.title
                    .contains(keyword)
            })
            .unwrap_or(true)
    }

    fn match_priority(
        &self,
        todo: &Todo,
    ) -> bool {
        self.priority
            .map(|p| p == todo.priority)
            .unwrap_or(true)
    }

    fn match_status(
        &self,
        todo: &Todo,
    ) -> bool {
        self.status
            .map(|s| s == todo.status)
            .unwrap_or(true)
    }

    fn match_deadline(
        deadline: &Option<i64>,
        todo: &Todo,
    ) -> bool {
        deadline
            .map(|deadline| {
                if let Some(before) =
                    todo.deadline
                {
                    before <= deadline
                } else {
                    true
                }
            })
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
    Clone,
    Debug,
    Eq,
    PartialEq,
    Getters,
    CopyGetters,
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

#[derive(Default)]
pub struct TodoList(
    HashMap<String, Todo>,
);
impl TodoList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(
        &mut self,
        item: NewTodo,
    ) -> AppResult<Todo> {
        let deadline = unix_time_from(
            &item.deadline,
        )?;

        let title = item
            .validate_title()?
            .to_string();

        let id =
            Uuid::new_v4().to_string();

        let now: i64 = unix_time_now!();

        let item = Todo {
            id,
            title,
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

    fn filter_by<'a>(
        &'a self,
        query: &'a Query,
        deadline: Option<i64>,
    ) -> impl Iterator<Item = &Todo>
    {
        self.0
            .values()
            .filter(move |t| {
                query.match_keyword(t) &&
                query.match_priority(t) &&
                query.match_status(t) &&
                Query::match_deadline(&deadline, t)
            })
    }

    pub fn search(
        &self,
        query: Query,
    ) -> AppResult<Vec<Todo>> {
        let deadline = unix_time_from(
            &query.deadline,
        )?;

        let top_n =
            query.validate_limit()?;

        let sort =
            SortBy::from(&query.sort);

        let mut heap =
            BinaryHeap::with_capacity_by_key(
                top_n,
                &sort
            );

        let mut count: usize = 0;

        for t in self
            .filter_by(&query, deadline)
        {
            if count < top_n {
                heap.push(t.clone());

                count += 1;
            } else if let Some(
                mut todo,
            ) =
                heap.peek_mut()
            {
                if sort(&todo) > sort(t)
                {
                    *todo = t.clone();
                }
            } else {
                unreachable!("DEFECT: Heap in `TodoList::search` is empty.");
            }
        }

        Ok(heap.into_sorted_vec())
    }

    pub fn count_by(
        &self,
        query: Query,
    ) -> AppResult<usize> {
        let deadline = unix_time_from(
            &query.deadline,
        )?;

        let count = self
            .filter_by(&query, deadline)
            .count();

        Ok(count)
    }

    pub fn count_all(&self) -> usize {
        self.0.len()
    }

    pub fn get(
        &self,
        id: &str,
    ) -> AppResult<Todo> {
        self.0
            .get(id)
            .cloned()
            .ok_or_else(|| {
                item_not_found(id)
            })
    }

    pub fn delete(
        &mut self,
        id: &str,
    ) -> AppResult<()> {
        self.0
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| {
                item_not_found(id)
            })
    }

    pub fn delete_by_status(
        &mut self,
        target_status: Status,
    ) -> u64 {
        let mut count: u64 = 0;

        self.0.retain(|_, item| {
            if item.status
                == target_status
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
    ) -> usize {
        let count = self.count_all();

        self.0.clear();

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    impl Query {
        fn empty() -> Self {
            Default::default()
        }
    }

    #[test]
    fn todolist_search_should_return_empty_vec_when_there_is_no_todos(
    ) {
        let todos = TodoList::new();

        assert!(todos
            .search(Query::empty())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn todolist_get_should_fail_when_there_is_no_todos(
    ) {
        let todos = TodoList::new();

        let id = "not-exist";

        assert_eq!(
            todos.get(id).unwrap_err(),
            item_not_found(id)
        );
    }

    #[test]
    fn todolist_count_should_be_0_when_there_is_no_todos(
    ) {
        let todos = TodoList::new();

        assert_eq!(
            todos.count_all(),
            0
        );
    }

    #[test]
    fn todolist_delete_should_fail_when_there_is_no_todos(
    ) {
        let mut todos = TodoList::new();

        let id = "not-exist";

        assert_eq!(
            todos
                .delete(id)
                .unwrap_err(),
            item_not_found(id)
        );
    }

    #[test]
    fn todolist_delete_by_status_should_return_0_when_there_is_no_todos(
    ) {
        let mut todos = TodoList::new();

        assert_eq!(
            todos.delete_by_status(
                Status::Done
            ),
            0
        );
    }

    #[test]
    fn todolist_delete_all_should_return_0_when_there_is_no_todos(
    ) {
        let mut todos = TodoList::new();

        assert_eq!(
            todos.delete_all(),
            0
        );
    }

    #[test]
    fn todolist_add_should_return_newly_created_todo(
    ) {
        let mut todos = TodoList::new();

        let title = "test";
        let priority = Priority::Medium;

        let item = NewTodo {
            title: title.to_string(),
            priority,
            deadline: None,
        };

        let actual =
            todos.add(item).unwrap();

        assert_eq!(actual.title, title);
        assert_eq!(
            actual.priority,
            priority
        );
        assert_eq!(
            actual.status,
            Status::Backlog
        );
        assert!(actual
            .deadline
            .is_none());
        assert_eq!(
            todos.count_all(),
            1
        );
    }

    #[test]
    fn todolist_search_should_return_matching_todos_in_requested_order(
    ) {
        let mut todos = TodoList::new();

        let todo1 = todos
            .add(NewTodo {
                title: "x".to_string(),
                priority: Priority::Low,
                deadline: None,
            })
            .unwrap();
        let todo2 = todos
            .add(NewTodo {
                title: "y".to_string(),
                priority:
                    Priority::Medium,
                deadline: None,
            })
            .unwrap();
        let todo3 = todos
            .add(NewTodo {
                title: "z".to_string(),
                priority:
                    Priority::High,
                deadline: None,
            })
            .unwrap();

        let actual = todos
            .search(Query {
                keyword: None,
                priority: None,
                status: None,
                deadline: None,
                limit: Some(2),
                sort: Some(
                    QuerySort::Priority,
                ),
            })
            .unwrap();

        let expected =
            vec![todo3, todo2.clone()];

        assert_eq!(actual, expected);

        let actual = todos
            .search(Query {
                keyword: None,
                priority: None,
                status: None,
                deadline: None,
                sort: None,
                limit: Some(2),
            })
            .unwrap();

        let expected =
            vec![todo1, todo2];

        assert_eq!(actual, expected);
    }
}
