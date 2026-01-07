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
}

pub async fn handler(
    headers: HeaderMap,
    jar: CookieJar,
    search_params: SearchParams,
) -> Result<Props> {
    let is_admin = crate::common::auth::is_admin(&jar);

    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}

// ---
// const exclusiveStartIdParam = Astro.url.searchParams.get("after");
// const exclusiveStartId = exclusiveStartIdParam
//   ? parseInt(exclusiveStartIdParam, 10)
//   : undefined;

// const posts = await dbGetPosts({ exclusiveStartId, isAdmin });

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
