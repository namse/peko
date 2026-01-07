use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;

pub struct SearchParams {
    pub after: Option<String>,
}

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
    DbErr { message: String },
}

pub async fn handler(
    headers: HeaderMap,
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

    // const displayedItems = posts.map((post) => {
    //   const url = new URL(post.url);
    //   return {
    //     ...post,
    //     domain: url.hostname,
    //   };
    // });

    // const hasMore = posts.length === 20;
    // ---

    // <Layout title="ls-news">
    //   {displayedItems.map((item) => <NewsItem item={item} />)}

    //   {
    //     displayedItems.length > 0 && (
    //       <NewsPagination
    //         lastKey={displayedItems[displayedItems.length - 1]!.id}
    //         hasMore={hasMore}
    //       />
    //     )
    //   }
    // </Layout>

    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}

struct Post {
    id: String,
}

async fn get_posts(after: Option<String>, is_admin: bool) -> Result<Vec<Post>> {
    let mut rows = forte_db::turso().query("posts", after.as_ref(), 10).await?;
    if is_admin {
        let mut deleted_posts_rows = forte_db::turso()
            .query("deleted_posts", after.as_ref(), 10)
            .await?;
        rows.append(&mut deleted_posts_rows);
        rows.sort_by(|a, b| b.0.cmp(&a.0));
    }

    let posts = rows.into_iter().map(|(id, _)| Post { id }).collect();
    Ok(posts)
}
