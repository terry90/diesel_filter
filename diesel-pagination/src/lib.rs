use diesel::{prelude::*, query_builder::*, sql_types::BigInt};

/// This trait has to be implemented for a type to be passed into
/// `Paginate::paginate`.
///
/// It's usually enough to use the default `PaginationParams`
/// provided with this crate, but if customizations are needed
/// create your own type and impl this trait.
pub trait GetPaginationParams {
    const DEFAULT_PER_PAGE: i64;

    fn per_page(&self) -> Option<i64>;
    fn page(&self) -> Option<i64>;
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct PaginationParams {
    pub per_page: Option<i64>,
    pub page: Option<i64>,
}

impl PaginationParams {
    pub fn page(page: i64) -> Self {
        Self {
            page: Some(page),
            per_page: None,
        }
    }

    pub fn per_page(mut self, per_page: i64) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

impl GetPaginationParams for PaginationParams {
    const DEFAULT_PER_PAGE: i64 = 50;

    fn page(&self) -> Option<i64> {
        self.page
    }

    fn per_page(&self) -> Option<i64> {
        self.per_page
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Paginated<T> {
    pub page: i64,
    pub per_page: i64,
    pub items: Vec<T>,
    pub num_total: i64,
}

pub trait Paginate: Sized {
    fn paginate<P: GetPaginationParams>(self, params: P) -> PaginatedQuery<Self>;
}

impl<T> Paginate for T {
    fn paginate<P: GetPaginationParams>(self, params: P) -> PaginatedQuery<Self> {
        let page = params.page().unwrap_or(1);
        let per_page = params.per_page().unwrap_or(P::DEFAULT_PER_PAGE);

        PaginatedQuery {
            query: self,
            per_page,
            page: page,
            offset: (page - 1) * per_page,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct PaginatedQuery<T> {
    query: T,
    page: i64,
    offset: i64,
    per_page: i64,
}

impl<T> PaginatedQuery<T> {
    #[cfg(not(feature = "diesel-async"))]
    pub fn load_and_count<'a, U, Conn>(self, conn: &mut Conn) -> QueryResult<Paginated<U>>
    where
        Self: diesel::query_dsl::methods::LoadQuery<'a, Conn, (U, i64)>,
        Conn: diesel::connection::Connection,
    {
        let Self { page, per_page, .. } = self;
        let results = self.load::<(U, i64)>(conn)?;
        let num_total = results.get(0).map(|x| x.1).unwrap_or(0);
        let items = results.into_iter().map(|x| x.0).collect();
        Ok(Paginated {
            page: self.page,
            per_page: self.per_page,
            items,
            num_total,
        })
    }

    #[cfg(feature = "diesel-async")]
    pub async fn load_and_count<'a, U, Conn>(self, conn: &mut Conn) -> QueryResult<Paginated<U>>
    where
        Self: diesel_async::methods::LoadQuery<'a, Conn, (U, i64)> + 'a,
        Conn: diesel_async::AsyncConnection,
        U: Send,
    {
        use diesel_async::RunQueryDsl;

        let Self { page, per_page, .. } = self;
        let results = <Self as RunQueryDsl<Conn>>::load::<(U, i64)>(self, conn).await?;
        let num_total = results.get(0).map(|x| x.1).unwrap_or(0);
        let items = results.into_iter().map(|x| x.0).collect();
        Ok(Paginated {
            page,
            per_page,
            items,
            num_total,
        })
    }
}

impl<T: Query> Query for PaginatedQuery<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T, Conn> RunQueryDsl<Conn> for PaginatedQuery<T> where Conn: diesel::connection::Connection {}

impl<T, DB> QueryFragment<DB> for PaginatedQuery<T>
where
    T: QueryFragment<DB>,
    DB: diesel::backend::Backend,
    i64: diesel::serialize::ToSql<BigInt, DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}
