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
            // Convert libsql: scheme to https, preserve http for local development
            match parts.scheme.as_ref().map(|s| s.as_str()) {
                Some("libsql") | None => {
                    parts.scheme = Some("https".parse().unwrap());
                }
                _ => {} // Keep http or other schemes as-is
            }
            parts.path_and_query = Some("/v2/pipeline".parse().unwrap());
            Uri::from_parts(parts)?
        };

        Ok(Self {
            http_url,
            client: Client::new(),
            auth_token,
        })
    }
    async fn execute_pipeline(&self, requests: Vec<StreamRequest>) -> Result<PipelineRespBody> {
        let body = PipelineReqBody {
            baton: None,
            requests,
        };

        let mut request_builder = Request::post(&self.http_url);

        if !self.auth_token.is_empty() {
            request_builder = request_builder.header(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", self.auth_token))?,
            );
        }

        let request = request_builder
            .header("Content-Type", HeaderValue::from_str("application/json")?)
            .body(
                serde_json::to_vec(&body)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize body: {}", e))?,
            )?;

        let response = self.client.send(request).await?;
        if !response.status().is_success() {
            bail!("Turso request failed with status: {}", response.status());
        }

        Ok(response.into_body().json().await?)
    }

    pub(crate) async fn execute_sql(&self, sql: &str) -> Result<()> {
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(sql.to_string()),
                        sql_id: None,
                        args: vec![],
                        named_args: vec![],
                        want_rows: Some(false),
                        replication_index: None,
                    },
                }),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("SQL error: {}", error.message);
            }
        }

        Ok(())
    }

    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let response = self
            .execute_pipeline(vec![
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
            ])
            .await?;

        for result in response.results {
            match result {
                StreamResult::Ok { response } => match response {
                    StreamResponse::Execute(exec_resp) => {
                        if let Some(Value::Blob { value }) = exec_resp
                            .result
                            .rows
                            .first()
                            .and_then(|row| row.values.first())
                        {
                            return Ok(Some(value.clone()));
                        }
                        return Ok(None);
                    }
                    StreamResponse::Close(_) => continue,
                    _ => {}
                },
                StreamResult::Error { error } => {
                    bail!("Turso error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }

        Ok(None)
    }

    pub(crate) async fn insert(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(
                            "INSERT OR REPLACE INTO docs (pk, sk, data) VALUES (?, ?, ?)"
                                .to_string(),
                        ),
                        sql_id: None,
                        args: vec![
                            Value::Text {
                                value: pk.to_string().into(),
                            },
                            Value::Text {
                                value: sk.to_string().into(),
                            },
                            Value::Blob {
                                value: data.to_vec().into(),
                            },
                        ],
                        named_args: vec![],
                        want_rows: Some(false),
                        replication_index: None,
                    },
                }),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Insert error: {}", error.message);
            }
        }

        Ok(())
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some("DELETE FROM docs WHERE pk = ? AND sk = ?".to_string()),
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
                        want_rows: Some(false),
                        replication_index: None,
                    },
                }),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Delete error: {}", error.message);
            }
        }

        Ok(())
    }
}
