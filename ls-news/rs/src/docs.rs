use forte_sdk::*;
use serde::{Deserialize, Serialize};

trait Doc {
    type Pk;
    type Sk;

    fn pk(pk: Self::Pk) -> String;
    fn sk(sk: Self::Sk) -> String;

    fn turso_query()
}

pub struct PostPk;
pub struct PostSk {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
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

impl Doc for Post {
    type Pk = PostPk;
    type Sk = PostSk;

    fn pk(_pk: Self::Pk) -> String {
        "posts".to_string()
    }

    fn sk(sk: Self::Sk) -> String {
        format!("id={}", sk.id)
    }
}

pub struct DeletedPostPk;
pub struct DeletedPostSk {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeletedPost {
    pub post: Post,
    pub deleted_at: DateTime,
}

impl Doc for DeletedPost {
    type Pk = DeletedPostPk;
    type Sk = DeletedPostSk;

    fn pk(_pk: Self::Pk) -> String {
        "deleted_posts".to_string()
    }

    fn sk(sk: Self::Sk) -> String {
        format!("id={}", sk.id)
    }
}
