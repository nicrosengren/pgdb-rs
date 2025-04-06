use futures_util::TryFutureExt;

use diesel_async::{methods::LoadQuery, return_futures::LoadFuture};

pub const DEFAULT_PAGE_SIZE: i64 = 50;

#[derive(Clone, Debug)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total_count: i64,
}

impl<T> Page<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn map_into<T2>(self, f: impl FnMut(T) -> T2) -> Page<T2> {
        Page {
            data: self.data.into_iter().map(f).collect(),
            total_count: self.total_count,
        }
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    // Required method
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

pub trait PaginateQuery: Sized {
    fn paginate(self, page: i64) -> PaginatedQuery<Self>;
}

impl<T> PaginateQuery for T {
    fn paginate(self, page: i64) -> PaginatedQuery<Self> {
        PaginatedQuery {
            query: self,
            page,
            page_size: DEFAULT_PAGE_SIZE,
            offset: (page - 1) * DEFAULT_PAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, diesel::query_builder::QueryId)]
pub struct PaginatedQuery<T> {
    query: T,
    page: i64,
    page_size: i64,
    offset: i64,
}

pub type InnerLoadFuture<
    'conn,
    'query,
    //Q: LoadQuery<'query, diesel_async::AsyncPgConnection, (U, i64)>,
    Q,
    U,
> = futures_util::future::MapOk<
    LoadFuture<'conn, 'query, Q, diesel_async::AsyncPgConnection, (U, i64)>,
    fn(Vec<(U, i64)>) -> Page<U>,
>;

impl<T> PaginatedQuery<T> {
    pub fn page_size(mut self, page_size: i64) -> Self {
        self.page_size = page_size;
        self.offset = (self.page - 1) * page_size;
        self
    }

    pub fn load_page<'query, 'conn, U>(
        self,
        conn: &'conn mut diesel_async::AsyncPgConnection,
    ) -> InnerLoadFuture<'conn, 'query, Self, U>
    where
        U: Send,
        Self: LoadQuery<'query, diesel_async::AsyncPgConnection, (U, i64)> + 'query,
    {
        fn convert_to_page<U>(res: Vec<(U, i64)>) -> Page<U> {
            let total_count = res.first().map(|x| x.1).unwrap_or(0);
            let data = res.into_iter().map(|x| x.0).collect();
            //let page_count = (total_count as f64 / page_size as f64).ceil() as i64;

            Page { data, total_count }
        }

        diesel_async::RunQueryDsl::load::<(U, i64)>(self, conn).map_ok(convert_to_page)
    }
}

impl<T, C> diesel::RunQueryDsl<C> for PaginatedQuery<T> where C: diesel::Connection {}

impl<T: diesel::query_builder::Query> diesel::query_builder::Query for PaginatedQuery<T> {
    type SqlType = (T::SqlType, diesel::sql_types::BigInt);
}

impl<T> diesel::query_builder::QueryFragment<diesel::pg::Pg> for PaginatedQuery<T>
where
    T: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
{
    fn walk_ast<'b>(
        &'b self,
        mut out: diesel::query_builder::AstPass<'_, 'b, diesel::pg::Pg>,
    ) -> diesel::QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") as paged_query_with LIMIT ");
        out.push_bind_param::<diesel::sql_types::BigInt, _>(&self.page_size)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<diesel::sql_types::BigInt, _>(&self.offset)?;
        Ok(())
    }
}
