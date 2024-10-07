use diesel::{
    backend::Backend,
    query_builder::{AstPass, QueryFragment, QueryId, SelectQuery},
    result::QueryResult,
};

mod private {
    use diesel::{
        backend::Backend,
        query_builder::{QueryFragment, QueryId},
    };

    #[derive(Debug, Clone, Copy, QueryId)]
    pub struct Empty;

    impl<DB: Backend> QueryFragment<DB> for Empty {
        fn walk_ast<'b>(
            &'b self,
            _pass: diesel::query_builder::AstPass<'_, 'b, DB>,
        ) -> diesel::QueryResult<()> {
            Ok(())
        }
    }
}

pub fn cte<Query: SelectQuery>(query: Query, name: &str) -> Cte<Query> {
    Cte::new(query, None, name)
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Cte<Query: SelectQuery, Inner = private::Empty> {
    query: Query,
    inner: Option<Inner>,
    name: String,
}

impl<Query: SelectQuery, Inner> Cte<Query, Inner> {
    pub fn new(query: Query, inner: Option<Inner>, name: &str) -> Self {
        Self {
            query,
            inner,
            name: name.to_string(),
        }
    }

    pub fn cte<Select: SelectQuery>(self, query: Select, name: &str) -> Cte<Select, Self> {
        Cte::new(query, Some(self), name)
    }

    pub fn select_stmt<Select: SelectQuery>(
        self,
        query: Select,
    ) -> CteSelectStmt<Query, Inner, Select> {
        CteSelectStmt {
            cte: self,
            select_stmt: query,
        }
    }
}

impl<Query: SelectQuery, Inner> SelectQuery for Cte<Query, Inner> {
    type SqlType = Query::SqlType;
}

impl<Query: SelectQuery, Inner> QueryId for Cte<Query, Inner> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<Query, Inner, DB> QueryFragment<DB> for Cte<Query, Inner>
where
    DB: Backend,
    Query: QueryFragment<DB> + SelectQuery,
    Inner: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        match &self.inner {
            None => out.push_sql("WITH "),
            Some(inner) => {
                inner.walk_ast(out.reborrow())?;
                out.push_sql(" , ");
            }
        }
        out.push_identifier(&self.name)?;
        out.push_sql(" AS ( ");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") ");
        Ok(())
    }
}

pub struct CteSelectStmt<Query, Inner, Select>
where
    Query: SelectQuery,
    Select: SelectQuery,
{
    cte: Cte<Query, Inner>,
    select_stmt: Select,
}

impl<Query, Inner, Select> CteSelectStmt<Query, Inner, Select>
where
    Query: SelectQuery,
    Select: SelectQuery,
{
    pub fn into_cte(self, name: &str) -> Cte<Select, Cte<Query, Inner>> {
        Cte::new(self.select_stmt, Some(self.cte), name)
    }
}

impl<Query, Inner, Select, DB> QueryFragment<DB> for CteSelectStmt<Query, Inner, Select>
where
    DB: Backend,
    Query: QueryFragment<DB> + SelectQuery,
    Inner: QueryFragment<DB>,
    Select: QueryFragment<DB> + SelectQuery,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        self.cte.walk_ast(out.reborrow())?;
        self.select_stmt.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<Query, Inner, Select> SelectQuery for CteSelectStmt<Query, Inner, Select>
where
    Query: SelectQuery,
    Select: SelectQuery,
{
    type SqlType = Select::SqlType;
}

impl<Query, Inner, Select> QueryId for CteSelectStmt<Query, Inner, Select>
where
    Query: SelectQuery,
    Select: SelectQuery,
{
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}
