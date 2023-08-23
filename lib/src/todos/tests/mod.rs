#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_app_error,
        deadline::USER_DATE_TIME_FORMAT,
    };
    use maplit::hashset;
    use memoize::memoize;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use strum::IntoEnumIterator;
    use uuid::uuid;

    macro_rules! new_todo_list {
        () => {
            TodoList::new()
        };
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

    impl UpdateTodo {
        fn empty() -> Self {
            UpdateTodo::builder()
                .build()
        }
    }

    impl TodoList {
        fn update_status(
            &mut self,
            id: Uuid,
            status: Status,
        ) -> AppResult<Todo> {
            self.update(
                id,
                &UpdateTodo::builder()
                    .status(Some(
                        status,
                    ))
                    .build(),
            )
        }

        fn update_priority(
            &mut self,
            id: Uuid,
            priority: Priority,
        ) -> AppResult<Todo> {
            self.update(
                id,
                &UpdateTodo::builder()
                    .priority(Some(
                        priority,
                    ))
                    .build(),
            )
        }

        fn update_deadline(
            &mut self,
            id: Uuid,
            deadline: OptionalDeadlineInput,
        ) -> AppResult<Todo> {
            self.update(
                id,
                &UpdateTodo::builder()
                    .deadline(deadline)
                    .build(),
            )
        }
    }

    #[memoize]
    fn too_long_title() -> String {
        ['a'; Title::MAX_LEN + 1]
            .into_iter()
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

    const NON_EXISTENT_ID: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

    #[test]
    fn todolist_get_should_fail_when_there_is_no_todos(
    ) {
        let actual = new_todo_list!()
            .get(NON_EXISTENT_ID);

        let expected =
            AppError::TodoNotFound(
                NON_EXISTENT_ID,
            );

        assert_app_error!(
            actual, expected
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
        let actual = new_todo_list!()
            .delete(NON_EXISTENT_ID);

        let expected =
            AppError::TodoNotFound(
                NON_EXISTENT_ID,
            );

        assert_app_error!(
            actual, expected
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
            deadline: OptionalDeadlineInput::none(),
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
        let invalid_date_time = "abc";

        let new_todo = NewTodo::builder()
            .title(
                Title::new("abc")
            )
            .priority(Priority::Medium)
            .deadline(
                OptionalDeadlineInput::some(invalid_date_time)
            )
            .build();

        let actual = new_todo_list!()
            .add(&new_todo);

        let expected = AppError::DateTimeParseError {
                input: invalid_date_time.into(),
                expected_format: USER_DATE_TIME_FORMAT.into()
            };

        assert_app_error!(
            actual, expected
        )
    }

    #[test]
    fn todolist_add_should_fail_when_title_is_empty(
    ) {
        let actual = new_todo_list!()
            .add(
            &NewTodo::builder()
                .title(Title::new(""))
                .priority(
                    Priority::Medium,
                )
                .build(),
        );

        let expected =
            AppError::EmptyTodoTitle;

        assert_app_error!(
            actual, expected
        )
    }

    #[test]
    fn todolist_add_should_fail_when_title_length_is_too_long(
    ) {
        let title = too_long_title();

        let actual = new_todo_list!()
            .add(
            &NewTodo::builder()
                .title(Title::new(
                    title.clone(),
                ))
                .priority(
                    Priority::Medium,
                )
                .build(),
        );

        let expected = AppError::TooLongTodoTitle {
            input: title,
            expected_len: Title::MAX_LEN
        };

        assert_app_error!(
            actual, expected
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
                .deadline(
                    OptionalDeadlineInput::some("2022-01-01 19")
                )
                .build();

        let v2 = todos
            .update(v1.id, &update)
            .unwrap();

        let item =
            todos.get(v1.id).unwrap();

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
            UpdateTodo::empty();

        let actual = todos
            .update(v1.id, &update);

        let expected =
            AppError::UpdateHasNoChanges;

        assert_app_error!(
            actual, expected
        )
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
                    Title::new("   "),
                ))
                .build();

        let actual = todos
            .update(v1.id, &update);

        let expected =
            AppError::EmptyTodoTitle;

        assert_app_error!(
            actual, expected
        )
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

        let title = too_long_title();

        let update =
            UpdateTodo::builder()
                .title(Some(
                    Title::new(
                        title.clone(),
                    ),
                ))
                .build();

        let actual = todos
            .update(v1.id, &update);

        let expected = AppError::TooLongTodoTitle {
                input: title,
                expected_len: Title::MAX_LEN
            };

        assert_app_error!(
            actual, expected
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

        let invalid_date_time = "abc";

        let update =
            UpdateTodo::builder()
                .deadline(
                    OptionalDeadlineInput::some(invalid_date_time)
                )
                .build();

        let actual = todos
            .update(v1.id, &update);

        let expected = AppError::DateTimeParseError {
                input: invalid_date_time.into(),
                expected_format: USER_DATE_TIME_FORMAT.into()
            };

        assert_app_error!(
            actual, expected
        )
    }

    fn add_todos(
        todos: &mut TodoList,
    ) -> AppResult<Vec<Todo>> {
        let low_todo = NewTodo {
            title: Title::new("a"),
            priority: Priority::Low,
            deadline: OptionalDeadlineInput::none(),
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
            // all::<Priority>().collect();
            Priority::iter().collect();

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
            let actual = todos
                .get(item.id)
                .unwrap();

            assert_eq!(actual, *item)
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
                    item.id, &update,
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
            todo_g, todo_h, todo_i
        ] =
            <[Todo; 3]>::try_from(
                items
                    .into_iter()
                    .skip(6)
                    .collect::<Vec<_>>()
            ).expect(
                "`items` vec should contain 9 elements"
            );

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
                .expect(
                    "`items` vec should contain 9 elements"
                );

        let query = Query::builder()
            .limit(
                OptionalResultLimit::some(5)
            )
            .sort(Some(QuerySort::Priority))
            .build();

        let search_result = todos
            .search(&query)
            .unwrap();

        let chunk_count = 2;

        let chunks: Vec<_> = search_result
            .chunks(3)
            .map(|chunk| {
                chunk.into_iter().collect::<HashSet<_>>()
            })
            .take(chunk_count)
            .collect();

        let [
            actual_highs,
            actual_mediums
        ] =
            <[HashSet<_>; 2]>::try_from(chunks).expect(
                format!(
                    "`chunks` vec should contain {} elements",
                    chunk_count
                ).as_str()
            );

        let expected_highs = hashset! {
            &todo_g,
            &todo_h,
            &todo_i,
        };
        let expected_mediums = hashset! {
            &todo_d,
            &todo_e,
            &todo_f,
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
        let query = Query::builder()
            .limit(
                OptionalResultLimit::some(5)
            )
            .build();

        let actual = todos
            .search(&query)
            .unwrap();

        let expected = vec![
            todo_a, todo_b, todo_c,
            todo_d, todo_e,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn todolist_search_should_sort_todos_by_status_in_order_of_inprogress_backlog_done(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            mut todo_a, mut todo_b, mut todo_c,
            mut todo_d, mut todo_e, mut todo_f,
            mut todo_g, mut todo_h, mut todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect(
                    "`items` vec should contain 9 elements"
                );

        todo_a = todos
            .update_status(
                todo_a.id,
                Status::Backlog,
            )
            .unwrap();
        todo_b = todos
            .update_status(
                todo_b.id,
                Status::InProgress,
            )
            .unwrap();
        todo_c = todos
            .update_status(
                todo_c.id,
                Status::Done,
            )
            .unwrap();
        todo_d = todos
            .update_status(
                todo_d.id,
                Status::Backlog,
            )
            .unwrap();
        todo_e = todos
            .update_status(
                todo_e.id,
                Status::InProgress,
            )
            .unwrap();
        todo_f = todos
            .update_status(
                todo_f.id,
                Status::Done,
            )
            .unwrap();
        todo_g = todos
            .update_status(
                todo_g.id,
                Status::Backlog,
            )
            .unwrap();
        todo_h = todos
            .update_status(
                todo_h.id,
                Status::InProgress,
            )
            .unwrap();
        todo_i = todos
            .update_status(
                todo_i.id,
                Status::Done,
            )
            .unwrap();

        let query = Query::builder()
            .sort(Some(
                QuerySort::Status,
            ))
            .build();

        let search_result = todos
            .search(&query)
            .unwrap();

        let chunk_count = 3;

        let chunks: Vec<_> = search_result
            .chunks(3)
            .map(|chunk| {
                chunk.into_iter().collect::<HashSet<_>>()
            })
            .take(chunk_count)
            .collect();

        let [
            actual_in_progress,
            actual_backlog,
            actual_done,
        ] =
            <[HashSet<_>; 3]>::try_from(chunks).expect(
                format!(
                    "`chunks` vec should contain {} elements",
                    chunk_count
                ).as_str()
            );

        assert_eq!(
            actual_in_progress,
            hashset! {
                &todo_b,
                &todo_e,
                &todo_h
            }
        );
        assert_eq!(
            actual_backlog,
            hashset! {
                &todo_a,
                &todo_d,
                &todo_g
            }
        );
        assert_eq!(
            actual_done,
            hashset! {
                &todo_c,
                &todo_f,
                &todo_i
            }
        );
    }

    #[test]
    fn todolist_search_should_sort_todos_by_priority_in_order_of_high_medium_low(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            mut todo_a, mut todo_b, mut todo_c,
            mut todo_d, mut todo_e, mut todo_f,
            mut todo_g, mut todo_h, mut todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect(
                    "`items` vec should contain 9 elements"
                );

        todo_a = todos
            .update_priority(
                todo_a.id,
                Priority::Medium,
            )
            .unwrap();
        todo_b = todos
            .update_priority(
                todo_b.id,
                Priority::High,
            )
            .unwrap();
        todo_c = todos
            .update_priority(
                todo_c.id,
                Priority::Low,
            )
            .unwrap();
        todo_d = todos
            .update_priority(
                todo_d.id,
                Priority::Medium,
            )
            .unwrap();
        todo_e = todos
            .update_priority(
                todo_e.id,
                Priority::High,
            )
            .unwrap();
        todo_f = todos
            .update_priority(
                todo_f.id,
                Priority::Low,
            )
            .unwrap();
        todo_g = todos
            .update_priority(
                todo_g.id,
                Priority::Medium,
            )
            .unwrap();
        todo_h = todos
            .update_priority(
                todo_h.id,
                Priority::High,
            )
            .unwrap();
        todo_i = todos
            .update_priority(
                todo_i.id,
                Priority::Low,
            )
            .unwrap();

        let query = Query::builder()
            .sort(Some(
                QuerySort::Priority,
            ))
            .build();

        let search_result = todos
            .search(&query)
            .unwrap();

        let chunk_count = 3;

        let chunks: Vec<_> = search_result
            .chunks(3)
            .map(|chunk| {
                chunk.into_iter().collect::<HashSet<_>>()
            })
            .take(chunk_count)
            .collect();

        let [
            actual_highs,
            actual_meds,
            actual_lows,
        ] =
            <[HashSet<_>; 3]>::try_from(chunks).expect(
                format!(
                    "`chunks` vec should contain {} elements",
                    chunk_count
                ).as_str()
            );

        assert_eq!(
            actual_highs,
            hashset! {
                &todo_b,
                &todo_e,
                &todo_h
            }
        );
        assert_eq!(
            actual_meds,
            hashset! {
                &todo_a,
                &todo_d,
                &todo_g
            }
        );
        assert_eq!(
            actual_lows,
            hashset! {
                &todo_c,
                &todo_f,
                &todo_i
            }
        );
    }

    #[test]
    fn todolist_search_should_sort_todos_by_deadline_in_ascending_order(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            mut todo_a, mut todo_b, mut todo_c,
            todo_d, todo_e, mut todo_f,
            todo_g, mut todo_h, todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect(
                    "`items` vec should contain 9 elements"
                );

        todo_a = todos
            .update_deadline(
                todo_a.id,
                OptionalDeadlineInput::some("2022-01-10 00")
            )
            .unwrap();
        todo_b = todos
            .update_deadline(
                todo_b.id,
                OptionalDeadlineInput::some("2022-01-07 00")
            )
            .unwrap();
        todo_c = todos
            .update_deadline(
                todo_c.id,
                OptionalDeadlineInput::some("2022-01-01 00")
            )
            .unwrap();
        let _todo_d = todos
            .update_deadline(
                todo_d.id,
                OptionalDeadlineInput::some("2022-01-22 00")
            )
            .unwrap();
        let _todo_e = todos
            .update_deadline(
                todo_e.id,
                OptionalDeadlineInput::some("2022-02-01 00")
            )
            .unwrap();
        todo_f = todos
            .update_deadline(
                todo_f.id,
                OptionalDeadlineInput::some("2022-01-03 00")
            )
            .unwrap();
        let _todo_g = todos
            .update_deadline(
                todo_g.id,
                OptionalDeadlineInput::some("2022-02-06 00")
            )
            .unwrap();
        todo_h = todos
            .update_deadline(
                todo_h.id,
                OptionalDeadlineInput::some("2022-01-18 00")
            )
            .unwrap();
        let _todo_i = todos
            .update_deadline(
                todo_i.id,
                OptionalDeadlineInput::some("2022-01-26 00")
            )
            .unwrap();

        let query = Query::builder()
            .sort(Some(
                QuerySort::Deadline,
            ))
            .limit(
                OptionalResultLimit::some(5)
            )
            .build();

        let search_result = todos
            .search(&query)
            .unwrap();

        assert_eq!(
            search_result,
            vec![
                todo_c, todo_f, todo_b,
                todo_a, todo_h
            ]
        );
    }

    #[test]
    fn todolist_search_should_fail_when_deadline_is_invalid(
    ) {
        let invalid_date_time = "abc";

        let query = Query::builder()
            .deadline(
                OptionalDeadlineInput::some(invalid_date_time)
            )
            .build();

        let actual = new_todo_list!()
            .search(&query);

        let expected = AppError::DateTimeParseError {
                input: invalid_date_time.into(),
                expected_format: USER_DATE_TIME_FORMAT.into()
            };

        assert_app_error!(
            actual, expected
        )
    }

    #[test]
    fn todolist_count_by_should_fail_when_deadline_is_invalid(
    ) {
        let invalid_date_time = "abc";

        let query = Query::builder()
            .deadline(
                OptionalDeadlineInput::some(invalid_date_time)
            )
            .build();

        let actual = new_todo_list!()
            .count_by(&query);

        let expected = AppError::DateTimeParseError {
                input: invalid_date_time.into(),
                expected_format: USER_DATE_TIME_FORMAT.into()
            };

        assert_app_error!(
            actual, expected
        )
    }

    #[test]
    fn todolist_delete_by_statuses_should_delete_todos_with_specified_statuses(
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
                .expect(
                    "`items` vec should contain 9 elements"
                );

        let _todo_a = todos
            .update_status(
                todo_a.id,
                Status::Backlog,
            )
            .unwrap();
        let _todo_b = todos
            .update_status(
                todo_b.id,
                Status::InProgress,
            )
            .unwrap();
        let _todo_c = todos
            .update_status(
                todo_c.id,
                Status::Done,
            )
            .unwrap();
        let _todo_d = todos
            .update_status(
                todo_d.id,
                Status::Backlog,
            )
            .unwrap();
        let _todo_e = todos
            .update_status(
                todo_e.id,
                Status::InProgress,
            )
            .unwrap();
        let _todo_f = todos
            .update_status(
                todo_f.id,
                Status::Done,
            )
            .unwrap();
        let _todo_g = todos
            .update_status(
                todo_g.id,
                Status::Backlog,
            )
            .unwrap();
        let _todo_h = todos
            .update_status(
                todo_h.id,
                Status::InProgress,
            )
            .unwrap();
        let _todo_i = todos
            .update_status(
                todo_i.id,
                Status::Done,
            )
            .unwrap();

        let deleted_count = todos
            .delete_by_statuses(&nes![
                Status::Backlog,
                Status::Done
            ]);

        assert_eq!(deleted_count, 6);

        let query = Query::builder()
            .status(Some(
                Status::InProgress,
            ))
            .build();

        let remaining_count = todos
            .count_by(&query)
            .unwrap();
        let count_all =
            todos.count_all();

        assert_eq!(
            remaining_count,
            count_all
        );
        assert_eq!(count_all, 3)
    }

    #[test]
    fn todolist_delete_by_priorities_should_delete_todos_with_specified_priorities(
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
                .expect(
                    "`items` vec should contain 9 elements"
                );

        let _todo_a = todos
            .update_priority(
                todo_a.id,
                Priority::Medium,
            )
            .unwrap();
        let _todo_b = todos
            .update_priority(
                todo_b.id,
                Priority::High,
            )
            .unwrap();
        let _todo_c = todos
            .update_priority(
                todo_c.id,
                Priority::Low,
            )
            .unwrap();
        let _todo_d = todos
            .update_priority(
                todo_d.id,
                Priority::Medium,
            )
            .unwrap();
        let _todo_e = todos
            .update_priority(
                todo_e.id,
                Priority::High,
            )
            .unwrap();
        let _todo_f = todos
            .update_priority(
                todo_f.id,
                Priority::Low,
            )
            .unwrap();
        let _todo_g = todos
            .update_priority(
                todo_g.id,
                Priority::Medium,
            )
            .unwrap();
        let _todo_h = todos
            .update_priority(
                todo_h.id,
                Priority::High,
            )
            .unwrap();
        let _todo_i = todos
            .update_priority(
                todo_i.id,
                Priority::Low,
            )
            .unwrap();

        let deleted_count = todos
            .delete_by_priorities(
                &nes![
                    Priority::Medium,
                    Priority::Low
                ],
            );

        assert_eq!(deleted_count, 6);

        let query = Query::builder()
            .priority(Some(
                Priority::High,
            ))
            .build();

        let remaining_count = todos
            .count_by(&query)
            .unwrap();
        let count_all =
            todos.count_all();

        assert_eq!(
            remaining_count,
            count_all
        );
        assert_eq!(count_all, 3);
    }

    #[test]
    fn todolist_delete_by_ids_should_delete_todos_with_specified_ids(
    ) {
        let mut todos =
            new_todo_list!();

        let items =
            add_todos(&mut todos)
                .unwrap();
        let [
            mut todo_a, mut todo_b, mut todo_c,
            mut todo_d, mut todo_e, mut todo_f,
            mut todo_g, mut todo_h, mut todo_i
        ] =
            <[Todo; 9]>::try_from(items)
                .expect(
                    "`items` vec should contain 9 elements"
                );

        todo_a = todos
            .update_priority(
                todo_a.id,
                Priority::Medium,
            )
            .unwrap();
        todo_b = todos
            .update_priority(
                todo_b.id,
                Priority::High,
            )
            .unwrap();
        todo_c = todos
            .update_priority(
                todo_c.id,
                Priority::Low,
            )
            .unwrap();
        todo_d = todos
            .update_priority(
                todo_d.id,
                Priority::Medium,
            )
            .unwrap();
        todo_e = todos
            .update_priority(
                todo_e.id,
                Priority::High,
            )
            .unwrap();
        todo_f = todos
            .update_priority(
                todo_f.id,
                Priority::Low,
            )
            .unwrap();
        todo_g = todos
            .update_priority(
                todo_g.id,
                Priority::Medium,
            )
            .unwrap();
        todo_h = todos
            .update_priority(
                todo_h.id,
                Priority::High,
            )
            .unwrap();
        todo_i = todos
            .update_priority(
                todo_i.id,
                Priority::Low,
            )
            .unwrap();

        let deleted_count = todos
            .delete_by_ids(&nes![
                todo_b.id, todo_d.id,
                todo_f.id, todo_h.id
            ]);

        assert_eq!(deleted_count, 4);

        let count_all =
            todos.count_all();

        assert_eq!(count_all, 5);

        let query = Query::builder()
            .limit(OptionalResultLimit::some(5))
            .build();

        let search_result = todos
            .search(&query)
            .unwrap();

        assert_eq!(
            search_result,
            vec![
                todo_a, todo_c, todo_e,
                todo_g, todo_i
            ]
        );
    }
}
