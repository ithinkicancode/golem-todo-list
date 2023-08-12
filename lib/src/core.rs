use crate::app_error::{
    AppError, AppResultExt, IntoReport,
    ResultExt,
};
use nonempty_collections::{
    NESet, NonEmptyIterator,
};
use uuid::Uuid;

// Not used in this crate; only used externally
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

pub fn uuid_set_from(
    targets: &NESet<String>,
) -> AppResult<NESet<Uuid>> {
    let (head, tail) =
        targets.iter().first();

    let head = uuid_from(head)?;

    let tail: AppResult<_> = tail
        .map(|t| {
            let id = uuid_from(t)?;
            Ok(id)
        })
        .collect();

    Ok(NESet { head, tail: tail? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_app_error;
    use nonempty_collections::nes;
    use pretty_assertions::assert_eq;
    use proptest::prelude::{
        any, prop, proptest, Strategy,
    };
    use std::collections::HashSet;

    #[test]
    fn uuid_from_should_fail_when_uuid_is_not_valid(
    ) {
        let invalid_uuid = "not_a_uuid";

        let actual =
            uuid_from(invalid_uuid);

        let expected =
            AppError::InvalidUuid(
                invalid_uuid.into(),
            );

        assert_app_error!(
            actual, expected
        );
    }

    #[test]
    fn uuid_set_from_should_fail_when_uuid_is_not_valid(
    ) {
        let invalid_uuid = "not_a_uuid";

        let actual =
            uuid_set_from(&nes![
                "67e55044-10b1-426f-9247-bb680e5fe0c8".to_string(),
                invalid_uuid.into()
            ]);

        let expected =
            AppError::InvalidUuid(
                invalid_uuid.into(),
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
        fn uuid_set_from_should_convert_string_set_to_uuid_set(
            uuid_vec in uuid_vec_gen_strategy()
        ) {
            let string_set: HashSet<_> = uuid_vec
                .iter()
                .map(|u| u.to_string())
                .collect();

            let string_set = NESet::from_set(string_set).expect(
                "`uuid_vec` should NOT be empty."
            );

            let uuid_set: HashSet<_> = uuid_vec
                .into_iter()
                .collect();

            let expected = NESet::from_set(uuid_set).expect(
                "`uuid_vec` should NOT be empty."
            );

            let actual = uuid_set_from(&string_set).unwrap();

            assert_eq!(actual, expected);
        }
    }
}
