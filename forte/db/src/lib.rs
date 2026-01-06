mod turso;

use anyhow::Result;
use turso::TursoDatabase;
use wstd::http::body::Bytes;

/// Create a Turso database instance from environment variables.
/// # Environment Variables
/// - `TURSO_URL`: The URL of the Turso database.
/// - `TURSO_AUTH_TOKEN`: The authentication token for the Turso database.
pub fn turso() -> Result<Database> {
    let url =
        std::env::var("TURSO_URL").map_err(|_| anyhow::anyhow!("TURSO_URL env var not set"))?;
    let auth_token = std::env::var("TURSO_AUTH_TOKEN")
        .map_err(|_| anyhow::anyhow!("TURSO_AUTH_TOKEN env var not set"))?;
    turso_with_config(url, auth_token)
}

/// Create a Turso database instance with explicit configuration.
/// - `url`: The URL of the Turso database (supports http, https, libsql schemes).
/// - `auth_token`: The authentication token. Pass empty string for no authentication.
pub fn turso_with_config(url: String, auth_token: String) -> Result<Database> {
    Ok(Database {
        inner: DatabaseInner::Turso(TursoDatabase::new(url, auth_token)?),
    })
}

pub struct Database {
    inner: DatabaseInner,
}

impl Database {
    // 이거 지우고, 그리고 테이블이 없다는 에러가 뜨면 그때 생성하고 재시도하는 그런 방식을 써보자.
    pub async fn execute_sql(&self, sql: &str) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_sql(sql).await,
        }
    }

    pub async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get(pk, sk).await,
        }
    }

    pub async fn insert(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.insert(pk, sk, data).await,
        }
    }

    pub async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.delete(pk, sk).await,
        }
    }
}

enum DatabaseInner {
    Turso(TursoDatabase),
}
