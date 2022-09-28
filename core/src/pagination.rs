use diesel::{
    pg::Pg, prelude::*, query_builder::*, query_dsl::methods::LoadQuery, sql_types::BigInt,
};

pub use diesel_filter_query::*;

#[cfg(feature = "serialize")]
pub use serialize::*;

pub const DEFAULT_PER_PAGE: i64 = 10;

#[cfg(feature = "serialize")]
pub mod serialize {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct PaginatedPayload<T>
    where
        T: Serialize,
    {
        pub data: Vec<T>,
        total: i64,
    }

    impl<T> From<(Vec<T>, i64)> for PaginatedPayload<T>
    where
        T: Serialize,
    {
        fn from(data: (Vec<T>, i64)) -> Self {
            Self {
                data: data.0,
                total: data.1,
            }
        }
    }
}

pub trait Paginate: Sized {
    fn paginate(self, page: Option<i64>) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: Option<i64>) -> Paginated<Self> {
        let page = page.unwrap_or(1);

        Paginated {
            query: self,
            per_page: DEFAULT_PER_PAGE,
            page: page,
            offset: (page - 1) * DEFAULT_PER_PAGE,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    offset: i64,
    per_page: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: Option<i64>) -> Self {
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);

        Paginated {
            per_page,
            offset: (self.page - 1) * per_page,
            ..self,
        }
    }

    pub fn load_and_count<'a, U>(self, conn: &mut PgConnection) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<'a, PgConnection, (U, i64)>,
    {
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        let total_pages = total as i64;
        Ok((records, total_pages))
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}

pub struct PaginationOptions {
    pub per_page: i64,
    pub page: i64,
}
