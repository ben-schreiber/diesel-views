use diesel::backend::Backend;
use diesel::query_builder::{AstPass, Query, QueryFragment, QueryId};
use diesel::sql_types::Untyped;
use diesel::QueryResult;

pub struct CreateOrReplaceStatement<D> {
    definition: D,
    name: String,
}

impl<D> QueryId for CreateOrReplaceStatement<D> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<D: Clone> Clone for CreateOrReplaceStatement<D> {
    fn clone(&self) -> Self {
        Self {
            definition: self.definition.clone(),
            name: self.name.clone(),
        }
    }
}

impl<D: std::fmt::Debug> std::fmt::Debug for CreateOrReplaceStatement<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateOrReplaceStatement")
            .field("definition", &self.definition)
            .field("name", &self.name)
            .finish()
    }
}

impl<D, DB> QueryFragment<DB> for CreateOrReplaceStatement<D>
where
    DB: Backend,
    D: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.push_sql("CREATE OR REPLACE VIEW ");
        out.push_identifier(&self.name)?;
        out.push_sql(" AS ");
        self.definition.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<D> Query for CreateOrReplaceStatement<D> {
    type SqlType = Untyped;
}

pub trait CreateOrReplace<D>: Sized {
    fn create_or_replace(self, name: &str) -> CreateOrReplaceStatement<D>;
}

impl<D> CreateOrReplace<D> for D {
    fn create_or_replace(self, name: &str) -> CreateOrReplaceStatement<D> {
        CreateOrReplaceStatement {
            definition: self,
            name: name.to_string(),
        }
    }
}
