use crate::route_generated::Redirect;
use anyhow::Result;
use cookie::CookieJar;
use forte_sdk::*;
use http::HeaderMap;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok {},
}

pub async fn handler(_headers: HeaderMap, jar: &mut CookieJar) -> Result<Props> {
    let user = crate::common::auth::get_me(jar);
    if user.is_some() {
        return Ok(Props::Ok {});
    }

    let github_auth_url = crate::common::auth::create_github_auth_url(jar, "", Redirect::Write);

    Err(Redirect::External {
        url: github_auth_url,
    }
    .into())
}
