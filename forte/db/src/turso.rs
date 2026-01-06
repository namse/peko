use anyhow::{Result, bail};
use libsql_hrana::proto::*;
use std::str::FromStr;
use wstd::http::{Client, HeaderValue, Request, Uri, body::Bytes};

pub(crate) struct TursoDatabase {
    http_url: Uri,
    client: Client,
    auth_token: String,
}

impl TursoDatabase {
    pub(crate) fn new(url: String, auth_token: String) -> Result<Self> {
        let http_url = {
            let url =
                Uri::from_str(&url).map_err(|e| anyhow::anyhow!("Failed to parse URI: {}", e))?;
            let mut parts = url.into_parts();
            parts.scheme = Some("https".parse().unwrap());
            parts.path_and_query = Some("/v2/pipeline".parse().unwrap());
            Uri::from_parts(parts)?
        };

        Ok(Self {
            http_url,
            client: Client::new(),
            auth_token,
        })
    }
    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let body = PipelineReqBody {
            baton: None,
            requests: vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some("SELECT data FROM docs WHERE pk = ? AND sk = ?".to_string()),
                        sql_id: None,
                        args: vec![
                            Value::Text {
                                value: pk.to_string().into(),
                            },
                            Value::Text {
                                value: sk.to_string().into(),
                            },
                        ],
                        named_args: vec![],
                        want_rows: Some(true),
                        replication_index: None,
                    },
                }),
                StreamRequest::Close(CloseStreamReq {}),
            ],
        };

        let request = Request::get(&self.http_url)
            .header(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", self.auth_token))?,
            )
            .header("Content-Type", HeaderValue::from_str("application/json")?)
            .body(
                serde_json::to_vec(&body)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize body: {}", e))?,
            )?;

        let response = self.client.send(request).await?;
        if !response.status().is_success() {
            bail!("Turso request failed with status: {}", response.status());
        }

        let response_body: PipelineRespBody = response.into_body().json().await?;

        for result in response_body.results {
            match result {
                StreamResult::Ok { response } => {
                    let unexpected_type = match response {
                        StreamResponse::Close(_close_stream_resp) => continue,
                        StreamResponse::Execute(execute_stream_resp) => {
                            let Some(first_row) = execute_stream_resp.result.rows.first() else {
                                bail!("No rows returned");
                            };
                            let Some(first_data) = first_row.values.first() else {
                                bail!("No data in first row");
                            };
                            let unexpected_data_type = match first_data {
                                Value::None => "None",
                                Value::Null => "Null",
                                Value::Integer { value: _ } => "Integer",
                                Value::Float { value: _ } => "Float",
                                Value::Text { value: _ } => "Text",
                                Value::Blob { value } => return Ok(Some(value.clone())),
                            };
                            bail!("Unexpected data type: {unexpected_data_type}");
                        }
                        StreamResponse::Batch(_batch_stream_resp) => "Batch",
                        StreamResponse::Sequence(_sequence_stream_resp) => "Sequence",
                        StreamResponse::Describe(_describe_stream_resp) => "Describe",
                        StreamResponse::StoreSql(_store_sql_stream_resp) => "StoreSql",
                        StreamResponse::CloseSql(_close_sql_stream_resp) => "CloseSql",
                        StreamResponse::GetAutocommit(_get_autocommit_stream_resp) => {
                            "GetAutocommit"
                        }
                    };
                    bail!("Turso response unexpected type: {unexpected_type}");
                }
                StreamResult::Error { error } => {
                    bail!("Turso error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }

        bail!("No data returned from Turso")
    }
}
