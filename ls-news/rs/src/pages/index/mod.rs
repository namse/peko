use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;

use crate::docs::{DeletedPost, Post};

pub struct SearchParams {
    pub after: Option<String>,
}

#[derive(Serialize)]
pub enum Props {
    Ok { posts: Vec<Post> },
    DbErr { message: String },
}

pub async fn handler(
    _headers: HeaderMap,
    jar: CookieJar,
    search_params: SearchParams,
) -> Result<Props> {
    let is_admin = crate::common::auth::is_admin(&jar);

    let posts = match get_posts(search_params.after, is_admin).await {
        Ok(posts) => posts,
        Err(err) => {
            eprintln!("Failed to get posts: {err}");
            return Ok(Props::DbErr {
                message: "Failed to get posts".to_string(),
            });
        }
    };

    Ok(Props::Ok { posts })
}

async fn get_posts(after: Option<String>, is_admin: bool) -> Result<Vec<Post>> {
    let mut posts = Post::query(&after, 10).await?;
    if is_admin {
        let deleted_posts_rows = DeletedPost::query(&after, 10).await?;
        posts.extend(deleted_posts_rows.into_iter().map(|d| d.post));
        posts.sort_by(|a, b| b.id.cmp(&a.id));
    }

    Ok(posts)
}
