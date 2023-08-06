use crate::{
    core::{unix_time_from, AppResult},
    title::Title,
};
use binary_heap_plus::BinaryHeap;
use chrono::Utc;
use enum_iterator::Sequence;
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
    Hash,
    Ord,
    PartialOrd,
    Sequence,
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
    Hash,
    Ord,
    PartialOrd,
    Sequence,
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
    Eq, PartialEq, Ord, PartialOrd,
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

#[derive(Default, TypedBuilder)]
#[builder(field_defaults(default))]
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

#[derive(Clone, TypedBuilder)]
pub struct NewTodo {
    title: Title,

    priority: Priority,

    #[builder(default)]
    deadline: Option<String>,
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct UpdateTodo {
    title: Option<Title>,

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
    Hash,
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
    const NO_CHANGE_PROVIDED: &str =
        "At least one change must be present.";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        item: &NewTodo,
    ) -> AppResult<Todo> {
        let deadline = unix_time_from(
            &item.deadline,
        )?;

        let title =
            item.title.validated()?;

        let id =
            Uuid::new_v4().to_string();

        let now = unix_time_now!();

        let todo = Todo {
            id,
            title,
            priority: item.priority,
            deadline,
            status: Status::Backlog,
            created_timestamp: now,
            updated_timestamp: now,
        };

        let result = todo.clone();

        self.0.insert(
            todo.id.clone(),
            todo,
        );

        Ok(result)
    }

    pub fn update(
        &mut self,
        id: &str,
        change: &UpdateTodo,
    ) -> AppResult<Todo> {
        if change.change_is_present() {
            let deadline_update =
                unix_time_from(
                    &change.deadline,
                )?;

            if let Some(todo) =
                self.0.get_mut(id)
            {
                let mut modified =
                    false;

                if let Some(
                    title_update,
                ) = &change.title
                {
                    let title_update =
                        title_update
                            .validated(
                            )?;

                    if todo.title
                        != title_update
                    {
                        todo.title = title_update;
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
                Err(item_not_found(id))
            }
        } else {
            Err(Self::NO_CHANGE_PROVIDED
                .to_string())
        }
    }

    fn filter_by<'a>(
        &'a self,
        query: &'a Query,
        deadline: &'a Option<i64>,
    ) -> impl Iterator<Item = &Todo>
    {
        self.0
            .values()
            .filter(move |t| {
                query.match_keyword(t) &&
                query.match_priority(t) &&
                query.match_status(t) &&
                Query::match_deadline(deadline, t)
            })
    }

    pub fn search(
        &self,
        query: &Query,
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
            .filter_by(query, &deadline)
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
        query: &Query,
    ) -> AppResult<usize> {
        let deadline = unix_time_from(
            &query.deadline,
        )?;

        let count = self
            .filter_by(query, &deadline)
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
        target_status: &Status,
    ) -> usize {
        let mut count = 0;

        self.0.retain(|_, item| {
            if item.status
                == *target_status
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
    use crate::core::INVALID_DATE_TIME_FORMAT;
    use enum_iterator::all;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    macro_rules! new_todo_list {
        () => {
            TodoList::new()
        };
    }

    impl Query {
        fn empty() -> Self {
            Self::default()
        }
    }

    impl NewTodo {
        fn cloned_with_title(
            &self,
            title: &str,
        ) -> Self {
            Self {
                title: Title::new(
                    title,
                ),
                ..self.clone()
            }
        }
    }

    fn too_long_title() -> String {
        ('a'..='z')
            .map(|c| c.to_string())
            .collect()
    }

    #[test]
    fn todolist_search_should_return_empty_vec_when_there_is_no_todos(
    ) {
        assert!(new_todo_list!()
            .search(&Query::empty())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn todolist_get_should_fail_when_there_is_no_todos(
    ) {
        let id = "not-exist";

        assert_eq!(
            new_todo_list!()
                .get(id)
                .unwrap_err(),
            item_not_found(id)
        );
    }

    #[test]
    fn todolist_count_should_be_0_when_there_is_no_todos(
    ) {
        assert_eq!(
            new_todo_list!()
                .count_all(),
            0
        );
    }

    #[test]
    fn todolist_delete_should_fail_when_there_is_no_todos(
    ) {
        let id = "not-exist";

        assert_eq!(
            new_todo_list!()
                .delete(id)
                .unwrap_err(),
            item_not_found(id)
        );
    }

    #[test]
    fn todolist_delete_by_status_should_return_0_when_there_is_no_todos(
    ) {
        assert_eq!(
            new_todo_list!()
                .delete_by_status(
                    &Status::Done
                ),
            0
        );
    }

    #[test]
    fn todolist_delete_all_should_return_0_when_there_is_no_todos(
    ) {
        assert_eq!(
            new_todo_list!()
                .delete_all(),
            0
        );
    }

    #[test]
    fn todolist_add_should_return_newly_created_todo(
    ) {
        let mut todos =
            new_todo_list!();

        let title = "test";
        let priority = Priority::Medium;

        let item = NewTodo {
            title: Title::new(title),
            priority,
            deadline: None,
        };

        let actual =
            todos.add(&item).unwrap();

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
    fn todolist_add_should_fail_when_deadline_is_invalid(
    ) {
        let actual = new_todo_list!()
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .deadline(Some("abc".to_string()))
                    .build()
            ).unwrap_err();

        assert!(actual.contains(
            INVALID_DATE_TIME_FORMAT
        ));
    }

    #[test]
    fn todolist_add_should_fail_when_title_is_empty(
    ) {
        let actual = new_todo_list!()
            .add(
                &NewTodo::builder()
                    .title(Title::new(""))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap_err();

        assert_eq!(
            actual,
            Title::EMPTY_TITLE_ERROR
        )
    }

    #[test]
    fn todolist_add_should_fail_when_title_length_is_too_long(
    ) {
        let actual = new_todo_list!()
            .add(
                &NewTodo::builder()
                    .title(Title::new(too_long_title()))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap_err();

        assert_eq!(
            actual,
            Title::EXCEEDING_MAX_LEN_ERROR
        )
    }

    #[test]
    fn todolist_update_should_return_updated_todo(
    ) {
        let mut todos =
            new_todo_list!();

        let v1 = todos
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap();

        let update =
            UpdateTodo::builder()
                .title(Some(
                    Title::new("abc"),
                ))
                .priority(Some(
                    Priority::High,
                ))
                .deadline(Some(
                    "2022-01-01 19"
                        .to_string(),
                ))
                .build();

        let v2 = todos
            .update(&v1.id, &update)
            .unwrap();

        let item =
            todos.get(&v1.id).unwrap();

        assert_eq!(v2, item);
    }

    #[test]
    fn todolist_update_should_fail_when_no_change_is_provided(
    ) {
        let mut todos =
            new_todo_list!();

        let v1 = todos
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap();

        let update =
            UpdateTodo::builder()
                .build();

        let actual = todos
            .update(&v1.id, &update)
            .unwrap_err();

        assert!(actual.contains(
            TodoList::NO_CHANGE_PROVIDED
        ));
    }

    #[test]
    fn todolist_update_should_fail_when_title_update_is_empty(
    ) {
        let mut todos =
            new_todo_list!();

        let v1 = todos
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap();

        let update =
            UpdateTodo::builder()
                .title(Some(
                    Title::new(""),
                ))
                .build();

        let actual = todos
            .update(&v1.id, &update)
            .unwrap_err();

        assert_eq!(
            actual,
            Title::EMPTY_TITLE_ERROR
        );
    }

    #[test]
    fn todolist_update_should_fail_when_title_update_is_too_long(
    ) {
        let mut todos =
            new_todo_list!();

        let v1 = todos
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap();

        let update =
            UpdateTodo::builder()
                .title(Some(
                    Title::new(
                        too_long_title(
                        ),
                    ),
                ))
                .build();

        let actual = todos
            .update(&v1.id, &update)
            .unwrap_err();

        assert_eq!(
            actual,
            Title::EXCEEDING_MAX_LEN_ERROR
        );
    }

    #[test]
    fn todolist_update_should_fail_when_deadline_is_invalid(
    ) {
        let mut todos =
            new_todo_list!();

        let v1 = todos
            .add(
                &NewTodo::builder()
                    .title(Title::new("abc"))
                    .priority(Priority::Medium)
                    .build()
            ).unwrap();

        let update =
            UpdateTodo::builder()
                .deadline(Some(
                    "xyz".to_string(),
                ))
                .build();

        let actual = todos
            .update(&v1.id, &update)
            .unwrap_err();

        assert!(actual.contains(
            INVALID_DATE_TIME_FORMAT
        ));
    }

    fn add_todos(
        todos: &mut TodoList,
    ) -> AppResult<Vec<Todo>> {
        let low_todo = NewTodo {
            title: Title::new("a"),
            priority: Priority::Low,
            deadline: None,
        };

        let todo_a = todos
            .add(&low_todo.clone())?;

        let todo_b = todos.add(
            &low_todo
                .cloned_with_title("b"),
        )?;
        let todo_c = todos.add(
            &low_todo
                .cloned_with_title("c"),
        )?;

        let med_todo = NewTodo {
            priority: Priority::Medium,
            ..low_todo
        };

        let todo_d = todos.add(
            &med_todo
                .cloned_with_title("d"),
        )?;
        let todo_e = todos.add(
            &med_todo
                .cloned_with_title("e"),
        )?;
        let todo_f = todos.add(
            &med_todo
                .cloned_with_title("f"),
        )?;

        let high_todo = NewTodo {
            priority: Priority::High,
            ..med_todo
        };

        let todo_g = todos.add(
            &high_todo
                .cloned_with_title("g"),
        )?;
        let todo_h = todos.add(
            &high_todo
                .cloned_with_title("h"),
        )?;
        let todo_i = todos.add(
            &high_todo
                .cloned_with_title("i"),
        )?;

        let result = vec![
            todo_a, todo_b, todo_c,
            todo_d, todo_e, todo_f,
            todo_g, todo_h, todo_i,
        ];

        Ok(result)
    }

    #[test]
    fn todolist_count_by_count_all_delete_all_should_all_work_as_expected(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let count = items.len();

        assert_eq!(
            todos.count_all(),
            count
        );

        let all_priorities: Vec<_> =
            all::<Priority>().collect();

        for p in all_priorities {
            let query =
                Query::builder()
                    .priority(Some(p))
                    .build();

            assert_eq!(
                todos
                    .count_by(&query)
                    .unwrap(),
                3
            );
        }

        assert_eq!(
            todos.delete_all(),
            count
        );
        assert_eq!(
            todos.count_all(),
            0
        );
    }

    #[test]
    fn todolist_update_get_delete_by_status_should_all_work_as_expected(
    ) {
        let the_status = Status::Done;

        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let count = items.len();

        for item in &items {
            assert_eq!(
                todos
                    .get(&item.id)
                    .unwrap(),
                *item
            )
        }

        let search_for_done_items =
            Query::builder()
                .status(Some(
                    the_status,
                ))
                .build();

        assert_eq!(
            todos
                .count_by(
                    &search_for_done_items
                )
                .unwrap(),
            0
        );

        for item in &items {
            let update =
                UpdateTodo::builder()
                    .status(Some(
                        the_status,
                    ))
                    .build();
            let updated = todos
                .update(
                    &item.id, &update,
                )
                .unwrap();

            assert_eq!(
                updated,
                Todo {
                    status: the_status,
                    ..item.clone()
                }
            )
        }

        assert_eq!(
            todos
                .count_by(
                    &search_for_done_items
                )
                .unwrap(),
            count
        );

        assert_eq!(
            todos.delete_by_status(
                &the_status
            ),
            count
        );
        assert_eq!(
            todos.count_all(),
            0
        );
    }

    #[test]
    fn todolist_search_should_return_matching_todos(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            _, _, _, _, _, _, todo_g, todo_h, todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect("Vec doesn't have 9 elements!");

        let query = Query::builder()
            .priority(Some(
                Priority::High,
            ))
            .build();
        let actual: HashSet<_> = todos
            .search(&query)
            .unwrap()
            .into_iter()
            .collect();

        assert_eq!(
            actual,
            hashset![
                todo_g, todo_h, todo_i
            ]
        );
    }

    #[test]
    fn todolist_search_should_return_todos_in_requested_order(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            todo_a, todo_b, todo_c,
            todo_d, todo_e, todo_f,
            todo_g, todo_h, todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect("Vec doesn't have 9 elements!");

        // sort by priority
        let mut actual_highs = todos
            .search(
                &Query::builder()
                    .limit(Some(5))
                    .sort(Some(QuerySort::Priority))
                    .build()
            )
            .unwrap();

        let split_at = 3;

        let actual_mediums: HashSet<_> =
            actual_highs
                .split_off(split_at)
                .into_iter()
                .collect();

        let actual_highs: HashSet<_> =
            actual_highs
                .into_iter()
                .collect();

        let expected_highs = hashset! {
            todo_g,
            todo_h,
            todo_i,
        };
        let expected_mediums = hashset! {
            todo_d.clone(),
            todo_e.clone(),
            todo_f,
        };

        assert_eq!(
            actual_highs,
            expected_highs
        );
        assert!(actual_mediums
            .is_subset(
                &expected_mediums
            ));

        // sort by title alphabetically
        let actual = todos
            .search(
                &Query::builder()
                    .limit(Some(5))
                    .build(),
            )
            .unwrap();

        let expected = vec![
            todo_a, todo_b, todo_c,
            todo_d, todo_e,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn todolist_search_should_fail_when_deadline_is_invalid(
    ) {
        let query = Query::builder()
            .deadline(Some(
                "abc".to_string(),
            ))
            .build();

        let actual = new_todo_list!()
            .search(&query)
            .unwrap_err();

        assert!(actual.contains(
            INVALID_DATE_TIME_FORMAT
        ));
    }

    #[test]
    fn todolist_count_by_should_fail_when_deadline_is_invalid(
    ) {
        let query = Query::builder()
            .deadline(Some(
                "abc".to_string(),
            ))
            .build();

        let actual = new_todo_list!()
            .count_by(&query)
            .unwrap_err();

        assert!(actual.contains(
            INVALID_DATE_TIME_FORMAT
        ));
    }
}
