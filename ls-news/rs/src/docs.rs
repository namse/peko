use forte_sdk::{anyhow::Result, *};
use serde::{Deserialize, Serialize};

pub trait AsOptStr {
    fn as_opt_str(&self) -> Option<&str>;
}

impl<T: AsRef<str>> AsOptStr for Option<T> {
    fn as_opt_str(&self) -> Option<&str> {
        self.as_ref().map(|s| s.as_ref())
    }
}

impl<T: AsRef<str>> AsOptStr for &Option<T> {
    fn as_opt_str(&self) -> Option<&str> {
        self.as_ref().map(|s| s.as_ref())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl User {
    pub async fn get(sk: impl AsRef<str>) -> Result<Option<User>> {
        Ok(forte_db::turso()
            .get("users", sk.as_ref())
            .await?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub author_id: String,
    pub likes: usize,
    pub dislikes: usize,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl Post {
    pub async fn query(after_sk: impl AsOptStr, limit: usize) -> Result<Vec<Post>> {
        Ok(forte_db::turso()
            .query("posts", after_sk.as_opt_str(), limit)
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<Post>, _>>()?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeletedPost {
    pub post: Post,
    pub deleted_at: DateTime,
}
impl DeletedPost {
    pub async fn query(after_sk: impl AsOptStr, limit: usize) -> Result<Vec<DeletedPost>> {
        Ok(forte_db::turso()
            .query("deleted_posts", after_sk.as_opt_str(), limit)
            .await?
            .into_iter()
            .map(|(_sk, data)| serde_json::from_slice(&data))
            .collect::<Result<Vec<DeletedPost>, _>>()?)
    }
}
