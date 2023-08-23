use crate::{
    app_error::{
        bail, report, AppError,
        AppResult,
    },
    core::UnixTime,
    deadline, query, result_limit,
    sort_by::SortBy,
    title,
};
use binary_heap_plus::BinaryHeap;
use chrono::Utc;
use getset::{CopyGetters, Getters};
use nonempty_collections::{
    nes, NESet,
};
use std::collections::HashMap;
use strum_macros::EnumIter;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub type OptionalDeadlineInput =
    deadline::OptionalDeadlineInput;
pub type Query = query::Query;
pub type QuerySort = query::QuerySort;
pub type OptionalResultLimit =
    result_limit::OptionalResultLimit;
pub type Title = title::Title;

mod tests;

macro_rules! unix_time_now {
    () => {
        Utc::now().timestamp()
    };
}

#[derive(
    Clone,
    Copy,
    Debug,
    EnumIter,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
pub enum Status {
    InProgress,
    Backlog,
    Done,
}

#[derive(
    Clone,
    Copy,
    Debug,
    EnumIter,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Clone, TypedBuilder)]
pub struct NewTodo {
    title: Title,

    priority: Priority,

    #[builder(default = OptionalDeadlineInput::default())]
    deadline: OptionalDeadlineInput,
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct UpdateTodo {
    title: Option<Title>,

    priority: Option<Priority>,

    status: Option<Status>,

    deadline: OptionalDeadlineInput,
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
    id: Uuid,

    #[getset(get = "pub")]
    title: String,

    #[getset(get_copy = "pub")]
    priority: Priority,

    #[getset(get_copy = "pub")]
    status: Status,

    #[getset(get_copy = "pub")]
    created_timestamp: UnixTime,

    #[getset(get_copy = "pub")]
    updated_timestamp: UnixTime,

    #[getset(get_copy = "pub")]
    deadline: Option<UnixTime>,
}
impl Todo {
    fn is_in_id_set(
        &self,
        ids: &NESet<Uuid>,
    ) -> bool {
        ids.contains(&self.id)
    }

    fn is_in_priority_set(
        &self,
        priorities: &NESet<Priority>,
    ) -> bool {
        priorities
            .contains(&self.priority)
    }

    fn is_in_status_set(
        &self,
        statuses: &NESet<Status>,
    ) -> bool {
        statuses.contains(&self.status)
    }
}

#[derive(Default)]
pub struct TodoList(
    HashMap<Uuid, Todo>,
);
impl TodoList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        item: &NewTodo,
    ) -> AppResult<Todo> {
        let deadline = item
            .deadline
            .unix_time()?;

        let title =
            item.title.validated()?;

        let id = Uuid::new_v4();

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

        self.0.insert(todo.id, todo);

        Ok(result)
    }

    pub fn update(
        &mut self,
        id: Uuid,
        change: &UpdateTodo,
    ) -> AppResult<Todo> {
        if change.change_is_present() {
            let deadline_update =
                change
                    .deadline
                    .unix_time()?;

            if let Some(todo) =
                self.0.get_mut(&id)
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
                bail!(
                    AppError::TodoNotFound(id)
                )
            }
        } else {
            bail!(
                AppError::UpdateHasNoChanges
            )
        }
    }

    fn filter_by<'a>(
        &'a self,
        query: &'a Query,
        deadline: &'a Option<UnixTime>,
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
        let deadline = query
            .deadline()
            .unix_time()?;

        let top_n = query
            .limit()
            .validated()?;

        let sort =
            SortBy::from(query.sort());

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
        let deadline = query
            .deadline()
            .unix_time()?;

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
        id: Uuid,
    ) -> AppResult<Todo> {
        self.0
            .get(&id)
            .cloned()
            .ok_or_else(|| {
                report!(
                    AppError::TodoNotFound(id)
                )
            })
    }

    pub fn delete(
        &mut self,
        id: Uuid,
    ) -> AppResult<()> {
        self.0
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| {
                report!(
                    AppError::TodoNotFound(id)
                )
            })
    }

    fn delete_by<T>(
        &mut self,
        targets: &NESet<T>,
        should_delete: impl Fn(
            &Todo,
            &NESet<T>,
        )
            -> bool,
    ) -> usize {
        let mut count = 0;

        self.0.retain(|_, item| {
            !{
                should_delete(
                    item, targets,
                )
            } || {
                count += 1;
                false
            }
        });

        count
    }

    pub fn delete_by_ids(
        &mut self,
        targets: &NESet<Uuid>,
    ) -> usize {
        self.delete_by(
            targets,
            Todo::is_in_id_set,
        )
    }

    pub fn delete_by_priorities(
        &mut self,
        targets: &NESet<Priority>,
    ) -> usize {
        self.delete_by(
            targets,
            Todo::is_in_priority_set,
        )
    }

    pub fn delete_by_statuses(
        &mut self,
        targets: &NESet<Status>,
    ) -> usize {
        self.delete_by(
            targets,
            Todo::is_in_status_set,
        )
    }

    pub fn delete_by_status(
        &mut self,
        target: &Status,
    ) -> usize {
        self.delete_by_statuses(&nes![
            *target
        ])
    }

    pub fn delete_all(
        &mut self,
    ) -> usize {
        let count = self.count_all();

        self.0.clear();

        count
    }
}
