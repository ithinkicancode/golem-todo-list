use bindings::{export, exports::golem::todos::api::*};
use lib::{
    app_error::AppResultExt,
    core::{u64_from, uuid_from, AppResult},
    todos::{self, OptionalDeadlineInput, OptionalResultLimit, Title, TodoList},
};
use once_cell::sync::Lazy;

const COMPONENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SCHEMA_VERSION: u64 = 1;

/*
 Unfortunately, I cannot implement the `From` trait because I own neither the
 wit-generated crate (in the target directory) nor the trait (Rust's orphan rule).
 I cannot use the `newtype` pattern as a workaround either because the wit-generated
 structs/enums are my APIs.
*/

fn priority_from_incoming(p: Priority) -> todos::Priority {
    match p {
        Priority::High => todos::Priority::High,
        Priority::Medium => todos::Priority::Medium,
        Priority::Low => todos::Priority::Low,
    }
}

fn priority_for_outgoing(p: todos::Priority) -> Priority {
    match p {
        todos::Priority::High => Priority::High,
        todos::Priority::Medium => Priority::Medium,
        todos::Priority::Low => Priority::Low,
    }
}

fn status_from_incoming(s: Status) -> todos::Status {
    match s {
        Status::Done => todos::Status::Done,
        Status::InProgress => todos::Status::InProgress,
        Status::Backlog => todos::Status::Backlog,
    }
}

fn status_for_outgoing(s: todos::Status) -> Status {
    match s {
        todos::Status::Done => Status::Done,
        todos::Status::InProgress => Status::InProgress,
        todos::Status::Backlog => Status::Backlog,
    }
}

fn query_sort_from_incoming(sort: QuerySort) -> todos::QuerySort {
    match sort {
        QuerySort::Deadline => todos::QuerySort::Deadline,
        QuerySort::Priority => todos::QuerySort::Priority,
        QuerySort::Status => todos::QuerySort::Status,
    }
}

fn new_todo_from_incoming(item: NewTodo) -> todos::NewTodo {
    todos::NewTodo::builder()
        .title(Title::new(item.title))
        .priority(priority_from_incoming(item.priority))
        .deadline(OptionalDeadlineInput::new(item.deadline))
        .build()
}

fn update_todo_from_incoming(item: UpdateTodo) -> todos::UpdateTodo {
    todos::UpdateTodo::builder()
        .title(item.title.map(Title::new))
        .priority(item.priority.map(priority_from_incoming))
        .status(item.status.map(status_from_incoming))
        .deadline(OptionalDeadlineInput::new(item.deadline))
        .build()
}

fn query_from_incoming(query: Query) -> todos::Query {
    todos::Query::builder()
        .keyword(query.keyword)
        .priority(query.priority.map(priority_from_incoming))
        .status(query.status.map(status_from_incoming))
        .deadline(OptionalDeadlineInput::new(query.deadline))
        .sort(query.sort.map(query_sort_from_incoming))
        .limit(OptionalResultLimit::new(query.limit))
        .build()
}

fn filter_from_incoming(filter: Filter) -> todos::Query {
    todos::Query::builder()
        .keyword(filter.keyword)
        .priority(filter.priority.map(priority_from_incoming))
        .status(filter.status.map(status_from_incoming))
        .deadline(OptionalDeadlineInput::new(filter.deadline))
        .build()
}

fn todo_for_outgoing(t: todos::Todo) -> Todo {
    Todo {
        id: t.id().to_string(),
        title: t.title().into(),
        priority: priority_for_outgoing(t.priority()),
        deadline: t.deadline(),
        status: status_for_outgoing(t.status()),
        created_timestamp: t.created_timestamp(),
        updated_timestamp: t.updated_timestamp(),
    }
}

struct AppState(TodoList);

static mut APP_STATE: Lazy<AppState> = Lazy::new(|| AppState(TodoList::new()));

fn with_app_state<T>(f: impl FnOnce(&mut AppState) -> T) -> T {
    unsafe { f(&mut APP_STATE) }
}

struct Todos;

impl Api for Todos {
    fn add(item: NewTodo) -> AppResult<Todo> {
        with_app_state(|AppState(todos)| {
            let result = todos.add(&new_todo_from_incoming(item)).err_as_string()?;

            Ok(todo_for_outgoing(result))
        })
    }

    fn update(id: String, change: UpdateTodo) -> AppResult<Todo> {
        with_app_state(|AppState(todos)| {
            let id = uuid_from(&id)?;

            let result = todos
                .update(id, &update_todo_from_incoming(change))
                .err_as_string()?;

            Ok(todo_for_outgoing(result))
        })
    }

    fn search(query: Query) -> AppResult<Vec<Todo>> {
        with_app_state(|AppState(todos)| {
            let found = todos.search(&query_from_incoming(query)).err_as_string()?;

            let result = found.into_iter().map(todo_for_outgoing).collect();

            Ok(result)
        })
    }

    fn count_by(filter: Filter) -> AppResult<u64> {
        with_app_state(|AppState(todos)| {
            let count = todos
                .count_by(&filter_from_incoming(filter))
                .err_as_string()?;

            u64_from(count)
        })
    }

    fn count_all() -> AppResult<u64> {
        with_app_state(|AppState(todos)| u64_from(todos.count_all()))
    }

    fn get(id: String) -> AppResult<Todo> {
        with_app_state(|AppState(todos)| {
            let id = uuid_from(&id)?;

            let result = todos.get(id).err_as_string()?;

            Ok(todo_for_outgoing(result))
        })
    }

    fn delete(id: String) -> AppResult<()> {
        with_app_state(|AppState(todos)| {
            let id = uuid_from(&id)?;

            todos.delete(id).err_as_string()
        })
    }

    fn delete_done_items() -> AppResult<u64> {
        with_app_state(|AppState(todos)| {
            let count = todos.delete_by_status(&todos::Status::Done);

            u64_from(count)
        })
    }

    fn delete_all() -> AppResult<u64> {
        with_app_state(|AppState(todos)| u64_from(todos.delete_all()))
    }

    fn meta() -> MetaData {
        MetaData {
            component_version: COMPONENT_VERSION.into(),
            schema_version: SCHEMA_VERSION,
        }
    }
}
export!(Todos);
