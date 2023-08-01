use bindings::{export, exports::golem::todos::api::*};
use lib::{core::AppResult, todos};

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
        .title(item.title)
        .priority(priority_from_incoming(item.priority))
        .deadline(item.deadline)
        .build()
}

fn update_todo_from_incoming(item: UpdateTodo) -> todos::UpdateTodo {
    todos::UpdateTodo::builder()
        .title(item.title)
        .priority(item.priority.map(priority_from_incoming))
        .status(item.status.map(status_from_incoming))
        .deadline(item.deadline)
        .build()
}

fn query_from_incoming(query: Query) -> todos::Query {
    todos::Query::builder()
        .keyword(query.keyword)
        .priority(query.priority.map(priority_from_incoming))
        .status(query.status.map(status_from_incoming))
        .deadline(query.deadline)
        .sort(query.sort.map(query_sort_from_incoming))
        .limit(query.limit)
        .build()
}

fn todo_for_outgoing(t: todos::Todo) -> Todo {
    Todo {
        id: t.id().to_string(),
        title: t.title().to_string(),
        priority: priority_for_outgoing(t.priority()),
        deadline: t.deadline(),
        status: status_for_outgoing(t.status()),
        created_timestamp: t.created_timestamp(),
        updated_timestamp: t.updated_timestamp(),
    }
}

struct Todos;

impl Api for Todos {
    fn add(item: NewTodo) -> AppResult<Todo> {
        let result = todos::add(new_todo_from_incoming(item))?;

        Ok(todo_for_outgoing(result))
    }

    fn update(id: String, change: UpdateTodo) -> AppResult<Todo> {
        let result = todos::update(id, update_todo_from_incoming(change))?;

        Ok(todo_for_outgoing(result))
    }

    fn search(query: Query) -> AppResult<Vec<Todo>> {
        let found = todos::search(query_from_incoming(query))?;

        let result = found.into_iter().map(todo_for_outgoing).collect();

        Ok(result)
    }

    fn get(id: String) -> AppResult<Todo> {
        let result = todos::get(id)?;

        Ok(todo_for_outgoing(result))
    }

    fn count() -> AppResult<u64> {
        todos::count()
    }

    fn delete(id: String) -> AppResult<()> {
        todos::delete(id)
    }

    fn delete_done_items() -> u64 {
        todos::delete_done_items()
    }

    fn delete_all() -> AppResult<u64> {
        todos::delete_all()
    }
}
export!(Todos);
