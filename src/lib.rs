mod build_error;
mod error;
mod optional_ext;
mod tls;

pub mod testing;

use {
    diesel::{ConnectionError, ConnectionResult},
    diesel_async::{
        async_connection_wrapper::AsyncConnectionWrapper,
        pooled_connection::{deadpool, AsyncDieselConnectionManager, ManagerConfig},
        AsyncPgConnection, SimpleAsyncConnection,
    },
    diesel_migrations::MigrationHarness,
    futures_util::{future::BoxFuture, FutureExt},
    tracing::warn,
};

pub use {
    build_error::*,
    diesel, diesel_async,
    diesel_migrations::{embed_migrations, EmbeddedMigrations},
    error::*,
    optional_ext::OptionalExt,
};

pub mod prelude {
    pub use {
        super::{build_error::BuildError, diesel, diesel_async, error::Error, OptionalExt},
        diesel::migration::MigrationSource,
        diesel_migrations::{self, embed_migrations, EmbeddedMigrations},
    };
}

pub type Connection = deadpool::Object<AsyncPgConnection>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Pool(deadpool::Pool<diesel_async::AsyncPgConnection>);

impl Pool {
    pub fn builder() -> PoolBuilder {
        PoolBuilder::default()
    }

    pub async fn get(&self) -> Result<Connection> {
        Ok(self.0.get().await?)
    }
}

pub struct PoolBuilder {
    max_connections: usize,
    migrations: Option<EmbeddedMigrations>,
    reset_db: bool,
}

impl Default for PoolBuilder {
    fn default() -> Self {
        Self {
            max_connections: 10,
            migrations: None,
            reset_db: false,
        }
    }
}

impl PoolBuilder {
    pub async fn build_from_env(self) -> std::result::Result<Pool, BuildError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| BuildError::other("environment variable `DATABASE_URL` not set"))?;
        self.build(database_url).await
    }

    pub async fn build(self, url: impl AsRef<str>) -> std::result::Result<Pool, BuildError> {
        let url = url.as_ref();

        let mut config = ManagerConfig::default();
        config.custom_setup = Box::new(establish_connection);

        let manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(url, config);
        let pool = deadpool::Pool::<diesel_async::AsyncPgConnection>::builder(manager)
            .max_size(10)
            .build()?;

        // Test connection
        let mut conn = pool.get().await.map_err(Error::from)?;
        conn.batch_execute("select 1").await.map_err(Error::from)?;

        if self.reset_db {
            if !url.ends_with("test") {
                panic!("refusing to reset database at {url} database name must end with \"test\"");
            }

            warn!("DROPPING public schema on database");

            conn.batch_execute("DROP SCHEMA public CASCADE")
                .await
                .expect("dropping public schema");

            conn.batch_execute("CREATE SCHEMA public")
                .await
                .expect("CREATING public schema");
        }

        // Apply migrations if needed
        if let Some(migrations) = self.migrations {
            let mut async_wrapper = AsyncConnectionWrapper::<Connection>::from(conn);

            tokio::task::spawn_blocking(move || {
                async_wrapper.run_pending_migrations(migrations).unwrap();
            })
            .await
            .map_err(|err| BuildError::other(format!("running migrations: {err}")))?;
        }

        Ok(Pool(pool))
    }

    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn with_migrations(mut self, migrations: EmbeddedMigrations) -> Self {
        self.migrations = Some(migrations);
        self
    }
}

fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls::client_config());
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };
    fut.boxed()
}
