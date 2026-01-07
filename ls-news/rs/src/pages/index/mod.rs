use anyhow::Result;
use http::HeaderMap;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(headers: HeaderMap) -> Result<Props> {
    let is_admin = crate::common::auth::is_admin(&headers);

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
