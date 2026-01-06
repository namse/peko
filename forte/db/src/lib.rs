mod turso;

use anyhow::Result;
use turso::TursoDatabase;
use wstd::http::body::Bytes;

/// Create a Turso database instance.
/// # Environment Variables
/// - `TURSO_URL`: The URL of the Turso database.
/// - `TURSO_AUTH_TOKEN`: The authentication token for the Turso database.
pub fn turso() -> Result<Database> {
    let url = std::env::var("TURSO_URL").map_err(|_| anyhow::anyhow!("TURSO_URL env var not set"))?;
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").map_err(|_| anyhow::anyhow!("TURSO_AUTH_TOKEN env var not set"))?;
    Ok(Database {
        inner: DatabaseInner::Turso(TursoDatabase::new(url, auth_token)?),
    })
}

pub struct Database {
    inner: DatabaseInner,
}

impl Database {
    pub async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get(pk, sk).await,
        }
    }
}

enum DatabaseInner {
    Turso(TursoDatabase),
}
