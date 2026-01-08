use crate::docs::{Comment, DeletedPost, Post, User};
use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;
use std::collections::HashSet;

pub struct PathParams {
    pub id: String,
}

#[derive(Serialize)]
pub enum Props {
    Ok { post: Post, comments: Vec<Comment> },
    DbErr { message: String },
}

pub async fn handler(
    _headers: HeaderMap,
    jar: CookieJar,
    path_params: PathParams,
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

async fn get_post_with_comments(post_id: &str, is_admin: bool) -> Result<(Post, Vec<Comment>)> {
    let post = Post::get(post_id).await?;
    //     export async function dbGetPostWithComments({
    //   postId,
    //   userId,
    //   isAdmin,
    // }: {
    //   postId: number;
    //   userId: number | undefined;
    //   isAdmin: boolean;
    // }): Promise<{
    //   post:
    //     | (typeof Posts.$inferSelect & {
    //         author: typeof Users.$inferSelect;
    //         userVote: "like" | "dislike" | null;
    //       })
    //     | null;
    //   comments: (typeof Comments.$inferSelect & {
    //     author: typeof Users.$inferSelect;
    //     userCommentVote: "like" | "dislike" | null;
    //   })[];
    // }> {
    //   const [[postResult], commentRows] = await db.batch([
    //     db
    //       .select({
    //         post: Posts,
    //         author: Users,
    //         userLiked: PostVotes.isLike,
    //       })
    //       .from(Posts)
    //       .where(
    //         isAdmin
    //           ? eq(Posts.id, postId)
    //           : and(eq(Posts.id, postId), isNull(Posts.deletedAt))
    //       )
    //       .innerJoin(Users, eq(Posts.authorId, Users.id))
    //       .leftJoin(
    //         PostVotes,
    //         and(
    //           eq(PostVotes.postId, Posts.id),
    //           eq(PostVotes.userId, userId ?? sql`null`)
    //         )
    //       ),
    //     db
    //       .select({
    //         comment: Comments,
    //         commentAuthor: Users,
    //         userCommentVote: CommentVotes.isLike,
    //       })
    //       .from(Comments)
    //       .where(eq(Comments.postId, postId))
    //       .innerJoin(Users, eq(Comments.authorId, Users.id))
    //       .leftJoin(
    //         CommentVotes,
    //         and(
    //           eq(CommentVotes.commentId, Comments.id),
    //           eq(CommentVotes.userId, userId ?? sql`null`)
    //         )
    //       ),
    //   ]);

    //   const post = postResult
    //     ? {
    //         ...postResult.post,
    //         author: postResult.author,
    //         userVote: (postResult.userLiked === true
    //           ? "like"
    //           : postResult.userLiked === false
    //             ? "dislike"
    //             : null) as "like" | "dislike" | null,
    //       }
    //     : null;

    //   const comments = commentRows.map((x) => {
    //     return {
    //       ...x.comment,
    //       content: x.comment.deletedAt && !isAdmin ? "" : x.comment.content,
    //       author: x.commentAuthor,
    //       userCommentVote: (x.userCommentVote === true
    //         ? "like"
    //         : x.userCommentVote === false
    //           ? "dislike"
    //           : null) as "like" | "dislike" | null,
    //     };
    //   });

    //   return { post, comments };
    // }

    todo!()
}
