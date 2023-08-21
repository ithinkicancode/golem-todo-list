use bindings::{export, exports::golem::todos::api::*};
use lib::{
    app_error::AppResultExt,
    core::{u64_from, uuid_from, AppResult},
    todos::{self, Title, TodoList},
};
use once_cell::sync::Lazy;
use paste::paste;
use std::mem;

const COMPONENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SCHEMA_VERSION: u64 = 1;

/*
 Unfortunately, I cannot implement the `From` trait because I own neither the
 wit-generated crate (in the target directory) nor the trait (Rust's orphan rule).
 I cannot use the `newtype` pattern as a workaround either because the wit-generated
 structs/enums are my APIs.
*/

macro_rules! convert_enum_from_incoming {
    (
        $the_enum:ty,
        $internal:ident
    ) => {
        paste! {
            fn [<$the_enum:lower _from_incoming>](
                the_enum: $the_enum
            ) -> $internal::$the_enum {
                unsafe { mem::transmute(the_enum) }
            }
        }
    };
}

macro_rules! convert_enum_for_outgoing {
    (
        $the_enum:ty,
        $internal:ident
    ) => {
        paste! {
            fn [<$the_enum:lower _for_outgoing>](
                the_enum: $internal::$the_enum
            ) -> $the_enum {
                unsafe { mem::transmute(the_enum) }
            }
        }
    };
}

macro_rules! convert_enum_both_ways {
    (
        $the_enum:ty,
        $internal:ident
    ) => {
        convert_enum_from_incoming!($the_enum, $internal);

        convert_enum_for_outgoing!($the_enum, $internal);
    };
}

convert_enum_both_ways!(Priority, todos);
convert_enum_both_ways!(Status, todos);

convert_enum_from_incoming!(QuerySort, todos);

fn new_todo_from_incoming(item: NewTodo) -> todos::NewTodo {
    todos::NewTodo::builder()
        .title(item.title.into())
        .priority(priority_from_incoming(item.priority))
        .deadline(item.deadline.into())
        .build()
}

fn update_todo_from_incoming(item: UpdateTodo) -> todos::UpdateTodo {
    todos::UpdateTodo::builder()
        .title(item.title.map(Title::new))
        .priority(item.priority.map(priority_from_incoming))
        .status(item.status.map(status_from_incoming))
        .deadline(item.deadline.into())
        .build()
}

fn query_from_incoming(query: Query) -> todos::Query {
    todos::Query::builder()
        .keyword(query.keyword)
        .priority(query.priority.map(priority_from_incoming))
        .status(query.status.map(status_from_incoming))
        .deadline(query.deadline.into())
        .sort(query.sort.map(querysort_from_incoming))
        .limit(query.limit.into())
        .build()
}

fn filter_from_incoming(filter: Filter) -> todos::Query {
    todos::Query::builder()
        .keyword(filter.keyword)
        .priority(filter.priority.map(priority_from_incoming))
        .status(filter.status.map(status_from_incoming))
        .deadline(filter.deadline.into())
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
