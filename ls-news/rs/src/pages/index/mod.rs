use crate::docs::{DeletedPost, Post, User};
use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;
use std::collections::HashSet;

pub struct SearchParams {
    pub after: Option<String>,
}

#[derive(Serialize)]
pub enum Props {
    Ok { rows: Vec<Row> },
    DbErr { message: String },
}

#[derive(Serialize)]
pub struct Row {
    pub post: Post,
    pub deleted_at: Option<DateTime>,
    pub author: User,
}

pub async fn handler(
    _headers: HeaderMap,
    jar: CookieJar,
    search_params: SearchParams,
) -> Result<Props> {
    let is_admin = crate::common::auth::is_admin(&jar);

    let rows = match get_rows(search_params.after, is_admin).await {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Failed to get posts: {err}");
            return Ok(Props::DbErr {
                message: "Failed to get posts".to_string(),
            });
        }
    };

    Ok(Props::Ok { rows })
}

async fn get_rows(after: Option<String>, is_admin: bool) -> Result<Vec<Row>> {
    pub struct RowWithoutAuthor {
        pub post: Post,
        pub deleted_at: Option<DateTime>,
    }

    let mut rows = Post::query(&after, 10)
        .await?
        .into_iter()
        .map(|post| RowWithoutAuthor {
            post,
            deleted_at: None,
        })
        .collect::<Vec<_>>();
    if is_admin {
        let deleted_posts_rows = DeletedPost::query(&after, 10).await?;
        rows.extend(deleted_posts_rows.into_iter().map(|d| RowWithoutAuthor {
            post: d.post,
            deleted_at: Some(d.deleted_at),
        }));
        rows.sort_by(|a, b| b.post.id.cmp(&a.post.id));
    }

    let user_ids = rows
        .iter()
        .map(|r| r.post.author_id.clone())
        .collect::<HashSet<_>>();

    let users = futures::future::try_join_all(user_ids.iter().map(User::get))
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    Ok(rows
        .into_iter()
        .map(|row| Row {
            deleted_at: row.deleted_at,
            author: users
                .iter()
                .find(|u| u.id == row.post.author_id)
                .cloned()
                .unwrap_or(User {
                    id: "0".to_string(),
                    username: "GHOST".to_string(),
                    avatar_url: "".to_string(),
                    created_at: DateTime::from_timestamp(0, 0).unwrap(),
                    updated_at: DateTime::from_timestamp(0, 0).unwrap(),
                }),
            post: row.post,
        })
        .collect())
}
