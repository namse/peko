pub mod cookie_sign;
mod generate_routes;

pub use anyhow;
pub use cookie;
pub use forte_db;
pub use forte_json;
pub use forte_macros::doc;
pub use generate_routes::*;
pub use http;
pub use serde;
pub use serde_json;
pub use sha2;
pub use wstd;
pub type DateTime = chrono::DateTime<chrono::Utc>;

pub trait Doc {
    type Pk;
    type Sk;

    fn pk(pk: Self::Pk) -> String;
    fn sk(sk: Self::Sk) -> String;
}
