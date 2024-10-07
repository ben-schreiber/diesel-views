use diesel::{pg::Pg, query_builder::BoxedSelectStatement};

pub trait ViewDefinition {
    type SqlType;
    type QuerySource;
    type GroupBy;

    fn name() -> String;
    fn definition<'a>(
    ) -> BoxedSelectStatement<'a, Self::SqlType, Self::QuerySource, Pg, Self::GroupBy>;
}
