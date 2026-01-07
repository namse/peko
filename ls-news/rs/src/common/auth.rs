use forte_sdk::cookie::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserInCookie {
    user_id: String,
}
const USER_COOKIE: &str = "USER";
const ADMIN_ID: &str = "3580430";

pub fn is_admin(jar: &CookieJar) -> bool {
    let Some(user) = forte_sdk::cookie_sign::unsign_cookie::<UserInCookie>(jar, USER_COOKIE) else {
        return false;
    };
    user.user_id == ADMIN_ID
}
