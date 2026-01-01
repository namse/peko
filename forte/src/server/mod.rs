mod cache;

pub use cache::SimpleCache;

use anyhow::Result;
use fn0::{CodeKind, DeploymentMap, Fn0};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct ServerConfig {
    pub port: u16,
    pub backend_path: String,
    pub frontend_path: String,
}

pub async fn run(config: ServerConfig) -> Result<()> {
    let mut deployment_map = DeploymentMap::new();
    deployment_map.register_code("backend", CodeKind::Wasm);
    deployment_map.register_code("frontend", CodeKind::Js);

    let cache = SimpleCache::new(config.backend_path, config.frontend_path);

    let fn0 = Arc::new(Fn0::new(cache.clone(), cache, deployment_map));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;
    println!("Forte SSR server listening on http://{}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let fn0_clone = fn0.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(socket);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0_clone.clone();
                        handle_request(req, fn0)
                    }),
                )
                .await
            {
                eprintln!("Failed to serve connection: {}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    fn0: Arc<Fn0<SimpleCache>>,
) -> Result<fn0::Response> {
    let uri = req.uri().clone();
    println!("Received {} {uri}", req.method());

    let backend_response = match fn0
        .run(
            "backend",
            req.map(|body| {
                UnsyncBoxBody::new(body)
                    .map_err(|e| anyhow::anyhow!(e))
                    .boxed_unsync()
            }),
        )
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Backend error: {:?}", e);
            return Err(anyhow::anyhow!("Backend error: {:?}", e));
        }
    };

    let backend_status = backend_response.status();

    println!("Backend response status: {}", backend_status);

    if !backend_status.is_success() {
        let (parts, body) = backend_response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("Backend error response body: {}", body_str);

        return Ok(fn0::Response::from_parts(
            parts,
            UnsyncBoxBody::new(body_str.to_string())
                .map_err(|e| anyhow::anyhow!(e))
                .boxed_unsync(),
        ));
    }

    println!("Preparing frontend request with backend response body");
    let frontend_request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(backend_response.into_body())?;

    println!("Calling frontend (ski::run)");
    match fn0.run("frontend", frontend_request).await {
        Ok(resp) => {
            println!("Frontend response status: {}", resp.status());
            Ok(resp)
        }
        Err(e) => {
            eprintln!("Frontend error: {:?}", e);
            Err(e)
        }
    }
}
