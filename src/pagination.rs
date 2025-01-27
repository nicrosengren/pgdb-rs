use std::future::Future;

use diesel_async::{methods::LoadQuery, return_futures::LoadFuture};
use futures_util::{future::AndThen, TryFutureExt};

pub const DEFAULT_PAGE_SIZE: i64 = 50;

#[derive(Clone, Debug)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_count: i64,
    pub page_count: i64,
}

impl<T> Page<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn map_into<T2>(self, f: impl Fn(T) -> T2) -> Page<T2> {
        Page {
            data: self.data.into_iter().map(f).collect(),
            page: self.page,
            page_size: self.page_size,
            total_count: self.total_count,
            page_count: self.page_count,
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

impl<T> PaginatedQuery<T> {
    pub fn page_size(mut self, page_size: i64) -> Self {
        self.page_size = page_size;
        self.offset = (self.page - 1) * page_size;
        self
    }

    pub fn load_page_asdasd<'query, 'conn, U>(
        self,
        conn: &'conn mut diesel_async::AsyncPgConnection,
    ) -> impl Future<Output = diesel::QueryResult<Page<U>>> + use<'query, 'conn, U, T>
    where
        U: Send + 'conn,
        Self: LoadQuery<'query, diesel_async::AsyncPgConnection, (U, i64)> + 'query,
    {
        let page_size = self.page_size;
        let page = self.page;
        // let res = diesel_async::RunQueryDsl::load::<(U, i64)>(self, conn).await?;
        // let total_count = res.first().map(|x| x.1).unwrap_or(0);
        // let data = res.into_iter().map(|x| x.0).collect();
        // let page_count = (total_count as f64 / page_size as f64).ceil() as i64;

        diesel_async::RunQueryDsl::load::<(U, i64)>(self, conn).map_ok(move |res| {
            let total_count = res.first().map(|x| x.1).unwrap_or(0);
            let data = res.into_iter().map(|x| x.0).collect();
            let page_count = (total_count as f64 / page_size as f64).ceil() as i64;

            Page {
                data,
                page,
                page_size,
                total_count,
                page_count,
            }
        })
    }

    pub async fn load_page<'query, U>(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> diesel::QueryResult<Page<U>>
    where
        U: Send,
        Self: LoadQuery<'query, diesel_async::AsyncPgConnection, (U, i64)> + 'query,
    {
        let page_size = self.page_size;
        let page = self.page;
        // let res = diesel_async::RunQueryDsl::load::<(U, i64)>(self, conn).await?;
        // let total_count = res.first().map(|x| x.1).unwrap_or(0);
        // let data = res.into_iter().map(|x| x.0).collect();
        // let page_count = (total_count as f64 / page_size as f64).ceil() as i64;
        Ok(Page {
            data: vec![],
            page,
            page_size,
            total_count: 1,
            page_count: 1,
        })
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
