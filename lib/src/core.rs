use crate::app_error::{
    AppError, AppResultExt, IntoReport,
    ResultExt,
};
use nonempty_collections::NESet;
use uuid::Uuid;

// Only used in this mod and by other crates; other modules in this crate use app_error::AppResult.
pub type AppResult<T> =
    Result<T, String>;

pub(crate) type UnixTime = i64;

pub fn u64_from(
    n: usize,
) -> AppResult<u64> {
    u64::try_from(n)
        .into_report()
        .change_context(
            AppError::DataConversionUsizeToU64(n),
        )
        .err_as_string()
}

pub fn uuid_from(
    s: &str,
) -> AppResult<Uuid> {
    Uuid::try_from(s.trim())
        .into_report()
        .change_context(
            AppError::InvalidUuid(
                s.into(),
            ),
        )
        .err_as_string()
}

pub fn uuid_set_from<I>(
    source: &mut I,
) -> AppResult<NESet<Uuid>>
where
    I: Iterator<Item = String>,
{
    if let Some(head) = source.next() {
        let head = uuid_from(&head)?;

        let tail: AppResult<_> = source
            .map(|t| {
                let id = uuid_from(&t)?;
                Ok(id)
            })
            .collect();

        Ok(NESet { head, tail: tail? })
    } else {
        Err(format!(
            "{}",
            AppError::CollectionIsEmpty,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_app_error;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use proptest::prelude::{
        any, prop, proptest, Strategy,
    };
    use std::collections::HashSet;

    const REAL_UUID: &str  =
        "67e55044-10b1-426f-9247-bb680e5fe0c8";

    const BOGUS_UUID: &str =
        "not_a_uuid";

    #[test]
    fn uuid_from_should_fail_when_uuid_is_not_valid(
    ) {
        let actual =
            uuid_from(BOGUS_UUID);

        let expected =
            AppError::InvalidUuid(
                BOGUS_UUID.into(),
            );

        assert_app_error!(
            actual, expected
        );
    }

    #[test]
    fn uuid_set_from_should_fail_when_source_hashset_is_empty(
    ) {
        let actual = uuid_set_from(
            &mut hashset! {}
                .into_iter(),
        );

        let expected =
            AppError::CollectionIsEmpty;

        assert_app_error!(
            actual, expected
        );
    }

    #[test]
    fn uuid_set_from_should_fail_when_source_vec_is_empty(
    ) {
        let actual = uuid_set_from(
            &mut vec![].into_iter(),
        );

        let expected =
            AppError::CollectionIsEmpty;

        assert_app_error!(
            actual, expected
        );
    }

    #[test]
    fn uuid_set_from_should_fail_when_source_hashset_contains_bogus_uuid(
    ) {
        let actual = uuid_set_from(
            &mut hashset! {
                REAL_UUID.into(),
                BOGUS_UUID.into()
            }
            .into_iter(),
        );

        let expected =
            AppError::InvalidUuid(
                BOGUS_UUID.into(),
            );

        assert_app_error!(
            actual, expected
        );
    }

    #[test]
    fn uuid_set_from_should_fail_when_source_vec_contains_bogus_uuid(
    ) {
        let actual = uuid_set_from(
            &mut vec![
                REAL_UUID.into(),
                BOGUS_UUID.into(),
            ]
            .into_iter(),
        );

        let expected =
            AppError::InvalidUuid(
                BOGUS_UUID.into(),
            );

        assert_app_error!(
            actual, expected
        );
    }

    fn uuid_gen_strategy(
    ) -> impl Strategy<Value = Uuid>
    {
        any::<[u8; 16]>().prop_map(
            |bytes| {
                Uuid::from_bytes(bytes)
            },
        )
    }

    fn uuid_vec_gen_strategy(
    ) -> impl Strategy<Value = Vec<Uuid>>
    {
        prop::collection::vec(
            uuid_gen_strategy(),
            1..=10,
        )
    }

    proptest! {
        #[test]
        fn uuid_set_from_should_convert_string_iterator_to_uuid_set(
            uuid_vec in uuid_vec_gen_strategy()
        ) {
            let mut strings = uuid_vec
                .iter()
                .map(|u| u.to_string());

            let actual = uuid_set_from(&mut strings).unwrap();

            let expected = {
                let uuid_set: HashSet<_> = uuid_vec
                    .into_iter()
                    .collect();

                NESet::from_set(uuid_set).expect(
                    "`uuid_vec` should NOT be empty."
                )
            };

            assert_eq!(actual, expected);
        }
    }
}
