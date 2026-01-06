use anyhow::Result;
use http::HeaderMap;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(_headers: HeaderMap) -> Result<Props> {
    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}
