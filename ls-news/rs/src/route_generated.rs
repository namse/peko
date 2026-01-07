#[path = "pages/index/mod.rs"]
mod pages_index;
use forte_sdk::anyhow::Result;
use forte_sdk::wstd::http::{Error, Request, Response, StatusCode, body::Body, HeaderMap};
use forte_sdk::http::header::COOKIE;
use forte_sdk::*;
#[forte_sdk::wstd::http_server]
pub async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
    let (parts, _body) = request.into_parts();
    let headers = parts.headers;
    let path = parts.uri.path();
    let cookie_jar = make_cookie_jar(&headers);
    if path == "/" {
        match pages_index::handler(headers, cookie_jar).await {
            Ok(props) => {
                let stream = forte_json::to_stream(&props);
                Ok(Response::new(Body::from_stream(stream)))
            }
            Err(e) => {
                Ok(
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!("Error: {:?}", e)))
                        .unwrap(),
                )
            }
        }
    } else {
        Ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        )
    }
}
fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {
    let mut jar = cookie::CookieJar::new();
    let Some(cookie) = headers.get(COOKIE) else {
        return jar;
    };
    let Ok(cookie_str) = cookie.to_str() else {
        return jar;
    };
    for cookie in cookie::Cookie::split_parse_encoded(cookie_str) {
        let Ok(cookie) = cookie else { continue };
        jar.add_original(cookie.into_owned());
    }
    jar
}
