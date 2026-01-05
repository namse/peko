mod turso;

use anyhow::Result;
use turso::TursoDatabase;
use wstd::http::body::Bytes;

pub fn turso(url: &str, auth_token: &str) -> Result<Database> {
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
